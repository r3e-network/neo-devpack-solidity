//! Contract State — Neo N3 native contract contract state operations
//!
//! Extracted from member_nativecalls.rs for maintainability.

#![allow(non_snake_case)]

use super::*;

pub(crate) fn lower_native_getContractById(
    ctx: &mut LoweringContext,
    args: &[Expression],
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    if let Some(false) = check_arg_count(ctx, "NativeCalls", "getContractById", args, 1) {
        return Some(false);
    }

    let tmp_id = ctx.next_label();
    let state_slot = ctx.allocate_local(
        format!("__native_calls_contract_state_{tmp_id}"),
        Some(ValueType::Any),
    );
    let result_slot = ctx.allocate_local(
        format!("__native_calls_contract_state_result_{tmp_id}"),
        Some(ValueType::Array(Box::new(ValueType::Any))),
    );

    if !lower_expression(&args[0], ctx, instructions) {
        return Some(false);
    }

    instructions.push(Instruction::CallBuiltin {
        builtin: BuiltinCall::NativeCall {
            contract: NativeContract::ContractManagement,
            method: "getContractById".to_string(),
        },
        arg_count: 1,
    });
    instructions.push(Instruction::StoreLocal(state_slot));

    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(4u8),
    )));
    instructions.push(Instruction::NewArray {
        element_type: ValueType::Any,
    });
    instructions.push(Instruction::StoreLocal(result_slot));

    // hash (index 2)
    instructions.push(Instruction::LoadLocal(result_slot));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::LoadLocal(state_slot));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(2u8),
    )));
    instructions.push(Instruction::ArrayGet);
    instructions.push(Instruction::ArraySet);

    // nef (index 3)
    instructions.push(Instruction::LoadLocal(result_slot));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::one(),
    )));
    instructions.push(Instruction::LoadLocal(state_slot));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(3u8),
    )));
    instructions.push(Instruction::ArrayGet);
    instructions.push(Instruction::ArraySet);

    // manifest (index 4) serialized
    instructions.push(Instruction::LoadLocal(result_slot));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(2u8),
    )));
    instructions.push(Instruction::LoadLocal(state_slot));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(4u8),
    )));
    instructions.push(Instruction::ArrayGet);
    instructions.push(Instruction::CallBuiltin {
        builtin: BuiltinCall::NativeCall {
            contract: NativeContract::StdLib,
            method: "serialize".to_string(),
        },
        arg_count: 1,
    });
    instructions.push(Instruction::ArraySet);

    // updateCounter (index 1)
    instructions.push(Instruction::LoadLocal(result_slot));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(3u8),
    )));
    instructions.push(Instruction::LoadLocal(state_slot));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::one(),
    )));
    instructions.push(Instruction::ArrayGet);
    instructions.push(Instruction::ArraySet);

    instructions.push(Instruction::LoadLocal(result_slot));
    Some(true)
}
