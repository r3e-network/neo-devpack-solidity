use super::*;

pub(crate) fn resolve_struct_type_by_name(
    ctx: &LoweringContext<'_>,
    name: &str,
) -> Option<ValueType> {
    ctx.defined_struct_types
        .iter()
        .chain(ctx.state_types.iter())
        .chain(ctx.param_types.iter())
        .chain(ctx.return_types.iter())
        .chain(ctx.local_types.values())
        .find_map(|ty| find_named_struct_type(ty, name))
}

pub(crate) fn try_lower_struct_constructor_call(
    func: &Expression,
    args: &[Expression],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    let struct_name = match func {
        Expression::Variable(identifier) => identifier.name.clone(),
        Expression::MemberAccess(_, _, identifier) => identifier.name.clone(),
        _ => return None,
    };

    let struct_type = resolve_struct_type_by_name(ctx, &struct_name)?;
    let ValueType::Struct { fields, .. } = &struct_type else {
        return None;
    };

    Some(lower_struct_constructor_positional(
        &struct_name,
        &struct_type,
        fields,
        args,
        ctx,
        instructions,
    ))
}

pub(crate) fn try_lower_struct_constructor_named_call(
    func: &Expression,
    args: &[solang_parser::pt::NamedArgument],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    let struct_name = match func {
        Expression::Variable(identifier) => identifier.name.clone(),
        Expression::MemberAccess(_, _, identifier) => identifier.name.clone(),
        _ => return None,
    };

    let struct_type = resolve_struct_type_by_name(ctx, &struct_name)?;
    let ValueType::Struct { fields, .. } = &struct_type else {
        return None;
    };

    Some(lower_struct_constructor_named(
        &struct_name,
        &struct_type,
        fields,
        args,
        ctx,
        instructions,
    ))
}

pub(crate) fn lower_struct_constructor_positional(
    call_name: &str,
    struct_type: &ValueType,
    fields: &[StructField],
    args: &[Expression],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    let tmp_id = ctx.next_label();
    let struct_local =
        ctx.allocate_local(format!("__struct_ctor_{tmp_id}"), Some(struct_type.clone()));

    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(fields.len() as u64),
    )));
    instructions.push(Instruction::NewArray {
        element_type: ValueType::Any,
    });
    instructions.push(Instruction::StoreLocal(struct_local));

    let mut success = true;

    for (index, field) in fields.iter().enumerate() {
        instructions.push(Instruction::LoadLocal(struct_local));
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::from(index as u64),
        )));
        if let Some(arg) = args.get(index) {
            // Canonicalize an integer-backed `bytesN` literal field to its
            // big-endian ByteArray so the stored field matches the cast/param
            // representation (else N<32 faults / N==32 reverses on encode).
            if !try_lower_bytesn_literal_canonical(arg, &field.ty, ctx, instructions)
                && !lower_expression(arg, ctx, instructions)
            {
                push_default_for_value_type(&field.ty, ctx, instructions);
                success = false;
            }
        } else {
            push_default_for_value_type(&field.ty, ctx, instructions);
        }
        instructions.push(Instruction::ArraySet);
    }

    if args.len() > fields.len() {
        ctx.record_error(format!(
            "struct constructor '{}' expects {} argument(s), got {}",
            call_name,
            fields.len(),
            args.len()
        ));
        for arg in &args[fields.len()..] {
            if lower_expression(arg, ctx, instructions) {
                instructions.push(Instruction::Drop(ValueType::Any));
            }
        }
        success = false;
    }

    instructions.push(Instruction::LoadLocal(struct_local));
    success
}

pub(crate) fn lower_struct_constructor_named(
    _call_name: &str,
    struct_type: &ValueType,
    fields: &[StructField],
    args: &[solang_parser::pt::NamedArgument],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    let tmp_id = ctx.next_label();
    let struct_local =
        ctx.allocate_local(format!("__struct_ctor_{tmp_id}"), Some(struct_type.clone()));

    let mut field_indexes = HashMap::with_capacity(fields.len());
    for (index, field) in fields.iter().enumerate() {
        field_indexes.insert(field.name.clone(), index);
    }

    let mut arg_locals: Vec<Option<usize>> = vec![None; fields.len()];
    let mut success = true;

    for arg in args {
        let Some(&field_index) = field_indexes.get(&arg.name.name) else {
            if lower_expression(&arg.expr, ctx, instructions) {
                instructions.push(Instruction::Drop(ValueType::Any));
            }
            continue;
        };

        if arg_locals[field_index].is_some() {
            if lower_expression(&arg.expr, ctx, instructions) {
                instructions.push(Instruction::Drop(ValueType::Any));
            }
            continue;
        }

        let field_ty = &fields[field_index].ty;
        let arg_local = ctx.allocate_local(
            format!("__struct_arg_{tmp_id}_{field_index}"),
            Some(field_ty.clone()),
        );

        if !try_lower_bytesn_literal_canonical(&arg.expr, field_ty, ctx, instructions)
            && !lower_expression(&arg.expr, ctx, instructions)
        {
            push_default_for_value_type(field_ty, ctx, instructions);
            success = false;
        }

        instructions.push(Instruction::StoreLocal(arg_local));
        arg_locals[field_index] = Some(arg_local);
    }

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
        if let Some(arg_local) = arg_locals[index] {
            instructions.push(Instruction::LoadLocal(arg_local));
        } else {
            push_default_for_value_type(&field.ty, ctx, instructions);
        }
        instructions.push(Instruction::ArraySet);
    }

    instructions.push(Instruction::LoadLocal(struct_local));
    success
}
