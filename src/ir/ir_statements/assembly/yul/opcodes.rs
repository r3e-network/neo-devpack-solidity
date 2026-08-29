use super::*;

/// Emit `mstore(offset, value)` — copy the 32 big-endian bytes of `value`
/// into `__yul_memory[offset .. offset+32]`.
pub(crate) fn lower_yul_mstore(
    offset_expr: &solang_parser::pt::YulExpression,
    value_expr: &solang_parser::pt::YulExpression,
    state: &mut YulLoweringState,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    let mem_local = state.ensure_memory(ctx);

    // Evaluate value first and stash it in a local so BE-encoding can work
    // without juggling offset on the stack at the same time.
    if !lower_yul_expression(value_expr, state, ctx, instructions) {
        return false;
    }
    let tmp_id = ctx.next_label();
    let value_local = ctx.allocate_local(format!("__yul_mstore_value_{tmp_id}"), None);
    instructions.push(Instruction::StoreLocal(value_local));

    // Evaluate offset into its own local (kept as a NeoVM integer).
    if !lower_yul_expression(offset_expr, state, ctx, instructions) {
        return false;
    }
    let offset_local = ctx.allocate_local(format!("__yul_mstore_offset_{tmp_id}"), None);
    instructions.push(Instruction::StoreLocal(offset_local));

    // Build the 32-byte BE encoding of `value` in a scratch buffer. Mirror
    // the well-worn `coerce_to_fixed_bytes(32, reverse=true)` recipe used by
    // bytesN(..) casts: LE bytes from Convert→ByteArray, MEMCPY into a
    // fresh 32-byte zero buffer at offset 0, then REVERSEITEMS in place.
    instructions.push(Instruction::LoadLocal(value_local));
    instructions.push(Instruction::Convert {
        target: ConvertTarget::ByteArray,
    });
    let src_local = ctx.allocate_local(format!("__yul_mstore_src_{tmp_id}"), None);
    instructions.push(Instruction::StoreLocal(src_local));

    let scratch_local = ctx.allocate_local(format!("__yul_mstore_scratch_{tmp_id}"), None);
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(32u64),
    )));
    instructions.push(Instruction::NewBuffer);
    instructions.push(Instruction::StoreLocal(scratch_local));

    // count = min(src.len(), 32)
    let size_local = ctx.allocate_local(format!("__yul_mstore_size_{tmp_id}"), None);
    let count_local = ctx.allocate_local(format!("__yul_mstore_count_{tmp_id}"), None);
    instructions.push(Instruction::LoadLocal(src_local));
    instructions.push(Instruction::GetSize);
    instructions.push(Instruction::StoreLocal(size_local));

    let ge_label = ctx.next_label();
    let end_label = ctx.next_label();
    instructions.push(Instruction::LoadLocal(size_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(32u64),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Lt));
    instructions.push(Instruction::JumpIf { target: ge_label });
    instructions.push(Instruction::LoadLocal(size_local));
    instructions.push(Instruction::StoreLocal(count_local));
    instructions.push(Instruction::Jump { target: end_label });
    instructions.push(Instruction::Label(ge_label));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(32u64),
    )));
    instructions.push(Instruction::StoreLocal(count_local));
    instructions.push(Instruction::Label(end_label));

    // Copy LE bytes into scratch[0 .. count].
    instructions.push(Instruction::LoadLocal(scratch_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::LoadLocal(src_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::LoadLocal(count_local));
    instructions.push(Instruction::MemCpy);
    // Real NeoVM MEMCPY: Pop 5, Push 0. Load scratch explicitly.
    instructions.push(Instruction::LoadLocal(scratch_local));
    instructions.push(Instruction::ReverseItems);

    // Copy scratch → __yul_memory[offset .. offset+32].
    instructions.push(Instruction::LoadLocal(mem_local));
    instructions.push(Instruction::LoadLocal(offset_local));
    instructions.push(Instruction::LoadLocal(scratch_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(32u64),
    )));
    instructions.push(Instruction::MemCpy);
    // Real NeoVM MEMCPY: Pop 5, Push 0. Nothing to discard.
    true
}

