use super::*;

impl ExecutionContext {
    pub(crate) fn execute_flow_instruction(&mut self, opcode: u8) -> Result<bool, RuntimeError> {
        if self.execute_flow_jumps(opcode)? {
            return Ok(true);
        }
        if self.execute_flow_calls(opcode)? {
            return Ok(true);
        }
        if self.execute_flow_exceptions(opcode)? {
            return Ok(true);
        }
        if self.execute_flow_try_frames(opcode)? {
            return Ok(true);
        }
        if self.execute_flow_returns(opcode)? {
            return Ok(true);
        }

        Ok(false)
    }
}
