//! ## Return Statement Lowering
//!
//! Lowers Solidity `return` statements into NeoVM instructions. Handles:
//!
//! * **Explicit return** — `lower_return_statement` for `return expr;`
//! * **Implicit return** — `lower_implicit_return` for trailing expression
//! * **External wrapping** — `wrap_external_single_array_return_value` for
//!   ABI-encoded multi-return values from cross-contract calls
//! * **ABI decode passthrough** — `lower_abi_decode_passthrough_return` for
//!   `return abi.decode(...)` optimization
//! * **Tuple flattening** — `flatten_tuple_return_params` for multi-return

use super::*;

pub(crate) fn lower_return_statement(
    expr: Option<&Expression>,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    // Task #91 — inside an inlined library-storage body, redirect
    // `return expr;` into the synthesised slot + jump-to-end label instead of
    // emitting a raw `Return` that would exit the caller function.
    if let Some((slot, end_label)) = ctx.inline_return_target() {
        if let Some(expression) = expr {
            if lower_expression(expression, ctx, instructions) {
                if let Some(index) = slot {
                    instructions.push(Instruction::StoreLocal(index));
                } else {
                    instructions.push(Instruction::Drop(ValueType::Any));
                }
            }
        }
        instructions.push(Instruction::Jump { target: end_label });
        return true;
    }

    // A `return` lexically inside one or more enclosing catch bodies executes
    // with those NeoVM TRY frames still live (state = Catch). Returning would
    // otherwise RET (or, under a modifier epilogue, JMP out of the catch) with a
    // non-empty try-stack, which the reference C# NeoVM faults on. Pop each
    // active frame here: ENDTRY pops the innermost frame and jumps to the label
    // immediately after it, leaving the next frame innermost. ENDTRY only
    // touches the try-stack (not the evaluation stack or locals), so the return
    // value lowered below is unaffected. Library-inline returns (handled above)
    // jump within the catch body and do not exit the function, so they skip
    // this.
    for _ in 0..ctx.active_catch_frames() {
        let after = ctx.next_label();
        instructions.push(Instruction::EndTry { target: after });
        instructions.push(Instruction::Label(after));
    }

    // Task #114 — inside a function whose body was expanded with modifier
    // epilogues (`had_modifier_epilogue == true`), redirect `return expr;`
    // into the synthesised return slots + jump to the innermost
    // modifier-wrap break label. That label sits between the inlined body
    // and the modifier epilogue statements, so the epilogue (`locked = 0;`
    // etc.) still runs before the function performs its actual RET. If we
    // have not yet entered the modifier-wrap `do { } while(false)` scope
    // (e.g. `return` before the wrap for some pathological rewrite), fall
    // back to the redirect's `end_label` which sits after the whole
    // expanded body.
    if let Some((slots, fallback_label)) = ctx.modifier_return_target() {
        let jump_label = ctx
            .innermost_modifier_break_label()
            .unwrap_or(fallback_label);

        if let Some(expression) = expr {
            // Multi-return: `return (a, b);` where expr is Expression::List.
            // Pair each tuple element with the matching synthetic slot.
            if let Expression::List(_, params) = expression {
                if params.len() == slots.len() && slots.iter().all(|s| s.is_some()) {
                    // Iterate in reverse-store order: lower each arg, store
                    // into slot. We lower left-to-right and store each as we
                    // go (popping the stack in the correct order).
                    //
                    // Safer approach: lower all args (stack: [v_0, v_1, ...]),
                    // then store in reverse (v_N-1 first).
                    let pre_len = instructions.len();
                    let mut all_ok = true;
                    for (_, param) in params.iter() {
                        let Some(parameter) = param else {
                            all_ok = false;
                            break;
                        };
                        if !lower_expression(&parameter.ty, ctx, instructions) {
                            all_ok = false;
                            break;
                        }
                    }
                    if all_ok {
                        // Pop in reverse: last value lowered is at TOS.
                        for slot in slots.iter().rev() {
                            if let Some(idx) = slot {
                                instructions.push(Instruction::StoreLocal(*idx));
                            } else {
                                instructions.push(Instruction::Drop(ValueType::Any));
                            }
                        }
                        instructions.push(Instruction::Jump { target: jump_label });
                        return true;
                    }
                    instructions.truncate(pre_len);
                }
            }

            // Single-value return (or fall-through from multi-return
            // lowering failure): lower the expression, store into the first
            // slot.
            if lower_expression(expression, ctx, instructions) {
                if let Some(first_slot) = slots.first().and_then(|s| *s) {
                    instructions.push(Instruction::StoreLocal(first_slot));
                } else {
                    instructions.push(Instruction::Drop(ValueType::Any));
                }
            }
        }
        instructions.push(Instruction::Jump { target: jump_label });
        return true;
    }
    if let Some(expression) = expr {
        // Task #64 — multi-value return lowering for Externally-callable
        // functions (Public/External).
        //
        // For `return (a, b);` on a function declared `returns (T, U)` we must
        // emit EVM-canonical ABI-encoded bytes (BE-padded 32-byte slots),
        // NOT a `StackItem::Array` that would serialize via serde_json at the
        // main-frame RET. We reuse the `abiEncode` StdLib handler wired up in
        // Task #44: lower each tuple element as a separate expression and
        // wrap the sequence in `CallBuiltin { AbiEncode, arg_count: N }`,
        // which the bytecode emitter lowers to PACK + StdLib.abiEncode.
        //
        // Guards:
        //   (1) only rewrite when the tuple arity matches the function's
        //       declared return arity (keeps `return (single_tuple);` or
        //       parenthesised single expressions on the Array path),
        //   (2) only rewrite for externally-callable functions — internal
        //       callers expect a StackItem::Array they can destructure via
        //       `ArrayGet` (see tuple_assignment_picks_items_from_returned_array).
        if let Expression::List(_, params) = expression {
            let return_types = ctx.return_types().to_vec();
            if ctx.is_externally_callable()
                && return_types.len() >= 2
                && params.len() == return_types.len()
            {
                let tuple_exprs: Option<Vec<Expression>> = params
                    .iter()
                    .map(|(_, param)| param.as_ref().map(|p| p.ty.clone()))
                    .collect();
                if let Some(tuple_exprs) = tuple_exprs.as_ref() {
                    // `lower_abi_encode_args_direct_from_slice` encodes each
                    // element by its INFERRED expression type, which for a bare
                    // hex literal is a number → a RIGHT-aligned (uint-style)
                    // slot. When the DECLARED return type is `bytesN`, the slot
                    // must be LEFT-aligned, so skip this fast path and fall
                    // through to the declared-type-aware loop below (which
                    // canonicalizes the literal via `lower_static_abi_return_expr_slot`).
                    let has_bytesn_literal_mismatch =
                        return_types.iter().zip(tuple_exprs.iter()).any(|(rt, e)| {
                            matches!(rt, ValueType::ByteArray { fixed_len: Some(_) })
                                && is_integer_backed_bytesn_operand(e, ctx)
                        });
                    if !has_bytesn_literal_mismatch
                        && !tuple_exprs
                            .iter()
                            .any(|expr| matches!(expr, Expression::List(_, _)))
                    {
                        let original_len = instructions.len();
                        if let Some(ok) =
                            lower_abi_encode_args_direct_from_slice(tuple_exprs, ctx, instructions)
                        {
                            if ok {
                                instructions.push(Instruction::Return);
                                return true;
                            }
                            instructions.truncate(original_len);
                        }
                    }
                }

                let original_len = instructions.len();
                if return_types.iter().all(is_static_abi_slot_value_type) {
                    let mut success = true;
                    for ((_, param), value_type) in params.iter().zip(return_types.iter()) {
                        let Some(parameter) = param else {
                            success = false;
                            break;
                        };
                        if !lower_static_abi_return_expr_slot(
                            &parameter.ty,
                            value_type,
                            ctx,
                            instructions,
                        ) {
                            success = false;
                            break;
                        }
                    }
                    if success {
                        instructions.push(Instruction::CallBuiltin {
                            builtin: BuiltinCall::BytesConcat,
                            arg_count: return_types.len(),
                        });
                        instructions.push(Instruction::Return);
                        return true;
                    }
                    instructions.truncate(original_len);
                }

                let original_len = instructions.len();
                // Task #94 — flatten nested tuple return expressions so
                // `return ((1, 2), 3)` for `returns ((uint,uint), uint)`
                // emits 3 head slots (`01 || 02 || 03`) rather than wrapping
                // the inner tuple in a StackItem::Array (which `abiEncode`
                // would classify as dynamic and encode with a junk tail).
                // Spec: Solidity inlines a static inner tuple into the
                // parent's head section.
                let mut flat_count = 0usize;
                let mut success =
                    flatten_tuple_return_params(params, ctx, instructions, &mut flat_count);
                if success && flat_count < params.len() {
                    // Defensive: the flatten walk must produce at least one
                    // leaf per declared param. An empty inner tuple would
                    // silently drop slots — fall back to the legacy path.
                    success = false;
                }
                if success {
                    instructions.push(Instruction::CallBuiltin {
                        builtin: BuiltinCall::BytesConcat,
                        arg_count: flat_count,
                    });
                    instructions.push(Instruction::Return);
                    return true;
                }
                // Fall through to legacy path on failure.
                instructions.truncate(original_len);
            }
        }

        // Task #116 — `return abi.decode(buf, (T1, ..., Tn));` where every
        // `Ti` is a static 32-byte ABI type (uintN/intN/address/bool/bytesN)
        // and the function is externally-callable with matching return arity
        // collapses to a pure pass-through: the ABI canonical layout of the
        // return tuple IS the input buffer, so round-tripping through
        // `abiDecode → Array → abiEncode` would be wasted work. More
        // importantly, the intermediate `StackItem::Array` would fall
        // through to `stack_item_to_bytes` at the main-frame RET and
        // serialize as a serde_json blob (see batch50 Z3 witness: 236 bytes
        // of JSON instead of the expected 96-byte BE-packed tuple).
        //
        // Short-circuit the round trip: lower only the buffer, reuse the
        // same panic(0x41) short-buffer guard that Task #84 installed on
        // the direct `abi.decode` path, and hand the buffer to RET. The
        // outer main-frame RET then treats the buffer as-is since the
        // ByteArray arm of `stack_item_to_bytes` is already a verbatim
        // copy.
        if let Some(abi_decode_args) = extract_abi_decode_call(expression) {
            let return_types = ctx.return_types().to_vec();
            if ctx.is_externally_callable()
                && return_types.len() >= 2
                && abi_decode_types_match_return_arity(&abi_decode_args, return_types.len())
            {
                if let Some(expected_bytes) = abi_decode_expected_static_bytes(&abi_decode_args) {
                    if let Some(buffer_expr) = abi_decode_args.first() {
                        if lower_abi_decode_passthrough_return(
                            buffer_expr,
                            expected_bytes,
                            BinaryOperator::Ne,
                            ctx,
                            instructions,
                        ) {
                            return true;
                        }
                    }
                }
            }

            // Task #127 — mixed-head/tail variant of the Task #116 short-
            // circuit. For `return abi.decode(buf, (uint, string, address))`
            // (or any tuple mixing static 32-byte slots with dynamic-tail
            // types like `string`/`bytes`/`T[]`) the EVM canonical encoding
            // of the return tuple is ALSO the input buffer verbatim: every
            // type (static or dynamic) occupies exactly one head slot, with
            // dynamic types' head slot holding an offset that points to
            // their length+payload tail section. Re-encoding would yield
            // the same byte sequence.
            //
            // The previous lowering path ran the buffer through the runtime
            // `abiDecode` handler, which for a mixed tuple returned a
            // `StackItem::Array` of per-slot items. That Array then fell
            // through to `stack_item_to_bytes` at the main-frame RET and
            // serialised as a serde_json blob (`{"type":"Array","value":...}`),
            // leaking 380 bytes of UTF-8 JSON where the caller expected the
            // 160-byte EVM-canonical re-encoding (batch58 HH4 witness).
            //
            // The short-circuit here is identical to the static-tuple case
            // (lower the buffer, guard its size, hand it to RET) with a
            // looser size guard: dynamic payloads vary, so we can only
            // assert `size >= head_slot_count * 32` — every declared type
            // contributes one head slot, and the tail must exist below.
            // When the guard fails we still panic(0x41) for parity with
            // the static case and Task #84's direct-decode guard.
            if ctx.is_externally_callable()
                && return_types.len() >= 2
                && abi_decode_types_match_return_arity_mixed(&abi_decode_args, return_types.len())
            {
                if let Some(buffer_expr) = abi_decode_args.first() {
                    // Minimum buffer size = one 32-byte head slot per
                    // declared ABI type. Every type (static or dynamic)
                    // contributes exactly one head slot; the dynamic
                    // tails live below the head block at the offsets
                    // declared within the head slots themselves.
                    let min_bytes: u32 = (return_types.len() as u32).saturating_mul(32);
                    if lower_abi_decode_passthrough_return(
                        buffer_expr,
                        min_bytes,
                        BinaryOperator::Lt,
                        ctx,
                        instructions,
                    ) {
                        return true;
                    }
                }
            }

            if ctx.is_externally_callable() && return_types.len() == 1 {
                if let Some(decoded_types) = abi_decode_value_types(&abi_decode_args[1], ctx) {
                    let decoded_type = decoded_types.first();
                    let return_type = return_types.first();
                    let matching_single_type = decoded_types.len() == 1
                        && decoded_type.zip(return_type).is_some_and(|(decoded, ret)| {
                            decoded == ret
                                || (abi_static_slot_count(decoded) == Some(1)
                                    && abi_static_slot_count(ret) == Some(1))
                                || (abi_value_type_is_dynamic(decoded)
                                    && abi_value_type_is_dynamic(ret))
                        });

                    if matching_single_type && decoded_type.is_some_and(abi_value_type_is_dynamic) {
                        if let Some(buffer_expr) = abi_decode_args.first() {
                            // Dynamic single-return: minimum buffer is 64 bytes
                            // (one 32-byte offset slot + one 32-byte length
                            // slot for the inner dynamic value's tail header).
                            if lower_abi_decode_passthrough_return(
                                buffer_expr,
                                64u32,
                                BinaryOperator::Lt,
                                ctx,
                                instructions,
                            ) {
                                return true;
                            }
                        }
                    }
                }
            }
        }

        // A bare integer-backed `bytesN` literal/constant returned directly
        // (`return 0x01020304;`, `return SELECTOR;`) would push its little-endian
        // Integer form and RET, so an external caller receives an Integer — not
        // the ByteString the manifest declares for `bytesN` (verified diverging
        // on a real node: `return 0x01020304` came back as Integer 16909060).
        // Canonicalize to the big-endian N-byte ByteString, matching the
        // binding-site canonicalization (`try_lower_bytesn_literal_canonical`)
        // and the manifest return type. Variables/casts/keccak are already
        // ByteString-backed, so only the integer-backed literal/constant case
        // is intercepted here.
        {
            let return_types = ctx.return_types();
            if let [ValueType::ByteArray { fixed_len: Some(n) }] = return_types {
                let n = *n;
                if is_integer_backed_bytesn_operand(expression, ctx) {
                    if let Some(be) = fixed_len_bytes_be_from_hex_or_const(expression, n, ctx) {
                        instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(be)));
                        instructions.push(Instruction::Return);
                        return true;
                    }
                }
            }
        }

        // NOTE: `return b[i]` (a byte index of bytes/bytesN returned as `bytes1`)
        // is handled at the SOURCE — `try_lower_expression_primary` coerces a
        // byte index to a 1-byte ByteString — so the generic path below returns
        // the correct ByteString without a special case here.

        if lower_expression(expression, ctx, instructions) {
            // Task #137 — single-value dynamic-array return. For
            // `function f(uint[] memory a) external pure returns (uint[] memory)
            // { ... return a; }` the `return a;` lowers to a raw
            // `StackItem::Array` push that, at the main-frame RET, falls through
            // to `stack_item_to_bytes` and gets serde_json-wrapped
            // (`{"type":"Array","value":[...]}`). Task #121 fixed the internally
            // constructed `return abi.decode(abi.encode(a), (uint[]))` round
            // trip by routing its payload through the runtime `abiencode`
            // helper; the PARAMETER-passthrough shape (`return a;`) never
            // entered that helper because no explicit `abi.encode` call exists
            // in the source. Wrap the single-value externally-callable return
            // with `AbiEncode(1 arg)` so the runtime produces the EVM-canonical
            // `offset=32 || length=N || N × 32-byte BE-padded elements` layout
            // instead of the JSON wrapper.
            //
            // Scope: only `ValueType::Array(_)` fires today — `bytes memory`
            // and `string memory` returns already land as `StackItem::ByteArray`
            // (verbatim in `stack_item_to_bytes`) and existing tests (e.g.
            // `abi_encodePacked_small_width_matches_spec`) pin the raw-bytes
            // shape, so re-wrapping them would regress. Struct / mapping
            // single-value returns are out of scope until a harness exists.
            if !wrap_external_single_array_return_value(ctx, instructions) {
                return false;
            }
            instructions.push(Instruction::Return);
            return true;
        }
    } else {
        let return_types = ctx.return_types().to_vec();
        let return_slots = ctx.return_slots().to_vec();

        if return_types.is_empty() {
            instructions.push(Instruction::ReturnVoid);
            return true;
        }
        return lower_implicit_return(&return_types, &return_slots, ctx, instructions);
    }
    false
}

