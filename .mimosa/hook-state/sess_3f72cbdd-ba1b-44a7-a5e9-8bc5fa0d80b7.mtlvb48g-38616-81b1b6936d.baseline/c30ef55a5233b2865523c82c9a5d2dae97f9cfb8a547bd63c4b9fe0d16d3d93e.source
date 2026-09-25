use super::*;
use crate::opcode::OpCode;

pub(crate) fn emit_load_struct_field(
    bytecode: &mut Vec<u8>,
    module: &ir::Module,
    state_index: usize,
    key_types: &[ValueType],
    field: StructFieldAccess<'_>,
    use_callt: bool,
    token_patches: &mut Vec<MethodTokenPatch>,
) {
    emit_struct_field_slot(
        bytecode,
        module,
        state_index,
        key_types,
        field.field_keys,
        use_callt,
        token_patches,
    );

    match field.ty {
        ValueType::Struct { fields, .. } => {
            emit_load_struct_value_from_slot(bytecode, fields, use_callt, token_patches);
        }
        _ => {
            emit_syscall(bytecode, "System.Storage.GetContext");
            emit_syscall(bytecode, "System.Storage.Get");
            emit_coerce_storage_value(bytecode, field.ty);
        }
    }
}

pub(crate) fn emit_store_struct_field(
    bytecode: &mut Vec<u8>,
    module: &ir::Module,
    state_index: usize,
    key_types: &[ValueType],
    field: StructFieldAccess<'_>,
    use_callt: bool,
    token_patches: &mut Vec<MethodTokenPatch>,
) {
    emit_struct_field_slot(
        bytecode,
        module,
        state_index,
        key_types,
        field.field_keys,
        use_callt,
        token_patches,
    );
    match field.ty {
        ValueType::Struct { fields, .. } => {
            bytecode.push(OpCode::SWAP.byte()); // SWAP -> [slot, value]
            emit_store_struct_value_to_slot(bytecode, fields, use_callt, token_patches);
        }
        _ => {
            emit_syscall(bytecode, "System.Storage.GetContext");
            emit_syscall(bytecode, "System.Storage.Put");
        }
    }
}
