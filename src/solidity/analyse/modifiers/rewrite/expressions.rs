use super::*;

pub(crate) fn rewrite_expression(
    expr: &Expression,
    subs: &std::collections::HashMap<String, Expression>,
) -> Expression {
    match expr {
        Expression::Variable(id) => subs.get(&id.name).cloned().unwrap_or_else(|| expr.clone()),
        Expression::PostIncrement(loc, inner) => {
            Expression::PostIncrement(*loc, Box::new(rewrite_expression(inner, subs)))
        }
        Expression::PostDecrement(loc, inner) => {
            Expression::PostDecrement(*loc, Box::new(rewrite_expression(inner, subs)))
        }
        Expression::New(loc, inner) => {
            Expression::New(*loc, Box::new(rewrite_expression(inner, subs)))
        }
        Expression::ArraySubscript(loc, inner, index) => Expression::ArraySubscript(
            *loc,
            Box::new(rewrite_expression(inner, subs)),
            index
                .as_ref()
                .map(|e| Box::new(rewrite_expression(e, subs))),
        ),
        Expression::ArraySlice(loc, inner, start, end) => Expression::ArraySlice(
            *loc,
            Box::new(rewrite_expression(inner, subs)),
            start
                .as_ref()
                .map(|e| Box::new(rewrite_expression(e, subs))),
            end.as_ref().map(|e| Box::new(rewrite_expression(e, subs))),
        ),
        Expression::Parenthesis(loc, inner) => {
            Expression::Parenthesis(*loc, Box::new(rewrite_expression(inner, subs)))
        }
        Expression::MemberAccess(loc, inner, member) => Expression::MemberAccess(
            *loc,
            Box::new(rewrite_expression(inner, subs)),
            member.clone(),
        ),
        Expression::FunctionCall(loc, func, args) => Expression::FunctionCall(
            *loc,
            Box::new(rewrite_expression(func, subs)),
            args.iter().map(|a| rewrite_expression(a, subs)).collect(),
        ),
        Expression::FunctionCallBlock(loc, func, block) => Expression::FunctionCallBlock(
            *loc,
            Box::new(rewrite_expression(func, subs)),
            Box::new(rewrite_statement(block, subs, None)),
        ),
        Expression::NamedFunctionCall(loc, func, args) => {
            let rewritten_args: Vec<NamedArgument> = args
                .iter()
                .map(|arg| NamedArgument {
                    loc: arg.loc,
                    name: arg.name.clone(),
                    expr: rewrite_expression(&arg.expr, subs),
                })
                .collect();
            Expression::NamedFunctionCall(
                *loc,
                Box::new(rewrite_expression(func, subs)),
                rewritten_args,
            )
        }
        Expression::Not(loc, inner) => {
            Expression::Not(*loc, Box::new(rewrite_expression(inner, subs)))
        }
        Expression::BitwiseNot(loc, inner) => {
            Expression::BitwiseNot(*loc, Box::new(rewrite_expression(inner, subs)))
        }
        Expression::Delete(loc, inner) => {
            Expression::Delete(*loc, Box::new(rewrite_expression(inner, subs)))
        }
        Expression::PreIncrement(loc, inner) => {
            Expression::PreIncrement(*loc, Box::new(rewrite_expression(inner, subs)))
        }
        Expression::PreDecrement(loc, inner) => {
            Expression::PreDecrement(*loc, Box::new(rewrite_expression(inner, subs)))
        }
        Expression::UnaryPlus(loc, inner) => {
            Expression::UnaryPlus(*loc, Box::new(rewrite_expression(inner, subs)))
        }
        Expression::Negate(loc, inner) => {
            Expression::Negate(*loc, Box::new(rewrite_expression(inner, subs)))
        }

        Expression::Power(loc, a, b) => Expression::Power(
            *loc,
            Box::new(rewrite_expression(a, subs)),
            Box::new(rewrite_expression(b, subs)),
        ),
        Expression::Multiply(loc, a, b) => Expression::Multiply(
            *loc,
            Box::new(rewrite_expression(a, subs)),
            Box::new(rewrite_expression(b, subs)),
        ),
        Expression::Divide(loc, a, b) => Expression::Divide(
            *loc,
            Box::new(rewrite_expression(a, subs)),
            Box::new(rewrite_expression(b, subs)),
        ),
        Expression::Modulo(loc, a, b) => Expression::Modulo(
            *loc,
            Box::new(rewrite_expression(a, subs)),
            Box::new(rewrite_expression(b, subs)),
        ),
        Expression::Add(loc, a, b) => Expression::Add(
            *loc,
            Box::new(rewrite_expression(a, subs)),
            Box::new(rewrite_expression(b, subs)),
        ),
        Expression::Subtract(loc, a, b) => Expression::Subtract(
            *loc,
            Box::new(rewrite_expression(a, subs)),
            Box::new(rewrite_expression(b, subs)),
        ),
        Expression::ShiftLeft(loc, a, b) => Expression::ShiftLeft(
            *loc,
            Box::new(rewrite_expression(a, subs)),
            Box::new(rewrite_expression(b, subs)),
        ),
        Expression::ShiftRight(loc, a, b) => Expression::ShiftRight(
            *loc,
            Box::new(rewrite_expression(a, subs)),
            Box::new(rewrite_expression(b, subs)),
        ),
        Expression::BitwiseAnd(loc, a, b) => Expression::BitwiseAnd(
            *loc,
            Box::new(rewrite_expression(a, subs)),
            Box::new(rewrite_expression(b, subs)),
        ),
        Expression::BitwiseXor(loc, a, b) => Expression::BitwiseXor(
            *loc,
            Box::new(rewrite_expression(a, subs)),
            Box::new(rewrite_expression(b, subs)),
        ),
        Expression::BitwiseOr(loc, a, b) => Expression::BitwiseOr(
            *loc,
            Box::new(rewrite_expression(a, subs)),
            Box::new(rewrite_expression(b, subs)),
        ),
        Expression::Less(loc, a, b) => Expression::Less(
            *loc,
            Box::new(rewrite_expression(a, subs)),
            Box::new(rewrite_expression(b, subs)),
        ),
        Expression::More(loc, a, b) => Expression::More(
            *loc,
            Box::new(rewrite_expression(a, subs)),
            Box::new(rewrite_expression(b, subs)),
        ),
        Expression::LessEqual(loc, a, b) => Expression::LessEqual(
            *loc,
            Box::new(rewrite_expression(a, subs)),
            Box::new(rewrite_expression(b, subs)),
        ),
        Expression::MoreEqual(loc, a, b) => Expression::MoreEqual(
            *loc,
            Box::new(rewrite_expression(a, subs)),
            Box::new(rewrite_expression(b, subs)),
        ),
        Expression::Equal(loc, a, b) => Expression::Equal(
            *loc,
            Box::new(rewrite_expression(a, subs)),
            Box::new(rewrite_expression(b, subs)),
        ),
        Expression::NotEqual(loc, a, b) => Expression::NotEqual(
            *loc,
            Box::new(rewrite_expression(a, subs)),
            Box::new(rewrite_expression(b, subs)),
        ),
        Expression::And(loc, a, b) => Expression::And(
            *loc,
            Box::new(rewrite_expression(a, subs)),
            Box::new(rewrite_expression(b, subs)),
        ),
        Expression::Or(loc, a, b) => Expression::Or(
            *loc,
            Box::new(rewrite_expression(a, subs)),
            Box::new(rewrite_expression(b, subs)),
        ),
        Expression::ConditionalOperator(loc, cond, a, b) => Expression::ConditionalOperator(
            *loc,
            Box::new(rewrite_expression(cond, subs)),
            Box::new(rewrite_expression(a, subs)),
            Box::new(rewrite_expression(b, subs)),
        ),
        Expression::Assign(loc, a, b) => Expression::Assign(
            *loc,
            Box::new(rewrite_expression(a, subs)),
            Box::new(rewrite_expression(b, subs)),
        ),
        Expression::AssignOr(loc, a, b) => Expression::AssignOr(
            *loc,
            Box::new(rewrite_expression(a, subs)),
            Box::new(rewrite_expression(b, subs)),
        ),
        Expression::AssignAnd(loc, a, b) => Expression::AssignAnd(
            *loc,
            Box::new(rewrite_expression(a, subs)),
            Box::new(rewrite_expression(b, subs)),
        ),
        Expression::AssignXor(loc, a, b) => Expression::AssignXor(
            *loc,
            Box::new(rewrite_expression(a, subs)),
            Box::new(rewrite_expression(b, subs)),
        ),
        Expression::AssignShiftLeft(loc, a, b) => Expression::AssignShiftLeft(
            *loc,
            Box::new(rewrite_expression(a, subs)),
            Box::new(rewrite_expression(b, subs)),
        ),
        Expression::AssignShiftRight(loc, a, b) => Expression::AssignShiftRight(
            *loc,
            Box::new(rewrite_expression(a, subs)),
            Box::new(rewrite_expression(b, subs)),
        ),
        Expression::AssignAdd(loc, a, b) => Expression::AssignAdd(
            *loc,
            Box::new(rewrite_expression(a, subs)),
            Box::new(rewrite_expression(b, subs)),
        ),
        Expression::AssignSubtract(loc, a, b) => Expression::AssignSubtract(
            *loc,
            Box::new(rewrite_expression(a, subs)),
            Box::new(rewrite_expression(b, subs)),
        ),
        Expression::AssignMultiply(loc, a, b) => Expression::AssignMultiply(
            *loc,
            Box::new(rewrite_expression(a, subs)),
            Box::new(rewrite_expression(b, subs)),
        ),
        Expression::AssignDivide(loc, a, b) => Expression::AssignDivide(
            *loc,
            Box::new(rewrite_expression(a, subs)),
            Box::new(rewrite_expression(b, subs)),
        ),
        Expression::AssignModulo(loc, a, b) => Expression::AssignModulo(
            *loc,
            Box::new(rewrite_expression(a, subs)),
            Box::new(rewrite_expression(b, subs)),
        ),

        Expression::List(loc, params) => Expression::List(
            *loc,
            params
                .iter()
                .map(|(loc, param)| {
                    (
                        *loc,
                        param.as_ref().map(|p| Parameter {
                            loc: p.loc,
                            annotation: p.annotation.clone(),
                            ty: rewrite_expression(&p.ty, subs),
                            storage: p.storage.clone(),
                            name: p.name.clone(),
                        }),
                    )
                })
                .collect(),
        ),
        Expression::ArrayLiteral(loc, values) => Expression::ArrayLiteral(
            *loc,
            values.iter().map(|v| rewrite_expression(v, subs)).collect(),
        ),

        // Literals and leaf nodes.
        Expression::BoolLiteral(_, _)
        | Expression::NumberLiteral(_, _, _, _)
        | Expression::RationalNumberLiteral(_, _, _, _, _)
        | Expression::HexNumberLiteral(_, _, _)
        | Expression::StringLiteral(_)
        | Expression::Type(_, _)
        | Expression::HexLiteral(_)
        | Expression::AddressLiteral(_, _) => expr.clone(),
    }
}
