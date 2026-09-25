//! ## ABI Decoding — Static Slots
//!
//! Helpers extracted from `abi_decode.rs` to keep the orchestration module
//! under the 800-line limit. This module decodes EVM-canonical static slots
//! (integers, booleans, addresses, fixed `bytesN`, and all-static structs)
//! both at compile-time slot indices and at runtime byte offsets.
//!
//! Functions are `pub(crate)` and re-exported from `builtins` so the main
//! `abi_decode` module and the dynamic-tail module can access them through a
//! single namespace.

use super::*;

/// Convert the little-endian 32-byte slot Buffer on top of the stack into the
/// canonical NeoVM integer for a decoded `uint256`.
///
/// The 32-byte little-endian slot IS the conformant uint256 representation: a
/// 32-byte two's-complement integer. NeoVM `CONVERT → Integer` reads it as a
/// SIGNED little-endian value, which is exactly the canonical form the rest of
/// the runtime uses — a `uint256 >= 2^255` reads back as its negative
/// two's-complement, byte-identical to the same value produced by a literal or
/// by arithmetic, so `abi.decode(abi.encode(x)) == x` holds across the whole
/// uint256 domain.
///
/// (An earlier revision appended a conditional `0x00` sign byte to force a
/// POSITIVE 33-byte magnitude — the non-conformant unsigned-magnitude model,
/// since removed: a real node stores two's-complement, and a 33-byte integer
/// exceeds NeoVM's 32-byte limit.)
///
/// Stack on entry: `[le_bytes_buffer]`.
/// Stack on exit:  `[integer]`.
pub(crate) fn emit_le_buffer_to_unsigned_integer(
    _ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) {
    // The 32-byte little-endian slot buffer IS the conformant uint256
    // representation: a 32-byte two's-complement integer. Reading it as a signed
    // NeoVM integer yields the canonical value — a `uint256 >= 2^255` reads back
    // as its negative two's-complement, byte-identical to the same value produced
    // by a literal or by arithmetic, so `abi.decode(abi.encode(x)) == x` holds.
    //
    // The previous code appended a `0x00` sign byte to coerce values >= 2^255
    // into a POSITIVE 33-byte magnitude — the old unsigned-magnitude model, which
    // is non-conformant (a real node stores two's-complement) and not equal to
    // the two's-complement form the rest of the runtime uses.
    instructions.push(Instruction::Convert {
        target: ConvertTarget::Integer,
    });
}

