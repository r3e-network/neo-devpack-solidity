use super::*;
use crate::opcode::OpCode;
use crate::runtime::execution::syscalls::CALL_FLAG_ALLOW_CALL;

/// Wave-#14 Finding #5 — per-byte gas surcharge for CryptoLib hash
/// methods invoked via `CALLT`. Neo's native hash ops charge roughly
/// `byte_len * 50` so an 800KB input costs ~40M gas instead of a flat
/// 512. Matches the rate used by `System.Crypto.*` syscalls.
pub(crate) const HASH_PER_BYTE_GAS: u64 = 50;

/// Known CryptoLib hash methods that take a single byte-array argument
/// and run in time linear in input length.
fn cryptolib_hash_method_per_byte_charge(method: &str) -> bool {
    matches!(
        method.to_ascii_lowercase().as_str(),
        "sha256" | "keccak256" | "ripemd160" | "sha1" | "murmur32"
    )
}

impl ExecutionContext {
    pub(crate) fn execute_flow_calls(&mut self, opcode: u8) -> Result<bool, RuntimeError> {
        const CALL: u8 = OpCode::CALL.byte();
        const CALL_L: u8 = OpCode::CALL_L.byte();
        const CALLA: u8 = OpCode::CALLA.byte();
        const CALLT: u8 = OpCode::CALLT.byte();

        match opcode {
            CALL => {
                // CALL (1-byte signed relative offset from instruction start)
                let offset = self.read_i8_offset("CALL")? as i32;
                let base = self.instruction_pointer;
                let target = self.compute_offset_target("CALL", base, offset)?;
                self.push_call_frame(base + 2)?;
                self.instruction_pointer = target;
            }
            CALL_L => {
                // CALL_L (4-byte signed relative offset from instruction start)
                let offset = self.read_i32_offset("CALL_L")?;
                let base = self.instruction_pointer;
                let target = self.compute_offset_target("CALL_L", base, offset)?;
                self.push_call_frame(base + 5)?;
                self.instruction_pointer = target;
            }
            CALLA => {
                // CALLA (absolute address from stack)
                let target_usize = self.pop_usize("CALLA")?;
                let target =
                    u32::try_from(target_usize).map_err(|_| RuntimeError::ExecutionError {
                        message: "CALLA: target out of bounds".to_string(),
                    })?;
                if target as usize >= self.bytecode.len() {
                    return Err(RuntimeError::ExecutionError {
                        message: "CALLA: target out of bounds".to_string(),
                    });
                }
                let return_address = self.instruction_pointer + 1;
                self.push_call_frame(return_address)?;
                self.instruction_pointer = target;
            }
            CALLT => {
                // CALLT (token-based contract call; u16 token index)
                let start = self.instruction_pointer as usize + 1;
                let end = start + 2;
                if end > self.bytecode.len() {
                    return Err(RuntimeError::ExecutionError {
                        message: "CALLT: insufficient bytecode for token index".to_string(),
                    });
                }

                let index = u16::from_le_bytes([self.bytecode[start], self.bytecode[start + 1]]);
                let token = self
                    .method_tokens
                    .get(index as usize)
                    .ok_or_else(|| RuntimeError::ExecutionError {
                        message: format!(
                            "CALLT: token index {} out of bounds ({} token(s) loaded)",
                            index,
                            self.method_tokens.len()
                        ),
                    })?
                    .clone();

                // S6 fix — CALLT must honor the same trust boundary as
                // `System.Contract.Call`. Previously it jumped straight to the
                // native handler, letting a callee granted ReadStates-only flags
                // invoke state-changing natives, and letting a contract call a
                // native its manifest never permitted. These two gates mirror
                // `handle_contract_call` and fire BEFORE the stack is touched so
                // a rejecting context leaves the eval stack balanced.
                if self.active_call_flags & CALL_FLAG_ALLOW_CALL == 0 {
                    return Err(RuntimeError::ExecutionError {
                        message: "CALLT: CallFlag.AllowCall is required to invoke a contract"
                            .to_string(),
                    });
                }
                if !self.manifest_permits(&token.hash, &token.method) {
                    return Err(RuntimeError::ExecutionError {
                        message: format!(
                            "CALLT: manifest does not permit call to {} on 0x{}",
                            token.method,
                            hex::encode(token.hash.iter().rev().copied().collect::<Vec<u8>>())
                        ),
                    });
                }

                let arg_count = token.parameters_count as usize;
                if arg_count > self.stack.len() {
                    return Err(RuntimeError::ExecutionError {
                        message: format!(
                            "CALLT: insufficient stack items for {arg_count} argument(s)"
                        ),
                    });
                }

                let mut args = Vec::with_capacity(arg_count);
                for _ in 0..arg_count {
                    args.push(self.pop_stack()?);
                }
                args.reverse();

                // Wave-#14 Finding #5 — CALLT's flat 512 base allowed
                // attackers to hash arbitrary-length inputs (sha256,
                // keccak256, ripemd160, sha1, murmur32) for free.
                // Add a per-byte surcharge BEFORE invoking the native
                // handler so DoS-shaped inputs fail fast on gas.
                if spec::native_contract_name(&token.hash) == Some("CryptoLib")
                    && cryptolib_hash_method_per_byte_charge(&token.method)
                {
                    let input_bytes =
                        args.first().map(stack_item_byte_len_for_hash).unwrap_or(0) as u64;
                    let extra = input_bytes.saturating_mul(HASH_PER_BYTE_GAS);
                    let projected = match self.gas_used.checked_add(extra) {
                        Some(p) => p,
                        None => {
                            return Err(RuntimeError::OutOfGas {
                                used: u64::MAX,
                                limit: self.gas_limit,
                            });
                        }
                    };
                    if projected > self.gas_limit {
                        return Err(RuntimeError::OutOfGas {
                            used: projected,
                            limit: self.gas_limit,
                        });
                    }
                    self.gas_used = projected;
                }

                let params = StackItem::array(args);
                let result = self.invoke_native_contract(&token.hash, &token.method, params);
                if token.has_return_value {
                    self.push_stack(result)?;
                }

                self.instruction_pointer += 3;
            }
            _ => return Ok(false),
        }

        Ok(true)
    }

