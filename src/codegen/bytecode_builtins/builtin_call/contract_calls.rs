use super::*;
use crate::opcode::OpCode;

pub(crate) fn emit_contract_call(
    bytecode: &mut Vec<u8>,
    use_callt: bool,
    token_patches: &mut Vec<MethodTokenPatch>,
) {
    // Stack: [contract_hash, method, params(bytes)]
    emit_native_contract_call(
        bytecode,
        ir::NativeContract::StdLib,
        "deserialize",
        1,
        use_callt,
        token_patches,
    );
    push_integer_bigint(bytecode, &BigInt::from(CALLFLAGS_ALL));
    bytecode.push(OpCode::SWAP.byte()); // SWAP -> [contract, method, flags, args]
    bytecode.push(OpCode::REVERSE4.byte()); // REVERSE4 -> [args, flags, method, contract]
    emit_syscall(bytecode, "System.Contract.Call");
    // Neo N3 has no native ABI encoder. Return data must still be executable
    // on-chain, so wrap non-null results with real StdLib.serialize. IR-level
    // static ABI fast paths can preserve EVM byte shapes where type
    // information is available; this low-level fallback must not emit
    // pseudo-native StdLib.abiEncode.
    emit_serialize_single_return(bytecode, use_callt, token_patches);
}

pub(crate) fn emit_contract_call_with_flags(
    bytecode: &mut Vec<u8>,
    use_callt: bool,
    token_patches: &mut Vec<MethodTokenPatch>,
) {
    // Stack: [contract_hash, method, params(bytes), flags]
    bytecode.push(OpCode::SWAP.byte()); // SWAP -> [contract, method, flags, params]
    emit_native_contract_call(
        bytecode,
        ir::NativeContract::StdLib,
        "deserialize",
        1,
        use_callt,
        token_patches,
    );
    bytecode.push(OpCode::REVERSE4.byte()); // REVERSE4 -> [args, flags, method, contract]
    emit_syscall(bytecode, "System.Contract.Call");
    // Same production-safe serialized wrapping as the non-flags path.
    emit_serialize_single_return(bytecode, use_callt, token_patches);
}

/// Wrap a single return `StackItem` on the top of the stack as low-level call
/// returndata. Null still maps to empty bytes; non-null values use real
/// StdLib.serialize so emitted NEFs do not depend on pseudo-native ABI helper
/// methods that are absent from Neo N3.
pub(crate) fn emit_serialize_single_return(
    bytecode: &mut Vec<u8>,
    use_callt: bool,
    token_patches: &mut Vec<MethodTokenPatch>,
) {
    // Stack: [returnValue]
    // NeoVM uses `Null` for "no return value". Solidity low-level calls
    // surface that as empty returndata, not as an ABI-encoded null sentinel.
    // Short-circuit the null case so void methods surface empty returndata.
    bytecode.push(OpCode::DUP.byte());
    bytecode.push(OpCode::ISNULL.byte());
    let jmp_if_not_null_pos = bytecode.len();
    bytecode.push(OpCode::JMPIFNOT_L.byte());
    let jmp_if_not_null_operand = bytecode.len();
    bytecode.extend_from_slice(&[0, 0, 0, 0]);

    // null path -> empty bytes
    bytecode.push(OpCode::DROP.byte());
    push_data(bytecode, &[]);
    let jmp_end_pos = bytecode.len();
    bytecode.push(OpCode::JMP_L.byte());
    let jmp_end_operand = bytecode.len();
    bytecode.extend_from_slice(&[0, 0, 0, 0]);

    let not_null_pos = bytecode.len();

    // PUSH1 — arity for PACK.
    bytecode.push(OpCode::PUSH1.byte());
    bytecode.push(OpCode::PACK.byte()); // PACK -> [Array([returnValue])]
    emit_native_contract_call(
        bytecode,
        ir::NativeContract::StdLib,
        "serialize",
        1,
        use_callt,
        token_patches,
    );

    let end_pos = bytecode.len();
    let rel_not_null = (not_null_pos as i32)
        .checked_sub(jmp_if_not_null_pos as i32)
        .unwrap_or(0);
    bytecode[jmp_if_not_null_operand..jmp_if_not_null_operand + 4]
        .copy_from_slice(&rel_not_null.to_le_bytes());

    let rel_end = (end_pos as i32)
        .checked_sub(jmp_end_pos as i32)
        .unwrap_or(0);
    bytecode[jmp_end_operand..jmp_end_operand + 4].copy_from_slice(&rel_end.to_le_bytes());
}
