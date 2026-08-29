use super::*;
use crate::opcode::OpCode;

/// Canonical NeoVM stack-item type for a mapping-key value of the given
/// Solidity type, or `None` when no canonicalization applies.
///
/// Real Neo N3's `StdLib.serialize` (BinarySerializer) embeds the stack-item
/// type byte in the serialized form (`Buffer` = 0x30 vs `ByteString` = 0x28,
/// `Integer` = 0x21, `Boolean` = 0x20), so two representations of the SAME
/// Solidity value — e.g. a `bytes.concat(..)` result, which CAT leaves as a
/// Buffer, vs an ABI `bytes` parameter, which is a ByteString — would derive
/// DIFFERENT `keccak256(serialize(key) || slot)` storage slots: a write via
/// one shape silently misses a read via the other. Mirror the SUBSTR slice
/// fix (`src/ir/expressions/arrays.rs`, Task #95) and CONVERT every key to
/// its canonical stack-item type before serializing.
pub(crate) fn canonical_key_convert_target(key_type: &ValueType) -> Option<ir::ConvertTarget> {
    match key_type {
        ValueType::Integer { .. } => Some(ir::ConvertTarget::Integer),
        ValueType::Boolean => Some(ir::ConvertTarget::Boolean),
        ValueType::Address | ValueType::String | ValueType::ByteArray { .. } => {
            Some(ir::ConvertTarget::ByteArray)
        }
        // Aggregates have no scalar canonical form and `Any` carries no type
        // information — leave those untouched (pre-fix behaviour).
        ValueType::Array(_)
        | ValueType::Mapping { .. }
        | ValueType::Struct { .. }
        | ValueType::Any => None,
    }
}

pub(crate) fn emit_serialize_key(
    bytecode: &mut Vec<u8>,
    key_type: &ValueType,
    use_callt: bool,
    token_patches: &mut Vec<MethodTokenPatch>,
) {
    if let Some(target) = canonical_key_convert_target(key_type) {
        emit_convert(bytecode, target);
    }
    emit_native_contract_call(
        bytecode,
        ir::NativeContract::StdLib,
        "serialize",
        1,
        use_callt,
        token_patches,
    );
}

pub(crate) fn emit_mapping_slot(
    bytecode: &mut Vec<u8>,
    module: &ir::Module,
    state_index: usize,
    key_types: &[ValueType],
    use_callt: bool,
    token_patches: &mut Vec<MethodTokenPatch>,
) {
    // `state_index` is always an in-range index into `state_variables` (the IR
    // builder derives it from the same `state_index_map`). A miss would silently
    // emit a zero base slot, so storage reads/writes for this mapping would land
    // on the wrong key — fail loudly in debug/test builds while keeping a
    // deterministic (never-panicking) fallback for release.
    debug_assert!(
        module.state_variables.get(state_index).is_some(),
        "emit_mapping_slot: state_index {state_index} out of range ({} state variables)",
        module.state_variables.len()
    );
    let base_slot = module
        .state_variables
        .get(state_index)
        .map(|state| state.storage_key.clone())
        .unwrap_or_else(|| vec![0u8; 32]);

    push_data(bytecode, &base_slot);

    for key_type in key_types {
        bytecode.push(OpCode::SWAP.byte()); // swap slot <-> key
        emit_serialize_key(bytecode, key_type, use_callt, token_patches);
        bytecode.push(OpCode::SWAP.byte()); // swap key_bytes <-> slot
        bytecode.push(OpCode::CAT.byte()); // concatenate key and slot
        emit_native_contract_call(
            bytecode,
            ir::NativeContract::CryptoLib,
            "keccak256",
            1,
            use_callt,
            token_patches,
        );
    }
}

pub(crate) fn emit_struct_field_slot(
    bytecode: &mut Vec<u8>,
    module: &ir::Module,
    state_index: usize,
    key_types: &[ValueType],
    field_keys: &[[u8; 32]],
    use_callt: bool,
    token_patches: &mut Vec<MethodTokenPatch>,
) {
    emit_mapping_slot(
        bytecode,
        module,
        state_index,
        key_types,
        use_callt,
        token_patches,
    );
    for field_key in field_keys {
        push_data(bytecode, field_key);
        bytecode.push(OpCode::SWAP.byte()); // swap slot and field key bytes
        bytecode.push(OpCode::CAT.byte()); // concatenate
        emit_native_contract_call(
            bytecode,
            ir::NativeContract::CryptoLib,
            "keccak256",
            1,
            use_callt,
            token_patches,
        );
    }
}

