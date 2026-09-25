use super::*;
use crate::opcode::OpCode;

impl ExecutionContext {
    pub(crate) fn dispatch_exception(&mut self, message: String) -> Result<(), RuntimeError> {
        self.uncaught_exception = Some(message.clone());

        loop {
            let Some(frame) = self.try_stack.last().cloned() else {
                // No try frame catches this revert. If a neo-test
                // `vm.expectRevert` guard is armed on a call frame, the
                // expectation is satisfied: swallow the revert there and resume
                // as if the guarded call had returned. Inert for every normal
                // execution (no guard armed).
                if let Some(result) = self.try_satisfy_expect_revert() {
                    return result;
                }
                return Err(RuntimeError::ExecutionError { message });
            };

            // Exceptions thrown inside FINALLY blocks, or inside CATCH blocks without a FINALLY,
            // are not handled by that frame and must propagate to outer frames.
            if frame.state == TryFrameState::Finally
                || (frame.state == TryFrameState::Catch && frame.finally_target.is_none())
            {
                self.try_stack.pop();
                continue;
            }

            // TRY state + catch handler => enter catch and clear the exception.
            if frame.state == TryFrameState::Try {
                if let Some(catch_target) = frame.catch_target {
                    // If the throw happened inside a callee (i.e. there are
                    // call_stack frames pushed after this TRY frame's owner),
                    // unwind them. Otherwise the catch body's RET would pop
                    // the callee's frame and resume the TRY body's post-call
                    // continuation — causing the catch's return value to leak
                    // into the success path's expression (batch31 H2b).
                    while self.call_stack.len() > frame.owner_call_depth {
                        if let Some(callee_frame) = self.call_stack.pop() {
                            // Restore the owner's locals and args. Don't push
                            // a return value — the stack unwound to the catch
                            // handler pushes the exception message itself.
                            self.stack.truncate(callee_frame.stack_base);
                            self.locals = callee_frame.saved_locals;
                            self.args = callee_frame.saved_args;
                            // S7 fix — if this frame crossed a contract-call
                            // boundary (handle_contract_call's self-offsets
                            // arm), restore the storage overlay to the snapshot
                            // taken at entry. Without this, the callee's dirty
                            // storage writes leak into the caller's overlay and
                            // get committed at top-level halt — diverging from
                            // Neo N3, which reverts storage on any inner fault.
                            if let Some(snapshot) = callee_frame.storage_snapshot {
                                self.restore_storage_snapshot(snapshot);
                            }
                            // S6 fix — restore the flags captured at the
                            // contract-call boundary. Unwinding may pop
                            // several frames; restoring in pop order walks
                            // back through each intermediate flags state to
                            // the TRY owner's caller flags.
                            if let Some(caller_flags) = callee_frame.saved_call_flags {
                                self.active_call_flags = caller_flags;
                            }
                        }
                    }
                    if let Some(top) = self.try_stack.last_mut() {
                        top.state = TryFrameState::Catch;
                    }
                    // Task #86 — the catch handler receives the raw revert
                    // payload so `catch (bytes memory data)` sees the
                    // EVM-canonical envelope (`selector || abi.encode(args)`)
                    // produced by THROW. Falling back to the UTF-8 rendering
                    // of the exception message (the pre-Task-#86 behavior)
                    // would leak the `"THROW: …"` prefix and the lossy U+FFFD
                    // replacements for non-UTF-8 payload bytes into the
                    // caller, breaking ABI decoding.
                    // Bug-hunt #29 — a bare no-reason revert (`require(false)` /
                    // `revert()`) lowers to `PUSH null; THROW`, which sets an
                    // empty revert_payload and the internal marker message
                    // "THROW". EVM returndata for a no-reason revert is EMPTY, so
                    // the catch must see zero bytes — not the 5-byte marker. Only
                    // the bare "THROW" marker maps to empty; genuine VM-fault
                    // messages (e.g. "index out of bounds") are preserved for
                    // `catch (bytes)` fidelity.
                    let payload = if !self.revert_payload.is_empty() {
                        self.revert_payload.clone()
                    } else if message == "THROW" {
                        Vec::new()
                    } else {
                        message.as_bytes().to_vec()
                    };
                    self.push_stack(StackItem::byte_array(payload))?;
                    self.uncaught_exception = None;
                    self.revert_payload.clear();
                    self.instruction_pointer = catch_target;
                    return Ok(());
                }
            }

            // Otherwise, route into FINALLY (must exist).
            if let Some(finally_target) = frame.finally_target {
                if let Some(top) = self.try_stack.last_mut() {
                    top.state = TryFrameState::Finally;
                }
                self.instruction_pointer = finally_target;
                return Ok(());
            }

            // Defensive: malformed try frame (no catch or finally).
            self.try_stack.pop();
        }
    }