/// Task #64/#137/#185 — wrap a single externally-callable ARRAY return
/// value (already on the stack) into its EVM-canonical byte encoding.
///
/// Shared by the explicit `return expr;` path and the implicit named-return
/// path so both produce the same on-stack shape for identical signatures
/// (previously `returns (T[] memory x)` with an implicit return leaked the
/// raw `StackItem::Array` while `return x;` emitted canonical bytes).
///
/// No-op (returns `true`) for internal functions and non-array /
/// multi-value returns. Returns `false` only on a fatal lowering error
/// (already recorded on `ctx`).
pub(crate) fn wrap_external_single_array_return_value(
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    if ctx.is_externally_callable() {
        let return_types = ctx.return_types().to_vec();
        if return_types.len() == 1 && matches!(return_types.first(), Some(ValueType::Array(_))) {
            // `return_types.len() == 1` (this guard) makes indexing `[0]` safe;
            // bind the single element once instead of re-`.unwrap()`ing it at
            // each use site, so a future refactor that relaxes the guard can't
            // turn a logic error into a panic.
            let first_ret_type = &return_types[0];
            // Task #185 — nested fixed-size array return (e.g.
            // `uint[3][2] memory`). The EVM canonical encoding is a
            // flat concat of leaf values: `pad32_be(a[0][0]) ||
            // pad32_be(a[0][1]) || ... || pad32_be(a[N-1][M-1])`,
            // NOT the dynamic-array `offset || length || elements`
            // wrapper that generic `AbiEncode(Array)` emits. We
            // detect this from the raw Solidity type string (e.g.
            // "uint[3][2]") since `ValueType::Array` alone does not
            // preserve fixed-size dimensions. When it matches, stash
            // the array in a local, unroll the nested index
            // traversal into per-leaf `ArrayGet` sequences, then emit
            // each leaf as a direct 32-byte ABI slot.
            let ret_ty_strings = ctx.return_type_strings();
            if let Some(first_ty) = ret_ty_strings.first() {
                if let Some(dims) = parse_nested_fixed_array_shape(first_ty) {
                    if let Some(leaf_type) = array_leaf_static_value_type(first_ret_type) {
                        // Cap the unrolled instruction count so a
                        // pathological return type like
                        // `uint[1000000000][1000000000] memory`
                        // (from a malicious source) doesn't OOM /
                        // time-out the compiler emitting billions
                        // of `LoadLocal`/`ArrayGet` pairs.
                        // Real contract returns top out at a few
                        // hundred leaves; 65_536 is a generous
                        // cap well above any legitimate use.
                        const MAX_FIXED_ARRAY_LEAVES: usize = 65_536;
                        let total_leaves: usize = dims
                            .iter()
                            .try_fold(1usize, |acc, d| acc.checked_mul(*d))
                            .filter(|n| *n <= MAX_FIXED_ARRAY_LEAVES)
                            .unwrap_or(0);
                        if total_leaves > 0 {
                            // Stash the outer array in a temp local
                            // so each per-leaf traversal can start
                            // from the same base.
                            let tmp_id = ctx.next_label();
                            let array_local =
                                ctx.allocate_local(format!("__flat_ret_arr_{tmp_id}"), None);
                            instructions.push(Instruction::StoreLocal(array_local));

                            // Enumerate every leaf coordinate in
                            // row-major (outer-most first) order so
                            // the flat output matches the Solidity
                            // static encoding layout.
                            let mut coord = vec![0usize; dims.len()];
                            loop {
                                // Walk from the stashed array down
                                // through each dimension: load
                                // array, PushLiteral(i_k), ArrayGet
                                // for k in 0..dims.len().
                                instructions.push(Instruction::LoadLocal(array_local));
                                for &idx in &coord {
                                    instructions.push(Instruction::PushLiteral(
                                        LiteralValue::Integer(BigInt::from(idx as u64)),
                                    ));
                                    instructions.push(Instruction::ArrayGet);
                                }
                                if !emit_static_abi_slot_for_value_type(
                                    &leaf_type,
                                    ctx,
                                    instructions,
                                ) {
                                    ctx.record_error("failed to encode fixed-array return leaf");
                                    return false;
                                }

                                // Advance to the next coordinate
                                // (least-significant dim increments
                                // first, with carry into outer
                                // dims) until all coords exhausted.
                                let mut i = dims.len();
                                let mut done = true;
                                while i > 0 {
                                    i -= 1;
                                    coord[i] += 1;
                                    if coord[i] < dims[i] {
                                        done = false;
                                        break;
                                    }
                                    coord[i] = 0;
                                }
                                if done {
                                    break;
                                }
                            }

                            instructions.push(Instruction::CallBuiltin {
                                builtin: BuiltinCall::BytesConcat,
                                arg_count: total_leaves,
                            });
                            return true;
                        }
                    }
                }
            }
            if emit_abi_encode_single_stack_value_for_type(first_ret_type, ctx, instructions)
                .is_none()
            {
                instructions.push(Instruction::CallBuiltin {
                    builtin: BuiltinCall::AbiEncode,
                    arg_count: 1,
                });
            }
        }
    }
    true
}

