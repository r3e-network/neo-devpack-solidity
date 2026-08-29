use super::*;
use crate::opcode::OpCode;

impl ExecutionContext {
    pub(crate) fn execute_arithmetic_numeric_unary(
        &mut self,
        opcode: u8,
    ) -> Result<bool, RuntimeError> {
        const SIGN: u8 = OpCode::SIGN.byte();
        const ABS: u8 = OpCode::ABS.byte();
        const NEGATE: u8 = OpCode::NEGATE.byte();
        const INC: u8 = OpCode::INC.byte();
        const DEC: u8 = OpCode::DEC.byte();
        const SQRT: u8 = OpCode::SQRT.byte();

        match opcode {
            SIGN => {
                // SIGN
                let value = self.pop_stack()?;
                let result = self.sign_stack_item(value)?;
                self.push_stack(StackItem::Integer(result))?;
                self.instruction_pointer += 1;
                Ok(true)
            }
            ABS => {
                // ABS
                let value = self.pop_stack()?;
                let result = self.abs_stack_item(value)?;
                self.push_stack(result)?;
                self.instruction_pointer += 1;
                Ok(true)
            }
            NEGATE => {
                // NEGATE
                let value = self.pop_stack()?;
                let result = self.negate_stack_item(value)?;
                self.push_stack(result)?;
                self.instruction_pointer += 1;
                Ok(true)
            }
            INC => {
                // INC
                let value = self.pop_stack()?;
                let result = self.inc_stack_item(value)?;
                self.push_stack(result)?;
                self.instruction_pointer += 1;
                Ok(true)
            }
            DEC => {
                // DEC
                let value = self.pop_stack()?;
                let result = self.dec_stack_item(value)?;
                self.push_stack(result)?;
                self.instruction_pointer += 1;
                Ok(true)
            }
            SQRT => {
                // SQRT
                let value = self.pop_stack()?;
                let result = self.sqrt_stack_item(value)?;
                self.push_stack(result)?;
                self.instruction_pointer += 1;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
