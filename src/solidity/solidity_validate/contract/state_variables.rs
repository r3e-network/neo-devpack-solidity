use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
struct ConstantStateSignature {
    ty: String,
    neo_type: Option<NeoType>,
    is_immutable: bool,
    has_initializer: bool,
    initializer_fingerprint: Option<String>,
}

#[derive(Debug, Default)]
struct SeenStateVariables {
    has_non_constant: bool,
    // M-FE5 fix — track the type of the first non-constant declaration seen
    // for this name so a later inheritance sibling declaring the SAME name
    // with a DIFFERENT type can be reported as a hard error (not a warning).
    // Two public state vars with the same name but different types silently
    // alias the same storage slot (slot is name-derived), corrupting both.
    first_non_constant_ty: Option<String>,
    constant_signatures: Vec<ConstantStateSignature>,
}

fn build_constant_state_signature(state: &StateVariableMetadata) -> ConstantStateSignature {
    ConstantStateSignature {
        ty: state.ty.trim().to_ascii_lowercase(),
        neo_type: state.neo_type.clone(),
        is_immutable: state.is_immutable,
        has_initializer: state.has_initializer,
        initializer_fingerprint: state
            .initializer
            .as_ref()
            .map(build_initializer_fingerprint),
    }
}

fn build_initializer_fingerprint(expr: &Expression) -> String {
    match expr {
        Expression::Parenthesis(_, inner) => build_initializer_fingerprint(inner),
        Expression::BoolLiteral(_, value) => format!("bool:{value}"),
        Expression::AddressLiteral(_, value) => format!("address:{}", value.to_ascii_lowercase()),
        Expression::NumberLiteral(_, integer, exponent, unit) => format!(
            "number:{}:{}:{}",
            integer.to_ascii_lowercase(),
            exponent.to_ascii_lowercase(),
            unit.as_ref()
                .map(|identifier| identifier.name.to_ascii_lowercase())
                .unwrap_or_default()
        ),
        Expression::HexNumberLiteral(_, value, unit) => format!(
            "hex-number:{}:{}",
            value.to_ascii_lowercase(),
            unit.as_ref()
                .map(|identifier| identifier.name.to_ascii_lowercase())
                .unwrap_or_default()
        ),
        Expression::RationalNumberLiteral(_, integer, fraction, exponent, unit) => format!(
            "rational:{}:{}:{}:{}",
            integer.to_ascii_lowercase(),
            fraction.to_ascii_lowercase(),
            exponent.to_ascii_lowercase(),
            unit.as_ref()
                .map(|identifier| identifier.name.to_ascii_lowercase())
                .unwrap_or_default()
        ),
        _ => format!("{expr:?}"),
    }
}

impl SeenStateVariables {
    fn from_state(state: &StateVariableMetadata) -> Self {
        if state.is_constant {
            Self {
                has_non_constant: false,
                first_non_constant_ty: None,
                constant_signatures: vec![build_constant_state_signature(state)],
            }
        } else {
            Self {
                has_non_constant: true,
                first_non_constant_ty: Some(state.ty.trim().to_ascii_lowercase()),
                constant_signatures: Vec::new(),
            }
        }
    }
}

