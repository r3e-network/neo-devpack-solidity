use super::*;

pub(crate) fn try_lower_expression_tuple(
    expr: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    match expr {
        Expression::List(_, params) => {
            // Solidity tuple/list expressions. On Neo we represent tuples as arrays.
            let element_count = params.len();
            let element_type = ValueType::Any;

            let array_local = ctx.allocate_local(
                "__tuple_literal".to_string(),
                Some(ValueType::Array(Box::new(element_type.clone()))),
            );

            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::from(element_count),
            )));
            instructions.push(Instruction::NewArray { element_type });
            instructions.push(Instruction::StoreLocal(array_local));

            for (index, (_, param)) in params.iter().enumerate() {
                instructions.push(Instruction::LoadLocal(array_local));
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                    BigInt::from(index as u64),
                )));

                if let Some(parameter) = param {
                    if !lower_expression(&parameter.ty, ctx, instructions) {
                        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                            BigInt::zero(),
                        )));
                    }
                } else {
                    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                        BigInt::zero(),
                    )));
                }

                instructions.push(Instruction::ArraySet);
            }

            instructions.push(Instruction::LoadLocal(array_local));
            Some(true)
        }
        _ => None,
    }
}
