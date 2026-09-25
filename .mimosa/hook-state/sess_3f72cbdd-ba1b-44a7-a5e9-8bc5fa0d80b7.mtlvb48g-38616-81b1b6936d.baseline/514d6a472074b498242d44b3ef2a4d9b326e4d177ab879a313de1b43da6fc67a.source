use super::*;
use crate::opcode::OpCode;

pub(crate) fn emit_load_state(bytecode: &mut Vec<u8>, module: &ir::Module, index: usize) {
    let key = module
        .state_variables
        .get(index)
        .map(|state| state.storage_key.as_slice())
        .unwrap_or(&[]);
    let value_type = module.state_variables.get(index).map(|state| &state.ty);

    push_data(bytecode, key);
    emit_syscall(bytecode, "System.Storage.GetContext");
    emit_syscall(bytecode, "System.Storage.Get");
    if let Some(ty) = value_type {
        emit_coerce_storage_value(bytecode, ty);
    }
}

pub(crate) fn emit_store_state(bytecode: &mut Vec<u8>, module: &ir::Module, index: usize) {
    let key = module
        .state_variables
        .get(index)
        .map(|state| state.storage_key.as_slice())
        .unwrap_or(&[]);

    push_data(bytecode, key);
    emit_syscall(bytecode, "System.Storage.GetContext");
    // Stack order for System.Storage.Put: [value, key, context]
    emit_syscall(bytecode, "System.Storage.Put");
}

pub(crate) fn emit_coerce_storage_value(bytecode: &mut Vec<u8>, ty: &ValueType) {
    fn with_null_default(
        bytecode: &mut Vec<u8>,
        default_value: impl Fn(&mut Vec<u8>),
        not_null: impl FnOnce(&mut Vec<u8>),
    ) {
        // Stack before: [value]
        // After this block: [coerced_or_default]
        bytecode.push(OpCode::DUP.byte());
        bytecode.push(OpCode::ISNULL.byte());

        // if !(value is null) -> jump to not_null
        let jmp_not_null_pos = bytecode.len();
        bytecode.push(OpCode::JMPIFNOT_L.byte());
        let jmp_not_null_operand = bytecode.len();
        bytecode.extend_from_slice(&[0, 0, 0, 0]);

        // null case: drop null and push default
        bytecode.push(OpCode::DROP.byte());
        default_value(bytecode);

        // jump to end
        let jmp_end_pos = bytecode.len();
        bytecode.push(OpCode::JMP_L.byte());
        let jmp_end_operand = bytecode.len();
        bytecode.extend_from_slice(&[0, 0, 0, 0]);

        // not_null:
        let not_null_pos = bytecode.len();
        // Treat an empty ByteString/Buffer (missing storage entry) as default.
        // Stack before: [value]
        // Stack after:  [value] (if not missing) or [default]
        bytecode.push(OpCode::DUP.byte());
        bytecode.push(OpCode::SIZE.byte());
        bytecode.push(OpCode::PUSH0.byte());
        bytecode.push(OpCode::EQUAL.byte());

        // if !(size == 0) -> jump to not_missing
        let jmp_not_missing_pos = bytecode.len();
        bytecode.push(OpCode::JMPIFNOT_L.byte());
        let jmp_not_missing_operand = bytecode.len();
        bytecode.extend_from_slice(&[0, 0, 0, 0]);

        // missing case: drop value and push default
        bytecode.push(OpCode::DROP.byte());
        default_value(bytecode);

        // jump to end
        let jmp_end2_pos = bytecode.len();
        bytecode.push(OpCode::JMP_L.byte());
        let jmp_end2_operand = bytecode.len();
        bytecode.extend_from_slice(&[0, 0, 0, 0]);

        // not_missing:
        let not_missing_pos = bytecode.len();
        not_null(bytecode);

        // end:
        let end_pos = bytecode.len();

        let rel_not_null = (not_null_pos as i32)
            .checked_sub(jmp_not_null_pos as i32)
            .unwrap_or(0);
        bytecode[jmp_not_null_operand..jmp_not_null_operand + 4]
            .copy_from_slice(&rel_not_null.to_le_bytes());

        let rel_not_missing = (not_missing_pos as i32)
            .checked_sub(jmp_not_missing_pos as i32)
            .unwrap_or(0);
        bytecode[jmp_not_missing_operand..jmp_not_missing_operand + 4]
            .copy_from_slice(&rel_not_missing.to_le_bytes());

        let rel_end = (end_pos as i32)
            .checked_sub(jmp_end_pos as i32)
            .unwrap_or(0);
        bytecode[jmp_end_operand..jmp_end_operand + 4].copy_from_slice(&rel_end.to_le_bytes());

        let rel_end2 = (end_pos as i32)
            .checked_sub(jmp_end2_pos as i32)
            .unwrap_or(0);
        bytecode[jmp_end2_operand..jmp_end2_operand + 4].copy_from_slice(&rel_end2.to_le_bytes());
    }

    match ty {
        ValueType::Integer { .. } => {
            with_null_default(
                bytecode,
                |bytecode| bytecode.push(OpCode::PUSH0.byte()),
                |bytecode| {
                    // Coerce ByteString -> Integer via CONVERT.
                    // The old `ADD(ByteString, Integer(0))` approach
                    // concatenated on real NeoVM (mixed-type ADD → CAT),
                    // producing a 33-byte ByteString for 32-byte values.
                    bytecode.push(OpCode::CONVERT.byte());
                    bytecode.push(0x21); // StackItem type tag = Integer
                },
            );
        }
        ValueType::Boolean => {
            with_null_default(
                bytecode,
                |bytecode| bytecode.push(OpCode::PUSHF.byte()),
                |bytecode| {
                    // Coerce ByteString -> Boolean via NZ.
                    bytecode.push(OpCode::NZ.byte());
                },
            );
        }
        ValueType::String => {
            with_null_default(
                bytecode,
                |bytecode| push_data(bytecode, &[]),
                |_bytecode| {},
            );
        }
        ValueType::Address => {
            with_null_default(
                bytecode,
                |bytecode| push_data(bytecode, &[0u8; 20]),
                |_bytecode| {},
            );
        }
        ValueType::ByteArray { fixed_len } => {
            let default_bytes = fixed_len
                .map(|len| vec![0u8; len as usize])
                .unwrap_or_default();
            with_null_default(
                bytecode,
                |bytecode| push_data(bytecode, &default_bytes),
                |_bytecode| {},
            );
        }
        ValueType::Array(_) => {
            with_null_default(
                bytecode,
                |bytecode| bytecode.push(OpCode::PUSH0.byte()),
                |bytecode| {
                    // Storage arrays store their length at the base slot. `System.Storage.Get` returns a
                    // ByteString, so coerce to Integer to keep slot hashing / arithmetic stable.
                    bytecode.push(OpCode::PUSH0.byte());
                    bytecode.push(OpCode::ADD.byte());
                },
            );
        }
        _ => {}
    }
}
