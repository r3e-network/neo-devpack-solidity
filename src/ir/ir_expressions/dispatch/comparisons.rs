use super::*;

pub(crate) fn try_lower_expression_comparisons(
    expr: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    match expr {
        Expression::Less(_, left, right) => Some(lower_binary_expr(
            left,
            right,
            ctx,
            instructions,
            BinaryOperator::Lt,
        )),
        Expression::LessEqual(_, left, right) => Some(lower_binary_expr(
            left,
            right,
            ctx,
            instructions,
            BinaryOperator::Le,
        )),
        Expression::More(_, left, right) => Some(lower_binary_expr(
            left,
            right,
            ctx,
            instructions,
            BinaryOperator::Gt,
        )),
        Expression::MoreEqual(_, left, right) => Some(lower_binary_expr(
            left,
            right,
            ctx,
            instructions,
            BinaryOperator::Ge,
        )),
        Expression::Equal(_, left, right) => Some(lower_binary_expr(
            left,
            right,
            ctx,
            instructions,
            BinaryOperator::Eq,
        )),
        Expression::NotEqual(_, left, right) => Some(lower_binary_expr(
            left,
            right,
            ctx,
            instructions,
            BinaryOperator::Ne,
        )),
        _ => None,
    }
}
