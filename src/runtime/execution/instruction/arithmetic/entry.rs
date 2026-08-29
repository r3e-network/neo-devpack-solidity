use super::*;

impl ExecutionContext {
    pub(crate) fn execute_arithmetic_instruction(
        &mut self,
        opcode: u8,
    ) -> Result<bool, RuntimeError> {
        if self.execute_arithmetic_bitwise(opcode)? {
            return Ok(true);
        }
        if self.execute_arithmetic_numeric_unary(opcode)? {
            return Ok(true);
        }
        if self.execute_arithmetic_binary(opcode)? {
            return Ok(true);
        }
        if self.execute_arithmetic_shifts(opcode)? {
            return Ok(true);
        }
        if self.execute_arithmetic_logical(opcode)? {
            return Ok(true);
        }
        if self.execute_arithmetic_comparison(opcode)? {
            return Ok(true);
        }

        Ok(false)
    }
}