pub(crate) fn emit_load_mapping(
    bytecode: &mut Vec<u8>,
    module: &ir::Module,
    state_index: usize,
    key_types: &[ValueType],
    use_callt: bool,
    token_patches: &mut Vec<MethodTokenPatch>,
) {
    emit_mapping_slot(
        bytecode,
        module,
        state_index,
        key_types,
        use_callt,
        token_patches,
    );
    emit_syscall(bytecode, "System.Storage.GetContext");
    emit_syscall(bytecode, "System.Storage.Get");
    if let Some(value_type) =
        resolve_loaded_mapping_value_type(module, state_index, key_types.len())
    {
        emit_coerce_storage_value(bytecode, value_type);
    }
}

pub(crate) fn resolve_loaded_mapping_value_type(
    module: &ir::Module,
    state_index: usize,
    key_depth: usize,
) -> Option<&ValueType> {
    let mut current = &module.state_variables.get(state_index)?.ty;

    for _ in 0..key_depth {
        match current {
            ValueType::Mapping { value, .. } => {
                current = value.as_ref();
            }
            ValueType::Array(element) => {
                current = element.as_ref();
            }
            _ => return Some(current),
        }
    }

    Some(current)
}

pub(crate) fn emit_store_mapping(
    bytecode: &mut Vec<u8>,
    module: &ir::Module,
    state_index: usize,
    key_types: &[ValueType],
    use_callt: bool,
    token_patches: &mut Vec<MethodTokenPatch>,
) {
    emit_mapping_slot(
        bytecode,
        module,
        state_index,
        key_types,
        use_callt,
        token_patches,
    );
    emit_syscall(bytecode, "System.Storage.GetContext");
    emit_syscall(bytecode, "System.Storage.Put");
}

pub(crate) fn emit_store_mapping_array_deep_copy(
    bytecode: &mut Vec<u8>,
    module: &ir::Module,
    state_index: usize,
    key_types: &[ValueType],
    use_callt: bool,
    token_patches: &mut Vec<MethodTokenPatch>,
) {
    emit_mapping_slot(
        bytecode,
        module,
        state_index,
        key_types,
        use_callt,
        token_patches,
    );
    // `emit_mapping_slot` leaves `[array, slot]`; the helper below expects
    // `[slot, array]`.
    bytecode.push(OpCode::SWAP.byte());
    emit_store_array_value_deep_copy(bytecode, use_callt, token_patches);
}

/// Task #205 — deep-copy a dynamic-array value into a storage slot so the
/// read side sees:
/// - length at the base slot
/// - element `i` at `keccak256(serialize(i) || slot)`
///
/// Stack on entry: `[..., slot, array]`
/// Stack on exit:  `[...]`
pub(crate) fn emit_store_array_value_deep_copy(
    bytecode: &mut Vec<u8>,
    use_callt: bool,
    token_patches: &mut Vec<MethodTokenPatch>,
) {
    bytecode.push(OpCode::DUP.byte()); // DUP -> [..., slot, array, array]
    bytecode.push(OpCode::SIZE.byte()); // SIZE -> [..., slot, array, length]

    let loop_start_pos = bytecode.len();

    bytecode.push(OpCode::DUP.byte()); // DUP -> [..., slot, array, i, i]
    bytecode.push(OpCode::PUSH0.byte()); // PUSH0 -> [..., slot, array, i, i, 0]
    bytecode.push(OpCode::EQUAL.byte()); // EQUAL -> [..., slot, array, i, (i==0)]
    let jmp_exit_pos = bytecode.len();
    bytecode.push(OpCode::JMPIF_L.byte());
    let jmp_exit_operand = bytecode.len();
    bytecode.extend_from_slice(&[0, 0, 0, 0]);

    bytecode.push(OpCode::DEC.byte()); // DEC -> [..., slot, array, current]

    bytecode.push(OpCode::DUP.byte()); // DUP -> [..., slot, array, current, current]
    emit_serialize_key(
        bytecode,
        &ValueType::Integer {
            signed: false,
            bits: 256,
        },
        use_callt,
        token_patches,
    );
    bytecode.push(OpCode::PUSH3.byte());
    bytecode.push(OpCode::PICK.byte()); // PICK -> [..., slot, array, current, current_bytes, slot]
    bytecode.push(OpCode::CAT.byte());
    emit_native_contract_call(
        bytecode,
        ir::NativeContract::CryptoLib,
        "keccak256",
        1,
        use_callt,
        token_patches,
    );

    bytecode.push(OpCode::PUSH2.byte());
    bytecode.push(OpCode::PICK.byte()); // PICK -> [..., slot, array, current, element_slot, array]
    bytecode.push(OpCode::PUSH2.byte());
    bytecode.push(OpCode::PICK.byte()); // PICK -> [..., slot, array, current, element_slot, array, current]
    bytecode.push(OpCode::PICKITEM.byte()); // PICKITEM -> [..., slot, array, current, element_slot, array[current]]

    bytecode.push(OpCode::SWAP.byte()); // SWAP -> [..., slot, array, current, array[current], element_slot]
    emit_syscall(bytecode, "System.Storage.GetContext");
    emit_syscall(bytecode, "System.Storage.Put");

    let jmp_back_pos = bytecode.len();
    bytecode.push(OpCode::JMP_L.byte());
    let jmp_back_operand = bytecode.len();
    bytecode.extend_from_slice(&[0, 0, 0, 0]);
    let rel_back = (loop_start_pos as i32)
        .checked_sub(jmp_back_pos as i32)
        .unwrap_or(0);
    bytecode[jmp_back_operand..jmp_back_operand + 4].copy_from_slice(&rel_back.to_le_bytes());

    let exit_pos = bytecode.len();
    let rel_exit = (exit_pos as i32)
        .checked_sub(jmp_exit_pos as i32)
        .unwrap_or(0);
    bytecode[jmp_exit_operand..jmp_exit_operand + 4].copy_from_slice(&rel_exit.to_le_bytes());

    bytecode.push(OpCode::DROP.byte()); // DROP -> [..., slot, array]
    bytecode.push(OpCode::SIZE.byte()); // SIZE -> [..., slot, length]
    bytecode.push(OpCode::SWAP.byte()); // SWAP -> [..., length, slot]
    emit_syscall(bytecode, "System.Storage.GetContext");
    emit_syscall(bytecode, "System.Storage.Put");
}

