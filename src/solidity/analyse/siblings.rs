use super::*;

/// Task #83 — walk a statement tree collecting every `new X()` target name
/// that matches a known primary contract. Mirrors the ast_scan permissions
/// pass but accumulates matches instead of returning a boolean.
pub(crate) fn collect_new_contract_refs(
    stmt: &Statement,
    primary_names: &std::collections::HashSet<String>,
    sink: &mut std::collections::HashSet<String>,
) {
    stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
        collect_new_contract_refs_inner(stmt, primary_names, sink)
    })
}

pub(crate) fn collect_new_contract_refs_inner(
    stmt: &Statement,
    primary_names: &std::collections::HashSet<String>,
    sink: &mut std::collections::HashSet<String>,
) {
    match stmt {
        Statement::Block { statements, .. } => {
            for s in statements {
                collect_new_contract_refs(s, primary_names, sink);
            }
        }
        Statement::If(_, cond, t, e) => {
            collect_new_refs_expr(cond, primary_names, sink);
            collect_new_contract_refs(t, primary_names, sink);
            if let Some(s) = e {
                collect_new_contract_refs(s, primary_names, sink);
            }
        }
        Statement::While(_, cond, body) | Statement::DoWhile(_, body, cond) => {
            collect_new_refs_expr(cond, primary_names, sink);
            collect_new_contract_refs(body, primary_names, sink);
        }
        Statement::Expression(_, expr) => collect_new_refs_expr(expr, primary_names, sink),
        Statement::VariableDefinition(_, _, Some(expr)) => {
            collect_new_refs_expr(expr, primary_names, sink);
        }
        Statement::VariableDefinition(_, _, None) => {}
        Statement::For(_, i, c, n, b) => {
            if let Some(s) = i {
                collect_new_contract_refs(s, primary_names, sink);
            }
            if let Some(e) = c {
                collect_new_refs_expr(e, primary_names, sink);
            }
            if let Some(e) = n {
                collect_new_refs_expr(e, primary_names, sink);
            }
            if let Some(s) = b {
                collect_new_contract_refs(s, primary_names, sink);
            }
        }
        Statement::Return(_, Some(expr)) | Statement::Emit(_, expr) => {
            collect_new_refs_expr(expr, primary_names, sink);
        }
        Statement::Revert(_, _, args) => {
            for e in args {
                collect_new_refs_expr(e, primary_names, sink);
            }
        }
        Statement::Try(_, expr, returns, clauses) => {
            collect_new_refs_expr(expr, primary_names, sink);
            if let Some((_, b)) = returns {
                collect_new_contract_refs(b, primary_names, sink);
            }
            for c in clauses {
                match c {
                    CatchClause::Simple(_, _, b) | CatchClause::Named(_, _, _, b) => {
                        collect_new_contract_refs(b, primary_names, sink);
                    }
                }
            }
        }
        _ => {}
    }
}

/// Task #115 — statement-level walk that collects every `I(expr).method(...)`
/// interface-cast receiver where `I` is a known interface declared in the
/// same source unit. Mirrors `collect_new_contract_refs` but tracks a
/// different alphabet of names (interface kinds, not primary contracts).
pub(crate) fn collect_interface_casts_stmt(
    stmt: &Statement,
    interface_names: &std::collections::HashSet<String>,
    sink: &mut std::collections::HashSet<String>,
) {
    stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
        collect_interface_casts_stmt_inner(stmt, interface_names, sink)
    })
}

