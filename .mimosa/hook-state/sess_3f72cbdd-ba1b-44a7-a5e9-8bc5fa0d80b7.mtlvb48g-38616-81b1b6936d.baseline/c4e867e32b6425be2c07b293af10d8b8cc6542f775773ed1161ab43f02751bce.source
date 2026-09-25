use super::*;

pub(crate) fn validate_library(metadata: &ContractMetadata, diagnostics: &mut Vec<Diagnostic>) {
    if !metadata.is_library {
        return;
    }

    // Built-in devpack libraries (Runtime, Storage, Syscalls, NativeCalls, Neo, abi)
    // use external stubs that are lowered to syscalls during IR generation.
    // Skip user-facing library validation for these intrinsic libraries.
    if matches!(
        metadata.name.as_str(),
        "Runtime" | "Storage" | "Syscalls" | "NativeCalls" | "Neo" | "abi"
    ) {
        return;
    }

    // Libraries cannot have state variables (constants are allowed).
    for state in &metadata.state_variables {
        if !state.is_constant {
            diagnostics.push(
                Diagnostic::error(format!(
                    "library '{}' cannot have state variable '{}'; \
                     libraries are stateless and cannot hold mutable storage",
                    metadata.name,
                    state.name.as_deref().unwrap_or("<unnamed>"),
                ))
                .with_suggestion(
                    "move the state variable into a contract, or declare it as 'constant'",
                ),
            );
        }
    }

    // Libraries cannot have constructors.
    for method in &metadata.methods {
        if matches!(method.kind, FunctionKind::Constructor) {
            diagnostics.push(
                Diagnostic::error(format!(
                    "library '{}' cannot have a constructor; \
                     libraries are not deployable and cannot be initialized",
                    metadata.name,
                ))
                .with_suggestion("remove the constructor from the library"),
            );
        }
    }

    // NeoVM libraries are inlined into the consuming contract. Keep emitting
    // diagnostics for user-facing visibility choices, but don't block lowering.
    for method in &metadata.methods {
        if !matches!(method.kind, FunctionKind::Regular) {
            continue;
        }
        match method.visibility {
            VisibilityKind::External => {
                diagnostics.push(
                    Diagnostic::warning(format!(
                        "library '{}' function '{}' is declared `external`; \
                         on NeoVM, user-defined library functions are inlined into \
                         the consuming contract, so `external` visibility is \
                         normalized to an internal helper",
                        metadata.name, method.name,
                    ))
                    .with_code("W124")
                    .with_suggestion(
                        "prefer `internal` visibility for library helpers to match Neo lowering semantics",
                    ),
                );
            }
            VisibilityKind::Public => {
                diagnostics.push(
                    Diagnostic::warning(format!(
                        "library '{}' function '{}' is declared `public`; \
                         on NeoVM, library functions are inlined into the calling \
                         contract and `public` visibility has no effect",
                        metadata.name, method.name,
                    ))
                    .with_code("W120")
                    .with_suggestion("use `internal` or `pure` visibility for library functions"),
                );
            }
            _ => {}
        }
    }
}