    pub(crate) fn push_call_frame(&mut self, return_address: u32) -> Result<(), RuntimeError> {
        if self.call_stack.len() as u32 >= self.call_stack_limit {
            // NeoVM MaxInvocationStackSize — UNCATCHABLE VMState.FAULT (see
            // `call_function`); prevents a CALL+TRY fault-storm DoS.
            return Err(RuntimeError::VmFault {
                message: "Call stack overflow (MaxInvocationStackSize)".to_string(),
            });
        }

        let frame = CallFrame {
            return_address,
            function_name: None,
            local_variables: HashMap::new(),
            stack_base: self.stack.len(),
            saved_locals: std::mem::take(&mut self.locals),
            saved_args: std::mem::take(&mut self.args),
            msg_sender_override: None,
            syscall_result_expected: false,
            storage_snapshot: None,
            saved_call_flags: None,
            expect_revert_guard: None,
        };
        self.call_stack.push(frame);
        Ok(())
    }
}

/// Byte-length of a `StackItem` for CALLT hash-method gas pricing.
/// Mirrors `Self::stack_item_to_bytes` so the surcharge tracks the
/// bytes the hash function actually digests. Non-bytes operands are
/// treated as their LE-encoded width to avoid a free path via integer
/// pushes (the handler converts them to bytes anyway).
fn stack_item_byte_len_for_hash(item: &StackItem) -> usize {
    match item {
        StackItem::ByteArray { data: bytes, .. } => bytes.borrow().len(),
        StackItem::Integer(_) | StackItem::UnsignedInteger(_) => 8,
        StackItem::Boolean(_) => 1,
        StackItem::Null => 0,
        StackItem::Array(items) => items.borrow().len().max(32),
        StackItem::Map(map) => map.borrow().len().max(32),
    }
}
