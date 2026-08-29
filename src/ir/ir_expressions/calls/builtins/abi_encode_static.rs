//! ## ABI Encoding — Static Slots & Static Arrays
//!
//! Helpers extracted from `abi_encode.rs` to keep the orchestration module
//! under the 800-line limit. This module covers:
//!
//! * **Static slot emission** — 32-byte BE-padded slots for integers,
//!   addresses, bools, and fixed `bytesN`.
//! * **Struct flattening** — all-static structs expand to one slot per field.
//! * **Static array buffers** — in-place encoding of static-element arrays,
//!   used by both `abi.encode()` (length-prefixed) and `abi.encodePacked()`
//!   (raw concatenation).
//!
//! Functions are `pub(crate)` and re-exported from `builtins` so the main
//! `abi_encode` module and external callers can access them through a single
//! namespace.

use super::*;

pub(crate) fn lower_static_abi_slots_for_expr(
    expr: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
    flatten_struct_params: bool,
) -> Option<usize> {
    let value_type = infer_type_from_expression(expr, ctx)?;
    match value_type {
        ValueType::Struct { fields, .. } => {
            if fields.is_empty()
                || !fields
                    .iter()
                    .all(|field| is_static_abi_type_value(&field.ty))
            {
                return None;
            }

            if flatten_struct_params {
                if let Some((param_slot, field_count)) = resolve_struct_param_flat_slots(expr, ctx)
                {
                    if field_count != fields.len() {
                        return None;
                    }
                    // The struct param arrives as a single `Array` (one arg
                    // slot); read field `index` by indexing into it (PICKITEM).
                    for (index, field) in fields.iter().enumerate() {
                        instructions.push(Instruction::LoadParameter(param_slot));
                        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                            BigInt::from(index),
                        )));
                        instructions.push(Instruction::ArrayGet);
                        emit_expr_static_abi_slot_for_value_type(&field.ty, ctx, instructions)?;
                    }
                    return Some(fields.len());
                }
            }

            let tmp_id = ctx.next_label();
            let struct_local = ctx.allocate_local(format!("__abi_static_struct_{tmp_id}"), None);
            if !lower_expression(expr, ctx, instructions) {
                return Some(0);
            }
            instructions.push(Instruction::StoreLocal(struct_local));

            for (index, field) in fields.iter().enumerate() {
                instructions.push(Instruction::LoadLocal(struct_local));
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                    BigInt::from(index as u64),
                )));
                instructions.push(Instruction::ArrayGet);
                emit_expr_static_abi_slot_for_value_type(&field.ty, ctx, instructions)?;
            }
            Some(fields.len())
        }
        other if is_static_abi_type_value(&other) => {
            // A `bytesN` argument that is INTEGER-backed (a hex literal or a
            // named `bytesN` constant) is pushed little-endian / pre-reversed,
            // which the slot encoder would emit byte-reversed (N==32) or fault on
            // (N<32 via `GetSize`). Resolve it to its big-endian bytes and emit a
            // left-aligned 32-byte slot directly. ByteArray-backed values
            // (keccak/cast/storage/param) keep the normal (correct) path.
            if let ValueType::ByteArray { fixed_len: Some(n) } = other {
                if is_integer_backed_bytesn_operand(expr, ctx) {
                    if let Some(be) = fixed_len_bytes_be_from_hex_or_const(expr, n, ctx) {
                        let mut slot = be; // exactly `n` big-endian bytes
                        slot.resize(32, 0); // left-aligned, zero-padded to the 32-byte slot
                        instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(slot)));
                        return Some(1);
                    }
                }
            }
            if !lower_expression(expr, ctx, instructions) {
                return Some(0);
            }
            emit_expr_static_abi_slot_for_value_type(&other, ctx, instructions)?;
            Some(1)
        }
        _ => None,
    }
}

pub(crate) fn emit_expr_static_abi_slot_for_value_type(
    value_type: &ValueType,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<()> {
    match value_type {
        // Bug #23: signed integers must be sign-extended (0xff..) when negative,
        // not zero-padded. Route signed-true through the sign-aware buffer.
        ValueType::Integer { signed: true, .. } => {
            instructions.push(Instruction::Convert {
                target: ConvertTarget::ByteArray,
            });
            emit_abi_fixed_buffer_signed(ctx, instructions, 32);
            Some(())
        }
        ValueType::Integer { signed: false, .. } | ValueType::Boolean | ValueType::Address => {
            instructions.push(Instruction::Convert {
                target: ConvertTarget::ByteArray,
            });
            emit_abi_fixed_buffer(ctx, instructions, 32, true);
            Some(())
        }
        ValueType::ByteArray {
            fixed_len: Some(len),
        } if *len == 32 => Some(()),
        ValueType::ByteArray {
            fixed_len: Some(len),
        } if *len < 32 => {
            // This function operates on an already-lowered stack VALUE, so it
            // cannot see the operand's backing. It relies on every `bytesN`
            // value reaching it being ByteArray-backed (big-endian): that holds
            // for keccak/sha output, `bytesN(..)` casts, storage, and params,
            // and ALSO for hex literals/`bytesN` constants because those are
            // canonicalized to a big-endian ByteArray at their BINDING site
            // (`try_lower_bytesn_literal_canonical`: variable declaration,
            // assignment, struct field, array element). A raw integer-backed
            // `bytesN` literal passed DIRECTLY (e.g. a top-level `abi.encode`
            // arg) is intercepted before this point by the integer-backed
            // special-case in `lower_static_abi_slots_for_expr`.
            emit_abi_bytesn_slot(ctx, instructions, *len as usize);
            Some(())
        }
        _ => None,
    }
}

pub(crate) fn emit_abi_static_slots_from_local(
    local: usize,
    value_type: &ValueType,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<usize> {
    match value_type {
        ValueType::Struct { fields, .. }
            if !fields.is_empty()
                && fields
                    .iter()
                    .all(|field| is_static_abi_type_value(&field.ty)) =>
        {
            for (index, field) in fields.iter().enumerate() {
                instructions.push(Instruction::LoadLocal(local));
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                    BigInt::from(index as u64),
                )));
                instructions.push(Instruction::ArrayGet);
                emit_expr_static_abi_slot_for_value_type(&field.ty, ctx, instructions)?;
            }
            Some(fields.len())
        }
        other if is_static_abi_type_value(other) => {
            instructions.push(Instruction::LoadLocal(local));
            emit_expr_static_abi_slot_for_value_type(other, ctx, instructions)?;
            Some(1)
        }
        _ => None,
    }
}

