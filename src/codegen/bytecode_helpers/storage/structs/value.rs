use super::*;
use crate::opcode::OpCode;

pub(crate) fn emit_load_struct_value_from_slot(
    bytecode: &mut Vec<u8>,
    fields: &[ir::StructField],
    use_callt: bool,
    token_patches: &mut Vec<MethodTokenPatch>,
) {
    // Stack before: [base_slot]
    // Stack after:  [struct_array]
    push_integer_bigint(bytecode, &BigInt::from(fields.len() as u64));
    bytecode.push(OpCode::NEWARRAY.byte());

    // Stack: [base_slot, out_array]
    for (field_index, field) in fields.iter().enumerate() {
        // Mapping members have no loadable storage value (entries live at
        // per-key derived slots). Push a Null placeholder — WITHOUT a
        // `Storage.Get` — but keep the SETITEM below so the struct array
        // stays at `fields.len()` entries and PICKITEM indices remain
        // aligned with the in-memory struct layout.
        if matches!(field.ty, ValueType::Mapping { .. }) {
            bytecode.push(OpCode::PUSHNULL.byte());
        } else {
            // Derive the field slot from the base slot:
            // field_slot = keccak256(field_key || base_slot)
            bytecode.push(OpCode::OVER.byte()); // OVER (duplicate base_slot)
            push_data(bytecode, &field.key);
            bytecode.push(OpCode::SWAP.byte()); // SWAP -> [base_slot, out_array, field_key, base_slot]
            bytecode.push(OpCode::CAT.byte());
            emit_native_contract_call(
                bytecode,
                ir::NativeContract::CryptoLib,
                "keccak256",
                1,
                use_callt,
                token_patches,
            );

            // Stack: [base_slot, out_array, field_slot]
            match &field.ty {
                ValueType::Struct { fields, .. } => {
                    emit_load_struct_value_from_slot(bytecode, fields, use_callt, token_patches);
                }
                _ => {
                    emit_syscall(bytecode, "System.Storage.GetContext");
                    emit_syscall(bytecode, "System.Storage.Get");
                    emit_coerce_storage_value(bytecode, &field.ty);
                }
            }
        }

        // Set `out_array[field_index] = field_value`, preserving the array reference.
        bytecode.push(OpCode::OVER.byte()); // OVER (duplicate out_array)
        bytecode.push(OpCode::SWAP.byte()); // SWAP -> [base_slot, out_array, out_array, field_value]
        push_integer_bigint(bytecode, &BigInt::from(field_index as u64));
        bytecode.push(OpCode::SWAP.byte()); // SWAP -> [base_slot, out_array, out_array, idx, value]
        bytecode.push(OpCode::SETITEM.byte());
    }

    // Drop base_slot, leaving the struct array.
    bytecode.push(OpCode::SWAP.byte()); // SWAP -> [out_array, base_slot]
    bytecode.push(OpCode::DROP.byte());
}

pub(crate) fn emit_store_struct_value_to_slot(
    bytecode: &mut Vec<u8>,
    fields: &[ir::StructField],
    use_callt: bool,
    token_patches: &mut Vec<MethodTokenPatch>,
) {
    // Stack before: [base_slot, struct_array]
    // Stack after:  []
    for (field_index, field) in fields.iter().enumerate() {
        // Skip Mapping members — Solidity `delete` / whole-struct writes
        // leave mapping entries untouched, and the in-memory value at the
        // mapping's field index is Null, which `Storage.Put` faults on for
        // real Neo N3. Each iteration only consumes copies of
        // `[base_slot, struct_array]`, so skipping is stack-safe.
        if matches!(field.ty, ValueType::Mapping { .. }) {
            continue;
        }
        // Derive field_slot = keccak256(field_key || base_slot)
        bytecode.push(OpCode::OVER.byte()); // OVER (duplicate base_slot)
        push_data(bytecode, &field.key);
        bytecode.push(OpCode::SWAP.byte());
        bytecode.push(OpCode::CAT.byte());
        emit_native_contract_call(
            bytecode,
            ir::NativeContract::CryptoLib,
            "keccak256",
            1,
            use_callt,
            token_patches,
        );

        // Stack: [base_slot, struct_array, field_slot]
        // Extract field_value = struct_array[field_index]
        bytecode.push(OpCode::OVER.byte()); // OVER (duplicate struct_array)
        push_integer_bigint(bytecode, &BigInt::from(field_index as u64));
        bytecode.push(OpCode::PICKITEM.byte()); // PICKITEM -> [base_slot, struct_array, field_slot, field_value]

        match &field.ty {
            ValueType::Struct { fields, .. } => {
                emit_store_struct_value_to_slot(bytecode, fields, use_callt, token_patches);
            }
            _ => {
                // Store expects [value, key, context], so swap to put value on top of the slot.
                bytecode.push(OpCode::SWAP.byte()); // SWAP -> [base_slot, struct_array, field_value, field_slot]
                emit_syscall(bytecode, "System.Storage.GetContext");
                emit_syscall(bytecode, "System.Storage.Put");
            }
        }
    }

    // Drop `[base_slot, struct_array]`.
    bytecode.push(OpCode::DROP.byte()); // DROP (struct_array)
    bytecode.push(OpCode::DROP.byte()); // DROP (base_slot)
}
