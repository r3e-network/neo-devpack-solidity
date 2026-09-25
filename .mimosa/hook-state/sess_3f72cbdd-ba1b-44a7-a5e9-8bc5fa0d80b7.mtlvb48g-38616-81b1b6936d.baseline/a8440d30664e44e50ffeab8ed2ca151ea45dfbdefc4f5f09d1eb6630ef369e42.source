use super::*;

pub(crate) fn emit_load_storage_dynamic(bytecode: &mut Vec<u8>) {
    emit_syscall(bytecode, "System.Storage.GetContext");
    emit_syscall(bytecode, "System.Storage.Get");
}
