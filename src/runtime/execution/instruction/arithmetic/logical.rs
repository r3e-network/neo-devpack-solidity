use super::*;
use crate::opcode::OpCode;

impl ExecutionContext {
    pub(crate) fn execute_arithmetic_logical(&mut self, opcode: u8) -> Result<bool, RuntimeError> {
        const NOT: u8 = OpCode::NOT.byte();
        const BOOLAND: u8 = OpCode::BOOLAND.byte();
        const BOOLOR: u8 = OpCode::BOOLOR.byte();
        const NZ: u8 = OpCode::NZ.byte();

        match opcode {
            NOT => {
                // NOT (logical)
                self.logical_not()?;
                self.instruction_pointer += 1;
                Ok(true)
            }
            BOOLAND => {
                // BOOLAND
                self.logical_and()?;
                self.instruction_pointer += 1;
                Ok(true)
            }
            BOOLOR => {
                // BOOLOR
                self.logical_or()?;
                self.instruction_pointer += 1;
                Ok(true)
            }
            NZ => {
                // NZ
                let value = self.pop_stack()?;
                self.push_stack(StackItem::Boolean(value.is_truthy()))?;
                self.instruction_pointer += 1;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
