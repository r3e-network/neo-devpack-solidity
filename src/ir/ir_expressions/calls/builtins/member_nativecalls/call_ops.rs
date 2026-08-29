//! Call Ops — Neo N3 native contract call ops operations
//!
//! Extracted from member_nativecalls.rs for maintainability.

#![allow(non_snake_case)]

use super::*;

pub(crate) fn lower_native_estimateNativeCallGas(
    ctx: &mut LoweringContext,
    args: &[Expression],
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    if let Some(false) = check_arg_count(ctx, "NativeCalls", "estimateNativeCallGas", args, 3) {
        return Some(false);
    }

    let tmp_id = ctx.next_label();
    let contract_slot = ctx.allocate_local(
        format!("__native_calls_estimate_contract_{tmp_id}"),
        Some(ValueType::Address),
    );
    let method_slot = ctx.allocate_local(
        format!("__native_calls_estimate_method_{tmp_id}"),
        Some(ValueType::String),
    );
    let params_slot = ctx.allocate_local(
        format!("__native_calls_estimate_params_{tmp_id}"),
        Some(ValueType::ByteArray { fixed_len: None }),
    );
    let result_slot = ctx.allocate_local(
        format!("__native_calls_estimate_result_{tmp_id}"),
        Some(ValueType::Integer {
            signed: false,
            bits: 256,
        }),
    );

    if !lower_expression(&args[0], ctx, instructions) {
        return Some(false);
    }
    instructions.push(Instruction::StoreLocal(contract_slot));

    if !lower_expression(&args[1], ctx, instructions) {
        return Some(false);
    }
    instructions.push(Instruction::StoreLocal(method_slot));

    if !lower_expression(&args[2], ctx, instructions) {
        return Some(false);
    }
    instructions.push(Instruction::StoreLocal(params_slot));

    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(1_000_000u64),
    )));
    instructions.push(Instruction::StoreLocal(result_slot));

    let end_label = ctx.next_label();

    // NEO contract heuristics
    let neo_skip_label = ctx.next_label();
    let neo_register_label = ctx.next_label();
    instructions.push(Instruction::LoadLocal(contract_slot));
    instructions.push(Instruction::PushLiteral(LiteralValue::Address(
        NATIVE_CONTRACTS[0].hash.to_vec(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Eq));
    instructions.push(Instruction::JumpIf {
        target: neo_skip_label,
    });

    instructions.push(Instruction::LoadLocal(method_slot));
    instructions.push(Instruction::PushLiteral(LiteralValue::String(
        b"vote".to_vec(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Eq));
    instructions.push(Instruction::JumpIf {
        target: neo_register_label,
    });
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(100_000_000u64),
    )));
    instructions.push(Instruction::StoreLocal(result_slot));
    instructions.push(Instruction::Jump { target: end_label });

    instructions.push(Instruction::Label(neo_register_label));
    instructions.push(Instruction::LoadLocal(method_slot));
    instructions.push(Instruction::PushLiteral(LiteralValue::String(
        b"registerCandidate".to_vec(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Eq));
    instructions.push(Instruction::JumpIf {
        target: neo_skip_label,
    });
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(1_000_000_000u64),
    )));
    instructions.push(Instruction::StoreLocal(result_slot));
    instructions.push(Instruction::Jump { target: end_label });

    instructions.push(Instruction::Label(neo_skip_label));

    // ContractManagement heuristics
    let cm_skip_label = ctx.next_label();
    let cm_update_label = ctx.next_label();
    instructions.push(Instruction::LoadLocal(contract_slot));
    instructions.push(Instruction::PushLiteral(LiteralValue::Address(
        NATIVE_CONTRACTS[2].hash.to_vec(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Eq));
    instructions.push(Instruction::JumpIf {
        target: cm_skip_label,
    });

    instructions.push(Instruction::LoadLocal(method_slot));
    instructions.push(Instruction::PushLiteral(LiteralValue::String(
        b"deploy".to_vec(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Eq));
    instructions.push(Instruction::JumpIf {
        target: cm_update_label,
    });
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(500_000_000u64),
    )));
    instructions.push(Instruction::StoreLocal(result_slot));
    instructions.push(Instruction::Jump { target: end_label });

    instructions.push(Instruction::Label(cm_update_label));
    instructions.push(Instruction::LoadLocal(method_slot));
    instructions.push(Instruction::PushLiteral(LiteralValue::String(
        b"update".to_vec(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Eq));
    instructions.push(Instruction::JumpIf {
        target: cm_skip_label,
    });
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(300_000_000u64),
    )));
    instructions.push(Instruction::StoreLocal(result_slot));
    instructions.push(Instruction::Jump { target: end_label });

    instructions.push(Instruction::Label(cm_skip_label));

    // Oracle heuristics
    let oracle_skip_label = ctx.next_label();
    instructions.push(Instruction::LoadLocal(contract_slot));
    instructions.push(Instruction::PushLiteral(LiteralValue::Address(
        NATIVE_CONTRACTS[4].hash.to_vec(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Eq));
    instructions.push(Instruction::JumpIf {
        target: oracle_skip_label,
    });

    instructions.push(Instruction::LoadLocal(method_slot));
    instructions.push(Instruction::PushLiteral(LiteralValue::String(
        b"request".to_vec(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Eq));
    instructions.push(Instruction::JumpIf {
        target: oracle_skip_label,
    });
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(50_000_000u64),
    )));
    instructions.push(Instruction::StoreLocal(result_slot));
    instructions.push(Instruction::Jump { target: end_label });

    instructions.push(Instruction::Label(oracle_skip_label));
    instructions.push(Instruction::Label(end_label));
    instructions.push(Instruction::LoadLocal(result_slot));
    Some(true)
}