pub(crate) fn emit_abi_static_slots_for_stack_value(
    value_type: &ValueType,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<usize> {
    match value_type {
        ValueType::Struct { fields, .. }
            if !fields.is_empty()
                && fields
                    .iter()
                    .all(|field| is_static_abi_type_value(&field.ty)) =>
        {
            let tmp_id = ctx.next_label();
            let struct_local = ctx.allocate_local(format!("__abi_stack_struct_{tmp_id}"), None);
            instructions.push(Instruction::StoreLocal(struct_local));
            for (index, field) in fields.iter().enumerate() {
                instructions.push(Instruction::LoadLocal(struct_local));
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                    BigInt::from(index as u64),
                )));
                instructions.push(Instruction::ArrayGet);
                emit_expr_static_abi_slot_for_value_type(&field.ty, ctx, instructions)?;
            }
            Some(fields.len())
        }
        other if is_static_abi_type_value(other) => {
            emit_expr_static_abi_slot_for_value_type(other, ctx, instructions)?;
            Some(1)
        }
        _ => None,
    }
}

pub(crate) fn emit_abi_packed_static_array(
    element_type: &ValueType,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<()> {
    emit_abi_static_array_buffer(element_type, ctx, instructions, false)
}

pub(crate) fn emit_abi_static_array_buffer(
    element_type: &ValueType,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
    include_length: bool,
) -> Option<()> {
    let element_slot_count = abi_static_slot_count(element_type)?;
    if element_slot_count == 0 {
        return None;
    }
    let element_byte_len = element_slot_count * 32;
    let tmp_id = ctx.next_label();
    let arr_local = ctx.allocate_local(format!("__abi_arr_{tmp_id}"), None);
    let len_local = ctx.allocate_local(format!("__abi_arr_len_{tmp_id}"), None);
    let out_local = ctx.allocate_local(format!("__abi_arr_out_{tmp_id}"), None);
    let idx_local = ctx.allocate_local(format!("__abi_arr_idx_{tmp_id}"), None);
    let elem_slot_local = ctx.allocate_local(format!("__abi_arr_elem_slot_{tmp_id}"), None);

    instructions.push(Instruction::StoreLocal(arr_local));
    instructions.push(Instruction::LoadLocal(arr_local));
    instructions.push(Instruction::GetSize);
    instructions.push(Instruction::StoreLocal(len_local));

    instructions.push(Instruction::LoadLocal(len_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(element_byte_len as u64),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Mul));
    if include_length {
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::from(32u8),
        )));
        instructions.push(Instruction::BinaryOp(BinaryOperator::Add));
    }
    instructions.push(Instruction::NewBuffer);
    instructions.push(Instruction::StoreLocal(out_local));

    if include_length {
        instructions.push(Instruction::LoadLocal(len_local));
        emit_abi_u256_slot(ctx, instructions);
        let len_slot_local = ctx.allocate_local(format!("__abi_arr_len_slot_{tmp_id}"), None);
        instructions.push(Instruction::StoreLocal(len_slot_local));
        instructions.push(Instruction::LoadLocal(out_local));
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::zero(),
        )));
        instructions.push(Instruction::LoadLocal(len_slot_local));
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::zero(),
        )));
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::from(32u8),
        )));
        instructions.push(Instruction::MemCpy);
    }

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

    instructions.push(Instruction::LoadLocal(arr_local));
    instructions.push(Instruction::LoadLocal(idx_local));
    instructions.push(Instruction::ArrayGet);
    let emitted_slots = emit_abi_static_slots_for_stack_value(element_type, ctx, instructions)?;
    if emitted_slots != element_slot_count {
        return None;
    }
    if emitted_slots > 1 {
        instructions.push(Instruction::CallBuiltin {
            builtin: BuiltinCall::BytesConcat,
            arg_count: emitted_slots,
        });
    }
    instructions.push(Instruction::StoreLocal(elem_slot_local));

    instructions.push(Instruction::LoadLocal(out_local));
    if include_length {
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::from(32u8),
        )));
    } else {
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::zero(),
        )));
    }
    instructions.push(Instruction::LoadLocal(idx_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(element_byte_len as u64),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Mul));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Add));
    instructions.push(Instruction::LoadLocal(elem_slot_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(element_byte_len as u64),
    )));
    instructions.push(Instruction::MemCpy);

    instructions.push(Instruction::LoadLocal(idx_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::one(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Add));
    instructions.push(Instruction::StoreLocal(idx_local));
    instructions.push(Instruction::Jump { target: loop_label });
    instructions.push(Instruction::Label(end_label));

    instructions.push(Instruction::LoadLocal(out_local));
    instructions.push(Instruction::Convert {
        target: ConvertTarget::ByteArray,
    });
    Some(())
}