/// Emit `mload(offset)` — read 32 bytes from `__yul_memory[offset..offset+32]`,
/// decoded as a big-endian uint256.
pub(crate) fn lower_yul_mload(
    offset_expr: &solang_parser::pt::YulExpression,
    state: &mut YulLoweringState,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    let mem_local = state.ensure_memory(ctx);

    // SUBSTR wants [bytes, index, count] — push in that order.
    instructions.push(Instruction::LoadLocal(mem_local));
    if !lower_yul_expression(offset_expr, state, ctx, instructions) {
        return false;
    }
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(32u64),
    )));
    instructions.push(Instruction::Substr);

    // SUBSTR returns a ByteString. Reverse into LE and CONVERT→Integer to
    // recover the big-endian magnitude.
    instructions.push(Instruction::Convert {
        target: ConvertTarget::ByteArray,
    });
    instructions.push(Instruction::Dup);
    instructions.push(Instruction::ReverseItems);
    instructions.push(Instruction::Convert {
        target: ConvertTarget::Integer,
    });
    true
}

/// Emit `return(offset, length)` — terminate the function with the memory
/// slice as its return value. When the enclosing function's declared single
/// return type is an integer, re-interpret the slice as BE-packed uint so
/// the main-frame RET emits the expected 32-byte value; otherwise hand the
/// raw buffer to RET.
pub(crate) fn lower_yul_return(
    offset_expr: &solang_parser::pt::YulExpression,
    length_expr: &solang_parser::pt::YulExpression,
    state: &mut YulLoweringState,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    let mem_local = state.ensure_memory(ctx);

    instructions.push(Instruction::LoadLocal(mem_local));
    if !lower_yul_expression(offset_expr, state, ctx, instructions) {
        return false;
    }
    if !lower_yul_expression(length_expr, state, ctx, instructions) {
        return false;
    }
    instructions.push(Instruction::Substr);

    // Decide the canonical return shape. If the function is declared
    // `returns (uintN)` / `returns (intN)` / `returns (bool)` (single
    // scalar), re-interpret the BE-packed slice as an integer so the
    // main-frame RET emits the spec-matching 32-byte value. Otherwise
    // leave the raw buffer.
    let return_types = ctx.return_types();
    let want_integer = return_types.len() == 1
        && matches!(
            return_types[0],
            ValueType::Integer { .. } | ValueType::Boolean
        );

    if want_integer {
        // Reverse the BE slice into LE and CONVERT→Integer. Mirror the
        // `uint256(bytes32)` reinterpret recipe from type_constructors.rs
        // so short slices (< 32 bytes) still decode as zero-extended uint.
        instructions.push(Instruction::Convert {
            target: ConvertTarget::ByteArray,
        });
        instructions.push(Instruction::Dup);
        instructions.push(Instruction::ReverseItems);
        instructions.push(Instruction::Convert {
            target: ConvertTarget::Integer,
        });
    }

    instructions.push(Instruction::Return);
    true
}

/// Emit `tstore(slot, value)` — store `value` in the transient map under
/// key `slot`. The `__yul_transient` map is lazily allocated in the first
/// yul block that uses transient storage and reused (by name lookup) by
/// every subsequent yul block in the same function, which gives EIP-1153's
/// per-transaction persistence for the lifetime of this invocation.
///
/// Stack shape for NeoVM SETITEM (0xD0): bottom → top = [collection, key, value].
/// After the op, the map has been mutated in place and the stack is empty.
pub(crate) fn lower_yul_tstore(
    slot_expr: &solang_parser::pt::YulExpression,
    value_expr: &solang_parser::pt::YulExpression,
    state: &mut YulLoweringState,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    let transient_local = state.ensure_transient(ctx);

    // Push [map, slot, value] in the order SETITEM expects.
    instructions.push(Instruction::LoadLocal(transient_local));
    if !lower_yul_expression(slot_expr, state, ctx, instructions) {
        return false;
    }
    if !lower_yul_expression(value_expr, state, ctx, instructions) {
        return false;
    }
    instructions.push(Instruction::ArraySet); // SETITEM 0xD0 — also works on Maps.
    true
}

