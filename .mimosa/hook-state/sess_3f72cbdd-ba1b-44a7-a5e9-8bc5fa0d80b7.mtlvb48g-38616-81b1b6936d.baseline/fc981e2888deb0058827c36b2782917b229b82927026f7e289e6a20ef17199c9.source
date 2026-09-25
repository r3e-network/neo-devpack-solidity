use super::*;

pub(crate) fn lower_generic_member_access(
    expr: &Expression,
    inner: &Expression,
    member: &Identifier,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    pub(crate) fn is_namespace_identifier(name: &str, ctx: &LoweringContext) -> bool {
        !ctx.param_index_map.contains_key(name)
            && ctx.resolve_local(name).is_none()
            && !ctx.state_index_map.contains_key(name)
    }

    // Enum value reference: `EnumName.VARIANT`.
    if let Expression::Variable(base) = inner {
        if let Some(variants) = ctx.enum_variant_map.get(&base.name) {
            if let Some(value) = variants.get(&member.name) {
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                    BigInt::from(*value),
                )));
                return true;
            }
        }
    }

    // Namespaced enum reference: `LibraryOrContract.EnumType.VARIANT`.
    if let Expression::MemberAccess(_, namespace_expr, enum_type) = inner {
        if let Expression::Variable(namespace_id) = namespace_expr.as_ref() {
            if is_namespace_identifier(&namespace_id.name, ctx)
                && ctx.is_contract_type_name(&namespace_id.name)
            {
                if let Some(variants) = ctx.enum_variant_map.get(&enum_type.name) {
                    if let Some(value) = variants.get(&member.name) {
                        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                            BigInt::from(*value),
                        )));
                        return true;
                    }
                }
            }
        }
    }

    // Namespaced constant/state-style access: `Library.CONSTANT`.
    if let Expression::Variable(namespace_id) = inner {
        if is_namespace_identifier(&namespace_id.name, ctx)
            && ctx.is_contract_type_name(&namespace_id.name)
        {
            if let Some(state_index) = ctx.state_index_map.get(&member.name).copied() {
                let state_type = ctx.state_type(state_index).cloned();
                let const_initializer = ctx.state_metadata(state_index).and_then(|meta| {
                    if meta.is_constant {
                        meta.initializer.clone()
                    } else {
                        None
                    }
                });

                if let Some(initializer) = const_initializer {
                    // Guard against infinite recursion: if this constant is already
                    // being resolved (its initializer references another constant
                    // that aliases back to this one), fall through to LoadState.
                    if ctx.is_resolving_constant(state_index) {
                        instructions.push(Instruction::LoadState(state_index));
                        return true;
                    }
                    if matches!(state_type.as_ref(), Some(ValueType::Address)) {
                        if let Some(bytes) = address_bytes_le_from_expression(&initializer) {
                            instructions
                                .push(Instruction::PushLiteral(LiteralValue::Address(bytes)));
                            return true;
                        }
                    }
                    ctx.push_resolving_constant(state_index);
                    let result = lower_expression(&initializer, ctx, instructions);
                    ctx.pop_resolving_constant();
                    return result;
                }

                instructions.push(Instruction::LoadState(state_index));
                return true;
            }

            // The namespace is a known contract/library/enum type, but `member`
            // is not one of its constants, state variables, or enum variants.
            // Previously this fell through to the generic handler below and was
            // silently folded to `0` (coerced to `address(0)` for an `address`
            // type) with no diagnostic — so a typo in a library constant (e.g.
            // `NativeContracts.ROLE_CONTRACT`, which does not exist) compiled to
            // an all-zero address. Fail loudly instead (A7).
            ctx.record_error_with_suggestion(
                format!(
                    "unknown member `{}::{}`: `{}` has no such constant, state variable, or type",
                    namespace_id.name, member.name, namespace_id.name
                ),
                "check the spelling, or import the library/contract that declares this member",
            );
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::zero(),
            )));
            return false;
        }
    }

    if let Some(reference) = resolve_storage_reference(expr, ctx) {
        return emit_storage_load(&reference, ctx, instructions);
    }

    // Memory struct field access: `tmp.field`
    if let Some(ValueType::Struct { fields, .. }) = infer_type_from_expression(inner, ctx) {
        if let Some(field_index) = fields.iter().position(|field| field.name == member.name) {
            if !lower_expression(inner, ctx, instructions) {
                return false;
            }
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::from(field_index as u64),
            )));
            instructions.push(Instruction::ArrayGet);
            return true;
        }
    }

    load_expression(inner, ctx, instructions);
    instructions.push(Instruction::Drop(ValueType::Any));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    true
}
