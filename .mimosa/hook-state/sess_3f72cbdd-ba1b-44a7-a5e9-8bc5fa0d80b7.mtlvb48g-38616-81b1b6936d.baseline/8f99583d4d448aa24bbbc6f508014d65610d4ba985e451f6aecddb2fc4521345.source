use crate::opcode::OpCode;

pub(crate) fn push_data(bytecode: &mut Vec<u8>, data: &[u8]) {
    bytecode.push(OpCode::push_data(data.len()).byte());
    match data.len() {
        len if len <= u8::MAX as usize => {
            bytecode.push(data.len() as u8);
        }
        len if len <= u16::MAX as usize => {
            bytecode.extend_from_slice(&(data.len() as u16).to_le_bytes());
        }
        _ => {
            bytecode.extend_from_slice(&(data.len() as u32).to_le_bytes());
        }
    }
    bytecode.extend_from_slice(data);
}