/// Emit `tload(slot)` — read the value stored under `slot` in the transient
/// map, or `0` if the slot was never tstore'd in this invocation. Uses
/// HASKEY to branch so that missing keys fall through to PUSH0 instead of
/// raising `PICKITEM: key not found`.
///
/// Generated shape (pseudocode):
///     if HAS_KEY(map, slot) { push(map_slot); } else { push(0); }
pub(crate) fn lower_yul_tload(
    slot_expr: &solang_parser::pt::YulExpression,
    state: &mut YulLoweringState,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    let transient_local = state.ensure_transient(ctx);

    // Stash the slot in a local so we can use it twice (once for HASKEY,
    // once for PICKITEM) without re-evaluating the expression.
    let tmp_id = ctx.next_label();
    let slot_local = ctx.allocate_local(format!("__yul_tload_slot_{tmp_id}"), None);
    if !lower_yul_expression(slot_expr, state, ctx, instructions) {
        return false;
    }
    instructions.push(Instruction::StoreLocal(slot_local));

    // HASKEY pops [collection, key] and pushes Boolean. NeoVM JMPIFNOT_L
    // branches on falsy, so we jump to the "push 0" arm when the key is
    // absent. IR's JumpIf has the same "jump when false" semantic
    // (see lower_yul_mstore above for the pattern).
    let missing_label = ctx.next_label();
    let end_label = ctx.next_label();

    instructions.push(Instruction::LoadLocal(transient_local));
    instructions.push(Instruction::LoadLocal(slot_local));
    instructions.push(Instruction::HasKey);
    instructions.push(Instruction::JumpIf {
        target: missing_label,
    });

    // key present → push map[slot]
    instructions.push(Instruction::LoadLocal(transient_local));
    instructions.push(Instruction::LoadLocal(slot_local));
    instructions.push(Instruction::ArrayGet); // PICKITEM 0xCE — works on Maps.
    instructions.push(Instruction::Jump { target: end_label });

    // key absent → push 0 (EIP-1153 default for unset transient slot)
    instructions.push(Instruction::Label(missing_label));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));

    instructions.push(Instruction::Label(end_label));
    true
}