/// Task #82: derive the storage slot for a mapping-in-struct-field access such as
/// `slots[k].balances[a]`. Stack before: `[trailing_keyN, ..., trailing_key0, outer_keyN, ..., outer_key0]`.
/// Stack after: `[slot_bytes]`.
pub(crate) struct StructFieldMappingSlot<'a> {
    pub(crate) module: &'a ir::Module,
    pub(crate) state_index: usize,
    pub(crate) key_types: &'a [ValueType],
    pub(crate) field_keys: &'a [[u8; 32]],
    pub(crate) trailing_key_types: &'a [ValueType],
    pub(crate) use_callt: bool,
}

pub(crate) fn emit_struct_field_mapping_slot(
    bytecode: &mut Vec<u8>,
    slot: &StructFieldMappingSlot<'_>,
    token_patches: &mut Vec<MethodTokenPatch>,
) {
    emit_struct_field_slot(
        bytecode,
        slot.module,
        slot.state_index,
        slot.key_types,
        slot.field_keys,
        slot.use_callt,
        token_patches,
    );
    // Stack now: [trailing_keyN, ..., trailing_key0, field_slot].
    for key_type in slot.trailing_key_types {
        bytecode.push(OpCode::SWAP.byte()); // swap slot <-> key
        emit_serialize_key(bytecode, key_type, slot.use_callt, token_patches);
        bytecode.push(OpCode::SWAP.byte()); // swap key_bytes <-> slot
        bytecode.push(OpCode::CAT.byte());
        emit_native_contract_call(
            bytecode,
            ir::NativeContract::CryptoLib,
            "keccak256",
            1,
            slot.use_callt,
            token_patches,
        );
    }
}

pub(crate) fn emit_load_struct_field_mapping_element(
    bytecode: &mut Vec<u8>,
    slot: &StructFieldMappingSlot<'_>,
    value_type: &ValueType,
    token_patches: &mut Vec<MethodTokenPatch>,
) {
    emit_struct_field_mapping_slot(bytecode, slot, token_patches);
    emit_syscall(bytecode, "System.Storage.GetContext");
    emit_syscall(bytecode, "System.Storage.Get");
    emit_coerce_storage_value(bytecode, value_type);
}

pub(crate) fn emit_store_struct_field_mapping_element(
    bytecode: &mut Vec<u8>,
    slot: &StructFieldMappingSlot<'_>,
    token_patches: &mut Vec<MethodTokenPatch>,
) {
    emit_struct_field_mapping_slot(bytecode, slot, token_patches);
    emit_syscall(bytecode, "System.Storage.GetContext");
    emit_syscall(bytecode, "System.Storage.Put");
}