/// Implicit `return;` (and the fall-off-end epilogue) for functions with
/// named return variables: load each declared return slot and emit the
/// shape matching the explicit-return path for the same signature.
pub(crate) fn lower_implicit_return(
    return_types: &[ValueType],
    return_slots: &[Option<usize>],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    {
        if return_slots.iter().any(|slot| slot.is_none()) {
            ctx.record_error_with_suggestion(
                "return without value requires named return variables for this function",
                "either use named return variables (e.g. function f() returns (uint256 result)) or provide an explicit return value",
            );
        }

        if return_types.len() == 1 {
            match return_slots.first().and_then(|slot| *slot) {
                Some(local_index) => instructions.push(Instruction::LoadLocal(local_index)),
                None => {
                    if let Some(value_type) = return_types.first() {
                        push_default_for_value_type(value_type, ctx, instructions);
                    } else {
                        instructions.push(Instruction::PushLiteral(LiteralValue::Null));
                    }
                }
            }
            // Keep the implicit named-return shape identical to the
            // explicit `return x;` shape: externally-callable array
            // returns are abi-encoded into canonical bytes.
            if !wrap_external_single_array_return_value(ctx, instructions) {
                return false;
            }
            instructions.push(Instruction::Return);
            return true;
        }

        // Task #64 — implicit multi-value return (named returns, no expr).
        // For externally-callable functions, push each declared return local
        // in order and hand the sequence to `abiEncode` so the main-frame RET
        // emits BE-packed bytes instead of a JSON-serialised Array. For
        // internal/private functions, preserve the legacy Array shape so
        // intra-contract callers can destructure via `ArrayGet`.
        if ctx.is_externally_callable() {
            let static_slot_return = return_types.iter().all(is_static_abi_slot_value_type);
            for (slot, value_type) in return_slots.iter().zip(return_types.iter()) {
                if let Some(local_index) = slot {
                    instructions.push(Instruction::LoadLocal(*local_index));
                } else {
                    push_default_for_value_type(value_type, ctx, instructions);
                }
                if static_slot_return
                    && !emit_static_abi_slot_for_value_type(value_type, ctx, instructions)
                {
                    ctx.record_error("failed to encode static ABI return slot");
                    return false;
                }
            }
            if static_slot_return {
                instructions.push(Instruction::CallBuiltin {
                    builtin: BuiltinCall::BytesConcat,
                    arg_count: return_types.len(),
                });
            } else {
                instructions.push(Instruction::CallBuiltin {
                    builtin: BuiltinCall::AbiEncode,
                    arg_count: return_types.len(),
                });
            }
            instructions.push(Instruction::Return);
            return true;
        }

        // Legacy Array-packed shape for internal/private multi-return.
        let tmp_id = ctx.next_label();
        let tuple_local = ctx.allocate_local(
            format!("__return_tuple_{tmp_id}"),
            Some(ValueType::Array(Box::new(ValueType::Any))),
        );

        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::from(return_types.len() as u64),
        )));
        instructions.push(Instruction::NewArray {
            element_type: ValueType::Any,
        });
        instructions.push(Instruction::StoreLocal(tuple_local));

        for (index, (slot, value_type)) in return_slots.iter().zip(return_types.iter()).enumerate()
        {
            instructions.push(Instruction::LoadLocal(tuple_local));
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::from(index as u64),
            )));

            if let Some(local_index) = slot {
                instructions.push(Instruction::LoadLocal(*local_index));
            } else {
                push_default_for_value_type(value_type, ctx, instructions);
            }

            instructions.push(Instruction::ArraySet);
        }

        instructions.push(Instruction::LoadLocal(tuple_local));
        instructions.push(Instruction::Return);
        true
    }
}