    /// neo-test `vm.expectRevert` support. If a call frame carries an armed
    /// expectation and the in-flight revert matches it, unwind to that frame
    /// (restoring storage / flags / locals exactly as the catch path does),
    /// resume as if the guarded call returned `Null`, and report the
    /// expectation as satisfied (`Some(Ok)`). Returns `None` when there is no
    /// guard, or the guard required a specific payload that does not match the
    /// actual revert — in which case the caller lets the revert propagate so
    /// the test fails with the real (unexpected) reason.
    fn try_satisfy_expect_revert(&mut self) -> Option<Result<(), RuntimeError>> {
        // Topmost guarded frame, if any.
        let guard_depth = self
            .call_stack
            .iter()
            .rposition(|f| f.expect_revert_guard.is_some())?;
        // `Some(Some(payload))` requires an exact match; `Some(None)` matches
        // any revert. A mismatch means the call reverted for the wrong reason —
        // do not swallow.
        if let Some(expected) = self.call_stack[guard_depth]
            .expect_revert_guard
            .clone()
            .flatten()
        {
            if expected != self.revert_payload {
                return None;
            }
        }
        // Unwind call frames from the top down to AND INCLUDING the guarded
        // frame, restoring each frame's caller state (mirror of the catch-path
        // unwinding above).
        let resume_ip;
        loop {
            let Some(callee) = self.call_stack.pop() else {
                return Some(Err(RuntimeError::ExecutionError {
                    message: "expectRevert: call-stack underflow".to_string(),
                }));
            };
            self.stack.truncate(callee.stack_base);
            self.locals = callee.saved_locals;
            self.args = callee.saved_args;
            if let Some(snapshot) = callee.storage_snapshot {
                self.restore_storage_snapshot(snapshot);
            }
            if let Some(flags) = callee.saved_call_flags {
                self.active_call_flags = flags;
            }
            if callee.expect_revert_guard.is_some() {
                resume_ip = callee.return_address;
                break;
            }
        }
        // The guarded call "returns" — push the single Null result the
        // external-call caller site expects (same contract as a void callee).
        if let Err(e) = self.push_stack(StackItem::Null) {
            return Some(Err(e));
        }
        self.uncaught_exception = None;
        self.revert_payload.clear();
        self.instruction_pointer = resume_ip;
        Some(Ok(()))
    }

