use super::*;

/// Task #112 — left-align a bytesN value (ByteArray of length N where
/// 1 ≤ N < 32) into a 32-byte buffer. The expected stack shape on entry is
/// `[.., src_bytes]`; on exit it is `[.., dst_bytes32]`.
///
/// Emits: allocate a fresh 32-byte NewBuffer, MEMCPY `N` bytes from `src[0..]`
/// into `dst[0..N]`, then re-push the destination and canonicalise as
/// ByteArray. The tail 32-N bytes remain zero-padded — matching
/// `abi.encode(bytesN)`'s spec (left-aligned content, zero-padded on the right).
pub(crate) fn emit_pad_bytesn_to_32(
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
    n: usize,
) {
    let tmp_id = ctx.next_label();
    let src_local = ctx.allocate_local(format!("__bytesn_pad_src_{tmp_id}"), None);
    let dst_local = ctx.allocate_local(format!("__bytesn_pad_dst_{tmp_id}"), None);

    // Store the source bytesN buffer (top of stack) into a local so we can
    // reference it twice (for MEMCPY src + the size-min computation below).
    instructions.push(Instruction::StoreLocal(src_local));

    // Allocate a zero-initialised 32-byte destination buffer.
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(32u64),
    )));
    instructions.push(Instruction::NewBuffer);
    instructions.push(Instruction::StoreLocal(dst_local));

    // Compute count = min(src.len(), n). Solidity `bytesN` guarantees the
    // source buffer has EXACTLY N bytes, but static bytesN produced by
    // `bytesN(<shorter_slice>)` can be shorter (the compiler's
    // `coerce_to_fixed_bytes` caps this at N) — defensively clamp.
    let size_local = ctx.allocate_local(format!("__bytesn_pad_size_{tmp_id}"), None);
    let count_local = ctx.allocate_local(format!("__bytesn_pad_count_{tmp_id}"), None);
    instructions.push(Instruction::LoadLocal(src_local));
    instructions.push(Instruction::GetSize);
    instructions.push(Instruction::StoreLocal(size_local));

    let ge_label = ctx.next_label();
    let end_label = ctx.next_label();
    instructions.push(Instruction::LoadLocal(size_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(n as u64),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Lt));
    instructions.push(Instruction::JumpIf { target: ge_label });
    // size >= n → count = n
    instructions.push(Instruction::LoadLocal(size_local));
    instructions.push(Instruction::StoreLocal(count_local));
    instructions.push(Instruction::Jump { target: end_label });
    instructions.push(Instruction::Label(ge_label));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(n as u64),
    )));
    instructions.push(Instruction::StoreLocal(count_local));
    instructions.push(Instruction::Label(end_label));

    // NeoVM MEMCPY stack order: [dst, dst_offset, src, src_offset, count].
    // dst_offset = 0 (left-aligned per EVM `abi.encode(bytesN)` layout).
    instructions.push(Instruction::LoadLocal(dst_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::LoadLocal(src_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::LoadLocal(count_local));
    instructions.push(Instruction::MemCpy);
    // Real NeoVM MEMCPY: Pop 5, Push 0. Load dst explicitly.
    instructions.push(Instruction::LoadLocal(dst_local));
    // Canonicalise via Convert so the downstream `abiEncode`
    // sees a stable ByteString (value equality with storage-loaded slots).
    instructions.push(Instruction::Convert {
        target: ConvertTarget::ByteArray,
    });
}

pub(crate) fn is_static_abi_slot_value_type(value_type: &ValueType) -> bool {
    matches!(
        value_type,
        ValueType::Integer { .. }
            | ValueType::Boolean
            | ValueType::Address
            | ValueType::ByteArray {
                fixed_len: Some(1..=32)
            }
    )
}

pub(crate) fn emit_static_abi_slot_for_value_type(
    value_type: &ValueType,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    match value_type {
        ValueType::Integer { signed: true, .. } => {
            // A NEGATIVE signed integer must sign-extend to the full 32-byte
            // ABI slot (EVM canonical: high bytes are 0xFF). The slot encoder
            // copies the minimal CONVERT bytes into a ZERO-filled buffer, which
            // would zero-extend a negative. Mask to 2^256 bits first so a
            // narrow negative is promoted to its canonical 32-byte
            // two's-complement, making the zero-pad encode correct.
            let mask: BigInt = (BigInt::one() << 256usize) - BigInt::one();
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(mask)));
            instructions.push(Instruction::BinaryOp(BinaryOperator::BitAnd));
            instructions.push(Instruction::Convert {
                target: ConvertTarget::ByteArray,
            });
            emit_revert_static_slot_32(ctx, instructions, true);
            true
        }
        ValueType::Integer { .. } | ValueType::Boolean | ValueType::Address => {
            instructions.push(Instruction::Convert {
                target: ConvertTarget::ByteArray,
            });
            emit_revert_static_slot_32(ctx, instructions, true);
            true
        }
        ValueType::ByteArray {
            fixed_len: Some(len),
        } if *len == 32 => true,
        ValueType::ByteArray {
            fixed_len: Some(len),
        } if *len < 32 => {
            emit_pad_bytesn_to_32(ctx, instructions, *len as usize);
            true
        }
        _ => false,
    }
}