/// Lower `return <buffer_expr>;` straight to a `RET` after a size guard.
/// Used by the three Task #116 / #127 / single-dynamic paths that
/// recognise an externally-callable `return abi.decode(buf, types)` whose
/// canonical byte encoding equals `buf` verbatim:
///
/// 1. Lower `buffer_expr` so the buffer sits on the stack.
/// 2. `Dup; GetSize; Push(expected); <cmp>; JumpIf(ok)` — branches over the
///    `Panic(0x41)` when the buffer violates the size contract.
/// 3. `Label(ok); Return` — RET the buffer verbatim.
///
/// The size check is `Lt` (size must be ≥ `expected`) for the
/// at-least-N-head-bytes paths (Task #127 mixed, single-dynamic) and `Ne`
/// (size must be exactly `expected`) for the static-tuple path
/// (Task #116). The `cmp` argument is the operator that produces the
/// BAD condition — the one whose FALSE result makes `JumpIf(ok)` skip the
/// panic.
///
/// On failure of step 1 (expression lowering already recorded the error),
/// `instructions` is rolled back to its pre-call length so the caller can
/// continue trying other lowering paths. Returns `true` on success.
fn lower_abi_decode_passthrough_return(
    buffer_expr: &Expression,
    expected_bytes: u32,
    cmp: BinaryOperator,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    let pre_len = instructions.len();
    if !lower_expression(buffer_expr, ctx, instructions) {
        instructions.truncate(pre_len);
        return false;
    }
    let decode_ok_label = ctx.next_label();
    instructions.push(Instruction::Dup);
    instructions.push(Instruction::GetSize);
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(expected_bytes),
    )));
    instructions.push(Instruction::BinaryOp(cmp));
    instructions.push(Instruction::JumpIf {
        target: decode_ok_label,
    });
    emit_panic(0x41, instructions);
    instructions.push(Instruction::Label(decode_ok_label));
    instructions.push(Instruction::Return);
    true
}

