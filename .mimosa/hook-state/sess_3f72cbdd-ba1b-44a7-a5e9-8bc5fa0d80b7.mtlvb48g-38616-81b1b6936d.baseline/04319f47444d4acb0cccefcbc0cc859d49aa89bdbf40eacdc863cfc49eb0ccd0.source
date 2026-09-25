use super::*;
use crate::opcode::OpCode;

pub(crate) fn emit_builtin_call(
    bytecode: &mut Vec<u8>,
    builtin: &ir::BuiltinCall,
    arg_count: usize,
    use_callt: bool,
    token_patches: &mut Vec<MethodTokenPatch>,
) {
    match builtin {
        ir::BuiltinCall::RuntimeNotify => {
            emit_runtime_notify(bytecode, use_callt, token_patches);
        }
        ir::BuiltinCall::RuntimeCheckWitness => emit_runtime_check_witness(bytecode),
        ir::BuiltinCall::Keccak256 => emit_keccak256(bytecode, use_callt, token_patches),
        ir::BuiltinCall::Ecrecover => emit_ecrecover(bytecode, use_callt, token_patches),
        ir::BuiltinCall::PrecompileEcrecover => {
            emit_precompile_ecrecover(bytecode, use_callt, token_patches)
        }
        ir::BuiltinCall::PrecompileModexp => emit_precompile_modexp(bytecode, use_callt),
        ir::BuiltinCall::StorageFind => emit_storage_find(bytecode, arg_count),
        ir::BuiltinCall::StoragePut => emit_storage_put(bytecode),
        ir::BuiltinCall::StorageGet => emit_storage_get(bytecode),
        ir::BuiltinCall::StorageDelete => emit_storage_delete(bytecode),
        ir::BuiltinCall::AbiEncode | ir::BuiltinCall::AbiEncodeCall => {
            emit_abi_encode(bytecode, arg_count, use_callt, token_patches)
        }
        ir::BuiltinCall::AbiEncodePacked => {
            emit_abi_encode_packed(bytecode, arg_count, use_callt, token_patches)
        }
        ir::BuiltinCall::AbiDecode => emit_abi_decode(bytecode, use_callt, token_patches),
        ir::BuiltinCall::NativeCall { contract, method } => emit_native_call(
            bytecode,
            *contract,
            method.as_str(),
            arg_count,
            use_callt,
            token_patches,
        ),
        ir::BuiltinCall::Syscall(name) => emit_syscall_builtin(bytecode, name, arg_count),
        ir::BuiltinCall::ContractCall => emit_contract_call(bytecode, use_callt, token_patches),
        ir::BuiltinCall::ContractCallWithFlags => {
            emit_contract_call_with_flags(bytecode, use_callt, token_patches)
        }
        ir::BuiltinCall::NotifySerialized => {
            emit_notify_serialized(bytecode, use_callt, token_patches)
        }
        ir::BuiltinCall::VerifySignature => {
            emit_verify_signature(bytecode, use_callt, token_patches)
        }
        ir::BuiltinCall::DeployContract => {
            emit_deploy_contract(bytecode, arg_count, use_callt, token_patches)
        }
        ir::BuiltinCall::GetContract => emit_get_contract(bytecode, use_callt, token_patches),
        ir::BuiltinCall::GetContractScript => {
            emit_get_contract_script(bytecode, use_callt, token_patches)
        }
        ir::BuiltinCall::GetNeoAccountState => {
            emit_get_neo_account_state(bytecode, use_callt, token_patches)
        }
        ir::BuiltinCall::TypeOf | ir::BuiltinCall::AbiEncodeWithSignature => {}
        ir::BuiltinCall::BytesConcat => {
            // bytes.concat(a, b, c, ...) → chain NeoVM CAT opcodes.
            // Args are already on the stack. For N args we need N-1 CAT ops.
            // CAT pops two byte arrays and pushes their concatenation.
            for _ in 1..arg_count {
                bytecode.push(OpCode::CAT.byte());
            }
            // If zero args, push empty buffer.
            if arg_count == 0 {
                bytecode.push(OpCode::PUSH0.byte());
                bytecode.push(OpCode::NEWBUFFER.byte());
            }
            // CAT (and NEWBUFFER) yield a NeoVM **Buffer** (0x30). Every concat
            // result — abi.encode/abi.encodePacked, bytes.concat, tuple-return
            // ABI payloads, selector/revert envelopes — flows through here, and
            // a Buffer that is later compared with `==`/`!=` (lowered to EQUAL)
            // against a ByteString literal, ISTYPE-checked, or returned where the
            // manifest declares ByteArray, mis-behaves on a real node (EQUAL is
            // type-strict: Buffer != ByteString). Normalize to ByteString (0x28).
            // The in-tree simulator already treats the concat result as a
            // ByteString-like byte_array, so this aligns the real node with it.
            bytecode.push(OpCode::CONVERT.byte());
            bytecode.push(0x28); // StackItemType::ByteString (CONVERT operand)
        }
    }
}
