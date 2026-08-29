use super::*;
use crate::opcode::OpCode;

impl ExecutionContext {
    pub(crate) fn execute_bytes_instruction(&mut self, opcode: u8) -> Result<bool, RuntimeError> {
        const NEWBUFFER: u8 = OpCode::NEWBUFFER.byte();
        const MEMCPY: u8 = OpCode::MEMCPY.byte();
        const CAT: u8 = OpCode::CAT.byte();
        const SUBSTR: u8 = OpCode::SUBSTR.byte();
        const LEFT: u8 = OpCode::LEFT.byte();
        const RIGHT: u8 = OpCode::RIGHT.byte();

        match opcode {
            NEWBUFFER => {
                // NEWBUFFER
                self.new_buffer()?;
                self.instruction_pointer += 1;
            }
            MEMCPY => {
                // MEMCPY
                self.memcpy_bytes()?;
                self.instruction_pointer += 1;
            }
            CAT => {
                // CAT
                self.concat_bytes()?;
                self.instruction_pointer += 1;
            }
            SUBSTR => {
                // SUBSTR
                self.substr_bytes()?;
                self.instruction_pointer += 1;
            }
            LEFT => {
                // LEFT
                self.left_bytes()?;
                self.instruction_pointer += 1;
            }
            RIGHT => {
                // RIGHT
                self.right_bytes()?;
                self.instruction_pointer += 1;
            }
            _ => return Ok(false),
        }

        Ok(true)
    }
}
