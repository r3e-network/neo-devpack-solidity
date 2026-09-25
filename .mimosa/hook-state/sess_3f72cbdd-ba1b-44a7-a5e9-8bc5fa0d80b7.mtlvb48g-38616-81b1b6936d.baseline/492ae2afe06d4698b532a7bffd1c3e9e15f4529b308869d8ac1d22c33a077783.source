use super::*;
use crate::opcode::OpCode;

pub(crate) fn emit_new_array(bytecode: &mut Vec<u8>) {
    bytecode.push(OpCode::NEWARRAY.byte());
}

pub(crate) fn emit_array_get(bytecode: &mut Vec<u8>) {
    bytecode.push(OpCode::PICKITEM.byte());
}

pub(crate) fn emit_array_set(bytecode: &mut Vec<u8>) {
    bytecode.push(OpCode::SETITEM.byte());
}

pub(crate) fn emit_load_runtime_value(
    bytecode: &mut Vec<u8>,
    value: &ir::RuntimeValue,
    use_callt: bool,
    token_patches: &mut Vec<MethodTokenPatch>,
) {
    match value {
        ir::RuntimeValue::MsgSender => {
            // Solidity `msg.sender` is the immediate caller:
            // - entry contract: Transaction.Sender (first signer)
            // - internal contract call: CallingScriptHash (caller contract)
            // - constructor (called from ContractManagement during deploy):
            //   ContractManagement.Hash would be a useless answer; the user
            //   actually wants the deploying signer, so we route to
            //   Transaction.Sender for that case too.
            //
            // Detection: check if CallingScriptHash matches EntryScriptHash
            // (top-level user invocation) OR ContractManagement.Hash (running
            // inside _deploy during a contract deploy). Either way, the
            // semantically correct `msg.sender` is the transaction's first
            // signer (Transaction.Sender). Otherwise we're in an internal
            // contract-to-contract call and CallingScriptHash is correct.
            //
            // ContractManagement native hash on Neo N3:
            //   0xfffdc93764dbaddd97c48f252a53ea4643faa3fd  (big-endian)
            // Stored on-chain in little-endian byte order, which is how
            // CallingScriptHash compares it.

            // Push CallingScriptHash, then test against EntryScriptHash and
            // ContractManagement.Hash. The result is whether either match.
            emit_syscall(bytecode, "System.Runtime.GetCallingScriptHash");
            bytecode.push(OpCode::DUP.byte()); // DUP — keep one copy for the actual return path

            // First test: == EntryScriptHash
            emit_syscall(bytecode, "System.Runtime.GetEntryScriptHash");
            bytecode.push(OpCode::EQUAL.byte());

            // Second test: == ContractManagement.Hash (LE bytes)
            // Stack: [calling_dup, calling_eq_entry?]
            bytecode.push(OpCode::SWAP.byte()); // SWAP — bring calling_dup back to top
                                                // PUSHDATA1 0x14 (20-byte hash, LE):
            bytecode.push(OpCode::PUSHDATA1.byte());
            bytecode.push(0x14); // length 20
            bytecode.extend_from_slice(&[
                0xFD, 0xA3, 0xFA, 0x43, 0x46, 0xEA, 0x53, 0x2A, 0x25, 0x8F, 0xC4, 0x97, 0xDD, 0xAD,
                0xDB, 0x64, 0x37, 0xC9, 0xFD, 0xFF,
            ]);
            bytecode.push(OpCode::EQUAL.byte()); // EQUAL — calling == ContractManagement?

            // OR the two boolean results.
            bytecode.push(OpCode::BOOLOR.byte());

            // if !(use_tx_sender) jump to "else" (return CallingScriptHash directly).
            let jmp_if_not_pos = bytecode.len();
            bytecode.push(OpCode::JMPIFNOT_L.byte());
            let jmp_if_not_operand = bytecode.len();
            bytecode.extend_from_slice(&[0, 0, 0, 0]);

            // then: Transaction.Sender
            emit_syscall(bytecode, "System.Runtime.GetScriptContainer");
            // Neo N3 Transaction stack item layout (devpack order):
            //   [Hash, Version, Nonce, Sender, ...]
            push_integer_bigint(bytecode, &BigInt::from(3u8));
            bytecode.push(OpCode::PICKITEM.byte()); // PICKITEM (Transaction.Sender field)

            // jump to end
            let jmp_end_pos = bytecode.len();
            bytecode.push(OpCode::JMP_L.byte());
            let jmp_end_operand = bytecode.len();
            bytecode.extend_from_slice(&[0, 0, 0, 0]);

            // else: re-fetch CallingScriptHash (we DUP'd and consumed it via the OR).
            let else_pos = bytecode.len();
            emit_syscall(bytecode, "System.Runtime.GetCallingScriptHash");

            // end:
            let end_pos = bytecode.len();

            let rel_else = (else_pos as i32)
                .checked_sub(jmp_if_not_pos as i32)
                .unwrap_or(0);
            bytecode[jmp_if_not_operand..jmp_if_not_operand + 4]
                .copy_from_slice(&rel_else.to_le_bytes());

            let rel_end = (end_pos as i32)
                .checked_sub(jmp_end_pos as i32)
                .unwrap_or(0);
            bytecode[jmp_end_operand..jmp_end_operand + 4].copy_from_slice(&rel_end.to_le_bytes());
        }
        ir::RuntimeValue::MsgValue => {
            // Neo N3 has NO EVM-style attached call value, and no
            // `System.Runtime.GetMsgValue` interop exists — emitting a SYSCALL for
            // it FAULTS on a real node (unknown interop service). Value transfer
            // on Neo arrives as the `amount` argument of `onNEP17Payment` /
            // `onNEP11Payment`, not as an ambient `msg.value`. So `msg.value`
            // lowers to the conformant constant 0 (PUSH0); contracts that need a
            // received amount must read their payment-callback argument.
            bytecode.push(OpCode::PUSH0.byte());
        }
        ir::RuntimeValue::MsgData => {
            // Solidity `msg.data` == the raw calldata bytes the runtime received at
            // `execute(bytecode, input)`. Neo N3 exposes those bytes through the
            // `Script` slot (index 7) of the Transaction-shaped array returned by
            // `System.Runtime.GetScriptContainer`:
            //   [Hash, Version, Nonce, Sender, SystemFee, NetworkFee, ValidUntilBlock, Script]
            // (see src/runtime/execution/syscalls/runtime.rs "GetScriptContainer").
            emit_syscall(bytecode, "System.Runtime.GetScriptContainer");
            push_integer_bigint(bytecode, &BigInt::from(7u8));
            bytecode.push(OpCode::PICKITEM.byte()); // PICKITEM — Transaction.Script (input_data)
        }
        ir::RuntimeValue::TxOrigin => {
            emit_syscall(bytecode, "System.Runtime.GetScriptContainer");
            // Neo N3 Transaction stack item layout (devpack order):
            // [Hash, Version, Nonce, Sender, ...]
            push_integer_bigint(bytecode, &BigInt::from(3u8));
            bytecode.push(OpCode::PICKITEM.byte()); // PICKITEM (Transaction.Sender field)
        }
        // Neo's System.Runtime.GetTime returns milliseconds since epoch, while Solidity's
        // block.timestamp is seconds. Normalize here to preserve Solidity semantics.
        ir::RuntimeValue::BlockTimestamp => {
            emit_syscall(bytecode, "System.Runtime.GetTime");
            push_integer_bigint(bytecode, &BigInt::from(1000u64));
            bytecode.push(OpCode::DIV.byte());
        }
        ir::RuntimeValue::BlockNumber => {
            emit_native_contract_call(
                bytecode,
                ir::NativeContract::Ledger,
                "currentIndex",
                0,
                use_callt,
                token_patches,
            );
        }
    }
}
