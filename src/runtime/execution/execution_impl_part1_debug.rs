use super::*;

impl ExecutionContext {
    /// Enable debugging
    pub fn enable_debugging(&mut self) {
        self.debugging_enabled = true;
    }

    /// Disable debugging
    pub fn disable_debugging(&mut self) {
        self.debugging_enabled = false;
    }

    /// Set breakpoint
    pub fn set_breakpoint(&mut self, address: u32) {
        self.breakpoints.insert(address);
    }

    /// Remove breakpoint
    pub fn remove_breakpoint(&mut self, address: u32) {
        self.breakpoints.remove(&address);
    }

    /// Step through one instruction.
    ///
    /// PERF: `step()` runs once per executed instruction. The `opcode` name
    /// String and the full evaluation-stack clone in the returned `StepResult`
    /// exist only for debug tracing (the driver pushes them into a
    /// `StackFrame` when debugging is enabled; nothing else reads them), so
    /// they are computed ONLY when `debugging_enabled` — the hot path returns
    /// alloc-free `String::new()` / `Vec::new()` placeholders instead of a
    /// per-instruction heap String + up-to-2048-item stack clone.
    pub fn step(&mut self) -> Result<StepResult, RuntimeError> {
        if self.instruction_pointer as usize >= self.bytecode.len() {
            return Ok(StepResult {
                instruction_pointer: self.instruction_pointer,
                opcode: if self.debugging_enabled {
                    "HALT".to_string()
                } else {
                    String::new()
                },
                stack_items: if self.debugging_enabled {
                    self.stack.clone()
                } else {
                    Vec::new()
                },
                gas_used: self.gas_used,
                memory_changes: Vec::new(),
                halted: true,
            });
        }

        let opcode = self.bytecode[self.instruction_pointer as usize];
        let opcode_name = if self.debugging_enabled {
            self.get_opcode_name(opcode)
        } else {
            String::new()
        };
        let old_gas = self.gas_used;

        // Execute the current instruction
        match self.execute_instruction(opcode) {
            Ok(()) => {}
            Err(e) => {
                // OutOfGas and UNCATCHABLE VM faults (ExecutionEngineLimits
                // violations like MaxTryNestingDepth) terminate execution — they
                // are NOT routed into an enclosing TRY/catch, matching real
                // NeoVM `VMState.FAULT`. Without this, a limit fault is catchable
                // and bytecode can build a fault-storm (runtime_exec fuzzer OOM).
                if matches!(
                    e,
                    RuntimeError::OutOfGas { .. }
                        | RuntimeError::VmFault { .. }
                        | RuntimeError::StackOverflow { .. }
                ) {
                    return Err(e);
                }

                if self.try_stack.is_empty() {
                    return Err(e);
                }

                self.dispatch_exception(e.to_string())?;
            }
        }

        self.instruction_count += 1;
        self.max_stack_depth = self.max_stack_depth.max(self.stack.len() as u32);
        let halted = self.instruction_pointer as usize >= self.bytecode.len();

        Ok(StepResult {
            instruction_pointer: self.instruction_pointer,
            opcode: opcode_name,
            stack_items: if self.debugging_enabled {
                self.stack.clone()
            } else {
                Vec::new()
            },
            gas_used: self.gas_used - old_gas,
            memory_changes: Vec::new(), // Would track actual changes
            halted,
        })
    }
}
