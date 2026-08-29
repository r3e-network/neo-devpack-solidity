use super::*;
use crate::opcode::OpCode;

impl ExecutionContext {
    pub(crate) fn execute_arithmetic_comparison(
        &mut self,
        opcode: u8,
    ) -> Result<bool, RuntimeError> {
        const NUMEQUAL: u8 = OpCode::NUMEQUAL.byte();
        const NUMNOTEQUAL: u8 = OpCode::NUMNOTEQUAL.byte();
        const LT: u8 = OpCode::LT.byte();
        const LE: u8 = OpCode::LE.byte();
        const GT: u8 = OpCode::GT.byte();
        const GE: u8 = OpCode::GE.byte();
        const MIN: u8 = OpCode::MIN.byte();
        const MAX: u8 = OpCode::MAX.byte();
        const WITHIN: u8 = OpCode::WITHIN.byte();

        match opcode {
            NUMEQUAL => {
                // NUMEQUAL
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                let result = self.stack_items_equal(&a, &b)?;
                self.push_stack(StackItem::Boolean(result))?;
                self.instruction_pointer += 1;
                Ok(true)
            }
            NUMNOTEQUAL => {
                // NUMNOTEQUAL
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                let result = self.stack_items_equal(&a, &b)?;
                self.push_stack(StackItem::Boolean(!result))?;
                self.instruction_pointer += 1;
                Ok(true)
            }
            LT => {
                // LT
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                let result = self.less_than(&a, &b)?;
                self.push_stack(StackItem::Boolean(result))?;
                self.instruction_pointer += 1;
                Ok(true)
            }
            LE => {
                // LE
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                let lt = self.less_than(&a, &b)?;
                let eq = self.stack_items_equal(&a, &b)?;
                self.push_stack(StackItem::Boolean(lt || eq))?;
                self.instruction_pointer += 1;
                Ok(true)
            }
            GT => {
                // GT
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                let result = self.greater_than(&a, &b)?;
                self.push_stack(StackItem::Boolean(result))?;
                self.instruction_pointer += 1;
                Ok(true)
            }
            GE => {
                // GE
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                let gt = self.greater_than(&a, &b)?;
                let eq = self.stack_items_equal(&a, &b)?;
                self.push_stack(StackItem::Boolean(gt || eq))?;
                self.instruction_pointer += 1;
                Ok(true)
            }
            MIN => {
                // MIN
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                let result = self.min_stack_items(a, b)?;
                self.push_stack(result)?;
                self.instruction_pointer += 1;
                Ok(true)
            }
            MAX => {
                // MAX
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                let result = self.max_stack_items(a, b)?;
                self.push_stack(result)?;
                self.instruction_pointer += 1;
                Ok(true)
            }
            WITHIN => {
                // WITHIN (left inclusive)
                let max_item = self.pop_stack()?;
                let min_item = self.pop_stack()?;
                let value = self.pop_stack()?;
                let result = self.within_stack_items(value, min_item, max_item)?;
                self.push_stack(StackItem::Boolean(result))?;
                self.instruction_pointer += 1;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
