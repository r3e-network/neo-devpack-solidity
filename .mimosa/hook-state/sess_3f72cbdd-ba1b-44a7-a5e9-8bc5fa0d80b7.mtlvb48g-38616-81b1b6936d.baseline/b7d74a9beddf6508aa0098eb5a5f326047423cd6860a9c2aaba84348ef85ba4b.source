use super::*;

pub fn validate_contract(metadata: &ContractMetadata) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    let method_name_counts = build_method_name_counts(metadata);
    let constructor_count = validate_methods(metadata, &mut diagnostics);

    if constructor_count > 1 {
        diagnostics.push(
            Diagnostic::error(format!(
                "multiple constructors defined ({constructor_count} total)"
            ))
            .with_code("MULTIPLE_CONSTRUCTORS"),
        );
    }

    validate_state_variables(metadata, &method_name_counts, &mut diagnostics);
    validate_events(metadata, &mut diagnostics);
    validate_return_types(metadata, &mut diagnostics);
    validate_erc_nep_patterns(metadata, &mut diagnostics);
    validate_using_directives(metadata, &mut diagnostics);
    validate_type_definitions(metadata, &mut diagnostics);
    validate_library(metadata, &mut diagnostics);
    validate_abstract_contract(metadata, &mut diagnostics);
    validate_flatten_warnings(metadata, &mut diagnostics);

    diagnostics
}

fn validate_using_directives(metadata: &ContractMetadata, diagnostics: &mut Vec<Diagnostic>) {
    // Member-style `using` (`x.f()`) is validated during IR lowering where the
    // receiver type and overload information are available. Here we surface a
    // frontend-only diagnostic for USER-DEFINED OPERATORS (Solidity 0.8.19+,
    // `using {add as +, eq as ==} for T global`): these parse and compile, but
    // neo-solc does NOT dispatch them — the operand's user-defined-value-type
    // identity is erased to its underlying type before operator lowering, so
    // `a + b` emits RAW arithmetic on the underlying representation instead of
    // a call to the bound function. Warn loudly so the result is never a
    // silent wrong value (a known gap; the alternative is a dangerous
    // accept-but-miscompile).
    for directive in &metadata.using_directives {
        if directive.overloaded_operators.is_empty() {
            continue;
        }
        let ops = directive.overloaded_operators.join(", ");
        let target = directive.target_type.as_deref().unwrap_or("*");
        diagnostics.push(
            Diagnostic::warning(format!(
                "user-defined operator overloading (`using {{ … as {ops} }} for {target}`) is not \
                 dispatched on NeoVM: operators on `{target}` compile to raw arithmetic on the \
                 underlying representation, NOT calls to the bound function(s)"
            ))
            .with_code("W_USER_DEFINED_OPERATOR")
            .with_suggestion(
                "call the bound function explicitly (e.g. `add(a, b)` instead of `a + b`) until \
                 operator dispatch is supported",
            ),
        );
    }
}

fn validate_type_definitions(_metadata: &ContractMetadata, _diagnostics: &mut [Diagnostic]) {
    // User-defined value types (`type X is Y`) are now supported.
    // They are treated as transparent type aliases; `wrap`/`unwrap` compile to no-ops.
}

fn validate_abstract_contract(metadata: &ContractMetadata, diagnostics: &mut Vec<Diagnostic>) {
    let unimplemented: Vec<&str> = metadata
        .methods
        .iter()
        .filter(|m| matches!(m.kind, FunctionKind::Regular) && m.body.is_none())
        .map(|m| m.name.as_str())
        .collect();

    if unimplemented.is_empty() {
        return;
    }

    if metadata.is_interface {
        return;
    }

    if metadata.is_abstract {
        // Abstract contracts: informational warning listing unimplemented methods.
        // This helps developers track what still needs implementation in derived contracts.
        diagnostics.push(
            Diagnostic::warning(format!(
                "abstract contract '{}' has {} unimplemented function(s): [{}]",
                metadata.name,
                unimplemented.len(),
                unimplemented.join(", "),
            ))
            .with_suggestion(
                "derived contracts must implement these functions or also be declared abstract",
            ),
        );
    } else {
        // Non-abstract contracts must implement all declared functions.
        diagnostics.push(
            Diagnostic::error(format!(
                "contract '{}' has {} unimplemented function(s) [{}] but is not declared abstract; \
                 either provide implementations or declare the contract as 'abstract contract {}'",
                metadata.name,
                unimplemented.len(),
                unimplemented.join(", "),
                metadata.name,
            ))
            .with_suggestion(
                "add 'abstract' before 'contract', or provide function bodies for all declared functions"
            ),
        );
    }
}

fn validate_flatten_warnings(metadata: &ContractMetadata, diagnostics: &mut Vec<Diagnostic>) {
    for warning in &metadata.flatten_warnings {
        diagnostics.push(
            Diagnostic::warning(warning.clone())
                .with_code("W200")
                .with_suggestion(
                    "mark the base function 'virtual' and the overriding function 'override'",
                ),
        );
    }
}
