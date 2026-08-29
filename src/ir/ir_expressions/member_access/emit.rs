use super::*;

pub(crate) fn lower_member_access_expression(
    expr: &Expression,
    inner: &Expression,
    member: &Identifier,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    // `super.member` in a non-call context (e.g., reading a parent state variable).
    // The compiler flattens inheritance so base-specific members are not separately addressable.
    if matches!(inner, Expression::Variable(id) if id.name == "super") {
        ctx.record_error_with_suggestion(
            format!(
                "super.{} member access is not supported; use super.method() calls only",
                member.name
            ),
            "access the member directly by name, call super.method(), or extract shared logic into a named internal function",
        );
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::zero(),
        )));
        return false;
    }

    if let Some(result) = try_lower_native_contract_constant(inner, member, instructions) {
        return result;
    }

    if let Some(result) = try_lower_syscalls_contract_constant(inner, member, instructions) {
        return result;
    }

    if let Some(result) = try_lower_selector_member_access(inner, member, ctx, instructions) {
        return result;
    }

    if let Some(result) = try_lower_runtime_member_access(inner, member, ctx, instructions) {
        return result;
    }

    if let Some(result) = try_lower_address_balance(inner, member, ctx, instructions) {
        return result;
    }

    if let Some(result) = try_lower_length_property(inner, member, ctx, instructions) {
        return result;
    }

    if let Some(result) = try_lower_current_key(inner, member, ctx, instructions) {
        return result;
    }

    if let Some(result) = try_lower_current_value(inner, member, ctx, instructions) {
        return result;
    }

    if let Some(result) = try_lower_code_property(inner, member, ctx, instructions) {
        return result;
    }

    if let Some(result) = try_lower_type_bound_max(inner, member, ctx, instructions) {
        return result;
    }

    if let Some(result) = try_lower_type_bound_min(inner, member, ctx, instructions) {
        return result;
    }

    if let Some(result) = try_lower_interface_id(inner, member, ctx, instructions) {
        return result;
    }

    if let Some(result) = try_lower_type_name(inner, member, ctx, instructions) {
        return result;
    }

    if let Some(result) = try_lower_type_code(inner, member, ctx, instructions) {
        return result;
    }

    lower_generic_member_access(expr, inner, member, ctx, instructions)
}
