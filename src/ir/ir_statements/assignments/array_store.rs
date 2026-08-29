use super::*;

pub(crate) fn lower_array_store(
    target: &Expression,
    rhs: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) {
    // Bug-hunt #11/#13 — element write to a storage `bytes` (`data[i] = v`).
    // Storage `bytes` is `ValueType::ByteArray{fixed_len:None}`, which
    // `resolve_storage_reference` does not model (it only handles
    // Array/Mapping/Struct), so the assignment lands here. The generic path
    // below would `ArraySet` a materialised copy that is never written back, so
    // the store silently vanishes. Emit a read-modify-write into the slot
    // instead: `data = data[0:i] ++ bytes1(v) ++ data[i+1 : len-(i+1)]`.
    if let Expression::ArraySubscript(_, array, Some(index)) = target {
        if let Expression::Variable(id) = array.as_ref() {
            let shadowed = ctx.storage_alias(&id.name).is_some()
                || ctx.param_index_map.contains_key(&id.name)
                || ctx.resolve_local(&id.name).is_some();
            if !shadowed {
                if let Some(&state_index) = ctx.state_index_map.get(&id.name) {
                    if matches!(
                        ctx.state_type(state_index),
                        Some(ValueType::ByteArray { fixed_len: None })
                    ) {
                        lower_storage_bytes_element_store(
                            state_index,
                            index,
                            rhs,
                            ctx,
                            instructions,
                        );
                        return;
                    }
                }
            }
        }
    }

    if let Expression::ArraySubscript(_, array, Some(index)) = target {
        let checkpoint = instructions.len();
        // A `bytesN` element assigned from an integer-backed literal must be
        // stored as its canonical big-endian ByteArray (matching the
        // cast/param representation) — else N<32 faults / N==32 reverses when
        // the element is later ABI-encoded.
        let elem_ty = match infer_type_from_expression(array, ctx) {
            Some(ValueType::Array(e)) => Some(*e),
            _ => None,
        };
        if lower_expression(array, ctx, instructions) && lower_expression(index, ctx, instructions)
        {
            let coerced = elem_ty
                .as_ref()
                .is_some_and(|et| try_lower_bytesn_literal_canonical(rhs, et, ctx, instructions));
            if coerced || lower_expression(rhs, ctx, instructions) {
                // No Swap; Drop needed: real NeoVM MEMCPY pushes nothing.
                instructions.push(Instruction::ArraySet);
                return;
            }
        }
        instructions.truncate(checkpoint);
    }

    load_expression(rhs, ctx, instructions);
    instructions.push(Instruction::Drop(ValueType::Any));
}

/// Read-modify-write a single byte of a storage `bytes` slot:
/// `data = data[0:i] ++ bytes1(v) ++ data[i+1 : len-(i+1)]`, then `StoreState`.
/// Mirrors the ByteString slicing/concat idiom used by `lower_state_bytes_pop`
/// (state_var.rs) — no in-place SETITEM (NeoVM ByteStrings are immutable, and a
/// Buffer SETITEM would need an integer, not the `bytes1` element value). The
/// element (Solidity type `bytes1`) is canonicalized from an integer-backed
/// literal to its big-endian byte, else lowered and coerced to a 1-byte
/// ByteString.
fn lower_storage_bytes_element_store(
    state_index: usize,
    index: &Expression,
    rhs: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) {
    let buf = ctx.allocate_local("__sbytes_buf".to_string(), None);
    let idx = ctx.allocate_local("__sbytes_idx".to_string(), None);

    // buf = ByteArray(current storage bytes)
    instructions.push(Instruction::LoadState(state_index));
    instructions.push(Instruction::Convert {
        target: ConvertTarget::ByteArray,
    });
    instructions.push(Instruction::StoreLocal(buf));

    // idx = index
    lower_expression(index, ctx, instructions);
    instructions.push(Instruction::StoreLocal(idx));

    // head = buf[0 : idx]
    instructions.push(Instruction::LoadLocal(buf));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::LoadLocal(idx));
    instructions.push(Instruction::Substr);

    // v1 = bytes1(rhs)
    let b1 = ValueType::ByteArray { fixed_len: Some(1) };
    if !try_lower_bytesn_literal_canonical(rhs, &b1, ctx, instructions) {
        lower_expression(rhs, ctx, instructions);
        instructions.push(Instruction::Convert {
            target: ConvertTarget::ByteArray,
        });
    }

    // head ++ v1
    instructions.push(Instruction::CallBuiltin {
        builtin: BuiltinCall::BytesConcat,
        arg_count: 2,
    });

    // tail = buf[idx+1 : len-(idx+1)]
    instructions.push(Instruction::LoadLocal(buf));
    instructions.push(Instruction::LoadLocal(idx));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::one(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Add)); // start = idx+1
    instructions.push(Instruction::LoadLocal(buf));
    instructions.push(Instruction::GetSize);
    instructions.push(Instruction::LoadLocal(idx));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::one(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Add));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Sub)); // count = len - (idx+1)
    instructions.push(Instruction::Substr);

    // (head ++ v1) ++ tail
    instructions.push(Instruction::CallBuiltin {
        builtin: BuiltinCall::BytesConcat,
        arg_count: 2,
    });
    instructions.push(Instruction::StoreState(state_index));
}