pub(crate) fn collect_interface_casts_stmt_inner(
    stmt: &Statement,
    interface_names: &std::collections::HashSet<String>,
    sink: &mut std::collections::HashSet<String>,
) {
    match stmt {
        Statement::Block { statements, .. } => {
            for s in statements {
                collect_interface_casts_stmt(s, interface_names, sink);
            }
        }
        Statement::If(_, cond, t, e) => {
            collect_interface_casts_expr(cond, interface_names, sink);
            collect_interface_casts_stmt(t, interface_names, sink);
            if let Some(s) = e {
                collect_interface_casts_stmt(s, interface_names, sink);
            }
        }
        Statement::While(_, cond, body) | Statement::DoWhile(_, body, cond) => {
            collect_interface_casts_expr(cond, interface_names, sink);
            collect_interface_casts_stmt(body, interface_names, sink);
        }
        Statement::Expression(_, expr) => collect_interface_casts_expr(expr, interface_names, sink),
        Statement::VariableDefinition(_, _, Some(expr)) => {
            collect_interface_casts_expr(expr, interface_names, sink);
        }
        Statement::VariableDefinition(_, _, None) => {}
        Statement::For(_, i, c, n, b) => {
            if let Some(s) = i {
                collect_interface_casts_stmt(s, interface_names, sink);
            }
            if let Some(e) = c {
                collect_interface_casts_expr(e, interface_names, sink);
            }
            if let Some(e) = n {
                collect_interface_casts_expr(e, interface_names, sink);
            }
            if let Some(s) = b {
                collect_interface_casts_stmt(s, interface_names, sink);
            }
        }
        Statement::Return(_, Some(expr)) | Statement::Emit(_, expr) => {
            collect_interface_casts_expr(expr, interface_names, sink);
        }
        Statement::Revert(_, _, args) => {
            for e in args {
                collect_interface_casts_expr(e, interface_names, sink);
            }
        }
        Statement::Try(_, expr, returns, clauses) => {
            collect_interface_casts_expr(expr, interface_names, sink);
            if let Some((_, b)) = returns {
                collect_interface_casts_stmt(b, interface_names, sink);
            }
            for c in clauses {
                match c {
                    CatchClause::Simple(_, _, b) | CatchClause::Named(_, _, _, b) => {
                        collect_interface_casts_stmt(b, interface_names, sink);
                    }
                }
            }
        }
        _ => {}
    }
}

/// Task #115 — expression-level half of `collect_interface_casts_stmt`.
/// Matches `FunctionCall(Variable(I), _)` where `I` is a known interface
/// name. The parser emits this shape for interface casts like `I(addr)`.
pub(crate) fn collect_interface_casts_expr(
    expr: &Expression,
    interface_names: &std::collections::HashSet<String>,
    sink: &mut std::collections::HashSet<String>,
) {
    stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
        collect_interface_casts_expr_inner(expr, interface_names, sink)
    })
}

pub(crate) fn collect_interface_casts_expr_inner(
    expr: &Expression,
    interface_names: &std::collections::HashSet<String>,
    sink: &mut std::collections::HashSet<String>,
) {
    if let Expression::FunctionCall(_, func, _) = expr {
        if let Expression::Variable(id) = func.as_ref() {
            if interface_names.contains(&id.name) {
                sink.insert(id.name.clone());
            }
        }
    }
    match expr {
        Expression::New(_, i)
        | Expression::Parenthesis(_, i)
        | Expression::MemberAccess(_, i, _)
        | Expression::Delete(_, i) => collect_interface_casts_expr(i, interface_names, sink),
        Expression::FunctionCall(_, func, args) => {
            collect_interface_casts_expr(func, interface_names, sink);
            for a in args {
                collect_interface_casts_expr(a, interface_names, sink);
            }
        }
        // Task #125 — symmetric fix with `collect_new_refs_expr`: the
        // `try-expr { success-block }` lowering parks the call inside a
        // FunctionCallBlock, so interface-cast chains such as
        // `try I(t).getR() returns (R r) { ... } catch ...` would also
        // silently skip the sibling-merge trigger without this arm.
        Expression::FunctionCallBlock(_, call, block) => {
            collect_interface_casts_expr(call, interface_names, sink);
            collect_interface_casts_stmt(block, interface_names, sink);
        }
        Expression::NamedFunctionCall(_, func, args) => {
            collect_interface_casts_expr(func, interface_names, sink);
            for a in args {
                collect_interface_casts_expr(&a.expr, interface_names, sink);
            }
        }
        Expression::ArraySubscript(_, a, b) => {
            collect_interface_casts_expr(a, interface_names, sink);
            if let Some(e) = b {
                collect_interface_casts_expr(e, interface_names, sink);
            }
        }
        Expression::ConditionalOperator(_, c, a, b) => {
            collect_interface_casts_expr(c, interface_names, sink);
            collect_interface_casts_expr(a, interface_names, sink);
            collect_interface_casts_expr(b, interface_names, sink);
        }
        Expression::Assign(_, a, b) => {
            collect_interface_casts_expr(a, interface_names, sink);
            collect_interface_casts_expr(b, interface_names, sink);
        }
        Expression::ArrayLiteral(_, values) => {
            for v in values {
                collect_interface_casts_expr(v, interface_names, sink);
            }
        }
        _ => {}
    }
}

