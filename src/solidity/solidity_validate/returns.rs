use super::*;

pub fn extract_return_expression(body: &Option<Statement>) -> Option<Expression> {
    let statement = body.as_ref()?;

    match statement {
        Statement::Block { statements, .. } => {
            if statements.len() == 1 {
                if let Statement::Return(_, expr) = &statements[0] {
                    expr.clone()
                } else {
                    None
                }
            } else {
                None
            }
        }
        Statement::Return(_, expr) => expr.clone(),
        _ => None,
    }
}

pub(crate) fn check_return_statements(
    statement: &Statement,
    expected_count: usize,
    function_name: &str,
    return_arities: &std::collections::HashMap<(String, usize), usize>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match statement {
        Statement::Block { statements, .. } => {
            for stmt in statements {
                check_return_statements(
                    stmt,
                    expected_count,
                    function_name,
                    return_arities,
                    diagnostics,
                );
            }
        }
        Statement::If(_, _, then_stmt, else_stmt) => {
            check_return_statements(
                then_stmt,
                expected_count,
                function_name,
                return_arities,
                diagnostics,
            );
            if let Some(else_stmt) = else_stmt {
                check_return_statements(
                    else_stmt,
                    expected_count,
                    function_name,
                    return_arities,
                    diagnostics,
                );
            }
        }
        Statement::While(_, _, body) => {
            check_return_statements(body, expected_count, function_name, return_arities, diagnostics);
        }
        Statement::DoWhile(_, body, _) => {
            check_return_statements(body, expected_count, function_name, return_arities, diagnostics);
        }
        Statement::For(_, init, _, _, body) => {
            if let Some(init_stmt) = init {
                check_return_statements(
                    init_stmt,
                    expected_count,
                    function_name,
                    return_arities,
                    diagnostics,
                );
            }
            if let Some(body_stmt) = body {
                check_return_statements(
                    body_stmt,
                    expected_count,
                    function_name,
                    return_arities,
                    diagnostics,
                );
            }
        }
        Statement::Try(_, _, handler, catches) => {
            if let Some((_, handler_stmt)) = handler {
                check_return_statements(
                    handler_stmt,
                    expected_count,
                    function_name,
                    return_arities,
                    diagnostics,
                );
            }
            for catch in catches {
                match catch {
                    CatchClause::Simple(_, _, stmt) => {
                        check_return_statements(
                            stmt,
                            expected_count,
                            function_name,
                            return_arities,
                            diagnostics,
                        )
                    }
                    CatchClause::Named(_, _, _, stmt) => {
                        check_return_statements(
                            stmt,
                            expected_count,
                            function_name,
                            return_arities,
                            diagnostics,
                        )
                    }
                }
            }
        }
        Statement::Return(_, expr) => match (expected_count, expr) {
            // Returning a value from a no-return function is invalid Solidity,
            // but the synthesized getter for a `public` state variable reaches
            // here with `expected_count == 0` (its return type is not tracked
            // in the metadata the validator sees), so promoting this to an
            // error false-positives on valid auto-getters. Keep it a warning;
            // only the explicit-tuple arity mismatches below (which a getter
            // never produces) are promoted to hard errors.
            (0, Some(_)) => diagnostics.push(Diagnostic::warning(format!(
                "function '{function_name}' returns a value but is declared without a return type"
            ))),
            (0, None) => {}
            // `return;` with a declared return is VALID when the returns are
            // NAMED (their current values are returned), so this stays a
            // warning to avoid false-positives on valid named-return functions.
            (1, None) => diagnostics.push(Diagnostic::warning(format!(
                "function '{function_name}' declares a return value but returns without one"
            ))),
            // A single declared return given an explicit multi-element tuple is
            // a definite arity mismatch — hard error.
            (1, Some(Expression::List(_, list))) if list.len() != 1 => {
                diagnostics.push(Diagnostic::error(format!(
                    "function '{function_name}' declares 1 return value but returns a {}-tuple",
                    list.len()
                )))
            }
            (1, Some(_)) => {}
            (expected, Some(expr)) => {
                if let Expression::List(_, list) = expr {
                    let actual = list.len();
                    if actual != expected {
                        // An explicit `return (a, b, ...)` whose element count
                        // disagrees with the declaration is a definite mismatch.
                        diagnostics.push(
                            Diagnostic::error(format!(
                                "function '{function_name}' expected {expected} return values but found {actual}"
                            ))
                            .with_code("RETURN_MISMATCH"),
                        );
                    }
                } else {
                    let inferred = match expr {
                        Expression::FunctionCall(_, callee, args) => {
                            let name = match callee.as_ref() {
                                Expression::Variable(id) => Some(id.name.as_str()),
                                Expression::MemberAccess(_, _, member) => Some(member.name.as_str()),
                                _ => None,
                            };
                            name.and_then(|name| {
                                return_arities
                                    .get(&(name.to_string(), args.len()))
                                    .copied()
                            })
                        }
                        _ => None,
                    };

                    if let Some(actual) = inferred {
                        if actual != expected {
                            diagnostics.push(
                                Diagnostic::warning(format!(
                                    "function '{function_name}' expected {expected} return values but call returns {actual}"
                                ))
                                .with_code("RETURN_MISMATCH"),
                            );
                        }
                    } else {
                        diagnostics.push(Diagnostic::warning(format!(
                            "function '{function_name}' should return {expected} values but expression does not match tuple"
                        )));
                    }
                }
            }
            (expected, None) => diagnostics.push(Diagnostic::warning(format!(
                "function '{function_name}' declares {expected} return values but returns without one"
            ))),
        },
        _ => {}
    }
}
