use super::*;

pub(crate) fn try_lower_expression_conditional(
    expr: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    match expr {
        Expression::ConditionalOperator(_, condition, then_expr, else_expr) => {
            // Ternary operator: condition ? then_expr : else_expr
            // Semantics: evaluate condition, then evaluate ONLY one branch based on result

            // Generate unique labels for this ternary expression
            let else_label = ctx.next_label();
            let end_label = ctx.next_label();

            // Step 1: Evaluate condition
            if !lower_expression(condition, ctx, instructions) {
                return Some(false);
            }

            // Step 2: Jump to else branch if condition is false (zero).
            instructions.push(Instruction::JumpIf { target: else_label });

            // Step 3: Evaluate then branch (condition was true/non-zero)
            if !lower_expression(then_expr, ctx, instructions) {
                return Some(false);
            }
            instructions.push(Instruction::Jump { target: end_label });

            // Step 4: Else branch label
            instructions.push(Instruction::Label(else_label));

            // Step 5: Evaluate else branch (condition was false/zero)
            if !lower_expression(else_expr, ctx, instructions) {
                return Some(false);
            }

            // Step 6: End label - result is on stack from whichever branch executed
            instructions.push(Instruction::Label(end_label));

            Some(true)
        }
        _ => None,
    }
}