/// Task #83 — expression-level half of `collect_new_contract_refs`. Matches
/// `Expression::New(FunctionCall(Variable(name), _))` and recurses through
/// the usual expression containers.
pub(crate) fn collect_new_refs_expr(
    expr: &Expression,
    primary_names: &std::collections::HashSet<String>,
    sink: &mut std::collections::HashSet<String>,
) {
    stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
        collect_new_refs_expr_inner(expr, primary_names, sink)
    })
}

pub(crate) fn collect_new_refs_expr_inner(
    expr: &Expression,
    primary_names: &std::collections::HashSet<String>,
    sink: &mut std::collections::HashSet<String>,
) {
    if let Expression::New(_, inner) = expr {
        if let Expression::FunctionCall(_, func, _) = inner.as_ref() {
            if let Expression::Variable(id) = func.as_ref() {
                if primary_names.contains(&id.name) {
                    sink.insert(id.name.clone());
                }
            }
        }
    }
    // Task K4 — `B(addr)` cast expressions mean A plans to call into B
    // through an address typed as B. The parser lowers these as
    // `FunctionCall(Variable("B"), [addr])`, identical in shape to a
    // `B.staticCall(addr)` helper, so we match on that before the generic
    // FunctionCall recursion below picks off the args.
    if let Expression::FunctionCall(_, func, _) = expr {
        if let Expression::Variable(id) = func.as_ref() {
            if primary_names.contains(&id.name) {
                sink.insert(id.name.clone());
            }
        }
    }
    match expr {
        Expression::New(_, i)
        | Expression::Parenthesis(_, i)
        | Expression::MemberAccess(_, i, _)
        | Expression::Delete(_, i) => collect_new_refs_expr(i, primary_names, sink),
        Expression::FunctionCall(_, func, args) => {
            collect_new_refs_expr(func, primary_names, sink);
            for a in args {
                collect_new_refs_expr(a, primary_names, sink);
            }
        }
        // Task #125 — `try X { ... } catch ...` parses the leading
        // `try-expr { success-block }` as `FunctionCallBlock(call, block)`
        // on the expression-tree side, so `try Target(t).willRevert() { ... }`
        // arrives here with `call = FunctionCall(MemberAccess(FunctionCall(
        // Variable("Target"), [t]), "willRevert"), [])` wrapped in a
        // FunctionCallBlock. Without this arm the walker's `_ => {}`
        // silently dropped the cast chain, so `Target` never made the
        // sibling-merge `referenced` set and `willRevert` never entered
        // C's `self_method_offsets` table — the runtime's
        // `handle_contract_call` then fell through to `invoke_native_contract`
        // which returned `Null` for the zero-placeholder hash, so the
        // target's `revert("bad")` was never dispatched and the outer
        // try-arm fired with its literal "ok" instead of the expected
        // `catch Error(string)` binding. The success block body is a
        // Statement, not an Expression, so we use the statement walker
        // for it — symmetric with the `Statement::Try` arm above.
        Expression::FunctionCallBlock(_, call, block) => {
            collect_new_refs_expr(call, primary_names, sink);
            collect_new_contract_refs(block, primary_names, sink);
        }
        Expression::NamedFunctionCall(_, func, args) => {
            collect_new_refs_expr(func, primary_names, sink);
            for a in args {
                collect_new_refs_expr(&a.expr, primary_names, sink);
            }
        }
        Expression::ArraySubscript(_, a, b) => {
            collect_new_refs_expr(a, primary_names, sink);
            if let Some(e) = b {
                collect_new_refs_expr(e, primary_names, sink);
            }
        }
        Expression::ConditionalOperator(_, c, a, b) => {
            collect_new_refs_expr(c, primary_names, sink);
            collect_new_refs_expr(a, primary_names, sink);
            collect_new_refs_expr(b, primary_names, sink);
        }
        Expression::Assign(_, a, b) => {
            collect_new_refs_expr(a, primary_names, sink);
            collect_new_refs_expr(b, primary_names, sink);
        }
        Expression::ArrayLiteral(_, values) => {
            for v in values {
                collect_new_refs_expr(v, primary_names, sink);
            }
        }
        _ => {}
    }
}

