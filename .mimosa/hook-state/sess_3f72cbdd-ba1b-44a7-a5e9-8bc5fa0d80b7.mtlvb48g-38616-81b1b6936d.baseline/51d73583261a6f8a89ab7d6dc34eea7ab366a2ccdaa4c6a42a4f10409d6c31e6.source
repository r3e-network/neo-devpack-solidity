use super::*;
use crate::opcode::OpCode;

pub(crate) fn emit_load_parameter(
    bytecode: &mut Vec<u8>,
    _method: &FunctionMetadata,
    index: usize,
) {
    if index <= 6 {
        bytecode.push(OpCode::ldarg(index as u8).byte());
    } else {
        bytecode.push(OpCode::LDARG.byte());
        bytecode.push(index as u8);
    }
}

pub(crate) fn emit_load_local(bytecode: &mut Vec<u8>, index: usize) {
    match index {
        0..=6 => bytecode.push(OpCode::ldloc(index as u8).byte()),
        _ => {
            bytecode.push(OpCode::LDLOC.byte());
            bytecode.push(index as u8);
        }
    }
}

pub(crate) fn emit_store_local(bytecode: &mut Vec<u8>, index: usize) {
    match index {
        0..=6 => bytecode.push(OpCode::stloc(index as u8).byte()),
        _ => {
            bytecode.push(OpCode::STLOC.byte());
            bytecode.push(index as u8);
        }
    }
}

// Task #156 — `(a, b) = (b, a)` on function parameters. NeoVM STARG0..6 are
// 0x80..=0x86; generic STARG is 0x87 with a one-byte index operand. Mirrors
// `emit_load_parameter` (0x78..=0x7E / 0x7F) so the store opcode pairs with
// the existing load for the same parameter slot.
pub(crate) fn emit_store_parameter(bytecode: &mut Vec<u8>, index: usize) {
    if index <= 6 {
        bytecode.push(OpCode::starg(index as u8).byte());
    } else {
        bytecode.push(OpCode::STARG.byte());
        bytecode.push(index as u8);
    }
}
