use super::*;

/// Lower a subscript's array operand. A `bytesN` value that is INTEGER-backed
/// (a hex literal or a named `bytesN` constant) is pushed little-endian, so
/// `b[i]` would index the wrong byte — Solidity's `b[0]` is the MOST-significant
/// byte. Push its big-endian bytes as a ByteString so the PICKITEM below reads
/// the Solidity byte order. Everything else lowers normally (a runtime bytesN
/// value is already big-endian, and non-bytesN operands are unaffected).
fn lower_indexable_array_operand(
    array: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    if let Some(ValueType::ByteArray { fixed_len: Some(n) }) =
        infer_type_from_expression(array, ctx)
    {
        if is_integer_backed_bytesn_operand(array, ctx) {
            if let Some(be) = fixed_len_bytes_be_from_hex_or_const(array, n, ctx) {
                instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(be)));
                return true;
            }
        }
    }
    lower_expression(array, ctx, instructions)
}

pub(crate) fn lower_array_subscript_expression(
    expr: &Expression,
    array: &Expression,
    index: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    if let Some(mapping) = resolve_mapping_access(expr, ctx) {
        // Task #199 — storage state-variable Array subscript (`arr[idx]`
        // where `arr` is `uint[] a`-style) must emit the same Panic(0x32)
        // bounds guard that Task #107 installed for memory arrays. Without
        // this, `a[99]` on an empty storage array reads through
        // `emit_load_mapping` → `System.Storage.Get` which returns zero
        // silently — the callee appears to succeed and the caller's
        // `try Target(t).getAt(99) { … } catch Panic(uint) { … }` routes
        // through the try-arm (observed as "ok") instead of the
        // catch-Panic arm. Same-contract memory arrays already work via
        // `lower_array_subscript_expression`'s third branch below; this
        // branch handles the storage state-variable Array case that
        // `resolve_mapping_access` intercepts first. Cross-contract
        // propagation rides on top: once Target.getAt emits the canonical
        // Panic envelope via `emit_panic(0x32)`, the existing
        // `dispatch_exception` machinery (see
        // src/runtime/execution/instruction/flow/try_frames.rs) unwinds
        // across the self-offsets dispatch frame and routes the payload
        // into C's `catch Panic(uint c)` arm.
        //
        // Guard only fires when the subscript's terminal step traverses
        // an Array: for pure mapping accesses (`m[k]`, `m[k1][k2]`, …),
        // or mapping-of-array head (`m[k]`) without a trailing array
        // index, no bounds concept applies. The helper below inspects
        // the resolved value_type chain to decide.
        let array_subscript_terminal = mapping_terminal_hits_array(&mapping, ctx);
        if let Some(array_head) = array_subscript_terminal {
            return emit_storage_array_subscript_with_bounds(
                &mapping,
                array_head,
                ctx,
                instructions,
            );
        }
        let reference = mapping.to_storage_reference();
        emit_storage_load(&reference, ctx, instructions)
    } else if let Some(reference) = resolve_storage_reference(expr, ctx) {
        // Task #82: nested mapping-in-struct reads such as `slots[k].balances[a]`.
        //
        // Storage-soundness fix — when the TERMINAL subscript traverses an
        // Array reached through a struct field (`s.arr[i]`, `m[k].arr[i]`),
        // emit the same Panic(0x32) bounds guard the direct state-var path
        // gets from Task #199 above. Without it the read goes straight to
        // `System.Storage.Get`: out-of-range indices return zero defaults
        // and data "deleted" via `delete s` (which only zeroes the length
        // field slot, never the element slots) is silently resurrected.
        if !emit_struct_field_array_bounds_guard(array, index, ctx, instructions) {
            return false;
        }
        emit_storage_load(&reference, ctx, instructions)
    } else if lower_indexable_array_operand(array, ctx, instructions)
        && lower_expression(index, ctx, instructions)
    {
        // Task #107 — emit an explicit bounds guard so array-index-OOB
        // reverts with the canonical EVM Panic(uint256) envelope
        //   keccak256("Panic(uint256)")[..4] || abi.encode(0x32)
        // (instead of the generic runtime "PICKITEM: index out of bounds"
        // fault which `catch Panic(uint code)` cannot bind). Skip when the
        // array expression is one of the dynamic-array surfaces whose native
        // lowering already covers the guard (string / bytes byte-indexing)
        // by only emitting when the inferred element type is non-bytes — the
        // SIZE instruction works uniformly on both Array and ByteString, so
        // the guard is safe in both cases, and we opt in for both.
        let tmp_id = ctx.next_label();
        let idx_local = ctx.allocate_local(
            format!("__aidx_{tmp_id}"),
            Some(ValueType::Integer {
                signed: false,
                bits: 256,
            }),
        );
        let arr_local = ctx.allocate_local(format!("__aarr_{tmp_id}"), None);

        // Stack is [array, index] — stash both into locals so we can bounds-
        // check without disturbing the eventual ArrayGet operands.
        instructions.push(Instruction::StoreLocal(idx_local));
        instructions.push(Instruction::StoreLocal(arr_local));

        // Guard 1: index < 0 → Panic(0x32).  Solidity `uint256` indices
        // cannot be negative, but the runtime's PICKITEM still handles
        // signed-int operands, so we emit the check for defensive parity
        // with Solidity's own lowering.
        let after_neg_label = ctx.next_label();
        instructions.push(Instruction::LoadLocal(idx_local));
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::zero(),
        )));
        instructions.push(Instruction::BinaryOp(BinaryOperator::Lt));
        // JumpIf-on-false: skip the THROW when idx < 0 is false.
        instructions.push(Instruction::JumpIf {
            target: after_neg_label,
        });
        emit_panic(0x32, instructions);
        instructions.push(Instruction::Label(after_neg_label));

        // Guard 2: index >= SIZE(array) → Panic(0x32).
        let ok_label = ctx.next_label();
        instructions.push(Instruction::LoadLocal(idx_local));
        instructions.push(Instruction::LoadLocal(arr_local));
        instructions.push(Instruction::GetSize);
        instructions.push(Instruction::BinaryOp(BinaryOperator::Ge));
        instructions.push(Instruction::JumpIf { target: ok_label });
        emit_panic(0x32, instructions);
        instructions.push(Instruction::Label(ok_label));

        // Re-push [array, index] for ArrayGet.
        instructions.push(Instruction::LoadLocal(arr_local));
        instructions.push(Instruction::LoadLocal(idx_local));
        instructions.push(Instruction::ArrayGet);
        true
    } else {
        false
    }
}