pub(crate) fn validate_state_variables(
    metadata: &ContractMetadata,
    method_name_counts: &std::collections::HashMap<String, usize>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    use std::collections::{HashMap, HashSet};

    let mut state_names: HashMap<String, SeenStateVariables> = HashMap::new();
    let mut warned_non_constant_duplicates: HashSet<String> = HashSet::new();
    let mut warned_conflicting_constant_duplicates: HashSet<String> = HashSet::new();

    for state in &metadata.state_variables {
        // `transient` data location (Solidity 0.8.28+, EIP-1153): NeoVM has no
        // transient store, so the variable compiles to REGULAR persistent
        // storage — it is NOT auto-cleared at the end of the transaction. A
        // transient-reentrancy-guard would stay locked forever; warn loudly so
        // the silent semantic difference is never a surprise (feature audit).
        if state.is_transient {
            let name = state.name.as_deref().unwrap_or("<unnamed>");
            diagnostics.push(
                Diagnostic::warning(format!(
                    "state variable '{name}' is declared `transient` but NeoVM has no transient storage (EIP-1153): it compiles to regular PERSISTENT storage and is NOT cleared at the end of the transaction"
                ))
                .with_code("W_TRANSIENT_PERSISTED")
                .with_suggestion(
                    "clear the variable manually at the end of every entry point (e.g. a reentrancy guard must reset its flag), or use ordinary storage semantics",
                ),
            );
        }
        match &state.name {
            Some(name) => {
                let current_non_constant = !state.is_constant;
                if let Some(seen) = state_names.get_mut(name) {
                    if seen.has_non_constant || current_non_constant {
                        // M-FE5 fix — if both the prior and current declarations
                        // are non-constant AND their types differ, escalate to a
                        // hard error. Two public state vars with the same name
                        // but different types silently alias the same storage
                        // slot (the Neo lowering derives the slot from the
                        // name), corrupting both declarations. Same-name +
                        // same-type duplicates stay a W122 warning.
                        let current_ty = state.ty.trim().to_ascii_lowercase();
                        let type_conflict = current_non_constant
                            && seen.has_non_constant
                            && seen
                                .first_non_constant_ty
                                .as_deref()
                                .is_some_and(|prev| prev != current_ty);
                        if type_conflict {
                            diagnostics.push(
                                Diagnostic::error(format!(
                                    "state variable '{name}' is declared with \
                                     conflicting types across inheritance \
                                     ({} vs {}) — on Neo N3 the storage slot is \
                                     derived from the name, so the two would \
                                     silently alias and corrupt each other",
                                    seen.first_non_constant_ty.as_deref().unwrap_or("?"),
                                    current_ty
                                ))
                                .with_code("E122")
                                .with_suggestion(
                                    "rename one of the conflicting declarations, or \
                                     align their types",
                                ),
                            );
                        } else if warned_non_constant_duplicates.insert(name.clone()) {
                            diagnostics.push(
                                Diagnostic::warning(format!(
                                    "duplicate state variable '{name}' detected while flattening/merging contracts"
                                ))
                                .with_code("W122")
                                .with_suggestion(
                                    "confirm inherited/storage layout expectations; Neo lowering keeps the first declaration semantics where possible",
                                ),
                            );
                        }
                        seen.has_non_constant = true;
                        if seen.first_non_constant_ty.is_none() && current_non_constant {
                            seen.first_non_constant_ty = Some(current_ty);
                        }
                    } else {
                        let current_signature = build_constant_state_signature(state);
                        let is_identical_duplicate = seen
                            .constant_signatures
                            .iter()
                            .any(|signature| signature == &current_signature);

                        if !is_identical_duplicate {
                            if warned_conflicting_constant_duplicates.insert(name.clone()) {
                                diagnostics.push(
                                    Diagnostic::warning(format!(
                                        "conflicting duplicate constant state variable '{name}' detected while merging libraries"
                                    ))
                                    .with_code("W121")
                                    .with_suggestion(
                                        "keep merged constants bit-for-bit identical, or qualify by library name to avoid ambiguity",
                                    ),
                                );
                            }
                            seen.constant_signatures.push(current_signature);
                        }
                    }
                } else {
                    state_names.insert(name.clone(), SeenStateVariables::from_state(state));
                }
            }
            None => diagnostics.push(
                Diagnostic::error("state variable declared without a name")
                    .with_code("STATE_VARIABLE_NAME_MISSING"),
            ),
        }

        if state
            .visibility
            .as_deref()
            .map(|v| v.eq_ignore_ascii_case("public"))
            == Some(true)
        {
            if let Some(name) = state.name.as_deref() {
                if method_name_counts.get(name).copied().unwrap_or(0) > 1 {
                    diagnostics.push(
                        Diagnostic::warning(format!(
                            "public state variable '{name}' conflicts with a function of the same name"
                        ))
                        .with_code("W123")
                        .with_suggestion(
                            "if this came from merged dependencies, prefer explicit wrapper names in user-facing ABIs",
                        ),
                    );
                }
            }
        }

        if state.neo_type.is_none() {
            let lower_ty = state.ty.to_ascii_lowercase();
            if lower_ty.starts_with("fixed") || lower_ty.starts_with("ufixed") {
                diagnostics.push(
                    Diagnostic::error(format!(
                        "state variable '{}' uses fixed-point type '{}' which is not supported on NeoVM",
                        state.name.as_deref().unwrap_or("<unnamed>"),
                        state.ty
                    ))
                    .with_suggestion(
                        "use scaled integer arithmetic instead (e.g., multiply by 10^18 for 18 decimal places)"
                    ),
                );
            } else {
                diagnostics.push(
                    Diagnostic::error(format!(
                        "state variable '{}' has unsupported type '{}'",
                        state.name.as_deref().unwrap_or("<unnamed>"),
                        state.ty
                    ))
                    .with_code("UNSUPPORTED_STATE_TYPE"),
                );
            }
        }

        if state.is_constant && !state.has_initializer {
            diagnostics.push(
                Diagnostic::error(format!(
                    "constant state variable '{}' must have an initializer",
                    state.name.as_deref().unwrap_or("<unnamed>")
                ))
                .with_code("CONSTANT_MISSING_INITIALIZER"),
            );
        }
    }
}
