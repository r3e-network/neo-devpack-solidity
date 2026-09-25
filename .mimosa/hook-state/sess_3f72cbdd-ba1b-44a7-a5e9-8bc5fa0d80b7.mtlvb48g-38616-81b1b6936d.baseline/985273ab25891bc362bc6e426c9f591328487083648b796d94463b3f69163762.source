use super::*;
use crate::opcode::OpCode;

pub(crate) fn emit_native_call(
    bytecode: &mut Vec<u8>,
    contract: ir::NativeContract,
    method: &str,
    arg_count: usize,
    use_callt: bool,
    token_patches: &mut Vec<MethodTokenPatch>,
) {
    emit_native_contract_call(
        bytecode,
        contract,
        method,
        arg_count,
        use_callt,
        token_patches,
    );
}

pub(crate) fn emit_deploy_contract(
    bytecode: &mut Vec<u8>,
    arg_count: usize,
    use_callt: bool,
    token_patches: &mut Vec<MethodTokenPatch>,
) {
    // ContractManagement.deploy(nef, manifest[, data]) returns a ContractState.
    // NativeCalls.deployContract is a devpack convenience wrapper that returns
    // the deployed contract hash (UInt160) only.
    emit_native_contract_call(
        bytecode,
        ir::NativeContract::ContractManagement,
        "deploy",
        arg_count,
        use_callt,
        token_patches,
    );
    bytecode.push(OpCode::PUSH2.byte()); // PUSH2 (ContractState.Hash field index)
    bytecode.push(OpCode::PICKITEM.byte());
}

pub(crate) fn emit_get_contract(
    bytecode: &mut Vec<u8>,
    use_callt: bool,
    token_patches: &mut Vec<MethodTokenPatch>,
) {
    // ContractManagement.getContract(UInt160) returns a ContractState struct:
    // [id, updateCounter, hash, nef, manifestStruct]
    //
    // NativeCalls.getContract reshapes it into:
    // [hash, nef, serialize(manifestStruct), updateCounter]
    emit_native_contract_call(
        bytecode,
        ir::NativeContract::ContractManagement,
        "getContract",
        1,
        use_callt,
        token_patches,
    );

    // Extract hash (index 2)
    bytecode.push(OpCode::DUP.byte());
    bytecode.push(OpCode::PUSH2.byte());
    bytecode.push(OpCode::PICKITEM.byte()); // PICKITEM -> [state, hash]

    // Extract nef (index 3)
    bytecode.push(OpCode::OVER.byte());
    bytecode.push(OpCode::PUSH3.byte());
    bytecode.push(OpCode::PICKITEM.byte()); // PICKITEM -> [state, hash, nef]

    // Extract manifest (index 4) and serialize it
    bytecode.push(OpCode::OVER.byte());
    bytecode.push(OpCode::PUSH4.byte());
    bytecode.push(OpCode::PICKITEM.byte()); // PICKITEM -> [state, hash, nef, manifestStruct]
    emit_native_contract_call(
        bytecode,
        ir::NativeContract::StdLib,
        "serialize",
        1,
        use_callt,
        token_patches,
    ); // -> [state, hash, nef, manifestBytes]

    // Extract updateCounter (index 1)
    bytecode.push(OpCode::PUSH3.byte()); // PUSH3 (duplicate state from depth 3)
    bytecode.push(OpCode::PICK.byte()); // PICK -> [state, hash, nef, manifestBytes, state]
    bytecode.push(OpCode::PUSH1.byte());
    bytecode.push(OpCode::PICKITEM.byte()); // PICKITEM -> [state, hash, nef, manifestBytes, updateCounter]

    // Drop original state and pack struct
    bytecode.push(OpCode::PUSH4.byte());
    bytecode.push(OpCode::ROLL.byte()); // ROLL -> [..., state]
    bytecode.push(OpCode::DROP.byte());

    bytecode.push(OpCode::PUSH4.byte());
    bytecode.push(OpCode::PACK.byte());
    // PACK reverses stack order; restore the field order expected by the devpack helper:
    // [hash, nef, manifestBytes, updateCounter]
    bytecode.push(OpCode::DUP.byte());
    bytecode.push(OpCode::REVERSEITEMS.byte());
}