/// Task #194 — statement walker that collects statically resolvable method
/// names from low-level `addr.call(...)` / `addr.staticcall(...)` payloads.
/// Mirrors the shape of `collect_new_contract_refs` but feeds a different
/// alphabet: plain method names (e.g. `"getValue"`) that the sibling-merge
/// pass later cross-references against every sibling primary's declared
/// method set.
pub(crate) fn collect_low_level_call_method_refs_stmt(
    stmt: &Statement,
    sink: &mut std::collections::HashSet<String>,
) {
    stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
        collect_low_level_call_method_refs_stmt_inner(stmt, sink)
    })
}

pub(crate) fn collect_low_level_call_method_refs_stmt_inner(
    stmt: &Statement,
    sink: &mut std::collections::HashSet<String>,
) {
    match stmt {
        Statement::Block { statements, .. } => {
            for s in statements {
                collect_low_level_call_method_refs_stmt(s, sink);
            }
        }
        Statement::If(_, cond, t, e) => {
            collect_low_level_call_method_refs_expr(cond, sink);
            collect_low_level_call_method_refs_stmt(t, sink);
            if let Some(s) = e {
                collect_low_level_call_method_refs_stmt(s, sink);
            }
        }
        Statement::While(_, cond, body) | Statement::DoWhile(_, body, cond) => {
            collect_low_level_call_method_refs_expr(cond, sink);
            collect_low_level_call_method_refs_stmt(body, sink);
        }
        Statement::Expression(_, expr) => {
            collect_low_level_call_method_refs_expr(expr, sink);
        }
        Statement::VariableDefinition(_, _, Some(expr)) => {
            collect_low_level_call_method_refs_expr(expr, sink);
        }
        Statement::VariableDefinition(_, _, None) => {}
        Statement::For(_, i, c, n, b) => {
            if let Some(s) = i {
                collect_low_level_call_method_refs_stmt(s, sink);
            }
            if let Some(e) = c {
                collect_low_level_call_method_refs_expr(e, sink);
            }
            if let Some(e) = n {
                collect_low_level_call_method_refs_expr(e, sink);
            }
            if let Some(s) = b {
                collect_low_level_call_method_refs_stmt(s, sink);
            }
        }
        Statement::Return(_, Some(expr)) | Statement::Emit(_, expr) => {
            collect_low_level_call_method_refs_expr(expr, sink);
        }
        Statement::Revert(_, _, args) => {
            for e in args {
                collect_low_level_call_method_refs_expr(e, sink);
            }
        }
        Statement::Try(_, expr, returns, clauses) => {
            collect_low_level_call_method_refs_expr(expr, sink);
            if let Some((_, b)) = returns {
                collect_low_level_call_method_refs_stmt(b, sink);
            }
            for c in clauses {
                match c {
                    CatchClause::Simple(_, _, b) | CatchClause::Named(_, _, _, b) => {
                        collect_low_level_call_method_refs_stmt(b, sink);
                    }
                }
            }
        }
        _ => {}
    }
}

