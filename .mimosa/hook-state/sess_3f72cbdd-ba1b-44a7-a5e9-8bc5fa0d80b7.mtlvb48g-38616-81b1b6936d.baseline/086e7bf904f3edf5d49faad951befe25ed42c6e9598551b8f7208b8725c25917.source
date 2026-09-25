use super::*;

impl VMBridge {
    /// Execute bytecode through the bridge
    pub fn execute(
        &mut self,
        context: &mut execution::ExecutionContext,
        _state: &mut state::StateManager,
        storage: &mut storage::StorageManager,
        gas: &mut execution::GasTracker,
    ) -> Result<ExecutionResult, RuntimeError> {
        context.bind_storage(&self.contract_account, storage)?;

        let mut state_changes = Vec::new();
        let mut stack_trace = Vec::new();

        loop {
            gas.sync_from_execution(context.gas_used());

            // Check gas limit using execution context accounting
            if gas.out_of_gas() {
                let result = ExecutionResult {
                    success: false,
                    return_data: Vec::new(),
                    gas_used: gas.used(),
                    gas_limit: gas.limit(),
                    exception: Some(RuntimeException {
                        exception_type: ExceptionType::OutOfGas,
                        message: "Execution ran out of gas".to_string(),
                        instruction_pointer: Some(context.instruction_count() as u32),
                        stack_trace: stack_trace.clone(),
                    }),
                    state_changes,
                    logs: context.logs().to_vec(),
                    stack_trace: Some(stack_trace),
                    metadata: ExecutionMetadata::default(),
                };
                context.unbind_storage();
                return Ok(result);
            }

            // Execute single step
            match context.step() {
                Ok(step_result) => {
                    gas.sync_from_execution(context.gas_used());

                    if step_result.halted {
                        let modified_accounts = self.apply_storage_overlay(context, storage)?;
                        for account in modified_accounts {
                            let changes = storage.commit(&account)?;
                            for change in changes {
                                state_changes.push(StateChange {
                                    change_type: crate::runtime::StateChangeType::StorageChange,
                                    account: account.clone(),
                                    key: Some(change.key),
                                    old_value: change.old_value,
                                    new_value: change.new_value.unwrap_or_default(),
                                });
                            }
                        }

                        gas.sync_from_execution(context.gas_used());

                        let result = ExecutionResult {
                            success: true,
                            return_data: self.extract_return_data(context)?,
                            gas_used: gas.used(),
                            gas_limit: gas.limit(),
                            exception: None,
                            state_changes,
                            logs: context.logs().to_vec(),
                            stack_trace: None,
                            metadata: ExecutionMetadata::default(),
                        };
                        context.unbind_storage();
                        return Ok(result);
                    }

                    // Add to stack trace if debugging enabled
                    if self.config.enable_debugging {
                        stack_trace.push(StackFrame {
                            function_name: None,
                            instruction_pointer: step_result.instruction_pointer,
                            opcode: step_result.opcode,
                            stack_items: step_result.stack_items,
                            local_variables: HashMap::new(),
                        });
                    }
                }
                Err(RuntimeError::OutOfGas { .. }) => {
                    gas.sync_from_execution(context.gas_used());

                    let result = ExecutionResult {
                        success: false,
                        return_data: Vec::new(),
                        gas_used: gas.used(),
                        gas_limit: gas.limit(),
                        exception: Some(RuntimeException {
                            exception_type: ExceptionType::OutOfGas,
                            message: "Execution ran out of gas".to_string(),
                            instruction_pointer: Some(context.instruction_count() as u32),
                            stack_trace: stack_trace.clone(),
                        }),
                        state_changes,
                        logs: context.logs().to_vec(),
                        stack_trace: Some(stack_trace),
                        metadata: ExecutionMetadata::default(),
                    };
                    context.unbind_storage();
                    return Ok(result);
                }
                Err(e) => {
                    gas.sync_from_execution(context.gas_used());

                    // Task #26 — distinguish Solidity-emitted THROW (user
                    // `revert`, `require`, custom errors, assert) from raw
                    // VM faults (invalid opcode, stack underflow, malformed
                    // jumps, ABORT, etc.).
                    //
                    // The IR lowers `revert Err(x)` / `revert "msg"` /
                    // `require(cond, msg)` / `assert(cond)` to the NeoVM
                    // `THROW` opcode. `ExecutionContext::execute_flow_exceptions`
                    // constructs the error message as either `"THROW"` (no
                    // payload) or `"THROW: <name_or_string>"` and routes it
                    // through `dispatch_exception`, which — when no `TRY`
                    // frame is present — surfaces as
                    // `RuntimeError::ExecutionError { message }` whose
                    // thiserror `Display` prefixes `"Execution failed: "`.
                    //
                    // We treat a substring of `"THROW"` in the rendered
                    // error as the discriminator: present → structured
                    // Solidity revert → `ExceptionType::RevertExecution`;
                    // absent → genuine VM fault → `ExceptionType::Fault`.
                    // This keeps VM faults indistinguishable from real
                    // panics (as callers expect) while letting tooling
                    // treat user-land reverts as recoverable.
                    //
                    // Note: `dispatch_exception` also rethrows messages via
                    // the `ENDFINALLY` path when an outer frame is absent;
                    // that path preserves the original string, so THROW-
                    // sourced messages still carry the `"THROW"` marker
                    // after a `finally` re-raise.
                    let rendered = e.to_string();
                    // M-RT4 fix — discriminate revert (THROW) vs VM fault by
                    // the revert_payload marker, NOT by string-matching the
                    // rendered Display. The THROW handler
                    // (exceptions.rs:35) sets `revert_payload` to the raw
                    // pushed bytes; a genuine VM fault (invalid opcode, stack
                    // underflow, ABORT) never touches it. The previous
                    // `rendered.contains("THROW")` check misclassified any
                    // fault whose message happened to contain "THROW" (e.g. a
                    // user `revert "THROW"` payload decoded as UTF-8, or an
                    // ABORTMSG with "THROW" in its text).
                    let has_revert_payload = !context.revert_payload().is_empty();
                    let exception_type = if has_revert_payload {
                        ExceptionType::RevertExecution
                    } else {
                        ExceptionType::Fault
                    };

                    // Task #27 (runtime slice) — on a Solidity-emitted revert
                    // (RevertExecution), surface the raw THROW payload bytes
                    // captured by `execute_flow_exceptions` as the EVM-style
                    // `return_data`. Today the IR lowerer drops revert args
                    // and pushes only the error-name bytes, so for
                    // `revert TooSmall(7)` this surfaces `b"TooSmall"`.
                    // When the compiler slice of Task #27 lands, the same
                    // plumbing carries the full `selector || abi.encode(args)`
                    // payload unchanged.
                    //
                    // VM faults (invalid opcode, stack underflow, ABORT, …)
                    // never push a meaningful payload, so we leave
                    // `return_data` empty for `ExceptionType::Fault`.
                    let return_data = if matches!(exception_type, ExceptionType::RevertExecution) {
                        context.revert_payload().to_vec()
                    } else {
                        Vec::new()
                    };

                    let result = ExecutionResult {
                        success: false,
                        return_data,
                        gas_used: gas.used(),
                        gas_limit: gas.limit(),
                        exception: Some(RuntimeException {
                            exception_type,
                            message: rendered,
                            instruction_pointer: Some(context.instruction_count() as u32),
                            stack_trace: stack_trace.clone(),
                        }),
                        state_changes,
                        logs: context.logs().to_vec(),
                        stack_trace: Some(stack_trace),
                        metadata: ExecutionMetadata::default(),
                    };
                    context.unbind_storage();
                    return Ok(result);
                }
            }
        }
    }
}
