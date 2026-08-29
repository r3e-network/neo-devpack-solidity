use super::*;
use crate::opcode::OpCode;

/// S2 fix — per-byte gas surcharge for `Storage.Put`.
///
/// Neo N3 mainnet `Policy.storagePrice` is **100_000 gas per byte** for
/// storage writes (key length + value length). The previous `100`/byte rate
/// was ~1000× too cheap — contracts that fit the simulator's gas budget
/// would exhaust gas on-chain, and any gas-assertion test passed locally
/// while failing at deployment. Align to the mainnet rate of 100_000/byte so
/// the simulator's gas profile matches a real node.
///
/// A 32-byte write (16-byte key + 16-byte value) now costs ~3.2M gas; the
/// default `RuntimeConfig::gas_limit` of 10M accommodates ~3 such writes
/// before OOG — same headroom as mainnet for typical contracts.
const STORAGE_PUT_PER_BYTE_GAS: u64 = 100_000;

/// Wave-#14 Finding #4 — per-verify gas surcharge for `CheckMultisig`.
///
/// secp256k1 verification is ~50µs, comparable to other ~200K-gas crypto
/// ops; charge `1000` per inner verify. With 64×64 = 4096 verifies the
/// upfront charge is 4_096_000, which exceeds typical test gas budgets
/// (10M default) only for genuinely worst-case inputs.
const CHECKMULTISIG_PER_VERIFY_GAS: u64 = 1_000;

/// CryptoLib native contract hash (UInt160, internal little-endian byte
/// order). Mirrored from `spec::native_contracts::NATIVE_CONTRACTS` so
/// the per-byte hash surcharge can recognize the target without pulling
/// the full lookup-table into a constant context.
///
/// `HASH_PER_BYTE_GAS` (the per-byte rate) lives in
/// `instruction/flow/calls.rs` next to the CALLT dispatch — both
/// dispatch sites share the same rate.
const CRYPTOLIB_CONTRACT_HASH: [u8; 20] =
    *b"\x1b\xf5\x75\xab\x11\x89\x68\x84\x13\x61\x0a\x35\xa1\x28\x86\xcd\xe0\xb6\x6c\x72";

impl ExecutionContext {
    /// Compute extra gas owed by a syscall whose handler does work
    /// proportional to user-controlled stack inputs (Wave-#14 Findings
    /// #2/#4/#5). Peeks the top stack items WITHOUT popping; the handler
    /// still pops them as before. Returns 0 for syscalls without
    /// per-input scaling.
    fn syscall_extra_input_gas(&self, syscall_id: [u8; 4]) -> u64 {
        let Some(name) = spec::syscall_name(&syscall_id) else {
            return 0;
        };
        match name {
            // Finding #2 — Storage.Put scales with key.len + value.len.
            // Stack layout (see execution/syscalls/storage.rs for the
            // canonical order):
            //   System.Storage.Put — stack = [value, key, context]
            //                        (context on top, then key, then value)
            "System.Storage.Put" => {
                let depth = self.stack.len();
                if depth < 3 {
                    return 0;
                }
                let key_len = stack_item_byte_len(&self.stack[depth - 2]) as u64;
                let value_len = stack_item_byte_len(&self.stack[depth - 3]) as u64;
                key_len
                    .saturating_add(value_len)
                    .saturating_mul(STORAGE_PUT_PER_BYTE_GAS)
            }
            // Finding #4 — CheckMultisig runs O(N*M) secp256k1 verifies.
            // Stack at entry: [..., pubkeys, signatures] (signatures on
            // top).
            "System.Crypto.CheckMultisig" => {
                let depth = self.stack.len();
                if depth < 2 {
                    return 0;
                }
                let sig_count = stack_item_array_len(&self.stack[depth - 1]) as u64;
                let pub_count = stack_item_array_len(&self.stack[depth - 2]) as u64;
                sig_count
                    .saturating_mul(pub_count)
                    .saturating_mul(CHECKMULTISIG_PER_VERIFY_GAS)
            }
            // Finding #5 — CryptoLib hash methods routed through
            // `System.Contract.Call` (the default lowering when
            // `use_callt=false`) hash arbitrary-length input for free.
            // Charge per-byte upfront, mirroring the CALLT path.
            // Stack at entry: [args_array, flags, method, hash] with
            // `hash` on top — see
            // execution/execution_impl_part2_contract_call.rs.
            "System.Contract.Call" => {
                let depth = self.stack.len();
                if depth < 4 {
                    return 0;
                }
                let hash_item = &self.stack[depth - 1];
                let method_item = &self.stack[depth - 2];
                let args_item = &self.stack[depth - 4];

                // Compare contract hash bytewise against CryptoLib's.
                // The hash arrives as a 20-byte ByteArray.
                let hash_matches_cryptolib = match hash_item {
                    StackItem::ByteArray { data: bytes, .. } => {
                        let b = bytes.borrow();
                        b.len() == 20 && b.as_slice() == &CRYPTOLIB_CONTRACT_HASH[..]
                    }
                    _ => false,
                };
                if !hash_matches_cryptolib {
                    return 0;
                }

                // Decode method name (utf8 ByteArray) and check
                // membership in the per-byte-charge set.
                let method_bytes = match method_item {
                    StackItem::ByteArray { data: b, .. } => b.borrow().clone(),
                    _ => return 0,
                };
                let Ok(method_str) = std::str::from_utf8(&method_bytes) else {
                    return 0;
                };
                if !is_cryptolib_hash_method(method_str) {
                    return 0;
                }

                // The args array typically has a single ByteArray
                // element (the input to be hashed). Murmur32 takes
                // (data, seed) — only the data byte-length matters
                // for cost.
                let input_bytes = match args_item {
                    StackItem::Array(items) => {
                        items.borrow().first().map(stack_item_byte_len).unwrap_or(0)
                    }
                    other => stack_item_byte_len(other),
                };
                (input_bytes as u64).saturating_mul(HASH_PER_BYTE_GAS)
            }
            _ => 0,
        }
    }