/// Task #194 — expression walker that recognises `<receiver>.call(payload)`
/// / `<receiver>.staticcall(payload)` / `<receiver>.delegatecall(payload)`
/// shapes, then peels the `abi.encodeWith{Selector,Signature}` /
/// `abi.encodeCall` wrapper on the payload to extract the Solidity method
/// name when it can be resolved at compile time. The extracted name is
/// later matched against every sibling primary's declared public/external
/// method set.
///
/// Static resolution handles:
///   - `abi.encodeWithSignature("m(T)", …)` — literal signature string,
///     name taken from the pre-`(` fragment.
///   - `abi.encodeWithSelector(bytes4(keccak256("m(T)")))` —
///     compile-time hash of a literal signature string.
///   - `abi.encodeWithSelector(Type.method.selector)` /
///     `abi.encodeCall(Type.method, (…))` — static member-access.
///
/// Runtime-computed selectors (e.g. `abi.encodeWithSelector(someRuntimeSel,
/// …)`) stay unresolved and yield nothing — the compiler's caller-side
/// lowering similarly cannot route those through sibling-merge, so they
/// fall through to the real cross-contract dispatch path.
pub(crate) fn collect_low_level_call_method_refs_expr(
    expr: &Expression,
    sink: &mut std::collections::HashSet<String>,
) {
    stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
        collect_low_level_call_method_refs_expr_inner(expr, sink)
    })
}

pub(crate) fn collect_low_level_call_method_refs_expr_inner(
    expr: &Expression,
    sink: &mut std::collections::HashSet<String>,
) {
    if let Expression::FunctionCall(_, func, args) = expr {
        if args.len() == 1 {
            if let Expression::MemberAccess(_, _recv, member) = func.as_ref() {
                let is_low_level =
                    matches!(member.name.as_str(), "call" | "staticcall" | "delegatecall");
                if is_low_level {
                    if let Some(name) = extract_static_method_name_from_payload(&args[0]) {
                        if !name.trim().is_empty() {
                            sink.insert(name);
                        }
                    }
                }
            }
        }
    }
    match expr {
        Expression::New(_, i)
        | Expression::Parenthesis(_, i)
        | Expression::MemberAccess(_, i, _)
        | Expression::Delete(_, i) => collect_low_level_call_method_refs_expr(i, sink),
        Expression::FunctionCall(_, func, args) => {
            collect_low_level_call_method_refs_expr(func, sink);
            for a in args {
                collect_low_level_call_method_refs_expr(a, sink);
            }
        }
        Expression::FunctionCallBlock(_, call, block) => {
            collect_low_level_call_method_refs_expr(call, sink);
            collect_low_level_call_method_refs_stmt(block, sink);
        }
        Expression::NamedFunctionCall(_, func, args) => {
            collect_low_level_call_method_refs_expr(func, sink);
            for a in args {
                collect_low_level_call_method_refs_expr(&a.expr, sink);
            }
        }
        Expression::ArraySubscript(_, a, b) => {
            collect_low_level_call_method_refs_expr(a, sink);
            if let Some(e) = b {
                collect_low_level_call_method_refs_expr(e, sink);
            }
        }
        Expression::ConditionalOperator(_, c, a, b) => {
            collect_low_level_call_method_refs_expr(c, sink);
            collect_low_level_call_method_refs_expr(a, sink);
            collect_low_level_call_method_refs_expr(b, sink);
        }
        Expression::Assign(_, a, b) => {
            collect_low_level_call_method_refs_expr(a, sink);
            collect_low_level_call_method_refs_expr(b, sink);
        }
        Expression::ArrayLiteral(_, values) => {
            for v in values {
                collect_low_level_call_method_refs_expr(v, sink);
            }
        }
        _ => {}
    }
}

/// Task #194 — peel the `abi.encodeWith{Selector,Signature}` /
/// `abi.encodeCall` wrapper of a low-level call payload to extract the
/// Solidity method name when it is statically resolvable.
pub(crate) fn extract_static_method_name_from_payload(expr: &Expression) -> Option<String> {
    stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
        extract_static_method_name_from_payload_inner(expr)
    })
}

