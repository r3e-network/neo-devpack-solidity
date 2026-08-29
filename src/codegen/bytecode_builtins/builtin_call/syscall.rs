use super::*;
use crate::opcode::OpCode;

pub(crate) fn emit_syscall_builtin(bytecode: &mut Vec<u8>, name: &str, arg_count: usize) {
    if name == "System.Storage.Find" && arg_count == 2 {
        // Neo N3 Storage.Find(context, prefix, options)
        // Emit PUSH0 (= FindOptions.None, 0) for the missing `options` arg.
        bytecode.push(OpCode::PUSH0.byte());
    }
    emit_syscall(bytecode, name);
    if is_void_syscall(name) {
        bytecode.push(OpCode::PUSH1.byte()); // PUSH1 for void syscalls
    }
}