/// Task #116 — true iff `expr` is `abi.decode(buf, types)` (with 2 args)
/// where `buf` is any expression and `types` is the tuple type spec.
/// Returns a `Vec<Expression>` clone of the arg slice for downstream
/// helpers that only take `&[Expression]` (matches the shape those
/// helpers already accept, e.g. `abi_decode_expected_static_bytes`).
pub(crate) fn extract_abi_decode_call(expr: &Expression) -> Option<Vec<Expression>> {
    let Expression::FunctionCall(_, func, args) = expr else {
        return None;
    };
    if args.len() != 2 {
        return None;
    }
    let Expression::MemberAccess(_, receiver, member) = func.as_ref() else {
        return None;
    };
    if member.name != "decode" {
        return None;
    }
    match receiver.as_ref() {
        Expression::Variable(id) if id.name == "abi" => Some(args.clone()),
        _ => None,
    }
}

/// Task #116 — true iff the second arg to `abi.decode` is a tuple of
/// exactly `expected_arity` static 32-byte types (uintN / intN / address /
/// bool / bytesN). Callers use this to decide whether
/// `return abi.decode(buf, tuple)` can short-circuit via a verbatim
/// buffer return: the ABI canonical layout of a static tuple IS the
/// input buffer, so no decode/re-encode round trip is needed.
pub(crate) fn abi_decode_types_match_return_arity(
    args: &[Expression],
    expected_arity: usize,
) -> bool {
    if expected_arity < 2 {
        return false;
    }
    let Some(types_arg) = args.get(1) else {
        return false;
    };
    match types_arg {
        Expression::List(_, params) => {
            if params.len() != expected_arity {
                return false;
            }
            params
                .iter()
                .all(|(_, param)| param.as_ref().is_some_and(|p| is_static_abi_type(&p.ty)))
        }
        // Parenthesis / single-type forms can never match arity >= 2.
        _ => false,
    }
}

