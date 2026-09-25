use super::*;

impl NeoRuntime {
    /// Enable debugging mode
    pub fn enable_debugging(&mut self) {
        self.execution_context.enable_debugging();
    }

    /// Disable debugging mode
    pub fn disable_debugging(&mut self) {
        self.execution_context.disable_debugging();
    }

    /// Set breakpoint at instruction
    pub fn set_breakpoint(&mut self, instruction_pointer: u32) {
        self.execution_context.set_breakpoint(instruction_pointer);
    }

    /// Remove breakpoint
    pub fn remove_breakpoint(&mut self, instruction_pointer: u32) {
        self.execution_context
            .remove_breakpoint(instruction_pointer);
    }

    /// Step through execution
    pub fn step(&mut self) -> Result<execution::StepResult, RuntimeError> {
        self.execution_context
            .step()
            .map_err(|e| RuntimeError::ExecutionError {
                message: e.to_string(),
            })
    }
}