/// Decode a single static element at a runtime byte offset inside
/// `buffer_local`. Mirrors [`emit_abi_decode_static_slot`] but operates
/// at a runtime offset rather than a compile-time slot index.
pub(crate) fn emit_abi_decode_static_slot_at_runtime_offset(
    buffer_local: usize,
    offset_local: usize,
    value_type: &ValueType,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) {
    match value_type {
        // Unsigned integers must NOT be read with NeoVM's signed-LE
        // CONVERT semantics — see `emit_le_buffer_to_unsigned_integer`.
        ValueType::Integer { signed: false, .. } => {
            instructions.push(Instruction::LoadLocal(buffer_local));
            instructions.push(Instruction::LoadLocal(offset_local));
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::from(32u8),
            )));
            instructions.push(Instruction::Substr);
            instructions.push(Instruction::Convert {
                target: ConvertTarget::ByteArray,
            });
            materialize_byte_array_buffer(ctx, instructions, true);
            emit_le_buffer_to_unsigned_integer(ctx, instructions);
        }
        ValueType::Integer { signed: true, .. } | ValueType::Boolean => {
            instructions.push(Instruction::LoadLocal(buffer_local));
            instructions.push(Instruction::LoadLocal(offset_local));
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::from(32u8),
            )));
            instructions.push(Instruction::Substr);
            instructions.push(Instruction::Convert {
                target: ConvertTarget::ByteArray,
            });
            materialize_byte_array_buffer(ctx, instructions, true);
            instructions.push(Instruction::Convert {
                target: ConvertTarget::Integer,
            });
            // A `bool` value must be a real Boolean stack item, not an Integer.
            // `decoded == true` lowers to NeoVM `EQUAL`, which distinguishes
            // Integer(1) from Boolean(true) and would be false even though the
            // value is 1. (The simulator masks this with a lenient EQUAL; on a
            // real node it breaks `abi.decode(..,(bool)) == true`.) Also makes
            // the returned item match the manifest's declared Boolean type.
            if matches!(value_type, ValueType::Boolean) {
                instructions.push(Instruction::Convert {
                    target: ConvertTarget::Boolean,
                });
            }
        }
        // All-static struct element: decode each field from its consecutive
        // 32-byte slot into a fresh `StackItem::Array` (the Array-of-fields
        // shape the struct-constructor lowering produces and the encode
        // side reads back via ArrayGet).
        ValueType::Struct { fields, .. } => {
            let tmp_id = ctx.next_label();
            let struct_local = ctx.allocate_local(format!("__abi_dec_rstruct_{tmp_id}"), None);
            let field_off_local =
                ctx.allocate_local(format!("__abi_dec_rstruct_off_{tmp_id}"), None);
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::from(fields.len() as u64),
            )));
            instructions.push(Instruction::NewArray {
                element_type: ValueType::Any,
            });
            instructions.push(Instruction::StoreLocal(struct_local));
            let mut field_byte_offset = 0usize;
            for (field_index, field) in fields.iter().enumerate() {
                // field_off = offset_local + field_byte_offset.
                instructions.push(Instruction::LoadLocal(offset_local));
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                    BigInt::from(field_byte_offset as u64),
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
                field_byte_offset += abi_static_slot_count(&field.ty).unwrap_or(1) * 32;
            }
            instructions.push(Instruction::LoadLocal(struct_local));
        }
        ValueType::Address => {
            // EVM ABI: addresses are 20 bytes left-padded with 12 zero
            // bytes inside a 32-byte slot, so the value lives in
            // `[slot+12 .. slot+32)`.
            instructions.push(Instruction::LoadLocal(buffer_local));
            instructions.push(Instruction::LoadLocal(offset_local));
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::from(12u8),
            )));
            instructions.push(Instruction::BinaryOp(BinaryOperator::Add));
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::from(20u8),
            )));
            instructions.push(Instruction::Substr);
            instructions.push(Instruction::Convert {
                target: ConvertTarget::ByteArray,
            });
            materialize_byte_array_buffer(ctx, instructions, true);
            // `materialize_byte_array_buffer` leaves a *Buffer* (0x30) on the
            // stack (NewBuffer + MemCpy + ReverseItems). A decoded address is
            // routinely compared with `==` against an address literal, which on
            // a real node lowers to `EQUAL` — and NeoVM's `EQUAL` distinguishes
            // Buffer from ByteString, so the comparison would be false even when
            // the bytes match. Coerce back to ByteString. (The in-tree simulator
            // masks this with a lenient EQUAL.)
            instructions.push(Instruction::Convert {
                target: ConvertTarget::ByteArray,
            });
        }
        ValueType::ByteArray {
            fixed_len: Some(len),
        } => {
            let len = (*len).min(32) as usize;
            instructions.push(Instruction::LoadLocal(buffer_local));
            instructions.push(Instruction::LoadLocal(offset_local));
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::from(len as u64),
            )));
            instructions.push(Instruction::Substr);
            instructions.push(Instruction::Convert {
                target: ConvertTarget::ByteArray,
            });
        }
        _ => {
            instructions.push(Instruction::LoadLocal(buffer_local));
            instructions.push(Instruction::LoadLocal(offset_local));
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::from(32u8),
            )));
            instructions.push(Instruction::Substr);
            instructions.push(Instruction::Convert {
                target: ConvertTarget::ByteArray,
            });
        }
    }
}

