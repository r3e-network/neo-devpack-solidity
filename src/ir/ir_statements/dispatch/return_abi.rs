use super::*;

/// Lower `return <buffer_expr;>` straight to a `RET` after a size guard.
pub(crate) fn lower_abi_decode_passthrough_return(
    buffer_expr: &Expression,
    expected_bytes: u32,
    cmp: BinaryOperator,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    let pre_len = instructions.len();
    if !lower_expression(buffer_expr, ctx, instructions) {
        instructions.truncate(pre_len);
        return false;
    }
    let decode_ok_label = ctx.next_label();
    instructions.push(Instruction::Dup);
    instructions.push(Instruction::GetSize);
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(expected_bytes),
    )));
    instructions.push(Instruction::BinaryOp(cmp));
    instructions.push(Instruction::JumpIf {
        target: decode_ok_label,
    });
    emit_panic(0x41, instructions);
    instructions.push(Instruction::Label(decode_ok_label));
    instructions.push(Instruction::Return);
    true
}

/// Task #116 — true iff `expr` is `abi.decode(buf, types)` (with 2 args).
pub(crate) fn extract_abi_decode_call(expr: &Expression) -> Option<Vec<Expression>> {
    let Expression::FunctionCall(_, func, args) = expr else {
        return None;
    };
    if args.len() != 2 {
        return None;
    }
    let Expression::MemberAccess(_, receiver, member) = func.as_ref() else {
        return None;
    };
    if member.name != "decode" {
        return None;
    }
    match receiver.as_ref() {
        Expression::Variable(id) if id.name == "abi" => Some(args.clone()),
        _ => None,
    }
}

/// Task #116 — match exactly `expected_arity` static ABI return types.
pub(crate) fn abi_decode_types_match_return_arity(
    args: &[Expression],
    expected_arity: usize,
) -> bool {
    if expected_arity < 2 {
        return false;
    }
    let Some(Expression::List(_, params)) = args.get(1) else {
        return false;
    };
    params.len() == expected_arity
        && params
            .iter()
            .all(|(_, param)| param.as_ref().is_some_and(|p| is_static_abi_type(&p.ty)))
}

/// Task #127 — match a mixed static/dynamic ABI tuple with at least one dynamic
/// type, used by the verbatim decode passthrough optimization.
pub(crate) fn abi_decode_types_match_return_arity_mixed(
    args: &[Expression],
    expected_arity: usize,
) -> bool {
    if expected_arity < 2 {
        return false;
    }
    let Some(Expression::List(_, params)) = args.get(1) else {
        return false;
    };
    if params.len() != expected_arity {
        return false;
    }
    let mut all_ok = true;
    let mut has_dynamic = false;
    for (_, param) in params.iter() {
        let Some(p) = param.as_ref() else {
            return false;
        };
        if is_static_abi_type(&p.ty) {
            continue;
        }
        if is_dynamic_abi_type(&p.ty) {
            has_dynamic = true;
            continue;
        }
        all_ok = false;
    }
    all_ok && has_dynamic
}