    pub(crate) fn execute_flow_try_frames(&mut self, opcode: u8) -> Result<bool, RuntimeError> {
        const TRY: u8 = OpCode::TRY.byte();
        const TRY_L: u8 = OpCode::TRY_L.byte();
        const ENDTRY: u8 = OpCode::ENDTRY.byte();
        const ENDTRY_L: u8 = OpCode::ENDTRY_L.byte();
        const ENDFINALLY: u8 = OpCode::ENDFINALLY.byte();
        // NeoVM ExecutionEngineLimits.MaxTryNestingDepth — a TRY/TRY_L that
        // would exceed this many ACTIVE try-contexts in the CURRENT execution
        // context faults. Without it, a `TRY` loop with no matching ENDTRY grows
        // the try-stack without bound — an O(n) memory + O(n) time-per-op DoS.
        // (Found by the runtime_exec fuzzer: a bytecode of stacked TRYs ran for
        // 1.4s and OOM'd under a 100k gas budget.)
        const MAX_TRY_NESTING_DEPTH: usize = 16;

        match opcode {
            TRY => {
                // TRY CatchOffset(sbyte) FinallyOffset(sbyte)
                let catch_offset = self.read_i8_offset("TRY")? as i32;
                let finally_offset = {
                    let idx = self.instruction_pointer as usize + 2;
                    if idx >= self.bytecode.len() {
                        return Err(RuntimeError::ExecutionError {
                            message: "TRY: insufficient bytecode for offset".to_string(),
                        });
                    }
                    (self.bytecode[idx] as i8) as i32
                };

                if catch_offset == 0 && finally_offset == 0 {
                    return Err(RuntimeError::ExecutionError {
                        message: "TRY: catchOffset and finallyOffset cannot both be 0".to_string(),
                    });
                }

                let catch_target = if catch_offset == 0 {
                    None
                } else {
                    Some(self.compute_offset_target(
                        "TRY",
                        self.instruction_pointer,
                        catch_offset,
                    )?)
                };

                let finally_target = if finally_offset == 0 {
                    None
                } else {
                    Some(self.compute_offset_target(
                        "TRY",
                        self.instruction_pointer,
                        finally_offset,
                    )?)
                };

                let current_depth = self
                    .try_stack
                    .iter()
                    .filter(|f| f.owner_call_depth == self.call_stack.len())
                    .count();
                if current_depth >= MAX_TRY_NESTING_DEPTH {
                    return Err(RuntimeError::VmFault {
                        message: format!(
                            "TRY: nesting depth exceeds NeoVM MaxTryNestingDepth ({MAX_TRY_NESTING_DEPTH})"
                        ),
                    });
                }
                self.try_stack.push(TryFrame {
                    catch_target,
                    finally_target,
                    end_target: None,
                    state: TryFrameState::Try,
                    owner_call_depth: self.call_stack.len(),
                });

                self.instruction_pointer += 3;
            }
            TRY_L => {
                // TRY_L CatchOffset(int) FinallyOffset(int)
                let catch_offset = self.read_i32_offset("TRY_L")?;
                let finally_offset = {
                    let start = self.instruction_pointer as usize + 5;
                    let end = start + 4;
                    if end > self.bytecode.len() {
                        return Err(RuntimeError::ExecutionError {
                            message: "TRY_L: insufficient bytecode for offset".to_string(),
                        });
                    }
                    let mut buf = [0u8; 4];
                    buf.copy_from_slice(&self.bytecode[start..end]);
                    i32::from_le_bytes(buf)
                };

                if catch_offset == 0 && finally_offset == 0 {
                    return Err(RuntimeError::ExecutionError {
                        message: "TRY_L: catchOffset and finallyOffset cannot both be 0"
                            .to_string(),
                    });
                }

                let catch_target = if catch_offset == 0 {
                    None
                } else {
                    Some(self.compute_offset_target(
                        "TRY_L",
                        self.instruction_pointer,
                        catch_offset,
                    )?)
                };

                let finally_target = if finally_offset == 0 {
                    None
                } else {
                    Some(self.compute_offset_target(
                        "TRY_L",
                        self.instruction_pointer,
                        finally_offset,
                    )?)
                };

                let current_depth = self
                    .try_stack
                    .iter()
                    .filter(|f| f.owner_call_depth == self.call_stack.len())
                    .count();
                if current_depth >= MAX_TRY_NESTING_DEPTH {
                    return Err(RuntimeError::VmFault {
                        message: format!(
                            "TRY_L: nesting depth exceeds NeoVM MaxTryNestingDepth ({MAX_TRY_NESTING_DEPTH})"
                        ),
                    });
                }
                self.try_stack.push(TryFrame {
                    catch_target,
                    finally_target,
                    end_target: None,
                    state: TryFrameState::Try,
                    owner_call_depth: self.call_stack.len(),
                });

                self.instruction_pointer += 9;
            }
            ENDTRY => {
                // ENDTRY: endOffset(sbyte)
                if self.try_stack.is_empty() {
                    return Err(RuntimeError::ExecutionError {
                        message: "ENDTRY: corresponding TRY block not found".to_string(),
                    });
                }

                if self
                    .try_stack
                    .last()
                    .is_some_and(|frame| frame.state == TryFrameState::Finally)
                {
                    return Err(RuntimeError::ExecutionError {
                        message: "ENDTRY: cannot execute inside FINALLY".to_string(),
                    });
                }

                let end_offset = self.read_i8_offset("ENDTRY")? as i32;
                let end_target =
                    self.compute_offset_target("ENDTRY", self.instruction_pointer, end_offset)?;

                let finally_target = self.try_stack.last().and_then(|frame| frame.finally_target);

                if let Some(finally_target) = finally_target {
                    if let Some(frame) = self.try_stack.last_mut() {
                        frame.state = TryFrameState::Finally;
                        frame.end_target = Some(end_target);
                    }
                    self.instruction_pointer = finally_target;
                } else {
                    self.try_stack.pop();
                    self.instruction_pointer = end_target;
                }
            }
            ENDTRY_L => {
                // ENDTRY_L: endOffset(int)
                if self.try_stack.is_empty() {
                    return Err(RuntimeError::ExecutionError {
                        message: "ENDTRY_L: corresponding TRY block not found".to_string(),
                    });
                }

                if self
                    .try_stack
                    .last()
                    .is_some_and(|frame| frame.state == TryFrameState::Finally)
                {
                    return Err(RuntimeError::ExecutionError {
                        message: "ENDTRY_L: cannot execute inside FINALLY".to_string(),
                    });
                }

                let end_offset = self.read_i32_offset("ENDTRY_L")?;
                let end_target =
                    self.compute_offset_target("ENDTRY_L", self.instruction_pointer, end_offset)?;

                let finally_target = self.try_stack.last().and_then(|frame| frame.finally_target);

                if let Some(finally_target) = finally_target {
                    if let Some(frame) = self.try_stack.last_mut() {
                        frame.state = TryFrameState::Finally;
                        frame.end_target = Some(end_target);
                    }
                    self.instruction_pointer = finally_target;
                } else {
                    self.try_stack.pop();
                    self.instruction_pointer = end_target;
                }
            }
            ENDFINALLY => {
                // ENDFINALLY
                let Some(frame) = self.try_stack.pop() else {
                    return Err(RuntimeError::ExecutionError {
                        message: "ENDFINALLY: corresponding TRY block not found".to_string(),
                    });
                };

                if self.uncaught_exception.is_none() {
                    let Some(end_target) = frame.end_target else {
                        return Err(RuntimeError::ExecutionError {
                            message: "ENDFINALLY: ENDTRY target not set".to_string(),
                        });
                    };

                    self.instruction_pointer = end_target;
                } else {
                    let message = self
                        .uncaught_exception
                        .clone()
                        .unwrap_or_else(|| "Unhandled exception".to_string());
                    self.dispatch_exception(message)?;
                }
            }
            _ => return Ok(false),
        }

        Ok(true)
    }
}
