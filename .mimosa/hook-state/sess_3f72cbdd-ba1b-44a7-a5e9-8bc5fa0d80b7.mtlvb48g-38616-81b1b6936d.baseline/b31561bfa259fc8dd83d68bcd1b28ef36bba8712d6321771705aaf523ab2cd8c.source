use super::*;

pub(crate) fn try_lower_expression_binary_ops(
    expr: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    match expr {
        Expression::Add(_, left, right) => Some(lower_binary_expr(
            left,
            right,
            ctx,
            instructions,
            BinaryOperator::Add,
        )),
        Expression::Subtract(_, left, right) => {
            // `0 - x` is exactly `-x`, including two's-complement overflow: for
            // `x == type(intN).min` both wrap to `intN.min`. In an unchecked
            // block the plain subtraction path produces the out-of-range
            // `+2^(N-1)` (and would fault NeoVM for int256), so reroute a
            // zero-minus-signed to the unary negate wrap that handles every
            // width correctly (bug-hunt #2). Only signed operands reroute;
            // unsigned `0 - x` stays on the underflow-checking sub path.
            if ctx.in_unchecked_block()
                && is_zero_literal(left)
                && matches!(
                    infer_type_from_expression(right, ctx),
                    Some(ValueType::Integer { signed: true, .. })
                )
            {
                return Some(lower_negate_expression(right, ctx, instructions));
            }
            Some(lower_binary_expr(
                left,
                right,
                ctx,
                instructions,
                BinaryOperator::Sub,
            ))
        }
        Expression::Multiply(_, left, right) => Some(lower_binary_expr(
            left,
            right,
            ctx,
            instructions,
            BinaryOperator::Mul,
        )),
        Expression::Divide(_, left, right) => Some(lower_binary_expr(
            left,
            right,
            ctx,
            instructions,
            BinaryOperator::Div,
        )),
        Expression::Modulo(_, left, right) => Some(lower_binary_expr(
            left,
            right,
            ctx,
            instructions,
            BinaryOperator::Mod,
        )),
        Expression::ShiftLeft(_, left, right) => Some(lower_binary_expr(
            left,
            right,
            ctx,
            instructions,
            BinaryOperator::Shl,
        )),
        Expression::ShiftRight(_, left, right) => Some(lower_binary_expr(
            left,
            right,
            ctx,
            instructions,
            BinaryOperator::Shr,
        )),
        Expression::BitwiseAnd(_, left, right) => Some(lower_binary_expr(
            left,
            right,
            ctx,
            instructions,
            BinaryOperator::BitAnd,
        )),
        Expression::BitwiseOr(_, left, right) => Some(lower_binary_expr(
            left,
            right,
            ctx,
            instructions,
            BinaryOperator::BitOr,
        )),
        Expression::BitwiseXor(_, left, right) => Some(lower_binary_expr(
            left,
            right,
            ctx,
            instructions,
            BinaryOperator::BitXor,
        )),
        _ => None,
    }
}