/// Task #199 — Describes how to load the runtime length of a storage
/// array whose terminal subscript is about to be indexed, for the
/// Panic(0x32) bounds guard.
///
/// * `DynamicStateVar` — the subscript chain is `arr[idx]` for a direct
///   dynamic `T[] storage` state variable. Length lives at
///   `LoadState(state_index)` (the same slot that `push`/`pop` update,
///   see `src/ir/expressions/calls/storage_array/state_var.rs`).
/// * `MappingOfDynamicArray` — the chain is `m[k1]...[kN][idx]` where
///   the mapping value-type is `T[]` and the LAST key (`idx`) is the
///   array subscript. Length lives at the `LoadMappingElement` head
///   slot (the outer keys up to but not including `idx`).
/// * `FixedSizeKnown` — the subscript chain is `arr[idx]` for a direct
///   `T[N]` state variable (or a mapping-of-fixed-array). The bound is
///   the compile-time constant `N`. Detected by parsing the state
///   variable's source type string (`uint256[3]`); distinct from the
///   dynamic case because fixed-size arrays do NOT maintain a length
///   slot at `LoadState(state_index)` (which would always read zero
///   and fire a spurious Panic(0x32) on every in-range access).
pub(crate) enum StorageArrayBound {
    DynamicStateVar {
        state_index: usize,
    },
    MappingOfDynamicArray {
        state_index: usize,
        head_key_types: Vec<ValueType>,
        head_key_expr_indices: Vec<usize>,
    },
    FixedSizeKnown(u64),
}

