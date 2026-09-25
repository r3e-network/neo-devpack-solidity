//! ## ABI Decoding — Dynamic Tails
//!
//! Helpers extracted from `abi_decode.rs` to keep the orchestration module
//! under the 800-line limit. This module decodes EVM-canonical dynamic tails
//! for `string`, `bytes`, dynamic arrays (static- and dynamic-element), and
//! dynamic structs at runtime-determined byte offsets.
//!
//! Functions are `pub(crate)` and re-exported from `builtins` so the main
//! `abi_decode` module can access them through a single namespace.

use super::*;

/// Decode a single top-level dynamic value from the encoded buffer in
/// `buffer_local`.
///
/// Layout produced by `abi.encode(x)` for a single dynamic `x`:
///   `BE32(0x20) || <tail at offset 0x20>`
/// i.e. a leading 32-byte head holding the offset 0x20 (= 32) of the tail,
/// followed by the tail payload (length + bytes for `string`/`bytes`, or
/// length + elements for `T[]`).
///
/// Stack on exit: `[decoded_value]` (ByteString for string/bytes, Array
/// of element values for `T[]`).
pub(crate) fn emit_abi_decode_dynamic_top_level(
    buffer_local: usize,
    value_type: &ValueType,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) {
    // Read the ACTUAL tail offset from head slot 0 rather than assuming the
    // canonical 0x20: a non-canonical (but valid) encoder may place the tail at
    // a different offset, and EVM decoders honor the encoded head pointer.
    let tmp_id = ctx.next_label();
    let offset_local = ctx.allocate_local(format!("__abi_dyn_top_off_{tmp_id}"), None);
    emit_abi_decode_u256_at(buffer_local, 0, ctx, instructions);
    instructions.push(Instruction::StoreLocal(offset_local));
    emit_abi_decode_dynamic_tail_runtime(
        buffer_local,
        offset_local,
        value_type,
        0,
        ctx,
        instructions,
    );
}

/// Decode a dynamic tuple member at head index `index`.
///
/// Reads the u256 offset from `buffer[index*32 .. index*32+32]` (which is
/// the byte offset of the member's tail relative to the start of the
/// encoded buffer) and dispatches to [`emit_abi_decode_dynamic_tail`] at
/// that runtime offset.
pub(crate) fn emit_abi_decode_dynamic_tuple_member(
    buffer_local: usize,
    index: usize,
    value_type: &ValueType,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) {
    // Read the offset slot. We can fold it through a temp local.
    let tmp_id = ctx.next_label();
    let offset_local = ctx.allocate_local(format!("__abi_dyn_off_{tmp_id}"), None);
    emit_abi_decode_u256_at(buffer_local, index * 32, ctx, instructions);
    instructions.push(Instruction::StoreLocal(offset_local));
    emit_abi_decode_dynamic_tail_runtime(
        buffer_local,
        offset_local,
        value_type,
        0,
        ctx,
        instructions,
    );
}

/// Same as [`emit_abi_decode_dynamic_tail`] but with the tail offset
/// supplied at runtime via `offset_local`. Used for tuple members where
/// the offset is read from a head slot.
pub(crate) fn emit_abi_decode_dynamic_tail_runtime(
    buffer_local: usize,
    offset_local: usize,
    value_type: &ValueType,
    depth: usize,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) {
    match value_type {
        ValueType::ByteArray { fixed_len: None } | ValueType::String => {
            emit_abi_decode_bytes_tail_runtime(buffer_local, offset_local, ctx, instructions);
        }
        ValueType::Array(element_type) if abi_static_slot_count(element_type).is_some() => {
            emit_abi_decode_static_element_array_tail_runtime(
                buffer_local,
                offset_local,
                element_type,
                ctx,
                instructions,
            );
        }
        // `T[]` with a dynamic element type (string[], bytes[], T[][]): each
        // head entry is itself a u256 offset (relative to the start of the
        // head section) pointing at that element's tail — decode recursively.
        ValueType::Array(element_type) => {
            emit_abi_decode_nested_array_tail_runtime(
                buffer_local,
                offset_local,
                element_type,
                depth,
                ctx,
                instructions,
            );
        }
        // Dynamic struct: walk the tuple head/tail at this offset.
        ValueType::Struct { fields, .. } => {
            emit_abi_decode_dynamic_struct_tail_runtime(
                buffer_local,
                offset_local,
                fields,
                depth,
                ctx,
                instructions,
            );
        }
        _ => {
            instructions.push(Instruction::PushLiteral(
                LiteralValue::ByteArray(Vec::new()),
            ));
        }
    }
}

