use super::*;

pub(crate) fn lower_special_assembly(
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    match ctx.function_name.as_str() {
        "extsload" | "exttload" => {
            lower_extsload_single(ctx, instructions)
                || lower_extsload_range(ctx, instructions)
                || lower_extsload_slots(ctx, instructions)
        }
        _ => false,
    }
}

pub(crate) fn lower_extsload_single(
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    let slot_index = match ctx.param_index_map.get("slot").copied() {
        Some(index) if ctx.param_index_map.len() == 1 => index,
        _ => return false,
    };

    instructions.push(Instruction::LoadParameter(slot_index));
    instructions.push(Instruction::LoadStorageDynamic);
    instructions.push(Instruction::Return);
    true
}

pub(crate) fn lower_extsload_range(
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    let start_index = match ctx.param_index_map.get("startSlot").copied() {
        Some(index) => index,
        None => return false,
    };
    let count_index = match ctx.param_index_map.get("nSlots").copied() {
        Some(index) => index,
        None => return false,
    };

    if ctx.param_index_map.len() != 2 {
        return false;
    }

    let start_local = ctx.allocate_local("__extsload_start".to_string(), None);
    instructions.push(Instruction::LoadParameter(start_index));
    instructions.push(Instruction::StoreLocal(start_local));

    let count_local = ctx.allocate_local("__extsload_count".to_string(), None);
    instructions.push(Instruction::LoadParameter(count_index));
    instructions.push(Instruction::StoreLocal(count_local));

    let array_element_type = ValueType::ByteArray {
        fixed_len: Some(32),
    };
    let array_value_type = ValueType::Array(Box::new(array_element_type.clone()));
    let array_local = ctx.allocate_local(
        "__extsload_array".to_string(),
        Some(array_value_type.clone()),
    );
    instructions.push(Instruction::LoadLocal(count_local));
    instructions.push(Instruction::NewArray {
        element_type: array_element_type,
    });
    instructions.push(Instruction::StoreLocal(array_local));

    let index_local = ctx.allocate_local("__extsload_index".to_string(), None);
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::StoreLocal(index_local));

    let value_local = ctx.allocate_local("__extsload_value".to_string(), None);

    let loop_label = ctx.next_label();
    let end_label = ctx.next_label();

    instructions.push(Instruction::Label(loop_label));
    instructions.push(Instruction::LoadLocal(index_local));
    instructions.push(Instruction::LoadLocal(count_local));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Lt));
    instructions.push(Instruction::JumpIf { target: end_label });

    instructions.push(Instruction::LoadLocal(start_local));
    instructions.push(Instruction::LoadStorageDynamic);
    instructions.push(Instruction::StoreLocal(value_local));

    instructions.push(Instruction::LoadLocal(array_local));
    instructions.push(Instruction::LoadLocal(index_local));
    instructions.push(Instruction::LoadLocal(value_local));
    instructions.push(Instruction::ArraySet);

    instructions.push(Instruction::LoadLocal(index_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(1u8),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Add));
    instructions.push(Instruction::StoreLocal(index_local));

    instructions.push(Instruction::LoadLocal(start_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(1u8),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Add));
    instructions.push(Instruction::StoreLocal(start_local));

    instructions.push(Instruction::Jump { target: loop_label });
    instructions.push(Instruction::Label(end_label));
    instructions.push(Instruction::LoadLocal(array_local));
    instructions.push(Instruction::Return);
    true
}

pub(crate) fn lower_extsload_slots(
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    let slots_index = match ctx.param_index_map.get("slots").copied() {
        Some(index) if ctx.param_index_map.len() == 1 => index,
        _ => return false,
    };

    let slots_local = ctx.allocate_local("__extsload_slots".to_string(), None);
    instructions.push(Instruction::LoadParameter(slots_index));
    instructions.push(Instruction::StoreLocal(slots_local));

    let count_local = ctx.allocate_local("__extsload_count".to_string(), None);
    instructions.push(Instruction::LoadLocal(slots_local));
    instructions.push(Instruction::GetSize);
    instructions.push(Instruction::StoreLocal(count_local));

    let slots_array_element = ValueType::ByteArray {
        fixed_len: Some(32),
    };
    let slots_array_type = ValueType::Array(Box::new(slots_array_element.clone()));
    let array_local = ctx.allocate_local(
        "__extsload_array".to_string(),
        Some(slots_array_type.clone()),
    );
    instructions.push(Instruction::LoadLocal(count_local));
    instructions.push(Instruction::NewArray {
        element_type: slots_array_element,
    });
    instructions.push(Instruction::StoreLocal(array_local));

    let index_local = ctx.allocate_local("__extsload_index".to_string(), None);
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::StoreLocal(index_local));

    let value_local = ctx.allocate_local("__extsload_value".to_string(), None);

    let loop_label = ctx.next_label();
    let end_label = ctx.next_label();

    instructions.push(Instruction::Label(loop_label));
    instructions.push(Instruction::LoadLocal(index_local));
    instructions.push(Instruction::LoadLocal(count_local));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Lt));
    instructions.push(Instruction::JumpIf { target: end_label });

    instructions.push(Instruction::LoadLocal(slots_local));
    instructions.push(Instruction::LoadLocal(index_local));
    instructions.push(Instruction::ArrayGet);
    instructions.push(Instruction::LoadStorageDynamic);
    instructions.push(Instruction::StoreLocal(value_local));

    instructions.push(Instruction::LoadLocal(array_local));
    instructions.push(Instruction::LoadLocal(index_local));
    instructions.push(Instruction::LoadLocal(value_local));
    instructions.push(Instruction::ArraySet);

    instructions.push(Instruction::LoadLocal(index_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(1u8),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Add));
    instructions.push(Instruction::StoreLocal(index_local));

    instructions.push(Instruction::Jump { target: loop_label });
    instructions.push(Instruction::Label(end_label));
    instructions.push(Instruction::LoadLocal(array_local));
    instructions.push(Instruction::Return);
    true
}