pub(crate) fn lower_native_batchNativeCalls(
    ctx: &mut LoweringContext,
    args: &[Expression],
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    if let Some(false) = check_arg_count(ctx, "NativeCalls", "batchNativeCalls", args, 3) {
        return Some(false);
    }

    let tmp_id = ctx.next_label();
    let contracts_slot = ctx.allocate_local(
        format!("__native_calls_batch_contracts_{tmp_id}"),
        Some(ValueType::Any),
    );
    let methods_slot = ctx.allocate_local(
        format!("__native_calls_batch_methods_{tmp_id}"),
        Some(ValueType::Any),
    );
    let params_slot = ctx.allocate_local(
        format!("__native_calls_batch_params_{tmp_id}"),
        Some(ValueType::Any),
    );
    let length_slot = ctx.allocate_local(
        format!("__native_calls_batch_length_{tmp_id}"),
        Some(ValueType::Integer {
            signed: false,
            bits: 256,
        }),
    );

    if !lower_expression(&args[0], ctx, instructions) {
        return Some(false);
    }
    instructions.push(Instruction::StoreLocal(contracts_slot));

    if !lower_expression(&args[1], ctx, instructions) {
        return Some(false);
    }
    instructions.push(Instruction::StoreLocal(methods_slot));

    if !lower_expression(&args[2], ctx, instructions) {
        return Some(false);
    }
    instructions.push(Instruction::StoreLocal(params_slot));

    instructions.push(Instruction::LoadLocal(contracts_slot));
    instructions.push(Instruction::GetSize);
    instructions.push(Instruction::StoreLocal(length_slot));

    // contracts.length == methods.length
    let methods_fail = ctx.next_label();
    let methods_ok = ctx.next_label();
    instructions.push(Instruction::LoadLocal(methods_slot));
    instructions.push(Instruction::GetSize);
    instructions.push(Instruction::LoadLocal(length_slot));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Eq));
    instructions.push(Instruction::JumpIf {
        target: methods_fail,
    });
    instructions.push(Instruction::Jump { target: methods_ok });
    instructions.push(Instruction::Label(methods_fail));
    emit_throw_with_message(instructions, "NativeCalls: array length mismatch");
    instructions.push(Instruction::Label(methods_ok));

    // contracts.length == params.length
    let params_fail = ctx.next_label();
    let params_ok = ctx.next_label();
    instructions.push(Instruction::LoadLocal(params_slot));
    instructions.push(Instruction::GetSize);
    instructions.push(Instruction::LoadLocal(length_slot));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Eq));
    instructions.push(Instruction::JumpIf {
        target: params_fail,
    });
    instructions.push(Instruction::Jump { target: params_ok });
    instructions.push(Instruction::Label(params_fail));
    emit_throw_with_message(instructions, "NativeCalls: array length mismatch");
    instructions.push(Instruction::Label(params_ok));

    // contracts.length > 0
    let non_empty_label = ctx.next_label();
    instructions.push(Instruction::LoadLocal(length_slot));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Eq));
    instructions.push(Instruction::JumpIf {
        target: non_empty_label,
    });
    emit_throw_with_message(instructions, "NativeCalls: empty arrays");
    instructions.push(Instruction::Label(non_empty_label));

    // contracts.length <= 10
    let length_fail = ctx.next_label();
    let length_ok_label = ctx.next_label();
    instructions.push(Instruction::LoadLocal(length_slot));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(10u8),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Le));
    instructions.push(Instruction::JumpIf {
        target: length_fail,
    });
    instructions.push(Instruction::Jump {
        target: length_ok_label,
    });
    instructions.push(Instruction::Label(length_fail));
    emit_throw_with_message(instructions, "NativeCalls: too many calls");
    instructions.push(Instruction::Label(length_ok_label));

    let results_slot = ctx.allocate_local(
        format!("__native_calls_batch_results_{tmp_id}"),
        Some(ValueType::Array(Box::new(ValueType::ByteArray {
            fixed_len: None,
        }))),
    );
    let index_slot = ctx.allocate_local(
        format!("__native_calls_batch_index_{tmp_id}"),
        Some(ValueType::Integer {
            signed: false,
            bits: 256,
        }),
    );
    let contract_slot = ctx.allocate_local(
        format!("__native_calls_batch_contract_{tmp_id}"),
        Some(ValueType::Address),
    );

    instructions.push(Instruction::LoadLocal(length_slot));
    instructions.push(Instruction::NewArray {
        element_type: ValueType::ByteArray { fixed_len: None },
    });
    instructions.push(Instruction::StoreLocal(results_slot));

    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::StoreLocal(index_slot));

    let loop_label = ctx.next_label();
    let done_label = ctx.next_label();
    instructions.push(Instruction::Label(loop_label));
    instructions.push(Instruction::LoadLocal(index_slot));
    instructions.push(Instruction::LoadLocal(length_slot));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Lt));
    instructions.push(Instruction::JumpIf { target: done_label });

    // contract = contracts[index]
    instructions.push(Instruction::LoadLocal(contracts_slot));
    instructions.push(Instruction::LoadLocal(index_slot));
    instructions.push(Instruction::ArrayGet);
    instructions.push(Instruction::StoreLocal(contract_slot));

    // require isNativeContract(contract)
    let native_ok = ctx.next_label();
    emit_is_native_contract_check(ctx, instructions, contract_slot);
    instructions.push(Instruction::JumpIf { target: native_ok });
    emit_throw_with_message(instructions, "NativeCalls: not a native contract");
    instructions.push(Instruction::Label(native_ok));

    // results[index] = contractCall(contract, method, params)
    instructions.push(Instruction::LoadLocal(results_slot));
    instructions.push(Instruction::LoadLocal(index_slot));
    instructions.push(Instruction::LoadLocal(contract_slot));
    instructions.push(Instruction::LoadLocal(methods_slot));
    instructions.push(Instruction::LoadLocal(index_slot));
    instructions.push(Instruction::ArrayGet);
    instructions.push(Instruction::LoadLocal(params_slot));
    instructions.push(Instruction::LoadLocal(index_slot));
    instructions.push(Instruction::ArrayGet);

    if ctx.is_safe {
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::from(0x05u8),
        )));
        instructions.push(Instruction::CallBuiltin {
            builtin: BuiltinCall::ContractCallWithFlags,
            arg_count: 4,
        });
    } else {
        instructions.push(Instruction::CallBuiltin {
            builtin: BuiltinCall::ContractCall,
            arg_count: 3,
        });
    }

    instructions.push(Instruction::ArraySet);

    instructions.push(Instruction::LoadLocal(index_slot));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::one(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Add));
    instructions.push(Instruction::StoreLocal(index_slot));
    instructions.push(Instruction::Jump { target: loop_label });

    instructions.push(Instruction::Label(done_label));
    instructions.push(Instruction::LoadLocal(results_slot));
    Some(true)
}