/// Decode a `string` / `bytes` tail at a runtime byte offset.
pub(crate) fn emit_abi_decode_bytes_tail_runtime(
    buffer_local: usize,
    offset_local: usize,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) {
    let tmp_id = ctx.next_label();
    let len_local = ctx.allocate_local(format!("__abi_dec_rblen_{tmp_id}"), None);

    emit_abi_decode_u256_at_runtime(buffer_local, offset_local, ctx, instructions);
    instructions.push(Instruction::StoreLocal(len_local));

    instructions.push(Instruction::LoadLocal(buffer_local));
    // payload starts at offset_local + 32.
    instructions.push(Instruction::LoadLocal(offset_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(32u8),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Add));
    instructions.push(Instruction::LoadLocal(len_local));
    instructions.push(Instruction::Substr);
    instructions.push(Instruction::Convert {
        target: ConvertTarget::ByteArray,
    });
}

/// Decode a `T[]` tail at a runtime byte offset (tuple-member case).
pub(crate) fn emit_abi_decode_static_element_array_tail_runtime(
    buffer_local: usize,
    offset_local: usize,
    element_type: &ValueType,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) {
    let element_byte_len = abi_static_slot_count(element_type).unwrap_or(1) * 32;
    let tmp_id = ctx.next_label();
    let len_local = ctx.allocate_local(format!("__abi_dec_ralen_{tmp_id}"), None);
    let base_local = ctx.allocate_local(format!("__abi_dec_rabase_{tmp_id}"), None);
    let arr_local = ctx.allocate_local(
        format!("__abi_dec_rarr_{tmp_id}"),
        Some(ValueType::Array(Box::new(element_type.clone()))),
    );
    let idx_local = ctx.allocate_local(format!("__abi_dec_raidx_{tmp_id}"), None);
    let elem_off_local = ctx.allocate_local(format!("__abi_dec_raelmoff_{tmp_id}"), None);

    emit_abi_decode_u256_at_runtime(buffer_local, offset_local, ctx, instructions);
    instructions.push(Instruction::StoreLocal(len_local));

    // base = offset_local + 32 (first element).
    instructions.push(Instruction::LoadLocal(offset_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(32u8),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Add));
    instructions.push(Instruction::StoreLocal(base_local));

    instructions.push(Instruction::LoadLocal(len_local));
    instructions.push(Instruction::NewArray {
        element_type: element_type.clone(),
    });
    instructions.push(Instruction::StoreLocal(arr_local));

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

    // elem_off = base + idx * element_byte_len.
    instructions.push(Instruction::LoadLocal(idx_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(element_byte_len as u64),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Mul));
    instructions.push(Instruction::LoadLocal(base_local));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Add));
    instructions.push(Instruction::StoreLocal(elem_off_local));

    instructions.push(Instruction::LoadLocal(arr_local));
    instructions.push(Instruction::LoadLocal(idx_local));
    emit_abi_decode_static_slot_at_runtime_offset(
        buffer_local,
        elem_off_local,
        element_type,
        ctx,
        instructions,
    );
    instructions.push(Instruction::ArraySet);

    instructions.push(Instruction::LoadLocal(idx_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::one(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Add));
    instructions.push(Instruction::StoreLocal(idx_local));
    instructions.push(Instruction::Jump { target: loop_label });
    instructions.push(Instruction::Label(end_label));

    instructions.push(Instruction::LoadLocal(arr_local));
}

