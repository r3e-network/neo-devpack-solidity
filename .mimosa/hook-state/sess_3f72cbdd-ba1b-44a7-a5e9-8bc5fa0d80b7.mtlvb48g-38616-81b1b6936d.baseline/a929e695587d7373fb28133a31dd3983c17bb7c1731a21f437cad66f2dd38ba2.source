use super::*;

pub(crate) fn validate_methods(
    metadata: &ContractMetadata,
    diagnostics: &mut Vec<Diagnostic>,
) -> usize {
    use std::collections::{HashMap, HashSet};

    let mut signatures = HashSet::new();
    let mut exposed_overload_counts: HashSet<(String, usize)> = HashSet::new();
    let mut constructor_count = 0usize;

    // Struct field map so the duplicate-signature check canonicalizes a struct
    // parameter to its ABI TUPLE shape, not its bare name. Two different
    // structs with the same shape (`struct A{uint x}` / `struct B{uint y}`)
    // produce the same 4-byte selector, so `f(A)` and `f(B)` collide on-chain —
    // solc rejects this, but a name-keyed check (`f(A)` != `f(B)`) misses it.
    let struct_fields_map: HashMap<String, Vec<(String, String)>> = metadata
        .structs
        .iter()
        .map(|s| {
            (
                s.name.clone(),
                s.fields
                    .iter()
                    .map(|f| (f.name.clone(), f.ty.clone()))
                    .collect(),
            )
        })
        .collect();

    // Used to reduce false-positive diagnostics for `return foo();` in
    // multi-return functions. NeoVM lowering represents tuples as arrays, so
    // returning another tuple-returning function call is valid.
    let mut return_arities: HashMap<(String, usize), usize> = HashMap::new();
    for function in &metadata.methods {
        return_arities.insert(
            (function.name.clone(), function.parameters.len()),
            function.return_parameters.len(),
        );
    }

    // Solidity 0.5.0+ requires an explicit visibility on every contract
    // function. Without this, an omitted specifier silently defaults to
    // `internal`, dropping the function from the ABI — a contract that "compiles
    // successfully" but exposes none of its intended entrypoints.
    for name in &metadata.functions_missing_visibility {
        diagnostics.push(
            Diagnostic::error(format!(
                "no visibility specified for function '{name}'; add one of \
                 external/public/internal/private"
            ))
            .with_code("NO_VISIBILITY_SPECIFIER")
            .with_suggestion(
                "Solidity 0.5.0+ requires explicit visibility; a missing specifier \
                 would default to `internal` and drop the function from the ABI",
            ),
        );
    }

    for function in &metadata.methods {
        let is_exposed = matches!(
            function.visibility,
            VisibilityKind::Public | VisibilityKind::External
        );

        match function.kind {
            FunctionKind::Constructor => {
                constructor_count += 1;
                if !function.return_parameters.is_empty() {
                    diagnostics.push(
                        Diagnostic::error("constructor must not specify a return type")
                            .with_code("INVALID_CONSTRUCTOR_RETURN"),
                    );
                }
            }
            FunctionKind::Regular => {
                let count_key = (function.name.clone(), function.parameters.len());
                if is_exposed && !exposed_overload_counts.insert(count_key.clone()) {
                    diagnostics.push(
                        Diagnostic::warning(format!(
                            "overloaded function '{}' with {} parameter(s) uses Neo overload mangling; \
                             external callers must invoke the generated Neo method names",
                            count_key.0, count_key.1
                        ))
                        .with_code("W130")
                        .with_suggestion(
                            "use generated neo_name entries (e.g. functionName(type1,type2)) when invoking this contract from Neo",
                        ),
                    );
                }

                let param_signature: Vec<String> = function
                    .parameters
                    .iter()
                    .map(|param| {
                        crate::utils::canonical_param_type_with_structs(
                            &param.ty,
                            &struct_fields_map,
                        )
                    })
                    .collect();
                let signature = format!("{}({})", function.name, param_signature.join(","));

                // Internal/private helper methods may be merged from multiple
                // libraries for compatibility; only enforce duplicate-signature
                // errors on externally visible ABI methods.
                if is_exposed && !signatures.insert(signature.clone()) {
                    diagnostics.push(
                        Diagnostic::error(format!("duplicate function signature '{signature}'"))
                            .with_code("DUPLICATE_SIGNATURE"),
                    );
                }
            }
        }

        let mut params = HashSet::new();
        for param in &function.parameters {
            if let Some(name) = &param.name {
                if !params.insert(name.clone()) {
                    diagnostics.push(
                        Diagnostic::error(format!(
                            "function '{}' has duplicate parameter name '{}'",
                            function.name, name
                        ))
                        .with_code("DUPLICATE_PARAMETER_NAME"),
                    );
                }
            }

            if param.neo_type.is_none() && is_exposed {
                let lower_ty = param.ty.to_ascii_lowercase();
                let param_name = param
                    .name
                    .clone()
                    .unwrap_or_else(|| "<unnamed>".to_string());
                if lower_ty.starts_with("fixed") || lower_ty.starts_with("ufixed") {
                    diagnostics.push(
                        Diagnostic::error(format!(
                            "function '{}' parameter '{}' uses fixed-point type '{}' which is not supported on NeoVM",
                            function.name, param_name, param.ty
                        ))
                        .with_suggestion(
                            "use scaled integer arithmetic instead (e.g., multiply by 10^18 for 18 decimal places)"
                        ),
                    );
                } else {
                    diagnostics.push(
                        Diagnostic::error(format!(
                            "function '{}' parameter '{}' uses unsupported type '{}'",
                            function.name, param_name, param.ty
                        ))
                        .with_code("UNSUPPORTED_PARAMETER_TYPE"),
                    );
                }
            }

            // Validate mapping key types: arrays, structs, and mappings are
            // not valid as mapping keys because they lack a stable hash on NeoVM.
            if let Some(NeoType::Mapping { ref key, .. }) = param.neo_type {
                fn is_invalid_mapping_key(ty: &NeoType) -> bool {
                    matches!(
                        ty,
                        NeoType::Array(..) | NeoType::Struct { .. } | NeoType::Mapping { .. }
                    )
                }
                if is_invalid_mapping_key(key) {
                    let param_name = param
                        .name
                        .clone()
                        .unwrap_or_else(|| "<unnamed>".to_string());
                    diagnostics.push(Diagnostic::error(format!(
                        "function '{}' parameter '{}': mapping key type must be \
                         an elementary type (integer, bool, address, string, bytes); \
                         arrays, structs, and mappings are not allowed as keys",
                        function.name, param_name
                    )));
                }
            }

            if let Some(storage) = &param.storage {
                if storage == "storage"
                    && matches!(
                        function.visibility,
                        VisibilityKind::External | VisibilityKind::Public
                    )
                {
                    diagnostics.push(
                        Diagnostic::error(format!(
                            "public/external function '{}' parameter '{}' may not use 'storage' data location",
                            function.name,
                            param
                                .name
                                .clone()
                                .unwrap_or_else(|| "<unnamed>".to_string())
                        ))
                        .with_code("INVALID_STORAGE_PARAM"),
                    );
                }
            }
        }

        // Warn when `payable` is used: Neo N3 does not have native value
        // transfers, so the modifier is a no-op.  Token receipts should use
        // the `onNEP17Payment` callback instead.
        if function.state_mutability == StateMutability::Payable
            && !matches!(function.kind, FunctionKind::Constructor)
        {
            diagnostics.push(
                Diagnostic::warning(format!(
                    "function '{}' is marked `payable`, but Neo N3 has no native coin \
                     transfer; the modifier is accepted for compatibility but has no \
                     effect. Use onNEP17Payment(address, uint256, bytes) to handle \
                     incoming NEP-17 token payments.",
                    function.name
                ))
                .with_code("W111")
                .with_suggestion(
                    "Remove `payable` or add an onNEP17Payment callback for token receipts",
                ),
            );
        }

        if let Some(body) = &function.body {
            check_return_statements(
                body,
                function.return_parameters.len(),
                &function.name,
                &return_arities,
                diagnostics,
            );
        } else if !function.return_parameters.is_empty()
            && !matches!(function.kind, FunctionKind::Constructor)
        {
            if metadata.is_abstract || metadata.is_interface {
                // Abstract contracts are allowed to have bodyless functions.
                // No diagnostic needed here; the abstract contract validation
                // in entry.rs handles the deployment check.
            } else {
                diagnostics.push(
                    Diagnostic::error(format!(
                        "function '{}' declares a return type but has no implementation; \
                         provide a body or mark the contract as 'abstract contract {}'",
                        function.name, metadata.name
                    ))
                    .with_suggestion("add a function body, or declare the contract as abstract")
                    .with_code("MISSING_IMPLEMENTATION_RETURN"),
                );
            }
        }
    }

    constructor_count
}
