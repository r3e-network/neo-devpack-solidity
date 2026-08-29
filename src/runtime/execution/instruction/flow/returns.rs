use super::*;
use crate::opcode::OpCode;

impl ExecutionContext {
    pub(crate) fn execute_flow_returns(&mut self, opcode: u8) -> Result<bool, RuntimeError> {
        const RET: u8 = OpCode::RET.byte();

        match opcode {
            RET => {
                // RET
                if self.call_stack.is_empty() {
                    if let Some(item) = self.stack.last() {
                        self.return_data = Self::stack_item_to_bytes(item.clone());
                    } else {
                        self.return_data.clear();
                    }
                    self.instruction_pointer = self.bytecode.len() as u32;
                } else {
                    self.return_from_function()?;
                }
            }
            _ => return Ok(false),
        }

        Ok(true)
    }
}