/// Task #199 — Determine whether the terminal subscript of a resolved
/// `MappingAccess` traverses an Array (so a Panic(0x32) bounds guard is
/// appropriate) or a Mapping (no bounds concept). Returns
/// `Some(bound)` when a bounds guard is required, `None` otherwise.
///
/// Clamp `local` against a constant or dynamic bound. Used by the three
/// slice-clamp sites in `lower_array_slice_expression` that all share the
/// same "branch on FALSE (bad case), then jump over the assignment" shape:
///
/// ```text
/// LoadLocal(local)
/// <bound>            // PushLiteral(0) for lower-zero, LoadLocal(bound) for upper
/// <cmp>              // Ge for lower, Le for upper
/// JumpIf(do_assign)  // JMPIFNOT: jumps on FALSE = the bad case
/// Jump(done)         // cond is TRUE → no clamp needed → skip the store
/// Label(do_assign)
/// <bound>            // re-push the same bound
/// StoreLocal(local)
/// Label(done)
/// ```
///
/// `ClampKind::LowerZero` clamps `local` to `>= 0`; `ClampKind::Upper`
/// clamps it to `<= bound_local`. The bound is re-pushed inside the
/// `do_assign` arm because the comparison already consumed it.
fn clamp_local(
    local: usize,
    kind: ClampKind,
    ctx: &mut LoweringContext,
    ins: &mut Vec<Instruction>,
) {
    let do_assign = ctx.next_label();
    let done = ctx.next_label();
    ins.push(Instruction::LoadLocal(local));
    match kind {
        ClampKind::LowerZero => {
            ins.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::zero(),
            )));
            ins.push(Instruction::BinaryOp(BinaryOperator::Ge));
        }
        ClampKind::Upper(bound_local) => {
            ins.push(Instruction::LoadLocal(bound_local));
            ins.push(Instruction::BinaryOp(BinaryOperator::Le));
        }
    }
    ins.push(Instruction::JumpIf { target: do_assign });
    ins.push(Instruction::Jump { target: done });
    ins.push(Instruction::Label(do_assign));
    match kind {
        ClampKind::LowerZero => {
            ins.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::zero(),
            )));
        }
        ClampKind::Upper(bound_local) => {
            ins.push(Instruction::LoadLocal(bound_local));
        }
    }
    ins.push(Instruction::StoreLocal(local));
    ins.push(Instruction::Label(done));
}

/// Which bound to clamp against. See `clamp_local`.
enum ClampKind {
    /// Clamp `local` to `>= 0` (the lower-zero case for slice start / length).
    LowerZero,
    /// Clamp `local` to `<= bound_local` (the upper case for slice end).
    Upper(usize),
}

/// Walks the state variable's type chain once per key to mirror what
/// `resolve_mapping_access` did, stopping one step short of the final
/// key to inspect the type the last subscript operates on. Also
/// disambiguates dynamic (`T[]`) from fixed-size (`T[N]`) arrays via
/// the state variable's source-type string: the IR's `ValueType::Array`
/// collapses the two into one variant, but fixed-size arrays do NOT
/// store a length at `LoadState` so we need the compile-time `N`.
pub(crate) fn mapping_terminal_hits_array(
    mapping: &MappingAccess<'_>,
    ctx: &LoweringContext,
) -> Option<StorageArrayBound> {
    let mut current = ctx.state_type(mapping.state_index)?.clone();
    let mut head_key_types: Vec<ValueType> = Vec::new();
    let mut head_key_expr_indices: Vec<usize> = Vec::new();
    let total_keys = mapping.key_expressions.len();
    if total_keys == 0 {
        return None;
    }
    // Parse the state variable's source type so we can distinguish a
    // direct dynamic `T[]` from a fixed-size `T[N]`. This string
    // preserves the original Solidity declaration (`uint256[3]`,
    // `mapping(address=>uint256[])[3]`, etc) — the IR-level ValueType
    // collapses both forms to `Array(uint256)`.
    let state_ty = ctx
        .state_metadata(mapping.state_index)
        .map(|m| m.ty.as_str())
        .unwrap_or("");
    for step in 0..total_keys {
        let is_terminal = step + 1 == total_keys;
        match &current {
            ValueType::Array(element) => {
                if is_terminal {
                    // Determine if this step's array is fixed-size by
                    // peeling brackets from the state type string. The
                    // depth is (step + 1) from the outermost type, but
                    // Solidity's array-type-string grammar stacks
                    // brackets right-to-left. We only need the
                    // outermost bracket at `step` depth-from-the-end
                    // for fixed-size detection of THIS access level.
                    if let Some(fixed_n) = extract_fixed_array_bound_at_depth(state_ty, step) {
                        return Some(StorageArrayBound::FixedSizeKnown(fixed_n));
                    }
                    if head_key_types.is_empty() {
                        return Some(StorageArrayBound::DynamicStateVar {
                            state_index: mapping.state_index,
                        });
                    }
                    return Some(StorageArrayBound::MappingOfDynamicArray {
                        state_index: mapping.state_index,
                        head_key_types,
                        head_key_expr_indices,
                    });
                }
                // Non-terminal Array step: we skip bounds guarding the
                // intermediate array layer (rare `T[][]` storage shape).
                // `emit_load_mapping`/`emit_load_struct_array_element`
                // collapse these into a flat keccak-chain key derivation,
                // so a missing guard just falls back to the legacy
                // no-guard behaviour for non-terminal steps.
                current = (**element).clone();
            }
            ValueType::Mapping { key: _, value } => {
                if is_terminal {
                    // Terminal step lands on a Mapping — no bounds concept.
                    return None;
                }
                if let Some(key_type) = mapping.key_types.get(step).cloned() {
                    head_key_types.push(key_type);
                    head_key_expr_indices.push(step);
                } else {
                    return None;
                }
                current = (**value).clone();
            }
            _ => return None,
        }
    }
    None
}

