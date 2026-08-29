//! ## ABI Encoding — Dynamic Tails
//!
//! Helpers extracted from `abi_encode.rs` to keep the orchestration module under
//! the 800-line limit. This module emits EVM-canonical dynamic tails for:
//!
//! * **bytes / string** — length word + zero-padded data
//! * **static-element arrays** — length word + inline element slots
//! * **dynamic-element arrays** — length + offsets + recursive element tails
//! * **dynamic structs** — head section with inline static fields and offsets for
//!   dynamic fields, followed by the dynamic-field tails
//!
//! Functions are `pub(crate)` and re-exported from `builtins` so the main
//! `abi_encode` module can access them through a single namespace.

use super::*;

pub(crate) fn emit_abi_dynamic_tail_for_value_type(
    value_type: &ValueType,
    depth: usize,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<()> {
    match value_type {
        ValueType::ByteArray { fixed_len: None } | ValueType::String => {
            emit_abi_dynamic_bytes_tail(ctx, instructions);
            Some(())
        }
        ValueType::Array(element_type) if abi_static_slot_count(element_type).is_some() => {
            emit_abi_dynamic_static_array_tail(element_type, ctx, instructions)
        }
        // `T[]` where each element is itself dynamic (string[], bytes[],
        // uint256[][], ...). The element offsets form a head section and the
        // element encodings a tail section, recursively — full EVM layout.
        ValueType::Array(element_type) if abi_dynamic_value_type_is_supported(element_type) => {
            emit_abi_dynamic_nested_array_tail(element_type, depth, ctx, instructions)
        }
        // Dynamic struct → encode as a tuple of its fields (head+tail).
        ValueType::Struct { fields, .. } if abi_dynamic_value_type_is_supported(value_type) => {
            emit_abi_dynamic_struct_tail(fields, depth, ctx, instructions)
        }
        _ => None,
    }
}

pub(crate) fn emit_abi_dynamic_bytes_tail(
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) {
    let tmp_id = ctx.next_label();
    let src_local = ctx.allocate_local(format!("__abi_dyn_bytes_src_{tmp_id}"), None);
    let len_local = ctx.allocate_local(format!("__abi_dyn_bytes_len_{tmp_id}"), None);
    let padded_len_local = ctx.allocate_local(format!("__abi_dyn_bytes_padded_len_{tmp_id}"), None);
    let padded_local = ctx.allocate_local(format!("__abi_dyn_bytes_padded_{tmp_id}"), None);
    let len_slot_local = ctx.allocate_local(format!("__abi_dyn_bytes_len_slot_{tmp_id}"), None);

    instructions.push(Instruction::StoreLocal(src_local));
    instructions.push(Instruction::LoadLocal(src_local));
    instructions.push(Instruction::GetSize);
    instructions.push(Instruction::StoreLocal(len_local));

    instructions.push(Instruction::LoadLocal(len_local));
    emit_abi_u256_slot(ctx, instructions);
    instructions.push(Instruction::StoreLocal(len_slot_local));

    instructions.push(Instruction::LoadLocal(len_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(31u8),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Add));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(32u8),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Div));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(32u8),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Mul));
    instructions.push(Instruction::StoreLocal(padded_len_local));

    instructions.push(Instruction::LoadLocal(padded_len_local));
    instructions.push(Instruction::NewBuffer);
    instructions.push(Instruction::StoreLocal(padded_local));

    instructions.push(Instruction::LoadLocal(padded_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::LoadLocal(src_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::LoadLocal(len_local));
    instructions.push(Instruction::MemCpy);

    instructions.push(Instruction::LoadLocal(len_slot_local));
    instructions.push(Instruction::LoadLocal(padded_local));
    instructions.push(Instruction::Convert {
        target: ConvertTarget::ByteArray,
    });
    instructions.push(Instruction::CallBuiltin {
        builtin: BuiltinCall::BytesConcat,
        arg_count: 2,
    });
}

pub(crate) fn emit_abi_dynamic_static_array_tail(
    element_type: &ValueType,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<()> {
    emit_abi_static_array_buffer(element_type, ctx, instructions, true)
}

/// Emit the ABI tail for a `T[]` whose element type `T` is itself dynamic
/// (e.g. `string[]`, `bytes[]`, `uint256[][]`). Consumes the array value from
/// the top of the stack and leaves the encoded tail (a `ByteString`) in its
/// place. Layout (all offsets relative to the start of the head section, i.e.
/// the slot immediately after the length word — standard EVM ABI):
///
/// ```text
///   [ length n ]                                  # 32-byte word
///   [ off_0 ][ off_1 ] ... [ off_{n-1} ]          # head: n × 32-byte words
///   [ tail_0 ][ tail_1 ] ... [ tail_{n-1} ]       # tail: element encodings
/// ```
///
/// where `off_i = n*32 + Σ_{j<i} len(tail_j)` and each `tail_i` is produced by
/// recursively encoding element `i` via [`emit_abi_dynamic_tail_for_value_type`].
pub(crate) fn emit_abi_dynamic_nested_array_tail(
    element_type: &ValueType,
    depth: usize,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<()> {
    // Seven reusable scratch locals for this nesting depth. Distinct depths
    // never collide (the inner element is encoded while these are live); call
    // sites at the same depth share the block.
    let scratch = ctx.abi_nested_scratch_locals(depth, 7);
    let arr_local = scratch[0];
    let n_local = scratch[1];
    let off_local = scratch[2];
    let heads_local = scratch[3];
    let tails_local = scratch[4];
    let idx_local = scratch[5];
    let et_local = scratch[6];

    // arr := top of stack; n := arr.length
    instructions.push(Instruction::StoreLocal(arr_local));
    instructions.push(Instruction::LoadLocal(arr_local));
    instructions.push(Instruction::GetSize);
    instructions.push(Instruction::StoreLocal(n_local));

    // off := n * 32  (head section size; first element tail begins here)
    instructions.push(Instruction::LoadLocal(n_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(32u8),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Mul));
    instructions.push(Instruction::StoreLocal(off_local));

    // heads := "" ; tails := ""
    instructions.push(Instruction::PushLiteral(
        LiteralValue::ByteArray(Vec::new()),
    ));
    instructions.push(Instruction::StoreLocal(heads_local));
    instructions.push(Instruction::PushLiteral(
        LiteralValue::ByteArray(Vec::new()),
    ));
    instructions.push(Instruction::StoreLocal(tails_local));

    // idx := 0
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::StoreLocal(idx_local));

    let loop_label = ctx.next_label();
    let end_label = ctx.next_label();
    instructions.push(Instruction::Label(loop_label));
    // while idx < n  (JumpIf -> JMPIFNOT exits when the condition is false)
    instructions.push(Instruction::LoadLocal(idx_local));
    instructions.push(Instruction::LoadLocal(n_local));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Lt));
    instructions.push(Instruction::JumpIf { target: end_label });

    // et := encode(arr[idx])  (recursive element tail)
    instructions.push(Instruction::LoadLocal(arr_local));
    instructions.push(Instruction::LoadLocal(idx_local));
    instructions.push(Instruction::ArrayGet);
    emit_abi_dynamic_tail_for_value_type(element_type, depth + 1, ctx, instructions)?;
    instructions.push(Instruction::StoreLocal(et_local));

    // heads := heads ++ u256_slot(off)
    instructions.push(Instruction::LoadLocal(heads_local));
    instructions.push(Instruction::LoadLocal(off_local));
    emit_abi_u256_slot(ctx, instructions);
    instructions.push(Instruction::CallBuiltin {
        builtin: BuiltinCall::BytesConcat,
        arg_count: 2,
    });
    instructions.push(Instruction::StoreLocal(heads_local));

    // off := off + len(et)
    instructions.push(Instruction::LoadLocal(off_local));
    instructions.push(Instruction::LoadLocal(et_local));
    instructions.push(Instruction::GetSize);
    instructions.push(Instruction::BinaryOp(BinaryOperator::Add));
    instructions.push(Instruction::StoreLocal(off_local));

    // tails := tails ++ et
    instructions.push(Instruction::LoadLocal(tails_local));
    instructions.push(Instruction::LoadLocal(et_local));
    instructions.push(Instruction::CallBuiltin {
        builtin: BuiltinCall::BytesConcat,
        arg_count: 2,
    });
    instructions.push(Instruction::StoreLocal(tails_local));

    // idx := idx + 1
    instructions.push(Instruction::LoadLocal(idx_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::one(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Add));
    instructions.push(Instruction::StoreLocal(idx_local));
    instructions.push(Instruction::Jump { target: loop_label });
    instructions.push(Instruction::Label(end_label));

    // result := u256_slot(n) ++ heads ++ tails
    instructions.push(Instruction::LoadLocal(n_local));
    emit_abi_u256_slot(ctx, instructions);
    instructions.push(Instruction::LoadLocal(heads_local));
    instructions.push(Instruction::LoadLocal(tails_local));
    instructions.push(Instruction::CallBuiltin {
        builtin: BuiltinCall::BytesConcat,
        arg_count: 3,
    });
    Some(())
}

/// Emit the ABI tail for a DYNAMIC struct (one with ≥1 dynamic field).
/// Consumes the struct value (a NeoVM `Array` of field values) from the top
/// of the stack and leaves its tuple encoding (a `ByteString`). The layout
/// is the standard EVM tuple: a head section (static fields inline, dynamic
/// fields as 32-byte offsets relative to the start of the head) followed by
/// the dynamic-field tails — identical to `lower_abi_encode_head_tail_direct`
/// but reading fields from the struct array rather than argument expressions.
pub(crate) fn emit_abi_dynamic_struct_tail(
    fields: &[StructField],
    depth: usize,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<()> {
    // Head size in 32-byte words: dynamic fields take one offset word; static
    // fields occupy their inline slot count.
    let mut head_slot_count = 0usize;
    for field in fields {
        if abi_value_type_is_dynamic(&field.ty) {
            if !abi_dynamic_value_type_is_supported(&field.ty) {
                return None;
            }
            head_slot_count += 1;
        } else if let Some(slots) = abi_static_slot_count(&field.ty) {
            head_slot_count += slots;
        } else {
            return None;
        }
    }

    let tmp_id = ctx.next_label();
    let struct_local = ctx.allocate_local(format!("__abi_dstruct_{tmp_id}"), None);
    instructions.push(Instruction::StoreLocal(struct_local));

    let offset_local = ctx.allocate_local(format!("__abi_dstruct_off_{tmp_id}"), None);
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from((head_slot_count * 32) as u64),
    )));
    instructions.push(Instruction::StoreLocal(offset_local));

    let mut tail_locals = Vec::new();
    let mut part_count = 0usize;

    for (field_index, field) in fields.iter().enumerate() {
        if abi_value_type_is_dynamic(&field.ty) {
            // Head: the offset word pointing at this field's tail.
            instructions.push(Instruction::LoadLocal(offset_local));
            emit_abi_u256_slot(ctx, instructions);
            part_count += 1;

            // Compute the field tail and stash it for the tail section.
            instructions.push(Instruction::LoadLocal(struct_local));
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::from(field_index as u64),
            )));
            instructions.push(Instruction::ArrayGet);
            emit_abi_dynamic_tail_for_value_type(&field.ty, depth + 1, ctx, instructions)?;
            let tail_local =
                ctx.allocate_local(format!("__abi_dstruct_tail_{tmp_id}_{field_index}"), None);
            instructions.push(Instruction::StoreLocal(tail_local));

            // offset += len(tail)
            instructions.push(Instruction::LoadLocal(offset_local));
            instructions.push(Instruction::LoadLocal(tail_local));
            instructions.push(Instruction::GetSize);
            instructions.push(Instruction::BinaryOp(BinaryOperator::Add));
            instructions.push(Instruction::StoreLocal(offset_local));
            tail_locals.push(tail_local);
        } else {
            // Static field: inline its slot(s) into the head section.
            instructions.push(Instruction::LoadLocal(struct_local));
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::from(field_index as u64),
            )));
            instructions.push(Instruction::ArrayGet);
            let slots = emit_abi_static_slots_for_stack_value(&field.ty, ctx, instructions)?;
            part_count += slots;
        }
    }

    for tail_local in tail_locals {
        instructions.push(Instruction::LoadLocal(tail_local));
        part_count += 1;
    }

    if part_count == 0 {
        return None;
    }
    instructions.push(Instruction::CallBuiltin {
        builtin: BuiltinCall::BytesConcat,
        arg_count: part_count,
    });
    Some(())
}