    pub(crate) fn execute_syscall_instruction(&mut self, opcode: u8) -> Result<bool, RuntimeError> {
        const SYSCALL: u8 = OpCode::SYSCALL.byte();

        if opcode != SYSCALL {
            return Ok(false);
        }

        if self.instruction_pointer + 4 >= self.bytecode.len() as u32 {
            return Err(RuntimeError::ExecutionError {
                message: "SYSCALL: insufficient bytecode".to_string(),
            });
        }

        let mut syscall_id = [0u8; 4];
        syscall_id.copy_from_slice(
            &self.bytecode
                [self.instruction_pointer as usize + 1..self.instruction_pointer as usize + 5],
        );
        // Consume syscall-specific gas if known
        // NOTE: This is ADDITIONAL gas on top of the base opcode cost (already charged
        // at the start of execute_instruction). The syscall_gas map contains only the
        // extra cost for each syscall, not the total cost.
        if let Some(extra_cost) = self.syscall_gas.get(&syscall_id) {
            // Wave-#14 Findings #2/#4/#5 — fold per-input scaling into
            // the upfront charge so DoS-shaped inputs fail fast on gas
            // instead of running the (expensive) handler body. Use
            // `checked_add` and bail with a graceful OutOfGas on
            // overflow.
            let extra_input = self.syscall_extra_input_gas(syscall_id);
            let total_extra = match extra_cost.checked_add(extra_input) {
                Some(t) => t,
                None => {
                    return Err(RuntimeError::OutOfGas {
                        used: u64::MAX,
                        limit: self.gas_limit,
                    });
                }
            };
            // Use checked_add to detect overflow instead of saturating silently
            let projected = match self.gas_used.checked_add(total_extra) {
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
        self.handle_syscall(syscall_id)?;
        // Task #70: `handle_contract_call` may route a `this.someFn()` self
        // external call by rewiring `instruction_pointer` directly to the
        // target method's compiled offset. In that case the SYSCALL's usual
        // `+= 5` post-increment would land past the target's INITSLOT
        // prologue, so skip it.
        if self.syscall_suppress_ip_advance {
            self.syscall_suppress_ip_advance = false;
        } else {
            self.instruction_pointer += 5;
        }

        Ok(true)
    }
}

/// Approximate byte-length of a `StackItem` for per-byte gas pricing.
/// Mirrors the heuristic used by `Self::stack_item_to_bytes` so the
/// gas-pricing peek matches the bytes the handler will actually
/// process (plus a small fixed cost for non-bytes types). For
/// arrays/maps we fall back to a small constant since they aren't
/// legal Storage.Put inputs in practice — pricing them at zero would
/// let an attacker substitute an Array wrapper to dodge the surcharge.
fn stack_item_byte_len(item: &StackItem) -> usize {
    match item {
        StackItem::ByteArray { data: bytes, .. } => bytes.borrow().len(),
        StackItem::Integer(_) | StackItem::UnsignedInteger(_) => 8,
        StackItem::Boolean(_) => 1,
        StackItem::Null => 0,
        StackItem::Array(items) => items.borrow().len().max(32),
        StackItem::Map(map) => map.borrow().len().max(32),
    }
}

/// Element count of a `StackItem::Array`; 1 for non-array operands
/// (treated as a single "implicit" element by the CheckMultisig
/// handler).
fn stack_item_array_len(item: &StackItem) -> usize {
    match item {
        StackItem::Array(items) => items.borrow().len(),
        _ => 1,
    }
}

/// True for CryptoLib hash methods that take a single byte-array
/// argument and run in time linear in input length (Wave-#14
/// Finding #5). Compared case-insensitively because the compiler
/// emits the case it inherits from Solidity (e.g. "keccak256") while
/// the runtime invocation path lowercases — match either form.
fn is_cryptolib_hash_method(method: &str) -> bool {
    let lower = method.to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "sha256" | "keccak256" | "ripemd160" | "sha1" | "murmur32"
    )
}