/// Task #199 — Extract the numeric bound `N` from a fixed-size array
/// layer in a Solidity type string, or return `None` for a dynamic
/// `T[]` (or absent) layer at that depth.
///
/// Solidity's type string stacks brackets on the right, e.g.:
///   - `uint256[]`     → outermost is dynamic
///   - `uint256[3]`    → outermost is fixed-size, bound = 3
///   - `uint256[3][]`  → outermost dynamic, inner fixed-size bound = 3
///   - `mapping(address => uint256[])[3]` → outermost fixed-size,
///     inner (the mapping) has no array layer
///
/// `depth = 0` asks about the OUTERMOST array layer (the one touched
/// by the first/leftmost subscript `a[…]`), and we walk inward by
/// stripping a trailing bracket group per step. A layer is dynamic
/// (`[]`) → returns `None`; fixed-size (`[N]`) → returns `Some(N)`.
pub(crate) fn extract_fixed_array_bound_at_depth(ty: &str, _depth: usize) -> Option<u64> {
    // For the types we need to support in the QQQ5 / MMM3 / MM2 / TT2 /
    // PPP2 set (single-level arrays at the state-var root, plus
    // `mapping(K=>T[])`), only the outermost array layer matters for
    // the terminal subscript — which is exactly the trailing `[...]`
    // suffix on the state variable's type string.
    //
    // We walk the string right-to-left, trimming whitespace, and look
    // at the trailing `[…]`:
    //   - `…[]`   → dynamic, return None.
    //   - `…[N]`  → fixed, return Some(N) (where N is all digits).
    //   - anything else → no array layer here, return None.
    let trimmed = ty.trim_end();
    if !trimmed.ends_with(']') {
        return None;
    }
    let open = trimmed.rfind('[')?;
    let inner = &trimmed[open + 1..trimmed.len() - 1];
    if inner.is_empty() {
        return None;
    }
    inner.parse::<u64>().ok()
}