/// Task #184 — emit `returndatacopy(dst, src, len)`.
///
/// EVM semantics: copy `len` bytes from the last-call returndata buffer
/// (starting at offset `src`) into yul memory at offset `dst`. If
/// `src + len > returndatasize()`, the contract MUST revert with
/// `Panic(0x32)` (read past returndatasize). When `len == 0`, the opcode
/// is a strict no-op regardless of `src`/`dst`.
///
/// Task #184 surface: the minimal harness (`batch76_zz4`) exercises the
/// "no preceding external call" case where `returndatasize() == 0`, so the
/// bounds check always fires on any non-zero-length read. We still
/// implement the full `MemCpy` path so a follow-up task can populate
/// `__yul_returndata` after external calls without revisiting this lower.
///
/// Generated shape (pseudocode):
/// ```text
///   // 1. Evaluate & stash args.
///   let dst = <dst_expr>;
///   let src = <src_expr>;
///   let len = <len_expr>;
///
///   // 2. Zero-length fast path (EVM spec: no-op even if src >= rdsize).
///   if len != 0 {
///     // 3. Bounds check: src + len > returndatasize() → revert.
///     if src + len > __yul_returndata.size() {
///       throw "returndata: read past returndatasize";
///     }
///
///     // 4. Copy.
///     MEMCPY(__yul_memory, dst, __yul_returndata, src, len);
///   }
/// ```
///
/// Diagnostic shape on the bounds-check fault: the exception message is
/// `THROW: returndata: read past returndatasize`, which contains the
/// `"returndata"` substring the `batch76_zz4` harness accepts as a clean
/// underflow marker (see `tests/fuzz_tests/batches_66_80.rs`).
pub(crate) fn lower_yul_returndatacopy(
    dst_expr: &solang_parser::pt::YulExpression,
    src_expr: &solang_parser::pt::YulExpression,
    len_expr: &solang_parser::pt::YulExpression,
    state: &mut YulLoweringState,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    let mem_local = state.ensure_memory(ctx);
    let rd_local = state.ensure_returndata(ctx);

    // Evaluate dst, src, len in that order (yul's left-to-right semantic) and
    // stash each in its own local so the bounds check and MEMCPY can reference
    // them multiple times without re-evaluating (which would double any side
    // effects in the source expressions).
    let tmp_id = ctx.next_label();

    if !lower_yul_expression(dst_expr, state, ctx, instructions) {
        return false;
    }
    let dst_local = ctx.allocate_local(format!("__yul_rdc_dst_{tmp_id}"), None);
    instructions.push(Instruction::StoreLocal(dst_local));

    if !lower_yul_expression(src_expr, state, ctx, instructions) {
        return false;
    }
    let src_local = ctx.allocate_local(format!("__yul_rdc_src_{tmp_id}"), None);
    instructions.push(Instruction::StoreLocal(src_local));

    if !lower_yul_expression(len_expr, state, ctx, instructions) {
        return false;
    }
    let len_local = ctx.allocate_local(format!("__yul_rdc_len_{tmp_id}"), None);
    instructions.push(Instruction::StoreLocal(len_local));

    let skip_label = ctx.next_label(); // target for len == 0 fast path
    let ok_label = ctx.next_label(); // target for (src+len) <= rdsize branch

    // Fast path: if `len == 0`, skip the whole copy (including the bounds
    // check — EVM spec treats zero-length returndatacopy as a no-op even
    // when returndatasize() is 0).
    //
    //   LoadLocal(len); Push(0); BinaryOp(Ne) → Boolean(len != 0)
    //   JumpIf(skip_label)   — IR JumpIf = JMPIFNOT: branches when falsy,
    //                          i.e. when (len != 0) is false, i.e. len == 0.
    instructions.push(Instruction::LoadLocal(len_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Ne));
    instructions.push(Instruction::JumpIf { target: skip_label });

    // Bounds check: compute (src + len) and compare with returndatasize.
    //
    //   Push (src + len)
    //   Push rdsize
    //   Compute (src + len) <= rdsize  ⇒  push Boolean
    //   JumpIf(ok_label) — branches when falsy, so we fall through to the
    //   "throw" arm when (src + len) > rdsize.
    instructions.push(Instruction::LoadLocal(src_local));
    instructions.push(Instruction::LoadLocal(len_local));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Add));

    instructions.push(Instruction::LoadLocal(rd_local));
    instructions.push(Instruction::GetSize);

    instructions.push(Instruction::BinaryOp(BinaryOperator::Le));
    instructions.push(Instruction::JumpIf { target: ok_label });

    // Fault arm: push a descriptive error message and THROW. The runtime's
    // `execute_flow_exceptions` UTF-8-lossies the payload into the exception
    // message with a `THROW: ` prefix, producing `THROW: returndata: read
    // past returndatasize`. The `"returndata"` substring matches the
    // `batch76_zz4` harness's `clean_underflow` acceptance set.
    instructions.push(Instruction::PushLiteral(LiteralValue::String(
        b"returndata: read past returndatasize".to_vec(),
    )));
    instructions.push(Instruction::Throw);

    // OK arm: execute the copy.
    //
    //   MemCpy stack order (bottom → top): [dst, dst_offset, src, src_offset, count]
    //   → copy src[src_offset .. src_offset+count] into dst[dst_offset .. dst_offset+count]
    //   → leaves dst on the stack; we `Drop` it because the buffer was
    //     already stored via the `__yul_memory` local.
    instructions.push(Instruction::Label(ok_label));
    instructions.push(Instruction::LoadLocal(mem_local));
    instructions.push(Instruction::LoadLocal(dst_local));
    instructions.push(Instruction::LoadLocal(rd_local));
    instructions.push(Instruction::LoadLocal(src_local));
    instructions.push(Instruction::LoadLocal(len_local));
    instructions.push(Instruction::MemCpy);
    // Real NeoVM MEMCPY: Pop 5, Push 0. Nothing to discard.

    // Skip target: zero-length fast path and end of the non-fault path meet
    // here. The `ok_label` arm falls through to `skip_label`; both paths
    // continue with the next yul statement.
    instructions.push(Instruction::Label(skip_label));
    true
}
