use super::*;
use crate::solidity::base_last_name;

pub(crate) fn build_modifier_definition_map(
    contract: &ContractIR,
) -> std::collections::HashMap<(String, usize), FunctionIR> {
    let mut map: std::collections::HashMap<(String, usize), FunctionIR> =
        std::collections::HashMap::new();
    for func in &contract.functions {
        if matches!(func.ty, FunctionTy::Modifier) {
            let key = (func.name.clone(), func.parameters.len());
            match map.get(&key) {
                Some(existing) => {
                    // Prefer concrete modifier implementations over stubs/bodyless declarations.
                    if existing.body.is_none() && func.body.is_some() {
                        map.insert(key, func.clone());
                    }
                }
                None => {
                    map.insert(key, func.clone());
                }
            }
        }
    }
    map
}

pub(crate) fn apply_modifier_calls_to_body(
    original_body: &Statement,
    modifier_calls: &[Base],
    modifier_defs: &std::collections::HashMap<(String, usize), FunctionIR>,
) -> Result<Statement, SolidityError> {
    let (body, _had_epilogue) =
        apply_modifier_calls_to_body_with_epilogue(original_body, modifier_calls, modifier_defs)?;
    Ok(body)
}

/// Task #114 — extended modifier expansion that also reports whether at least
/// one applied modifier contained an epilogue (statements after the `_;`
/// placeholder). The caller uses this flag to set
/// `FunctionIR::had_modifier_epilogue`, which in turn tells the IR lowerer
/// to redirect `return expr;` inside the expanded body to synthetic return
/// slots + a jump past the epilogue so tail statements still run.
pub(crate) fn apply_modifier_calls_to_body_with_epilogue(
    original_body: &Statement,
    modifier_calls: &[Base],
    modifier_defs: &std::collections::HashMap<(String, usize), FunctionIR>,
) -> Result<(Statement, bool), SolidityError> {
    let mut current = original_body.clone();
    let mut has_epilogue = false;
    // Task #114 — first pass: detect whether any applied modifier has an
    // epilogue. We need to know this up front so the body we substitute for
    // `_;` can be wrapped in a `do { ... } while(false)` scope — giving
    // `return expr;` inside the body a well-defined break target that
    // falls through to the epilogue instead of skipping it.
    for modifier_call in modifier_calls.iter() {
        let Some(name) = base_last_name(modifier_call) else {
            continue;
        };
        let arg_count = modifier_call.args.as_ref().map(|a| a.len()).unwrap_or(0);
        let Some(modifier_def) = modifier_defs.get(&(name, arg_count)) else {
            continue;
        };
        let Some(modifier_body) = modifier_def.body.as_ref() else {
            continue;
        };
        if modifier_body_has_epilogue(modifier_body) {
            has_epilogue = true;
            break;
        }
    }

    // If any modifier has an epilogue, wrap the ORIGINAL body in
    // `do { body } while(false)` so `return expr;` lowers to "store synthetic
    // slot + break" (break lands at the do-while end label, which then falls
    // through to the modifier epilogue statements). See the `modifier_break_stack`
    // machinery in `src/ir/context/lowering_context.rs` + the redirect in
    // `src/ir/statements/dispatch/return_revert.rs`.
    if has_epilogue {
        current = Statement::DoWhile(
            Loc::Implicit,
            Box::new(current),
            Expression::BoolLiteral(Loc::Implicit, false),
        );
    }

    for modifier_call in modifier_calls.iter().rev() {
        let Some(name) = base_last_name(modifier_call) else {
            continue;
        };
        let arg_list: Vec<Expression> = modifier_call.args.clone().unwrap_or_default();
        let key = (name.clone(), arg_list.len());
        let Some(modifier_def) = modifier_defs.get(&key) else {
            return Err(SolidityError::Analysis(format!(
                "unresolved modifier '{name}' with {} argument(s)",
                arg_list.len()
            )));
        };

        let Some(modifier_body) = modifier_def.body.as_ref() else {
            // Compatibility fallback for abstract/bodyless modifier declarations:
            // treat as a no-op wrapper when no concrete body is available.
            continue;
        };

        let mut normalized_args = arg_list.clone();
        if normalized_args.len() < modifier_def.parameters.len() {
            let missing = modifier_def.parameters.len() - normalized_args.len();
            for _ in 0..missing {
                normalized_args.push(Expression::NumberLiteral(
                    Default::default(),
                    "0".to_string(),
                    "".to_string(),
                    None,
                ));
            }
        } else if normalized_args.len() > modifier_def.parameters.len() {
            normalized_args.truncate(modifier_def.parameters.len());
        }

        // Evaluate each non-trivial modifier argument EXACTLY ONCE into a
        // synthetic temporary at the top of the inlined modifier body, then
        // substitute parameter references with that temporary. Without this, a
        // side-effecting argument (e.g. `check(tick())`) would be cloned at
        // every parameter use and run once per use, corrupting state.
        let prefix = next_arg_temp_prefix("modarg");
        let (substitutions, temp_decls) = build_parameter_substitutions_single_eval(
            &modifier_def.parameters,
            &normalized_args,
            &prefix,
        )?;
        let rewritten = rewrite_statement(modifier_body, &substitutions, Some(&current));
        current = prepend_arg_temp_decls(temp_decls, rewritten);
    }

    Ok((current, has_epilogue))
}

/// Task #114 — returns `true` if the modifier body contains at least one
/// statement AFTER an `_;` placeholder. Used to decide whether expansion
/// must route returns through the synthetic-slot redirect in the IR layer.
///
/// We look across nested blocks: `_;` can appear inside an if-branch, for
/// example `modifier guarded() { if (cond) { _; log(); } }` — the `log()` is
/// an epilogue that must run after the inlined body on the taken branch.
pub(crate) fn modifier_body_has_epilogue(body: &Statement) -> bool {
    match body {
        Statement::Block { statements, .. } => {
            // Check if any statement after the first `_;` exists (top level),
            // or any nested block contains an epilogue.
            let mut seen_placeholder = false;
            for stmt in statements {
                if seen_placeholder {
                    return true;
                }
                if modifier_placeholder_stmt(stmt) {
                    seen_placeholder = true;
                    continue;
                }
                // Recurse into nested control-flow to catch placeholders
                // buried inside if/for/while bodies.
                if modifier_body_has_epilogue(stmt) {
                    return true;
                }
            }
            false
        }
        Statement::If(_, _, then_stmt, else_stmt) => {
            modifier_body_has_epilogue(then_stmt)
                || else_stmt
                    .as_ref()
                    .map(|s| modifier_body_has_epilogue(s))
                    .unwrap_or(false)
        }
        Statement::While(_, _, body) | Statement::DoWhile(_, body, _) => {
            modifier_body_has_epilogue(body)
        }
        Statement::For(_, _, _, _, body) => body
            .as_ref()
            .map(|b| modifier_body_has_epilogue(b))
            .unwrap_or(false),
        _ => false,
    }
}

pub(crate) fn statement_list_from_body(body: &Statement) -> Vec<Statement> {
    match body {
        Statement::Block { statements, .. } => statements.clone(),
        stmt => vec![stmt.clone()],
    }
}