pub(crate) fn extract_static_method_name_from_payload_inner(expr: &Expression) -> Option<String> {
    match expr {
        Expression::Parenthesis(_, inner) => extract_static_method_name_from_payload(inner),
        Expression::FunctionCall(_, func, args) => {
            // `bytes(<inner>)` / `bytes4(<inner>)` / `string(<inner>)` casts
            // are transparent — recurse through them.
            if args.len() == 1 {
                if let Expression::Variable(id) = func.as_ref() {
                    if id.name == "bytes" || id.name == "string" {
                        return extract_static_method_name_from_payload(&args[0]);
                    }
                }
                if matches!(func.as_ref(), Expression::Type(_, _)) {
                    return extract_static_method_name_from_payload(&args[0]);
                }
            }

            let Expression::MemberAccess(_, inner, member) = func.as_ref() else {
                return None;
            };

            if !matches!(inner.as_ref(), Expression::Variable(id) if id.name == "abi") {
                return None;
            }

            match member.name.as_str() {
                "encodeWithSignature" => {
                    let first = args.first()?;
                    let signature = extract_static_signature_string(first)?;
                    crate::utils::method_name_from_signature(&signature)
                }
                "encodeWithSelector" => {
                    let first = args.first()?;
                    extract_static_selector_method_name(first)
                }
                "encodeCall" => {
                    // `abi.encodeCall(X.method, (…))` — member-access
                    // function reference resolves to the member name.
                    let first = args.first()?;
                    extract_static_encode_call_method_name(first)
                }
                _ => None,
            }
        }
        _ => None,
    }
}

/// Task #194 — analogue of `resolve_selector_method_name` in
/// `ir/build/selectors.rs` that operates on raw `solang_parser::pt`
/// expressions (the analyse pass runs before the IR is built).
pub(crate) fn extract_static_selector_method_name(expr: &Expression) -> Option<String> {
    stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
        extract_static_selector_method_name_inner(expr)
    })
}

pub(crate) fn extract_static_selector_method_name_inner(expr: &Expression) -> Option<String> {
    match expr {
        Expression::Parenthesis(_, inner) => extract_static_selector_method_name(inner),
        Expression::MemberAccess(_, inner, member) => {
            if member.name == "selector" {
                match inner.as_ref() {
                    Expression::MemberAccess(_, _, function_name) => {
                        let name = function_name.name.trim();
                        if name.is_empty() {
                            None
                        } else {
                            Some(name.to_string())
                        }
                    }
                    Expression::Variable(function_name) => {
                        let name = function_name.name.trim();
                        if name.is_empty() {
                            None
                        } else {
                            Some(name.to_string())
                        }
                    }
                    _ => None,
                }
            } else {
                None
            }
        }
        Expression::FunctionCall(_, func, args) => {
            if matches!(func.as_ref(), Expression::Type(_, _)) && args.len() == 1 {
                return extract_static_selector_method_name(&args[0]);
            }
            if let Expression::Variable(id) = func.as_ref() {
                if (id.name == "bytes" || id.name == "string") && args.len() == 1 {
                    return extract_static_selector_method_name(&args[0]);
                }
                if id.name == "keccak256" && args.len() == 1 {
                    let signature = extract_static_signature_string(&args[0])?;
                    return crate::utils::method_name_from_signature(&signature);
                }
            }
            None
        }
        _ => None,
    }
}

/// Task #194 — recognise the function reference argument of
/// `abi.encodeCall(funcRef, tuple)`. Accepts `Type.method`,
/// `instance.method`, or nested member-access chains and returns the
/// outermost member name.
pub(crate) fn extract_static_encode_call_method_name(expr: &Expression) -> Option<String> {
    stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
        extract_static_encode_call_method_name_inner(expr)
    })
}

pub(crate) fn extract_static_encode_call_method_name_inner(expr: &Expression) -> Option<String> {
    match expr {
        Expression::Parenthesis(_, inner) => extract_static_encode_call_method_name(inner),
        Expression::MemberAccess(_, _inner, member) => {
            if member.name == "selector" {
                // `abi.encodeCall(X.method.selector, …)` — uncommon but we
                // can still recover the method name by looking one level up.
                if let Expression::MemberAccess(_, _, function_name) = _inner.as_ref() {
                    let name = function_name.name.trim();
                    if !name.is_empty() {
                        return Some(name.to_string());
                    }
                }
                return None;
            }
            let name = member.name.trim();
            if name.is_empty() {
                None
            } else {
                Some(name.to_string())
            }
        }
        _ => None,
    }
}