pub(crate) fn emit_get_contract_script(
    bytecode: &mut Vec<u8>,
    use_callt: bool,
    token_patches: &mut Vec<MethodTokenPatch>,
) {
    // Syscalls.getContractScript is implemented as a convenience wrapper
    // that fetches the NEF file from a contract state.
    emit_native_contract_call(
        bytecode,
        ir::NativeContract::ContractManagement,
        "getContract",
        1,
        use_callt,
        token_patches,
    );
    bytecode.push(OpCode::PUSH3.byte()); // PUSH3 (ContractState.Nef field index)
    bytecode.push(OpCode::PICKITEM.byte());
}

pub(crate) fn emit_get_neo_account_state(
    bytecode: &mut Vec<u8>,
    use_callt: bool,
    token_patches: &mut Vec<MethodTokenPatch>,
) {
    // NeoToken.getAccountState(UInt160) returns NeoAccountState? (nullable) with shape:
    //   [balance, balanceHeight, voteTo (ECPoint bytes or null), lastGasPerVote]
    //
    // The devpack helper expects a non-null struct, so we replace `null` with a
    // default struct and normalize `voteTo` to an empty byte string.
    emit_native_contract_call(
        bytecode,
        ir::NativeContract::Neo,
        "getAccountState",
        1,
        use_callt,
        token_patches,
    );

    // If result is null, replace with [0, 0, "", 0].
    bytecode.push(OpCode::DUP.byte());
    bytecode.push(OpCode::ISNULL.byte());
    let jmp_if_not_null_pos = bytecode.len();
    bytecode.push(OpCode::JMPIFNOT_L.byte());
    let jmp_if_not_null_operand = bytecode.len();
    bytecode.extend_from_slice(&[0, 0, 0, 0]);

    // null path
    bytecode.push(OpCode::DROP.byte());
    bytecode.push(OpCode::PUSH0.byte()); // PUSH0 (balance)
    bytecode.push(OpCode::PUSH0.byte()); // PUSH0 (balanceHeight)
    push_data(bytecode, &[]); // empty voteTo
    bytecode.push(OpCode::PUSH0.byte()); // PUSH0 (lastGasPerVote)
    bytecode.push(OpCode::PUSH4.byte());
    bytecode.push(OpCode::PACK.byte());
    // PACK reverses stack order; restore [balance, balanceHeight, voteTo, lastGasPerVote].
    bytecode.push(OpCode::DUP.byte());
    bytecode.push(OpCode::REVERSEITEMS.byte());

    let jmp_end_pos = bytecode.len();
    bytecode.push(OpCode::JMP_L.byte());
    let jmp_end_operand = bytecode.len();
    bytecode.extend_from_slice(&[0, 0, 0, 0]);

    // not-null path
    let not_null_pos = bytecode.len();

    // If voteTo (index 2) is null, set it to "".
    bytecode.push(OpCode::DUP.byte());
    bytecode.push(OpCode::PUSH2.byte());
    bytecode.push(OpCode::PICKITEM.byte()); // PICKITEM -> voteTo
    bytecode.push(OpCode::ISNULL.byte());
    let jmp_vote_non_null_pos = bytecode.len();
    bytecode.push(OpCode::JMPIFNOT_L.byte());
    let jmp_vote_non_null_operand = bytecode.len();
    bytecode.extend_from_slice(&[0, 0, 0, 0]);

    // voteTo is null path: state[2] = ""
    bytecode.push(OpCode::DUP.byte());
    bytecode.push(OpCode::PUSH2.byte());
    push_data(bytecode, &[]);
    bytecode.push(OpCode::SETITEM.byte());

    let vote_non_null_pos = bytecode.len();

    let rel_vote_non_null = (vote_non_null_pos as i32)
        .checked_sub(jmp_vote_non_null_pos as i32)
        .unwrap_or(0);
    bytecode[jmp_vote_non_null_operand..jmp_vote_non_null_operand + 4]
        .copy_from_slice(&rel_vote_non_null.to_le_bytes());

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