pub(crate) fn emit_abi_decode_static_slot(
    buffer_local: usize,
    index: usize,
    value_type: &ValueType,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) {
    match value_type {
        // Unsigned integers must NOT be read with NeoVM's signed-LE
        // CONVERT semantics — see `emit_le_buffer_to_unsigned_integer`.
        ValueType::Integer { signed: false, .. } => {
            emit_abi_decode_slot_slice(buffer_local, index, 0, 32, instructions);
            instructions.push(Instruction::Convert {
                target: ConvertTarget::ByteArray,
            });
            materialize_byte_array_buffer(ctx, instructions, true);
            emit_le_buffer_to_unsigned_integer(ctx, instructions);
        }
        // Signed intN: the canonical slot is sign-extended big-endian, so
        // the signed-LE CONVERT after the reversal is exactly right.
        ValueType::Integer { signed: true, .. } | ValueType::Boolean => {
            emit_abi_decode_slot_slice(buffer_local, index, 0, 32, instructions);
            instructions.push(Instruction::Convert {
                target: ConvertTarget::ByteArray,
            });
            materialize_byte_array_buffer(ctx, instructions, true);
            instructions.push(Instruction::Convert {
                target: ConvertTarget::Integer,
            });
            // See the runtime-offset Boolean arm: a decoded `bool` must be a
            // real Boolean stack item so `decoded == true` (NeoVM `EQUAL`)
            // matches on a real node and the return matches the manifest type.
            if matches!(value_type, ValueType::Boolean) {
                instructions.push(Instruction::Convert {
                    target: ConvertTarget::Boolean,
                });
            }
        }
        // All-static struct: decode each field from consecutive head slots
        // starting at `index` into a fresh `StackItem::Array` (the
        // Array-of-fields shape the struct-constructor lowering produces
        // and the encode side reads back via ArrayGet). Mirrors
        // `lower_static_abi_slots_for_expr` on the encode side.
        ValueType::Struct { fields, .. } => {
            let tmp_id = ctx.next_label();
            let struct_local = ctx.allocate_local(format!("__abi_dec_struct_{tmp_id}"), None);
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::from(fields.len() as u64),
            )));
            instructions.push(Instruction::NewArray {
                element_type: ValueType::Any,
            });
            instructions.push(Instruction::StoreLocal(struct_local));
            let mut field_slot = index;
            for (field_index, field) in fields.iter().enumerate() {
                instructions.push(Instruction::LoadLocal(struct_local));
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                    BigInt::from(field_index as u64),
                )));
                emit_abi_decode_static_slot(buffer_local, field_slot, &field.ty, ctx, instructions);
                instructions.push(Instruction::ArraySet);
                field_slot += abi_static_slot_count(&field.ty).unwrap_or(1);
            }
            instructions.push(Instruction::LoadLocal(struct_local));
        }
        ValueType::Address => {
            emit_abi_decode_slot_slice(buffer_local, index, 12, 20, instructions);
            instructions.push(Instruction::Convert {
                target: ConvertTarget::ByteArray,
            });
            materialize_byte_array_buffer(ctx, instructions, true);
            // Coerce the reversed *Buffer* back to a ByteString so on-chain
            // `EQUAL` against an address literal matches (see the runtime-offset
            // Address arm above for the full rationale).
            instructions.push(Instruction::Convert {
                target: ConvertTarget::ByteArray,
            });
        }
        ValueType::ByteArray {
            fixed_len: Some(len),
        } => {
            let len = (*len).min(32) as usize;
            emit_abi_decode_slot_slice(buffer_local, index, 0, len, instructions);
            instructions.push(Instruction::Convert {
                target: ConvertTarget::ByteArray,
            });
        }
        _ => {
            emit_abi_decode_slot_slice(buffer_local, index, 0, 32, instructions);
            instructions.push(Instruction::Convert {
                target: ConvertTarget::ByteArray,
            });
        }
    }
}
