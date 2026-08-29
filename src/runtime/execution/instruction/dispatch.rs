use super::*;

impl ExecutionContext {
    pub(crate) fn execute_instruction(&mut self, opcode: u8) -> Result<(), RuntimeError> {
        // Check and charge gas before execution (single charge point)
        let gas_cost = self.get_instruction_gas_cost(opcode);
        let projected_gas = self.gas_used.saturating_add(gas_cost);
        if projected_gas > self.gas_limit {
            return Err(RuntimeError::OutOfGas {
                used: projected_gas,
                limit: self.gas_limit,
            });
        }
        self.gas_used = projected_gas;

        let handled = self.execute_push_instruction(opcode)?
            || self.execute_flow_instruction(opcode)?
            || self.execute_syscall_instruction(opcode)?
            || self.execute_stack_instruction(opcode)?
            || self.execute_bytes_instruction(opcode)?
            || self.execute_arithmetic_instruction(opcode)?
            || self.execute_slots_instruction(opcode)?
            || self.execute_collection_instruction(opcode)?;

        if !handled {
            return Err(RuntimeError::ExecutionError {
                message: format!("Unsupported opcode: 0x{opcode:02X}"),
            });
        }

        Ok(())
    }
}