/// Task #199 — Emit a Panic(0x32) bounds guard for a storage-backed
/// array subscript, then delegate to `emit_storage_load` for the
/// actual element read. This is the storage analogue of Task #107's
/// in-memory guard (see the `else if` branch in
/// `lower_array_subscript_expression`):
///
///   1. Evaluate the index expression and stash in a local so both
///      the guard and the subsequent `emit_storage_load` re-evaluation
///      observe the same value (and to avoid duplicate side effects
///      for expression-valued indices).
///   2. Guard 1: `index < 0 → Panic(0x32)`.
///   3. Guard 2: `index >= length → Panic(0x32)`, where `length` is
///      loaded per the `StorageArrayBound` variant.
///   4. Fall through to `emit_storage_load` with the original
///      `MappingAccess → StorageReference`, which handles struct
///      elements (via `LoadStructField` loops), struct-array
///      elements, plain mapping reads, etc.
///
/// The Panic payload is the canonical EVM envelope
/// (`keccak256("Panic(uint256)")[..4] || abi.encode(0x32)`), which
/// cross-contract dispatch propagates through the runtime's
/// `dispatch_exception` (see
/// `src/runtime/execution/instruction/flow/try_frames.rs`) into the
/// caller's `catch Panic(uint c)` arm.
pub(crate) fn emit_storage_array_subscript_with_bounds(
    mapping: &MappingAccess<'_>,
    bound: StorageArrayBound,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    let tmp_id = ctx.next_label();

    // Guard 1 + 2 operate on `index`, which we want to observe exactly
    // once for side-effect purity. However we can't easily substitute
    // a pre-evaluated local back into the `MappingAccess` expression
    // tree without allocating a lot of IR-level machinery, so we
    // accept that simple indices (literal numbers, local variable
    // loads, arithmetic on same) re-evaluate twice — which is exactly
    // the same compromise the memory-array branch in
    // `lower_array_subscript_expression` makes (it stores `idx_local`
    // once then re-loads it for ArrayGet; here we evaluate the AST
    // expression twice, but the IR optimiser has no less information
    // than before).

    // Evaluate the final index once and stash for the bounds check.
    let last_index_expr = match mapping.key_expressions.last() {
        Some(expr) => *expr,
        None => return false,
    };
    let idx_local = ctx.allocate_local(
        format!("__storage_aidx_{tmp_id}"),
        Some(ValueType::Integer {
            signed: false,
            bits: 256,
        }),
    );
    if !lower_expression(last_index_expr, ctx, instructions) {
        return false;
    }
    instructions.push(Instruction::StoreLocal(idx_local));

    // Guard 1: index < 0 → Panic(0x32). Solidity `uint` indices cannot
    // be negative, but the runtime integer stack item is signed —
    // defensive parity with the memory-array guard above.
    let after_neg_label = ctx.next_label();
    instructions.push(Instruction::LoadLocal(idx_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Lt));
    instructions.push(Instruction::JumpIf {
        target: after_neg_label,
    });
    emit_panic(0x32, instructions);
    instructions.push(Instruction::Label(after_neg_label));

    // Guard 2: index >= length → Panic(0x32). Load length per the
    // bound variant.
    let ok_label = ctx.next_label();
    instructions.push(Instruction::LoadLocal(idx_local));
    match &bound {
        StorageArrayBound::DynamicStateVar { state_index } => {
            // Length of a state-var `T[]` is stored at
            // `LoadState(state_index)` (see `emit_load_state` +
            // `uint[] a; a.length` in `address_ops.rs` which uses the
            // same lookup). `emit_coerce_storage_value` coerces the
            // raw ByteString to Integer via `value + 0`.
            instructions.push(Instruction::LoadState(*state_index));
        }
        StorageArrayBound::MappingOfDynamicArray {
            state_index,
            head_key_types,
            head_key_expr_indices,
        } => {
            // Evaluate outer keys (all but the last) and fire a
            // `LoadMappingElement` against the head slot — the same
            // slot that `push`/`pop` on `m[k]` updates via
            // `lower_storage_reference_push`.
            for expr_idx in head_key_expr_indices {
                if let Some(expr) = mapping.key_expressions.get(*expr_idx) {
                    if !lower_expression(expr, ctx, instructions) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            instructions.push(Instruction::LoadMappingElement {
                state_index: *state_index,
                key_types: head_key_types.clone(),
            });
        }
        StorageArrayBound::FixedSizeKnown(n) => {
            // Compile-time constant bound — fixed-size arrays (`T[N]`)
            // do NOT write a length slot (writes go directly to the
            // per-index slot via `StoreMappingElement`), so the
            // runtime length cannot be derived from storage. Emit the
            // literal `N`.
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::from(*n),
            )));
        }
    }
    instructions.push(Instruction::BinaryOp(BinaryOperator::Ge));
    instructions.push(Instruction::JumpIf { target: ok_label });
    emit_panic(0x32, instructions);
    instructions.push(Instruction::Label(ok_label));

    // Bounds check passed — delegate to the canonical storage-load
    // path so struct-element arrays (`LoadStructField` loops), flat
    // mapping reads, nested mapping-in-struct paths, etc. all go
    // through the SAME machinery that was in use before Task #199.
    // This is the cleanest way to avoid duplicating the element-type
    // routing inside this helper.
    let reference = mapping.to_storage_reference();
    emit_storage_load(&reference, ctx, instructions)
}

