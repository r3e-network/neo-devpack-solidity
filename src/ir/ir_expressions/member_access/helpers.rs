use super::*;

pub(crate) fn typeof_argument(expr: &Expression) -> Option<&Expression> {
    if let Expression::FunctionCall(_, func, args) = expr {
        if args.len() == 1 && matches!(func.as_ref(), Expression::Variable(id) if id.name == "type")
        {
            return Some(&args[0]);
        }
    }

    None
}
