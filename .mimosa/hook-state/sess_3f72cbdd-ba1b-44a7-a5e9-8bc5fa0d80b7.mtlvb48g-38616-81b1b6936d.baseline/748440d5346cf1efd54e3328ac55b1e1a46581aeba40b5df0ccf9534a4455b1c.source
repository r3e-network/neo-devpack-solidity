//! Network Config — Neo N3 native contract network config operations
//!
//! Extracted from member_nativecalls.rs for maintainability.

#![allow(non_snake_case)]

use super::*;

pub(crate) fn lower_native_getNetworkConfiguration(
    ctx: &mut LoweringContext,
    args: &[Expression],
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    if let Some(false) = check_arg_count(ctx, "NativeCalls", "getNetworkConfiguration", args, 0) {
        return Some(false);
    }

    let tmp_id = ctx.next_label();
    let config_slot = ctx.allocate_local(
        format!("__native_calls_network_config_{tmp_id}"),
        Some(ValueType::Array(Box::new(ValueType::Any))),
    );

    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(6u8),
    )));
    instructions.push(Instruction::NewArray {
        element_type: ValueType::Any,
    });
    instructions.push(Instruction::StoreLocal(config_slot));

    // feePerByte
    instructions.push(Instruction::LoadLocal(config_slot));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::CallBuiltin {
        builtin: BuiltinCall::NativeCall {
            contract: NativeContract::Policy,
            method: "getFeePerByte".to_string(),
        },
        arg_count: 0,
    });
    instructions.push(Instruction::ArraySet);

    // execFeeFactor
    instructions.push(Instruction::LoadLocal(config_slot));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::one(),
    )));
    instructions.push(Instruction::CallBuiltin {
        builtin: BuiltinCall::NativeCall {
            contract: NativeContract::Policy,
            method: "getExecFeeFactor".to_string(),
        },
        arg_count: 0,
    });
    instructions.push(Instruction::ArraySet);

    // storagePrice
    instructions.push(Instruction::LoadLocal(config_slot));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(2u8),
    )));
    instructions.push(Instruction::CallBuiltin {
        builtin: BuiltinCall::NativeCall {
            contract: NativeContract::Policy,
            method: "getStoragePrice".to_string(),
        },
        arg_count: 0,
    });
    instructions.push(Instruction::ArraySet);

    // gasPerBlock
    instructions.push(Instruction::LoadLocal(config_slot));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(3u8),
    )));
    instructions.push(Instruction::CallBuiltin {
        builtin: BuiltinCall::NativeCall {
            contract: NativeContract::Neo,
            method: "getGasPerBlock".to_string(),
        },
        arg_count: 0,
    });
    instructions.push(Instruction::ArraySet);

    // oraclePrice
    instructions.push(Instruction::LoadLocal(config_slot));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(4u8),
    )));
    instructions.push(Instruction::CallBuiltin {
        builtin: BuiltinCall::NativeCall {
            contract: NativeContract::Oracle,
            method: "getPrice".to_string(),
        },
        arg_count: 0,
    });
    instructions.push(Instruction::ArraySet);

    // minimumDeploymentFee
    instructions.push(Instruction::LoadLocal(config_slot));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(5u8),
    )));
    instructions.push(Instruction::CallBuiltin {
        builtin: BuiltinCall::NativeCall {
            contract: NativeContract::ContractManagement,
            method: "getMinimumDeploymentFee".to_string(),
        },
        arg_count: 0,
    });
    instructions.push(Instruction::ArraySet);

    instructions.push(Instruction::LoadLocal(config_slot));
    Some(true)
}
