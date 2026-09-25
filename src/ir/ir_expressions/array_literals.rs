use super::*;

pub(crate) fn lower_array_literal_expression(
    elements: &[Expression],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    let element_type = infer_literal_array_element_type(elements);
    let array_local = ctx.allocate_local(
        "__array_literal".to_string(),
        Some(ValueType::Array(Box::new(element_type.clone()))),
    );
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(elements.len()),
    )));
    instructions.push(Instruction::NewArray { element_type });
    instructions.push(Instruction::StoreLocal(array_local));

    for (index, element) in elements.iter().enumerate() {
        instructions.push(Instruction::LoadLocal(array_local));
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::from(index as u64),
        )));
        if !lower_expression(element, ctx, instructions) {
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::zero(),
            )));
        }
        instructions.push(Instruction::ArraySet);
    }

    instructions.push(Instruction::LoadLocal(array_local));
    true
}
