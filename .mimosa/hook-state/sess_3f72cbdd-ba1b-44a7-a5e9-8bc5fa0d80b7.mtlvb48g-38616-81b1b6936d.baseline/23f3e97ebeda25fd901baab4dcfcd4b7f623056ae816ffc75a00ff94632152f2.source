use super::*;
use crate::opcode::OpCode;

impl ExecutionContext {
    pub(crate) fn execute_arithmetic_shifts(&mut self, opcode: u8) -> Result<bool, RuntimeError> {
        const SHL: u8 = OpCode::SHL.byte();
        const SHR: u8 = OpCode::SHR.byte();

        match opcode {
            SHL => {
                // SHL
                let shift = self.pop_stack()?;
                let value = self.pop_stack()?;
                let result = self.shift_left(value, shift)?;
                self.push_stack(result)?;
                self.instruction_pointer += 1;
                Ok(true)
            }
            SHR => {
                // SHR
                let shift = self.pop_stack()?;
                let value = self.pop_stack()?;
                let result = self.shift_right(value, shift)?;
                self.push_stack(result)?;
                self.instruction_pointer += 1;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
