use super::*;

pub(crate) fn rewrite_statement(
    stmt: &Statement,
    subs: &std::collections::HashMap<String, Expression>,
    placeholder_body: Option<&Statement>,
) -> Statement {
    if modifier_placeholder_stmt(stmt) {
        if let Some(body) = placeholder_body {
            return body.clone();
        }
    }

    match stmt {
        Statement::Block {
            loc,
            unchecked,
            statements,
        } => Statement::Block {
            loc: *loc,
            unchecked: *unchecked,
            statements: statements
                .iter()
                .map(|s| rewrite_statement(s, subs, placeholder_body))
                .collect(),
        },
        Statement::Assembly { .. } => stmt.clone(),
        Statement::Args(loc, args) => {
            let rewritten_args: Vec<NamedArgument> = args
                .iter()
                .map(|arg| NamedArgument {
                    loc: arg.loc,
                    name: arg.name.clone(),
                    expr: rewrite_expression(&arg.expr, subs),
                })
                .collect();
            Statement::Args(*loc, rewritten_args)
        }
        Statement::If(loc, cond, then_stmt, else_stmt) => Statement::If(
            *loc,
            rewrite_expression(cond, subs),
            Box::new(rewrite_statement(then_stmt, subs, placeholder_body)),
            else_stmt
                .as_ref()
                .map(|s| Box::new(rewrite_statement(s, subs, placeholder_body))),
        ),
        Statement::While(loc, cond, body) => Statement::While(
            *loc,
            rewrite_expression(cond, subs),
            Box::new(rewrite_statement(body, subs, placeholder_body)),
        ),
        Statement::Expression(loc, expr) => {
            Statement::Expression(*loc, rewrite_expression(expr, subs))
        }
        Statement::VariableDefinition(loc, decl, init) => Statement::VariableDefinition(
            *loc,
            decl.clone(),
            init.as_ref().map(|e| rewrite_expression(e, subs)),
        ),
        Statement::For(loc, init, cond, next, body) => Statement::For(
            *loc,
            init.as_ref()
                .map(|s| Box::new(rewrite_statement(s, subs, placeholder_body))),
            cond.as_ref().map(|e| Box::new(rewrite_expression(e, subs))),
            next.as_ref().map(|e| Box::new(rewrite_expression(e, subs))),
            body.as_ref()
                .map(|s| Box::new(rewrite_statement(s, subs, placeholder_body))),
        ),
        Statement::DoWhile(loc, body, cond) => Statement::DoWhile(
            *loc,
            Box::new(rewrite_statement(body, subs, placeholder_body)),
            rewrite_expression(cond, subs),
        ),
        Statement::Continue(_) | Statement::Break(_) => stmt.clone(),
        Statement::Return(loc, expr) => {
            Statement::Return(*loc, expr.as_ref().map(|e| rewrite_expression(e, subs)))
        }
        Statement::Revert(loc, path, args) => Statement::Revert(
            *loc,
            path.clone(),
            args.iter().map(|e| rewrite_expression(e, subs)).collect(),
        ),
        Statement::RevertNamedArgs(loc, path, args) => {
            let rewritten_args: Vec<NamedArgument> = args
                .iter()
                .map(|arg| NamedArgument {
                    loc: arg.loc,
                    name: arg.name.clone(),
                    expr: rewrite_expression(&arg.expr, subs),
                })
                .collect();
            Statement::RevertNamedArgs(*loc, path.clone(), rewritten_args)
        }
        Statement::Emit(loc, expr) => Statement::Emit(*loc, rewrite_expression(expr, subs)),
        Statement::Try(loc, expr, handler, catches) => Statement::Try(
            *loc,
            rewrite_expression(expr, subs),
            handler.as_ref().map(|(params, stmt)| {
                (
                    params.clone(),
                    Box::new(rewrite_statement(stmt, subs, placeholder_body)),
                )
            }),
            catches
                .iter()
                .map(|clause| match clause {
                    CatchClause::Simple(loc, param, stmt) => CatchClause::Simple(
                        *loc,
                        param.clone(),
                        rewrite_statement(stmt, subs, placeholder_body),
                    ),
                    CatchClause::Named(loc, path, param, stmt) => CatchClause::Named(
                        *loc,
                        path.clone(),
                        param.clone(),
                        rewrite_statement(stmt, subs, placeholder_body),
                    ),
                })
                .collect(),
        ),
        Statement::Error(_) => stmt.clone(),
    }
}
