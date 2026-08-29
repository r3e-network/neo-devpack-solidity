use super::*;
use crate::opcode::OpCode;

pub(crate) fn emit_new_buffer(bytecode: &mut Vec<u8>) {
    bytecode.push(OpCode::NEWBUFFER.byte());
}

// Stack item type tags (operands of CONVERT/ISTYPE). These are NOT
// opcodes — they're the StackItem type tag bytes that follow the
// CONVERT (0xDB) or ISTYPE (0xD9) opcode in the script.
pub(crate) fn stack_item_type_code(target: ir::ConvertTarget) -> u8 {
    match target {
        ir::ConvertTarget::Any => 0x00,
        ir::ConvertTarget::Boolean => 0x20,
        ir::ConvertTarget::Integer => 0x21,
        ir::ConvertTarget::ByteArray => 0x28,
        ir::ConvertTarget::Array => 0x40,
        ir::ConvertTarget::Map => 0x48,
    }
}

pub(crate) fn emit_convert(bytecode: &mut Vec<u8>, target: ir::ConvertTarget) {
    bytecode.push(OpCode::CONVERT.byte());
    bytecode.push(stack_item_type_code(target));
}

pub(crate) fn emit_is_type(bytecode: &mut Vec<u8>, target: ir::ConvertTarget) {
    bytecode.push(OpCode::ISTYPE.byte());
    bytecode.push(stack_item_type_code(target));
}
