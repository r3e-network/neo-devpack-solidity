use super::*;

pub(crate) fn lower_neo_serialized_arg_array(
    args: &[Expression],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    let tmp_id = ctx.next_label();
    let array_local = ctx.allocate_local(format!("__neo_serialized_args_{tmp_id}"), None);

    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(args.len() as u64),
    )));
    instructions.push(Instruction::NewArray {
        element_type: ValueType::Any,
    });
    instructions.push(Instruction::StoreLocal(array_local));

    let mut success = true;
    for (index, arg) in args.iter().enumerate() {
        instructions.push(Instruction::LoadLocal(array_local));
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::from(index as u64),
        )));
        if !lower_expression(arg, ctx, instructions) {
            success = false;
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::zero(),
            )));
        }
        instructions.push(Instruction::ArraySet);
    }

    instructions.push(Instruction::LoadLocal(array_local));
    instructions.push(Instruction::CallBuiltin {
        builtin: BuiltinCall::NativeCall {
            contract: NativeContract::StdLib,
            method: "serialize".to_string(),
        },
        arg_count: 1,
    });

    success
}