/// Storage-soundness fix — Panic(0x32) bounds guard for array subscripts
/// that reach storage through a struct field: `s.arr[i]`, `m[k].arr[i]`,
/// `s.inner.arr[i]`. These shapes bypass `resolve_mapping_access` (it bails
/// on any MemberAccess hop), so Task #199's guard never fired for them and
/// reads went straight to `System.Storage.Get` — out-of-range indices
/// silently returned zero defaults and `delete s` data (length field slot
/// zeroed, element slots intact) stayed fully readable.
///
/// `array` is the collection expression of the subscript (`s.arr`), `index`
/// is the subscript expression. Returns:
///   * `true` after emitting the guard when the shape matches a struct-field
///     array subscript (the array length comes from the same
///     `LoadStructField` slot that `s.arr.length` / `push` / `pop` use, or
///     the compile-time `N` for a fixed-size `T[N]` field which maintains no
///     length slot);
///   * `true` WITHOUT emitting anything for non-matching shapes (mapping
///     fields, `bytes`/`string` fields, nested `T[][]` chains, storage
///     pointer aliases) so those keep their pre-fix behaviour;
///   * `false` only when lowering the index expression fails.
///
/// The index expression is evaluated once here for the guard and again
/// inside `emit_storage_load`'s trailing-key evaluation — the same
/// double-evaluation compromise documented in
/// `emit_storage_array_subscript_with_bounds` above.
pub(crate) fn emit_struct_field_array_bounds_guard(
    array: &Expression,
    index: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    // Shape: the collection must be a direct struct-field access.
    let Expression::MemberAccess(_, struct_expr, field_ident) = array else {
        return true;
    };
    let Some(collection) = resolve_storage_reference(array, ctx) else {
        return true;
    };
    // The guard only applies when the terminal subscript traverses an Array
    // (`bytes`/`string` fields are ValueType::ByteArray and keep byte-index
    // semantics; Mapping fields have no bounds concept) reached through a
    // struct field whose own subscript chain is empty.
    if !matches!(collection.value_type, ValueType::Array(_))
        || collection.field_path.is_empty()
        || !collection.trailing_key_expressions.is_empty()
    {
        return true;
    }
    // Fixed-size `T[N]` struct fields maintain no length slot — a naive
    // length load would read 0 and panic on every in-range access. Look up
    // the declared bound via the owning struct's metadata; dynamic fields
    // load the runtime length from the field slot instead.
    let struct_name = match resolve_storage_reference(struct_expr, ctx)
        .map(|base| base.value_type)
        .or_else(|| infer_type_from_expression(struct_expr, ctx))
    {
        Some(ValueType::Struct { name, .. }) => name,
        _ => return true,
    };
    let fixed_bound = ctx.struct_fixed_array_bound(&struct_name, &field_ident.name);

    let tmp_id = ctx.next_label();
    let idx_local = ctx.allocate_local(
        format!("__struct_aidx_{tmp_id}"),
        Some(ValueType::Integer {
            signed: false,
            bits: 256,
        }),
    );
    if !lower_expression(index, ctx, instructions) {
        return false;
    }
    instructions.push(Instruction::StoreLocal(idx_local));

    // Guard 1: index < 0 → Panic(0x32).
    let after_neg_label = ctx.next_label();
    instructions.push(Instruction::LoadLocal(idx_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Lt));
    instructions.push(Instruction::JumpIf {
        target: after_neg_label,
    });
    emit_panic(0x32, instructions);
    instructions.push(Instruction::Label(after_neg_label));

    // Guard 2: index >= length → Panic(0x32).
    let ok_label = ctx.next_label();
    instructions.push(Instruction::LoadLocal(idx_local));
    match fixed_bound {
        Some(bound) => {
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::from(bound),
            )));
        }
        None => {
            // Dynamic field — the length lives at the struct-field slot,
            // loaded through the same machinery `s.arr.length` uses (see
            // the Task #161 branch in `try_lower_length_property`).
            if !emit_storage_load(&collection, ctx, instructions) {
                return false;
            }
        }
    }
    instructions.push(Instruction::BinaryOp(BinaryOperator::Ge));
    instructions.push(Instruction::JumpIf { target: ok_label });
    emit_panic(0x32, instructions);
    instructions.push(Instruction::Label(ok_label));
    true
}

pub(crate) fn lower_array_slice_expression(
    array: &Expression,
    start: Option<&Expression>,
    end: Option<&Expression>,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    // Task #95: Solidity `bytes` / `bytes memory` / `bytes calldata` slicing must
    // produce a contiguous ByteString, not an Array-of-bytes. Route bytes-typed
    // slices to a SUBSTR-based path; keep the element-wise Array copy path for
    // `T[]` dynamic arrays (where Solidity semantics require a new Array value).
    if is_bytes_slice_target(array, ctx) {
        return lower_bytes_slice_expression(array, start, end, ctx, instructions);
    }

    let array_local = ctx.allocate_local("__slice_array".to_string(), None);
    if !lower_expression(array, ctx, instructions) {
        return false;
    }
    instructions.push(Instruction::StoreLocal(array_local));

    let start_local = ctx.allocate_local("__slice_start".to_string(), None);
    if let Some(start_expr) = start {
        if !lower_expression(start_expr, ctx, instructions) {
            return false;
        }
    } else {
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::zero(),
        )));
    }
    instructions.push(Instruction::StoreLocal(start_local));

    let end_local = ctx.allocate_local("__slice_end".to_string(), None);
    if let Some(end_expr) = end {
        if !lower_expression(end_expr, ctx, instructions) {
            return false;
        }
    } else {
        instructions.push(Instruction::LoadLocal(array_local));
        instructions.push(Instruction::GetSize);
    }
    instructions.push(Instruction::StoreLocal(end_local));

    // Clamp start to >= 0
    clamp_local(start_local, ClampKind::LowerZero, ctx, instructions);

    // Clamp end to array length
    let size_local = ctx.allocate_local("__slice_size".to_string(), None);
    instructions.push(Instruction::LoadLocal(array_local));
    instructions.push(Instruction::GetSize);
    instructions.push(Instruction::StoreLocal(size_local));

    clamp_local(end_local, ClampKind::Upper(size_local), ctx, instructions);

    let len_local = ctx.allocate_local("__slice_len".to_string(), None);
    instructions.push(Instruction::LoadLocal(end_local));
    instructions.push(Instruction::LoadLocal(start_local));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Sub));
    instructions.push(Instruction::StoreLocal(len_local));

    clamp_local(len_local, ClampKind::LowerZero, ctx, instructions);

    let element_type = infer_array_element_type(array, ctx).unwrap_or(ValueType::Any);
    let slice_array_type = ValueType::Array(Box::new(element_type.clone()));
    let out_local = ctx.allocate_local("__slice_out".to_string(), Some(slice_array_type));
    instructions.push(Instruction::LoadLocal(len_local));
    instructions.push(Instruction::NewArray { element_type });
    instructions.push(Instruction::StoreLocal(out_local));

    let idx_local = ctx.allocate_local("__slice_index".to_string(), None);
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::StoreLocal(idx_local));

    let loop_label = ctx.next_label();
    let end_label = ctx.next_label();

    instructions.push(Instruction::Label(loop_label));
    instructions.push(Instruction::LoadLocal(idx_local));
    instructions.push(Instruction::LoadLocal(len_local));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Lt));
    instructions.push(Instruction::JumpIf { target: end_label });

    instructions.push(Instruction::LoadLocal(out_local));
    instructions.push(Instruction::LoadLocal(idx_local));
    instructions.push(Instruction::LoadLocal(array_local));
    instructions.push(Instruction::LoadLocal(start_local));
    instructions.push(Instruction::LoadLocal(idx_local));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Add));
    instructions.push(Instruction::ArrayGet);
    instructions.push(Instruction::ArraySet);

    instructions.push(Instruction::LoadLocal(idx_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(1u8),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Add));
    instructions.push(Instruction::StoreLocal(idx_local));

    instructions.push(Instruction::Jump { target: loop_label });
    instructions.push(Instruction::Label(end_label));
    instructions.push(Instruction::LoadLocal(out_local));
    true
}