/// Task #194 — compile-time constant string extraction. Peels `bytes(…)`
/// / `string(…)` casts and unwraps `Parenthesis` but stops at the first
/// non-literal (e.g. `constant`-stored strings are not read here because
/// the analyse pass doesn't have access to the lowering context yet).
pub(crate) fn extract_static_signature_string(expr: &Expression) -> Option<String> {
    stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
        extract_static_signature_string_inner(expr)
    })
}

pub(crate) fn extract_static_signature_string_inner(expr: &Expression) -> Option<String> {
    match expr {
        Expression::Parenthesis(_, inner) => extract_static_signature_string(inner),
        Expression::StringLiteral(parts) => {
            let mut bytes = Vec::new();
            for part in parts {
                bytes.extend_from_slice(part.string.as_bytes());
            }
            Some(String::from_utf8_lossy(&bytes).to_string())
        }
        Expression::FunctionCall(_, func, args) if args.len() == 1 => match func.as_ref() {
            Expression::Type(_, _) => extract_static_signature_string(&args[0]),
            Expression::Variable(id) if id.name == "bytes" || id.name == "string" => {
                extract_static_signature_string(&args[0])
            }
            _ => None,
        },
        _ => None,
    }
}

/// Task #206 — compute the DIRECT sibling-primary references a contract body
/// introduces. Used by the sibling-merge closure so multi-hop cross-contract
/// call chains pull every reachable primary into the root artifact's
/// self-dispatch table.
pub(crate) fn collect_direct_sibling_contract_refs(
    contract: &ContractIR,
    primary_names: &std::collections::HashSet<String>,
    interface_names: &std::collections::HashSet<String>,
    interface_impls: &std::collections::HashMap<String, Vec<String>>,
    primary_method_names: &std::collections::HashMap<String, std::collections::HashSet<String>>,
) -> std::collections::HashSet<String> {
    let mut referenced: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut iface_refs: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut low_level_method_refs: std::collections::HashSet<String> =
        std::collections::HashSet::new();

    for function in &contract.functions {
        if let Some(body) = function.body.as_ref() {
            collect_new_contract_refs(body, primary_names, &mut referenced);
            collect_interface_casts_stmt(body, interface_names, &mut iface_refs);
            collect_low_level_call_method_refs_stmt(body, &mut low_level_method_refs);
        }
        for p in function.parameters.iter().chain(function.returns.iter()) {
            if primary_names.contains(&p.ty) {
                referenced.insert(p.ty.clone());
            }
            if interface_names.contains(&p.ty) {
                iface_refs.insert(p.ty.clone());
            }
        }
    }

    for state in &contract.state_variables {
        if primary_names.contains(&state.ty) {
            referenced.insert(state.ty.clone());
        }
        if interface_names.contains(&state.ty) {
            iface_refs.insert(state.ty.clone());
        }
        if let Some(init) = state.initializer.as_ref() {
            collect_new_refs_expr(init, primary_names, &mut referenced);
            collect_interface_casts_expr(init, interface_names, &mut iface_refs);
            collect_low_level_call_method_refs_expr(init, &mut low_level_method_refs);
        }
    }

    for iface in &iface_refs {
        if let Some(impls) = interface_impls.get(iface) {
            for prim in impls {
                if prim != &contract.name {
                    referenced.insert(prim.clone());
                }
            }
        }
    }

    if !low_level_method_refs.is_empty() {
        for (prim_name, prim_methods) in primary_method_names {
            if prim_name == &contract.name {
                continue;
            }
            if low_level_method_refs
                .iter()
                .any(|method| prim_methods.contains(method))
            {
                referenced.insert(prim_name.clone());
            }
        }
    }

    referenced.remove(&contract.name);
    referenced
}
