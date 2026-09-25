use super::*;
use crate::opcode::OpCode;

impl ExecutionContext {
    pub(crate) fn execute_flow_exceptions(&mut self, opcode: u8) -> Result<bool, RuntimeError> {
        const ABORT: u8 = OpCode::ABORT.byte();
        const ASSERT: u8 = OpCode::ASSERT.byte();
        const THROW: u8 = OpCode::THROW.byte();

        match opcode {
            ABORT => {
                // ABORT
                return Err(RuntimeError::ExecutionError {
                    message: "ABORT instruction executed".to_string(),
                });
            }
            ASSERT => {
                // ASSERT
                let condition = self.pop_stack()?;
                if condition.is_truthy() {
                    self.instruction_pointer += 1;
                } else {
                    return Err(RuntimeError::ExecutionError {
                        message: "ASSERT failed".to_string(),
                    });
                }
            }
            THROW => {
                // THROW
                //
                // Task #27 (runtime slice) — preserve the raw stack-top bytes
                // verbatim in `self.revert_payload` BEFORE lossy-decoding them
                // into a UTF-8 String for the exception message. The bridge
                // reads `revert_payload` on the error path and routes it into
                // `ExecutionResult.return_data`, giving callers an EVM-style
                // `return_data = selector || abi.encode(args)` surface when
                // the compiler lowering pushes structured bytes (follow-up
                // slice) or when users hand-roll the payload today.
                let payload = Self::stack_item_to_bytes(self.pop_stack()?);
                self.revert_payload = payload.clone();
                let message = String::from_utf8_lossy(&payload);
                let message = if message.is_empty() {
                    "THROW".to_string()
                } else {
                    format!("THROW: {message}")
                };
                self.dispatch_exception(message)?;
            }
            _ => return Ok(false),
        }

        Ok(true)
    }
}