pub(crate) fn lower_native_safeNativeCall(
    ctx: &mut LoweringContext,
    args: &[Expression],
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    if let Some(false) = check_arg_count(ctx, "NativeCalls", "safeNativeCall", args, 3) {
        return Some(false);
    }

    let tmp_id = ctx.next_label();
    let contract_slot = ctx.allocate_local(
        format!("__native_calls_safe_contract_{tmp_id}"),
        Some(ValueType::Address),
    );
    let method_slot = ctx.allocate_local(
        format!("__native_calls_safe_method_{tmp_id}"),
        Some(ValueType::String),
    );
    let params_slot = ctx.allocate_local(
        format!("__native_calls_safe_params_{tmp_id}"),
        Some(ValueType::ByteArray { fixed_len: None }),
    );
    let data_slot = ctx.allocate_local(
        format!("__native_calls_safe_data_{tmp_id}"),
        Some(ValueType::ByteArray { fixed_len: None }),
    );
    let tuple_slot = ctx.allocate_local(
        format!("__native_calls_safe_tuple_{tmp_id}"),
        Some(ValueType::Any),
    );

    if !lower_expression(&args[0], ctx, instructions) {
        return Some(false);
    }
    instructions.push(Instruction::StoreLocal(contract_slot));

    if !lower_expression(&args[1], ctx, instructions) {
        return Some(false);
    }
    instructions.push(Instruction::StoreLocal(method_slot));

    if !lower_expression(&args[2], ctx, instructions) {
        return Some(false);
    }
    instructions.push(Instruction::StoreLocal(params_slot));

    // require(isNativeContract(contract))
    let native_ok = ctx.next_label();
    emit_is_native_contract_check(ctx, instructions, contract_slot);
    instructions.push(Instruction::JumpIf { target: native_ok });
    emit_throw_with_message(instructions, "NativeCalls: not a native contract");
    instructions.push(Instruction::Label(native_ok));

    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(2u8),
    )));
    instructions.push(Instruction::NewArray {
        element_type: ValueType::Any,
    });
    instructions.push(Instruction::StoreLocal(tuple_slot));

    let catch_label = ctx.next_label();
    let end_label = ctx.next_label();
    instructions.push(Instruction::Try {
        catch_target: catch_label,
    });

    instructions.push(Instruction::LoadLocal(contract_slot));
    instructions.push(Instruction::LoadLocal(method_slot));
    instructions.push(Instruction::LoadLocal(params_slot));

    if ctx.is_safe {
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::from(0x05u8),
        )));
        instructions.push(Instruction::CallBuiltin {
            builtin: BuiltinCall::ContractCallWithFlags,
            arg_count: 4,
        });
    } else {
        instructions.push(Instruction::CallBuiltin {
            builtin: BuiltinCall::ContractCall,
            arg_count: 3,
        });
    }

    instructions.push(Instruction::StoreLocal(data_slot));

    instructions.push(Instruction::LoadLocal(tuple_slot));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::PushLiteral(LiteralValue::Boolean(true)));
    instructions.push(Instruction::ArraySet);

    instructions.push(Instruction::LoadLocal(tuple_slot));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::one(),
    )));
    instructions.push(Instruction::LoadLocal(data_slot));
    instructions.push(Instruction::ArraySet);

    instructions.push(Instruction::EndTry { target: end_label });

    instructions.push(Instruction::Label(catch_label));
    instructions.push(Instruction::Drop(ValueType::Any));

    instructions.push(Instruction::LoadLocal(tuple_slot));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::PushLiteral(LiteralValue::Boolean(false)));
    instructions.push(Instruction::ArraySet);

    instructions.push(Instruction::LoadLocal(tuple_slot));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::one(),
    )));
    instructions.push(Instruction::PushLiteral(
        LiteralValue::ByteArray(Vec::new()),
    ));
    instructions.push(Instruction::ArraySet);

    instructions.push(Instruction::EndTry { target: end_label });
    instructions.push(Instruction::Label(end_label));
    instructions.push(Instruction::LoadLocal(tuple_slot));
    Some(true)
}