/// Task #95: Returns true when the sliced expression's IR type is a
/// Solidity `bytes` (`ValueType::ByteArray`) — a contiguous byte-string —
/// rather than a `T[]` dynamic array (`ValueType::Array(_)`). Covers
/// `bytes memory`, `bytes calldata`, `bytes storage`, and byte-string
/// literals such as `hex"..."`.
pub(crate) fn is_bytes_slice_target(array: &Expression, ctx: &LoweringContext) -> bool {
    matches!(
        infer_type_from_expression(array, ctx),
        Some(ValueType::ByteArray { .. })
    )
}

/// Task #95: Lower `bytes b[start:end]` to a contiguous ByteString using
/// NeoVM SUBSTR (opcode 0x8C). SUBSTR stack order (bottom -> top):
/// `[bytes, index, count]` → `[bytes.sub(index, count)]`.
///
/// Both `start` and `end` are clamped to `[0, len(bytes)]` and `end` is
/// further clamped to `>= start` so out-of-range slices yield an empty
/// ByteString instead of trapping — matches Solidity's saturating
/// semantics on `bytes` views and avoids a NeoVM "SUBSTR: out of bounds"
/// fault for degenerate ranges.
pub(crate) fn lower_bytes_slice_expression(
    array: &Expression,
    start: Option<&Expression>,
    end: Option<&Expression>,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    // Evaluate the source bytes once and stash in a local — we need the
    // length both for end-clamping and for the final SUBSTR push.
    let bytes_local = ctx.allocate_local(
        "__bytes_slice_src".to_string(),
        Some(ValueType::ByteArray { fixed_len: None }),
    );
    if !lower_expression(array, ctx, instructions) {
        return false;
    }
    instructions.push(Instruction::StoreLocal(bytes_local));

    // len = SIZE(bytes)
    let size_local = ctx.allocate_local("__bytes_slice_size".to_string(), None);
    instructions.push(Instruction::LoadLocal(bytes_local));
    instructions.push(Instruction::GetSize);
    instructions.push(Instruction::StoreLocal(size_local));

    // Evaluate `start` (default 0) and clamp to [0, len].
    let start_local = ctx.allocate_local("__bytes_slice_start".to_string(), None);
    if let Some(start_expr) = start {
        if !lower_expression(start_expr, ctx, instructions) {
            return false;
        }
    } else {
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::zero(),
        )));
    }
    instructions.push(Instruction::StoreLocal(start_local));

    // NOTE on branch shape: IR `JumpIf { target }` jumps when the cond on
    // the stack is FALSE (lowers to JMPIFNOT_L). So the pattern
    //     push <cond>; JumpIf clamp; Jump done; Label clamp; <fix-up>; Label done
    // runs <fix-up> iff <cond> is false.

    // start < 0 → start = 0
    let start_clamp_lo = ctx.next_label();
    let start_clamp_lo_done = ctx.next_label();
    instructions.push(Instruction::LoadLocal(start_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Ge));
    instructions.push(Instruction::JumpIf {
        target: start_clamp_lo,
    });
    instructions.push(Instruction::Jump {
        target: start_clamp_lo_done,
    });
    instructions.push(Instruction::Label(start_clamp_lo));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::StoreLocal(start_local));
    instructions.push(Instruction::Label(start_clamp_lo_done));

    // start > len → start = len
    let start_clamp_hi = ctx.next_label();
    let start_clamp_hi_done = ctx.next_label();
    instructions.push(Instruction::LoadLocal(start_local));
    instructions.push(Instruction::LoadLocal(size_local));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Le));
    instructions.push(Instruction::JumpIf {
        target: start_clamp_hi,
    });
    instructions.push(Instruction::Jump {
        target: start_clamp_hi_done,
    });
    instructions.push(Instruction::Label(start_clamp_hi));
    instructions.push(Instruction::LoadLocal(size_local));
    instructions.push(Instruction::StoreLocal(start_local));
    instructions.push(Instruction::Label(start_clamp_hi_done));

    // Evaluate `end` (default = len) and clamp to [start, len].
    let end_local = ctx.allocate_local("__bytes_slice_end".to_string(), None);
    if let Some(end_expr) = end {
        if !lower_expression(end_expr, ctx, instructions) {
            return false;
        }
    } else {
        instructions.push(Instruction::LoadLocal(size_local));
    }
    instructions.push(Instruction::StoreLocal(end_local));

    // end > len → end = len
    let end_clamp_hi = ctx.next_label();
    let end_clamp_hi_done = ctx.next_label();
    instructions.push(Instruction::LoadLocal(end_local));
    instructions.push(Instruction::LoadLocal(size_local));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Le));
    instructions.push(Instruction::JumpIf {
        target: end_clamp_hi,
    });
    instructions.push(Instruction::Jump {
        target: end_clamp_hi_done,
    });
    instructions.push(Instruction::Label(end_clamp_hi));
    instructions.push(Instruction::LoadLocal(size_local));
    instructions.push(Instruction::StoreLocal(end_local));
    instructions.push(Instruction::Label(end_clamp_hi_done));

    // end < start → end = start (produces a zero-length slice rather than
    // a negative count that would trap SUBSTR).
    let end_clamp_lo = ctx.next_label();
    let end_clamp_lo_done = ctx.next_label();
    instructions.push(Instruction::LoadLocal(end_local));
    instructions.push(Instruction::LoadLocal(start_local));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Ge));
    instructions.push(Instruction::JumpIf {
        target: end_clamp_lo,
    });
    instructions.push(Instruction::Jump {
        target: end_clamp_lo_done,
    });
    instructions.push(Instruction::Label(end_clamp_lo));
    instructions.push(Instruction::LoadLocal(start_local));
    instructions.push(Instruction::StoreLocal(end_local));
    instructions.push(Instruction::Label(end_clamp_lo_done));

    // SUBSTR expects [bytes, index, count] on the stack.
    instructions.push(Instruction::LoadLocal(bytes_local));
    instructions.push(Instruction::LoadLocal(start_local));
    instructions.push(Instruction::LoadLocal(end_local));
    instructions.push(Instruction::LoadLocal(start_local));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Sub));
    instructions.push(Instruction::Substr);
    // SUBSTR yields a Buffer on NeoVM; coerce to ByteString so equality /
    // ABI-return layers see a canonical contiguous bytes value.
    instructions.push(Instruction::Convert {
        target: ConvertTarget::ByteArray,
    });
    true
}

pub(crate) fn lower_array_literal_expression(
    elements: &[Expression],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    let element_type = infer_literal_array_element_type(elements);
    let array_local = ctx.allocate_local(
        "__array_literal".to_string(),
        Some(ValueType::Array(Box::new(element_type.clone()))),
    );
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(elements.len()),
    )));
    instructions.push(Instruction::NewArray { element_type });
    instructions.push(Instruction::StoreLocal(array_local));

    for (index, element) in elements.iter().enumerate() {
        instructions.push(Instruction::LoadLocal(array_local));
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::from(index as u64),
        )));
        if !lower_expression(element, ctx, instructions) {
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::zero(),
            )));
        }
        instructions.push(Instruction::ArraySet);
    }

    instructions.push(Instruction::LoadLocal(array_local));
    true
}
