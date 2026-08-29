//! Utility — Neo N3 native contract utility operations
//!
//! Extracted from member_nativecalls.rs for maintainability.

#![allow(non_snake_case)]

use super::*;

pub(crate) fn lower_native_isNativeContract(
    ctx: &mut LoweringContext,
    args: &[Expression],
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    if let Some(false) = check_arg_count(ctx, "NativeCalls", "isNativeContract", args, 1) {
        return Some(false);
    }

    let tmp_id = ctx.next_label();
    let contract_slot = ctx.allocate_local(
        format!("__native_calls_is_native_contract_{tmp_id}"),
        Some(ValueType::Address),
    );

    if !lower_expression(&args[0], ctx, instructions) {
        return Some(false);
    }
    instructions.push(Instruction::StoreLocal(contract_slot));

    emit_is_native_contract_check(ctx, instructions, contract_slot);
    Some(true)
}

pub(crate) fn lower_native_getNativeContractName(
    ctx: &mut LoweringContext,
    args: &[Expression],
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    if let Some(false) = check_arg_count(ctx, "NativeCalls", "getNativeContractName", args, 1) {
        return Some(false);
    }

    let tmp_id = ctx.next_label();
    let contract_slot = ctx.allocate_local(
        format!("__native_calls_contract_name_{tmp_id}"),
        Some(ValueType::Address),
    );

    if !lower_expression(&args[0], ctx, instructions) {
        return Some(false);
    }
    instructions.push(Instruction::StoreLocal(contract_slot));

    emit_native_contract_name(ctx, instructions, contract_slot);
    Some(true)
}

pub(crate) fn lower_native_getAllNativeContracts(
    ctx: &mut LoweringContext,
    args: &[Expression],
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    if let Some(false) = check_arg_count(ctx, "NativeCalls", "getAllNativeContracts", args, 0) {
        return Some(false);
    }

    let tmp_id = ctx.next_label();
    let contracts_slot = ctx.allocate_local(
        format!("__native_calls_all_contracts_{tmp_id}"),
        Some(ValueType::Array(Box::new(ValueType::Address))),
    );

    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(NATIVE_CONTRACTS.len() as u64),
    )));
    instructions.push(Instruction::NewArray {
        element_type: ValueType::Address,
    });
    instructions.push(Instruction::StoreLocal(contracts_slot));

    for (index, contract) in NATIVE_CONTRACTS.iter().enumerate() {
        instructions.push(Instruction::LoadLocal(contracts_slot));
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::from(index as u64),
        )));
        instructions.push(Instruction::PushLiteral(LiteralValue::Address(
            contract.hash.to_vec(),
        )));
        instructions.push(Instruction::ArraySet);
    }

    instructions.push(Instruction::LoadLocal(contracts_slot));
    Some(true)
}

pub(crate) fn lower_native_getNativeContractManifest(
    ctx: &mut LoweringContext,
    args: &[Expression],
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    if let Some(false) = check_arg_count(ctx, "NativeCalls", "getNativeContractManifest", args, 1) {
        return Some(false);
    }

    if !lower_expression(&args[0], ctx, instructions) {
        return Some(false);
    }
    instructions.push(Instruction::CallBuiltin {
        builtin: BuiltinCall::GetContract,
        arg_count: 1,
    });
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(2u8),
    )));
    instructions.push(Instruction::ArrayGet);
    Some(true)
}