/// Decode a `T[]` tail whose element type `T` is itself dynamic (string[],
/// bytes[], T[][]) at a runtime byte offset. The mirror of
/// [`emit_abi_dynamic_nested_array_tail`]: read the length, then for each
/// element read its offset (relative to the start of the head section) from
/// the head entry and decode the element tail recursively at that absolute
/// offset. Stack on exit: `[Array<decoded elements>]`.
pub(crate) fn emit_abi_decode_nested_array_tail_runtime(
    buffer_local: usize,
    offset_local: usize,
    element_type: &ValueType,
    depth: usize,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) {
    // Six reusable scratch locals for this nesting depth (see the encode-side
    // `emit_abi_dynamic_nested_array_tail` for the pooling rationale).
    let scratch = ctx.abi_nested_scratch_locals(depth, 6);
    let len_local = scratch[0];
    let base_local = scratch[1];
    let arr_local = scratch[2];
    let idx_local = scratch[3];
    let head_off_local = scratch[4];
    let elem_off_local = scratch[5];

    // n = length word at the tail offset.
    emit_abi_decode_u256_at_runtime(buffer_local, offset_local, ctx, instructions);
    instructions.push(Instruction::StoreLocal(len_local));

    // base = offset_local + 32 (start of the head section; element offsets are
    // relative to here).
    instructions.push(Instruction::LoadLocal(offset_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(32u8),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Add));
    instructions.push(Instruction::StoreLocal(base_local));

    instructions.push(Instruction::LoadLocal(len_local));
    instructions.push(Instruction::NewArray {
        element_type: element_type.clone(),
    });
    instructions.push(Instruction::StoreLocal(arr_local));

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

    // head_off = base + idx*32 (byte position of element idx's head entry).
    instructions.push(Instruction::LoadLocal(idx_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(32u8),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Mul));
    instructions.push(Instruction::LoadLocal(base_local));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Add));
    instructions.push(Instruction::StoreLocal(head_off_local));

    // elem_off = base + relative_offset(head_off).
    instructions.push(Instruction::LoadLocal(base_local));
    emit_abi_decode_u256_at_runtime(buffer_local, head_off_local, ctx, instructions);
    instructions.push(Instruction::BinaryOp(BinaryOperator::Add));
    instructions.push(Instruction::StoreLocal(elem_off_local));

    // arr[idx] = decode_dynamic_tail(elem_off, element_type)  (recursive)
    instructions.push(Instruction::LoadLocal(arr_local));
    instructions.push(Instruction::LoadLocal(idx_local));
    emit_abi_decode_dynamic_tail_runtime(
        buffer_local,
        elem_off_local,
        element_type,
        depth + 1,
        ctx,
        instructions,
    );
    instructions.push(Instruction::ArraySet);

    instructions.push(Instruction::LoadLocal(idx_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::one(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Add));
    instructions.push(Instruction::StoreLocal(idx_local));
    instructions.push(Instruction::Jump { target: loop_label });
    instructions.push(Instruction::Label(end_label));

    instructions.push(Instruction::LoadLocal(arr_local));
}

/// Decode a DYNAMIC struct (≥1 dynamic field) tail at a runtime byte offset
/// into a fresh `StackItem::Array` of field values — the mirror of
/// [`emit_abi_dynamic_struct_tail`]. The head section layout is statically
/// known from the field types (static fields inline, dynamic fields one
/// offset word each), so the per-field head position is a compile-time
/// constant; dynamic fields read a relative offset and decode their tail
/// recursively. Stack on exit: `[Array<field values>]`.
pub(crate) fn emit_abi_decode_dynamic_struct_tail_runtime(
    buffer_local: usize,
    offset_local: usize,
    fields: &[StructField],
    depth: usize,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) {
    let tmp_id = ctx.next_label();
    let struct_local = ctx.allocate_local(format!("__abi_dec_dstruct_{tmp_id}"), None);
    let field_off_local = ctx.allocate_local(format!("__abi_dec_dstruct_foff_{tmp_id}"), None);

    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(fields.len() as u64),
    )));
    instructions.push(Instruction::NewArray {
        element_type: ValueType::Any,
    });
    instructions.push(Instruction::StoreLocal(struct_local));

    let mut head_byte_offset = 0usize;
    for (field_index, field) in fields.iter().enumerate() {
        if abi_value_type_is_dynamic(&field.ty) {
            // head slot position = offset_local + head_byte_offset.
            instructions.push(Instruction::LoadLocal(offset_local));
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::from(head_byte_offset as u64),
            )));
            instructions.push(Instruction::BinaryOp(BinaryOperator::Add));
            instructions.push(Instruction::StoreLocal(field_off_local));
            // absolute tail offset = offset_local + rel(head slot).
            instructions.push(Instruction::LoadLocal(offset_local));
            emit_abi_decode_u256_at_runtime(buffer_local, field_off_local, ctx, instructions);
            instructions.push(Instruction::BinaryOp(BinaryOperator::Add));
            instructions.push(Instruction::StoreLocal(field_off_local));
            // arr[field_index] = decode_dynamic_tail(field_off, field.ty)
            instructions.push(Instruction::LoadLocal(struct_local));
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::from(field_index as u64),
            )));
            emit_abi_decode_dynamic_tail_runtime(
                buffer_local,
                field_off_local,
                &field.ty,
                depth + 1,
                ctx,
                instructions,
            );
            instructions.push(Instruction::ArraySet);
            head_byte_offset += 32;
        } else {
            // static field at offset_local + head_byte_offset.
            instructions.push(Instruction::LoadLocal(offset_local));
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::from(head_byte_offset as u64),
            )));
            instructions.push(Instruction::BinaryOp(BinaryOperator::Add));
            instructions.push(Instruction::StoreLocal(field_off_local));
            instructions.push(Instruction::LoadLocal(struct_local));
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::from(field_index as u64),
            )));
            emit_abi_decode_static_slot_at_runtime_offset(
                buffer_local,
                field_off_local,
                &field.ty,
                ctx,
                instructions,
            );
            instructions.push(Instruction::ArraySet);
            head_byte_offset += abi_static_slot_count(&field.ty).unwrap_or(1) * 32;
        }
    }

    instructions.push(Instruction::LoadLocal(struct_local));
}
