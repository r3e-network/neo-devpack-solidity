use super::*;

/// Task #94 — lower the children of a tuple-return expression, recursively
/// flattening nested `Expression::List` into leaf values. Each leaf is a
/// single expression lowered via `lower_expression`; `flat_count` is
/// incremented once per leaf so the caller can pass the right `arg_count`
/// to `AbiEncode`.
pub(crate) fn flatten_tuple_return_params(
    params: &[(solang_parser::pt::Loc, Option<solang_parser::pt::Parameter>)],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
    flat_count: &mut usize,
) -> bool {
    for (_, param) in params.iter() {
        let Some(parameter) = param else {
            return false;
        };
        match &parameter.ty {
            Expression::List(_, inner_params) => {
                if !flatten_tuple_return_params(inner_params, ctx, instructions, flat_count) {
                    return false;
                }
            }
            expr => {
                let Some(value_type) = infer_type_from_expression(expr, ctx) else {
                    return false;
                };
                if !lower_static_abi_return_expr_slot(expr, &value_type, ctx, instructions) {
                    return false;
                }
                *flat_count += 1;
            }
        }
    }
    true
}