/// Task #127 — true iff the second arg to `abi.decode` is a tuple of
/// exactly `expected_arity` ABI-decodable types (either static 32-byte
/// types OR dynamic-tail types like `string`/`bytes`/`T[]`), AND at
/// least one element is dynamic. This is the "mixed head/tail" shape
/// that the all-static matcher (`abi_decode_types_match_return_arity`)
/// deliberately rejects.
///
/// The short-circuit for this shape is still "pass the input buffer
/// through verbatim": Solidity's EVM canonical encoding of a tuple
/// with dynamic tails uses offset indirection in the head slots to
/// point at the length+payload tail — which is exactly the input
/// layout the caller handed to `abi.decode`. Re-encoding would yield
/// an identical byte sequence (modulo trivial offset arithmetic that's
/// already baked into the buffer).
///
/// This matcher exists separately from `abi_decode_types_match_return_arity`
/// because (a) the static case can emit a tighter size guard
/// (`size == N*32`) whereas the mixed case must use a loose lower
/// bound (`size >= head_slots*32`) since dynamic payloads vary, and
/// (b) future refinements (e.g. enforcing that offsets are in-range)
/// can slot in here without perturbing the static-tuple path.
pub(crate) fn abi_decode_types_match_return_arity_mixed(
    args: &[Expression],
    expected_arity: usize,
) -> bool {
    if expected_arity < 2 {
        return false;
    }
    let Some(types_arg) = args.get(1) else {
        return false;
    };
    let Expression::List(_, params) = types_arg else {
        return false;
    };
    if params.len() != expected_arity {
        return false;
    }
    let mut all_ok = true;
    let mut has_dynamic = false;
    for (_, param) in params.iter() {
        let Some(p) = param.as_ref() else {
            return false;
        };
        if is_static_abi_type(&p.ty) {
            continue;
        }
        if is_dynamic_abi_type(&p.ty) {
            has_dynamic = true;
            continue;
        }
        all_ok = false;
    }
    all_ok && has_dynamic
}

