use super::*;
use crate::opcode::OpCode;

pub(crate) fn emit_storage_find(bytecode: &mut Vec<u8>, arg_count: usize) {
    // Stack input: [prefix] or [prefix, options]
    // Neo N3 Find signature: Find(context, prefix, options)
    // Stack order for syscall: [options, prefix, context]
    if arg_count == 1 {
        // PUSH0 (which evaluates to 0, i.e. FindOptions.None on the stack).
        bytecode.push(OpCode::PUSH0.byte());
        bytecode.push(OpCode::SWAP.byte()); // SWAP -> [options, prefix]
    } else {
        // Assume the last arg is `options`.
        bytecode.push(OpCode::SWAP.byte()); // SWAP -> [options, prefix]
    }
    emit_syscall(bytecode, "System.Storage.GetContext");
    emit_syscall(bytecode, "System.Storage.Find");
}

pub(crate) fn emit_storage_put(bytecode: &mut Vec<u8>) {
    // Stack input: [key, value]
    // Stack order for System.Storage.Put: [value, key, context]
    bytecode.push(OpCode::SWAP.byte()); // SWAP -> [value, key]
    emit_syscall(bytecode, "System.Storage.GetContext");
    emit_syscall(bytecode, "System.Storage.Put");
    bytecode.push(OpCode::PUSH1.byte()); // PUSH1 to avoid stack underflow on statement calls
}

pub(crate) fn emit_storage_get(bytecode: &mut Vec<u8>) {
    emit_syscall(bytecode, "System.Storage.GetReadOnlyContext");
    emit_syscall(bytecode, "System.Storage.Get");
}

pub(crate) fn emit_storage_delete(bytecode: &mut Vec<u8>) {
    emit_syscall(bytecode, "System.Storage.GetContext");
    emit_syscall(bytecode, "System.Storage.Delete");
    bytecode.push(OpCode::PUSH1.byte());
}
