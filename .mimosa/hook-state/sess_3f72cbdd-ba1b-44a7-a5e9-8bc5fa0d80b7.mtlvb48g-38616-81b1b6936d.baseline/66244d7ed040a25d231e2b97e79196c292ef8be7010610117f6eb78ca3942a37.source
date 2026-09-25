use super::*;
use crate::opcode::OpCode;

impl ExecutionContext {
    pub(crate) fn handle_contract_syscall(&mut self, name: &str) -> Result<bool, RuntimeError> {
        match name {
            "System.Contract.Call" => {
                self.handle_contract_call()?;
                Ok(true)
            }
            "System.Contract.GetCallFlags" => {
                // S6 fix: return the *active* CallFlags for this execution
                // (top-level = All = 0x0F; a restricted context set via
                // `override_call_flags` returns its armed value). Previously
                // hard-coded to 0x0F, hiding staticcall-shaped read-only
                // contexts and letting them write storage.
                self.push_stack(StackItem::UnsignedInteger(self.active_call_flags as u64))?;
                Ok(true)
            }
            "System.Contract.CreateStandardAccount" => {
                // Signature: CreateStandardAccount(pubkey: ByteString) -> UInt160
                let pubkey_item = self.pop_stack()?;
                let pubkey = Self::stack_item_to_bytes(pubkey_item);

                // Script = PUSH <pubkey> + SYSCALL System.Crypto.CheckSig
                let mut script = Vec::with_capacity(2 + pubkey.len() + 1 + 4);
                Self::append_pushdata(&mut script, &pubkey);
                script.push(OpCode::SYSCALL.byte());
                script
                    .extend_from_slice(&crate::interop::interop_id_bytes("System.Crypto.CheckSig"));

                // UInt160 = RIPEMD160(SHA256(script))
                let sha = Sha256::digest(&script);
                let hash160 = Ripemd160::digest(sha);
                self.push_stack(StackItem::byte_array(hash160[..].to_vec()))?;
                Ok(true)
            }
            "System.Contract.CreateMultisigAccount" => {
                // Signature: CreateMultisigAccount(m: Integer, pubkeys: Array) -> UInt160
                //
                // S4 fix: build the real Neo N3 multisig verification script
                //   PUSHINT <m> ; PUSHDATA <pk_1> ... PUSHDATA <pk_n> ;
                //   PUSHINT <n> ; SYSCALL System.Crypto.CheckMultisig
                // and return RIPEMD160(SHA256(script)) — matching the on-chain
                // account-derivation rule. The previous implementation returned
                // SHA256(m || pubkeys)[..20], which produced a UInt160 that
                // matched nothing on-chain.
                //
                // m and n are integers, so they use the PUSHINT8/16/32/64
                // encoding (mirroring how a real verification script would be
                // assembled); public keys are byte strings and use PUSHDATA1/2/4.
                let pubkeys_item = self.pop_stack()?;
                let m_item = self.pop_stack()?;
                let m: i64 = match m_item {
                    StackItem::Integer(i) => i,
                    StackItem::UnsignedInteger(u) => u as i64,
                    _ => {
                        return Err(RuntimeError::ExecutionError {
                            message: "CreateMultisigAccount: m must be an Integer".to_string(),
                        })
                    }
                };
                if m < 0 {
                    return Err(RuntimeError::ExecutionError {
                        message: "CreateMultisigAccount: m must be non-negative".to_string(),
                    });
                }

                let pubkeys: Vec<Vec<u8>> = match &pubkeys_item {
                    StackItem::Array(items) => items
                        .borrow()
                        .iter()
                        .map(|it| Self::stack_item_to_bytes(it.clone()))
                        .collect(),
                    _ => vec![Self::stack_item_to_bytes(pubkeys_item)],
                };

                let mut script: Vec<u8> = Vec::new();
                Self::append_push_int(&mut script, m);
                for pk in &pubkeys {
                    Self::append_pushdata(&mut script, pk);
                }
                Self::append_push_int(&mut script, pubkeys.len() as i64);
                script.push(OpCode::SYSCALL.byte());
                script.extend_from_slice(&crate::interop::interop_id_bytes(
                    "System.Crypto.CheckMultisig",
                ));

                // UInt160 = RIPEMD160(SHA256(script))
                let sha = Sha256::digest(&script);
                let hash160 = Ripemd160::digest(sha);
                self.push_stack(StackItem::byte_array(hash160[..].to_vec()))?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Append a `PUSHDATA1/2/4`-encoded operand to `script`.
    ///
    /// Shared by [`System.Contract.CreateStandardAccount`] and
    /// [`System.Contract.CreateMultisigAccount`] so the single-sig and
    /// multi-sig account-derivation paths emit byte-identical verification
    /// scripts for the same operand. This matters because the resulting
    /// `UInt160 = RIPEMD160(SHA256(script))` is the on-chain account hash and
    /// any encoding divergence would silently produce a different address.
    pub(crate) fn append_pushdata(script: &mut Vec<u8>, data: &[u8]) {
        script.push(OpCode::push_data(data.len()).byte());
        match data.len() {
            len if len <= u8::MAX as usize => {
                script.push(data.len() as u8);
            }
            len if len <= u16::MAX as usize => {
                script.extend_from_slice(&(data.len() as u16).to_le_bytes());
            }
            _ => {
                script.extend_from_slice(&(data.len() as u32).to_le_bytes());
            }
        }
        script.extend_from_slice(data);
    }

    /// Append a small integer to `script` using the NeoVM `PUSHINT8/16/32/64`
    /// opcodes, little-endian. Used by
    /// [`System.Contract.CreateMultisigAccount`] to emit the `m` and `n`
    /// operands of a multisig verification script the way a real Neo
    /// assembler would, so the derived account hash matches the on-chain
    /// derivation. Values must be non-negative.
    pub(crate) fn append_push_int(script: &mut Vec<u8>, value: i64) {
        // Match real Neo's `ScriptBuilder.EmitPush` (and the production helper
        // `emit_push_u32` in src/neo/contract_hash.rs): small integers use the
        // single-byte `PUSHM1`/`PUSH0..PUSH16` opcodes, NOT `PUSHINT8`. Multisig
        // `m`/`n` are <= 16, so the previous always-`PUSHINT8` form built a
        // different verification script and therefore a different account hash
        // than the chain derives.
        if value == -1 {
            script.push(OpCode::PUSHM1.byte());
        } else if value == 0 {
            script.push(OpCode::PUSH0.byte());
        } else if (1..=16).contains(&value) {
            // # Invariant: `value` is in 1..=16, which is exactly the range
            // `push_small` accepts. This cannot be None.
            script.push(
                OpCode::push_small(value as u8)
                    .expect("value in 1..=16")
                    .byte(),
            );
        } else if (i8::MIN as i64..=i8::MAX as i64).contains(&value) {
            script.push(OpCode::PUSHINT8.byte());
            script.push(value as i8 as u8);
        } else if (i16::MIN as i64..=i16::MAX as i64).contains(&value) {
            script.push(OpCode::PUSHINT16.byte());
            script.extend_from_slice(&(value as i16).to_le_bytes());
        } else if (i32::MIN as i64..=i32::MAX as i64).contains(&value) {
            script.push(OpCode::PUSHINT32.byte());
            script.extend_from_slice(&(value as i32).to_le_bytes());
        } else {
            script.push(OpCode::PUSHINT64.byte());
            script.extend_from_slice(&value.to_le_bytes());
        }
    }
}