/// Task #94 — lower the children of a tuple-return expression, recursively
/// flattening nested `Expression::List` into leaf values. Each leaf is a
/// single expression lowered via `lower_expression`; `flat_count` is
/// incremented once per leaf so the caller can pass the right `arg_count`
/// to `AbiEncode`.
///
/// For `return ((1, 2), 3)` with `returns ((uint,uint), uint)`, this emits
/// pushes for `1`, `2`, `3` — three static head slots, which `abiEncode`
/// packs into `32z|01 || 32z|02 || 32z|03`, matching Solidity's inlining
/// of a static inner tuple into its parent's head section.
pub(crate) fn flatten_tuple_return_params(
    params: &[(solang_parser::pt::Loc, Option<solang_parser::pt::Parameter>)],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
    flat_count: &mut usize,
) -> bool {
    for (_, param) in params.iter() {
        let Some(parameter) = param else {
            return false;
        };
        match &parameter.ty {
            Expression::List(_, inner_params) => {
                if !flatten_tuple_return_params(inner_params, ctx, instructions, flat_count) {
                    return false;
                }
            }
            expr => {
                let Some(value_type) = infer_type_from_expression(expr, ctx) else {
                    return false;
                };
                if !lower_static_abi_return_expr_slot(expr, &value_type, ctx, instructions) {
                    return false;
                }
                *flat_count += 1;
            }
        }
    }
    true
}
