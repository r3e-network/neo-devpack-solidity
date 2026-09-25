use super::*;

pub(crate) fn load_expression(
    expr: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) {
    let mut tmp = Vec::new();
    if lower_expression(expr, ctx, &mut tmp) {
        instructions.append(&mut tmp);
    }
}

pub(crate) fn push_default_for_type(ty: &PtType, instructions: &mut Vec<Instruction>) {
    match ty {
        PtType::Address | PtType::AddressPayable => {
            instructions.push(Instruction::PushLiteral(LiteralValue::Address(vec![
                0u8;
                20
            ])));
        }
        PtType::Bool => instructions.push(Instruction::PushLiteral(LiteralValue::Boolean(false))),
        PtType::String => {
            instructions.push(Instruction::PushLiteral(LiteralValue::String(Vec::new())))
        }
        PtType::Uint(_) | PtType::Int(_) => {
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::zero(),
            )));
        }
        PtType::Bytes(len) => instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(
            vec![0u8; *len as usize],
        ))),
        PtType::DynamicBytes => instructions.push(Instruction::PushLiteral(
            LiteralValue::ByteArray(Vec::new()),
        )),
        _ => instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::zero(),
        ))),
    }
}

pub(crate) fn push_default_for_value_type(
    value_type: &ValueType,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    match value_type {
        ValueType::Integer { .. } => {
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::zero(),
            )));
            true
        }
        ValueType::Boolean => {
            instructions.push(Instruction::PushLiteral(LiteralValue::Boolean(false)));
            true
        }
        ValueType::String => {
            instructions.push(Instruction::PushLiteral(LiteralValue::String(Vec::new())));
            true
        }
        ValueType::Address => {
            instructions.push(Instruction::PushLiteral(LiteralValue::Address(vec![
                0u8;
                20
            ])));
            true
        }
        ValueType::ByteArray { fixed_len } => {
            let bytes = fixed_len
                .map(|len| vec![0u8; len as usize])
                .unwrap_or_else(Vec::new);
            instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(bytes)));
            true
        }
        ValueType::Array(element_type) => {
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::zero(),
            )));
            instructions.push(Instruction::NewArray {
                element_type: (**element_type).clone(),
            });
            true
        }
        ValueType::Struct { fields, .. } => {
            let tmp_id = ctx.next_label();
            let struct_local = ctx.allocate_local(
                format!("__default_struct_{tmp_id}"),
                Some(ValueType::Array(Box::new(ValueType::Any))),
            );

            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::from(fields.len() as u64),
            )));
            instructions.push(Instruction::NewArray {
                element_type: ValueType::Any,
            });
            instructions.push(Instruction::StoreLocal(struct_local));

            for (index, field) in fields.iter().enumerate() {
                instructions.push(Instruction::LoadLocal(struct_local));
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                    BigInt::from(index as u64),
                )));
                push_default_for_value_type(&field.ty, ctx, instructions);
                instructions.push(Instruction::ArraySet);
            }

            instructions.push(Instruction::LoadLocal(struct_local));
            true
        }
        ValueType::Mapping { .. } | ValueType::Any => {
            instructions.push(Instruction::PushLiteral(LiteralValue::Null));
            true
        }
    }
}

pub(crate) fn push_default_for_storage_value_type(
    value_type: &ValueType,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    match value_type {
        // Storage arrays are represented by their length stored at the base slot.
        ValueType::Array(_) => {
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::zero(),
            )));
            true
        }
        // Structs are stored field-by-field; represent the default as an array whose elements
        // match the per-field storage encoding (notably: nested arrays use their length).
        ValueType::Struct { fields, .. } => {
            let tmp_id = ctx.next_label();
            let struct_local = ctx.allocate_local(
                format!("__default_storage_struct_{tmp_id}"),
                Some(ValueType::Array(Box::new(ValueType::Any))),
            );

            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::from(fields.len() as u64),
            )));
            instructions.push(Instruction::NewArray {
                element_type: ValueType::Any,
            });
            instructions.push(Instruction::StoreLocal(struct_local));

            for (index, field) in fields.iter().enumerate() {
                instructions.push(Instruction::LoadLocal(struct_local));
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                    BigInt::from(index as u64),
                )));
                push_default_for_storage_value_type(&field.ty, ctx, instructions);
                instructions.push(Instruction::ArraySet);
            }

            instructions.push(Instruction::LoadLocal(struct_local));
            true
        }
        _ => push_default_for_value_type(value_type, ctx, instructions),
    }
}
