use super::*;
use crate::opcode::OpCode;

impl ExecutionContext {
    pub(crate) fn execute_arithmetic_bitwise(&mut self, opcode: u8) -> Result<bool, RuntimeError> {
        const INVERT: u8 = OpCode::INVERT.byte();
        const AND: u8 = OpCode::AND.byte();
        const OR: u8 = OpCode::OR.byte();
        const XOR: u8 = OpCode::XOR.byte();
        const EQUAL: u8 = OpCode::EQUAL.byte();
        const NOTEQUAL: u8 = OpCode::NOTEQUAL.byte();

        match opcode {
            INVERT => {
                // INVERT
                let value = self.pop_stack()?;
                let result = self.bitwise_not(value)?;
                self.push_stack(result)?;
                self.instruction_pointer += 1;
                Ok(true)
            }
            AND => {
                // AND
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                let result = self.bitwise_and(a, b)?;
                self.push_stack(result)?;
                self.instruction_pointer += 1;
                Ok(true)
            }
            OR => {
                // OR
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                let result = self.bitwise_or(a, b)?;
                self.push_stack(result)?;
                self.instruction_pointer += 1;
                Ok(true)
            }
            XOR => {
                // XOR
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                let result = self.bitwise_xor(a, b)?;
                self.push_stack(result)?;
                self.instruction_pointer += 1;
                Ok(true)
            }
            EQUAL => {
                // EQUAL
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                let result = self.stack_items_equal(&a, &b)?;
                self.push_stack(StackItem::Boolean(result))?;
                self.instruction_pointer += 1;
                Ok(true)
            }
            NOTEQUAL => {
                // NOTEQUAL
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                let result = self.stack_items_equal(&a, &b)?;
                self.push_stack(StackItem::Boolean(!result))?;
                self.instruction_pointer += 1;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