pub(crate) fn lower_static_abi_return_expr_slot(
    expr: &Expression,
    value_type: &ValueType,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    // A bare hex literal / named constant implicitly returned as `bytesN` is
    // INTEGER-backed (pushed little-endian), so the generic slot encoder would
    // emit it RIGHT-aligned (uint-style) — but `bytesN` is LEFT-aligned.
    // Resolve it to its big-endian bytes against the DECLARED return type and
    // emit a left-aligned slot directly (mirrors the abi.encode fix).
    if let ValueType::ByteArray { fixed_len: Some(n) } = value_type {
        if is_integer_backed_bytesn_operand(expr, ctx) {
            if let Some(be) = fixed_len_bytes_be_from_hex_or_const(expr, *n, ctx) {
                let mut slot = be; // exactly `n` big-endian bytes
                slot.resize(32, 0); // left-aligned, zero-padded to the 32-byte slot
                instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(slot)));
                return true;
            }
        }
    }
    if !lower_expression(expr, ctx, instructions) {
        return false;
    }
    emit_static_abi_slot_for_value_type(value_type, ctx, instructions)
}

pub(crate) fn is_direct_static_revert_arg(expr: &Expression, ctx: &LoweringContext) -> bool {
    if resolve_struct_type_for_revert_arg(expr, ctx).is_some() {
        return false;
    }
    matches!(
        infer_type_from_expression(expr, ctx),
        Some(ValueType::Integer { signed: false, .. })
            | Some(ValueType::Boolean)
            | Some(ValueType::Address)
            | Some(ValueType::ByteArray { fixed_len: Some(_) })
    )
}

pub(crate) fn emit_revert_static_slot_32(
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
    reverse: bool,
) {
    let tmp_id = ctx.next_label();
    let src_local = ctx.allocate_local(format!("__revert_slot_src_{tmp_id}"), None);
    let dst_local = ctx.allocate_local(format!("__revert_slot_dst_{tmp_id}"), None);
    let size_local = ctx.allocate_local(format!("__revert_slot_size_{tmp_id}"), None);
    let count_local = ctx.allocate_local(format!("__revert_slot_count_{tmp_id}"), None);

    instructions.push(Instruction::StoreLocal(src_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(32u64),
    )));
    instructions.push(Instruction::NewBuffer);
    instructions.push(Instruction::StoreLocal(dst_local));

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

    instructions.push(Instruction::LoadLocal(dst_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::LoadLocal(src_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::LoadLocal(count_local));
    instructions.push(Instruction::MemCpy);
    // Real NeoVM MEMCPY: Pop 5, Push 0. Nothing to discard.

    if reverse {
        instructions.push(Instruction::LoadLocal(dst_local));
        instructions.push(Instruction::LoadLocal(dst_local));
        instructions.push(Instruction::ReverseItems);
        instructions.push(Instruction::Convert {
            target: ConvertTarget::ByteArray,
        });
    } else {
        instructions.push(Instruction::LoadLocal(dst_local));
        instructions.push(Instruction::Convert {
            target: ConvertTarget::ByteArray,
        });
    }
}

pub(crate) fn lower_direct_static_revert_arg_slot(
    expr: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    let Some(value_type) = infer_type_from_expression(expr, ctx) else {
        return false;
    };
    if !lower_expression(expr, ctx, instructions) {
        return false;
    }
    // No Swap; Drop needed: real NeoVM MEMCPY pushes nothing.

    match value_type {
        ValueType::Integer { signed: false, .. } | ValueType::Boolean | ValueType::Address => {
            instructions.push(Instruction::Convert {
                target: ConvertTarget::ByteArray,
            });
            emit_revert_static_slot_32(ctx, instructions, true);
            true
        }
        ValueType::ByteArray {
            fixed_len: Some(len),
        } => {
            if len == 32 {
                true
            } else if len < 32 {
                emit_pad_bytesn_to_32(ctx, instructions, len as usize);
                true
            } else {
                false
            }
        }
        _ => false,
    }
}
