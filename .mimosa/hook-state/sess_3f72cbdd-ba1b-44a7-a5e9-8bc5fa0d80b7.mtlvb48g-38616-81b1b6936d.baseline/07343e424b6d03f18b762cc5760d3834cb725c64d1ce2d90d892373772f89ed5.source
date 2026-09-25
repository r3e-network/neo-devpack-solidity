use super::*;

// Task #55: AST-level scan for `new Contract(...)` expressions.
//
// The IR lowering for `new Contract(...)` pushes a 20-byte zero address
// without emitting any builtin call, so the permission-inference passes that
// run over `ir::Module` cannot see the intent. Walk the parsed function bodies
// in `ContractMetadata` to detect `Expression::New(FunctionCall(Variable(T)))`
// where `T` is a known contract type, and wire `ContractManagement.deploy`
// into the manifest so a later compiler revision that actually performs the
// deployment is not blocked by a missing permission.
pub(crate) fn contract_uses_new_contract(metadata: &ContractMetadata) -> bool {
    let contract_types: std::collections::HashSet<&str> =
        metadata.contract_types.iter().map(String::as_str).collect();

    metadata.methods.iter().any(|method| {
        method
            .body
            .as_ref()
            .is_some_and(|body| scan_stmt(body, &contract_types))
    })
}

pub(crate) fn is_new_contract(
    expr: &solang_parser::pt::Expression,
    contract_types: &std::collections::HashSet<&str>,
) -> bool {
    use solang_parser::pt::Expression as E;
    if let E::New(_, inner) = expr {
        if let E::FunctionCall(_, func, _) = inner.as_ref() {
            if let E::Variable(id) = func.as_ref() {
                return contract_types.contains(id.name.as_str());
            }
        }
    }
    false
}

pub(crate) fn scan_stmt(
    stmt: &solang_parser::pt::Statement,
    contract_types: &std::collections::HashSet<&str>,
) -> bool {
    use solang_parser::pt::Statement as S;
    match stmt {
        S::Block { statements, .. } => statements.iter().any(|s| scan_stmt(s, contract_types)),
        S::If(_, cond, t, e) => {
            scan_expr(cond, contract_types)
                || scan_stmt(t, contract_types)
                || e.as_ref().is_some_and(|s| scan_stmt(s, contract_types))
        }
        S::While(_, cond, body) | S::DoWhile(_, body, cond) => {
            scan_expr(cond, contract_types) || scan_stmt(body, contract_types)
        }
        S::Expression(_, expr) => scan_expr(expr, contract_types),
        S::VariableDefinition(_, _, init) => {
            init.as_ref().is_some_and(|e| scan_expr(e, contract_types))
        }
        S::For(_, i, c, n, b) => {
            i.as_ref().is_some_and(|s| scan_stmt(s, contract_types))
                || c.as_ref().is_some_and(|e| scan_expr(e, contract_types))
                || n.as_ref().is_some_and(|e| scan_expr(e, contract_types))
                || b.as_ref().is_some_and(|s| scan_stmt(s, contract_types))
        }
        S::Return(_, Some(expr)) | S::Emit(_, expr) => scan_expr(expr, contract_types),
        S::Revert(_, _, args) => args.iter().any(|e| scan_expr(e, contract_types)),
        S::Try(_, expr, returns, clauses) => {
            scan_expr(expr, contract_types)
                || returns
                    .as_ref()
                    .is_some_and(|(_, b)| scan_stmt(b, contract_types))
                || clauses.iter().any(|c| match c {
                    solang_parser::pt::CatchClause::Simple(_, _, b)
                    | solang_parser::pt::CatchClause::Named(_, _, _, b) => {
                        scan_stmt(b, contract_types)
                    }
                })
        }
        _ => false,
    }
}

// Walk children of an expression. We only need to recurse into the structural
// containers that can wrap a `new Contract(...)` sub-expression. The match
// deliberately enumerates common shapes rather than every binary operator;
// `new T(...)` normally appears as a statement, assignment RHS, return value,
// argument, or conditional branch, all of which are covered below.
pub(crate) fn scan_expr(
    expr: &solang_parser::pt::Expression,
    contract_types: &std::collections::HashSet<&str>,
) -> bool {
    use solang_parser::pt::Expression as E;
    if is_new_contract(expr, contract_types) {
        return true;
    }
    match expr {
        E::New(_, i) | E::Parenthesis(_, i) | E::MemberAccess(_, i, _) | E::Delete(_, i) => {
            scan_expr(i, contract_types)
        }
        E::FunctionCall(_, func, args) => {
            scan_expr(func, contract_types) || args.iter().any(|a| scan_expr(a, contract_types))
        }
        E::NamedFunctionCall(_, func, args) => {
            scan_expr(func, contract_types)
                || args.iter().any(|a| scan_expr(&a.expr, contract_types))
        }
        E::ArraySubscript(_, a, b) => {
            scan_expr(a, contract_types) || b.as_ref().is_some_and(|e| scan_expr(e, contract_types))
        }
        E::ConditionalOperator(_, c, a, b) => {
            scan_expr(c, contract_types)
                || scan_expr(a, contract_types)
                || scan_expr(b, contract_types)
        }
        // Assignments and binary ops carry a left + right expression we must
        // descend into. Rather than enumerate 30 variants, we fall through
        // to `_` because in practice `new Contract(...)` is not nested
        // inside arithmetic/comparison expressions — it is the RHS of an
        // assignment or variable definition, which the statement-level
        // handler already visits.
        E::Assign(_, a, b) => scan_expr(a, contract_types) || scan_expr(b, contract_types),
        E::ArrayLiteral(_, values) => values.iter().any(|v| scan_expr(v, contract_types)),
        _ => false,
    }
}
