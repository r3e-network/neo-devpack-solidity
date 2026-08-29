use super::*;

/// Parse Solidity source into `ContractIR` values.
pub fn parse_source(source: &str) -> Result<Vec<ContractIR>, FrontendError> {
    let (source_unit, comments) = parse_solidity_guarded(source).map_err(|diags| {
        FrontendError::ParseDiagnostics(collect_parse_diagnostics(source, &diags))
    })?;

    // Build a map of end positions to preceding doc comments
    let comment_map = build_comment_map(&comments, source);

    let mut contracts = Vec::new();
    // Collect file-level `type X is Y` definitions so they can be injected into all contracts.
    let mut file_level_type_aliases: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    let mut file_level_structs: Vec<StructIR> = Vec::new();
    let mut file_level_enums: Vec<EnumIR> = Vec::new();
    // File-scope custom `error` declarations (Solidity 0.8.4+). Merged into
    // every contract so revert-site lowering can resolve the declared
    // signature regardless of where the error was declared.
    let mut file_level_errors: Vec<ErrorIR> = Vec::new();
    // Task #187 — collect file-scope free functions (Solidity 0.7+). A free
    // function like `function helper(uint a, uint b) pure returns (uint) { ... }`
    // declared outside any contract is conceptually internal to every contract
    // in the source unit; merging it into each primary contract's function
    // table lets call-site dispatch (`ctx.function_names.contains(...)`)
    // resolve the reference as a regular internal call.
    let mut file_level_free_functions: Vec<FunctionIR> = Vec::new();
    // Task #188 — Solidity 0.8.13+ file-level `using { L.f1, L.f2 } for T;`
    // (and `using L for T global;`) attach directives apply to every contract
    // declared in the same source unit. Collect them here and merge into each
    // converted ContractIR below, symmetric with the file-level type-alias /
    // struct / enum / free-function injection passes. Without this, the IR
    // lowering stage only sees contract-scope `using` directives and the
    // member-style call resolver hard-errors on `x.double()` for any
    // attachment declared at file scope.
    let mut file_level_usings: Vec<Using> = Vec::new();

    // Track the declared pragma's minimum Solidity version so we can reject
    // features that were introduced later (solc-compatible behavior).
    let mut pragma_min_version: Option<Version> = None;

    for part in source_unit.0 {
        match part {
            SourceUnitPart::PragmaDirective(pragma) => {
                if let Some(min) = enforce_supported_pragma(&pragma)? {
                    // The combined source unit's effective minimum compiler
                    // version is the *intersection* of every file's pragma
                    // range. Since each file's pragma gives a lower bound on
                    // the version that file accepts, the chosen compiler
                    // version must be `>= max(file_mins)`. Earlier versions
                    // tracked the MIN here, which incorrectly lowered the
                    // effective version when one imported file declared
                    // `>=0.4.16` (a broad lower bound used by ENS / Aave / some
                    // OZ utility files); that caused legitimate uses of
                    // `string.concat` / `bytes.concat` in the entry contract
                    // to fail the feature-version gate.
                    pragma_min_version = match pragma_min_version {
                        Some(existing) if existing >= min => Some(existing),
                        _ => Some(min),
                    };
                }
            }
            SourceUnitPart::ContractDefinition(contract) => {
                contracts.push(convert_contract(*contract, &comment_map));
            }
            SourceUnitPart::TypeDefinition(td) => {
                let underlying = format!("{}", td.ty);
                file_level_type_aliases.insert(td.name.name, underlying);
            }
            SourceUnitPart::StructDefinition(def) => {
                file_level_structs.push(convert_struct(*def));
            }
            SourceUnitPart::EnumDefinition(def) => {
                file_level_enums.push(convert_enum(*def));
            }
            SourceUnitPart::ErrorDefinition(def) => {
                file_level_errors.push(convert_error(*def));
            }
            SourceUnitPart::FunctionDefinition(def) => {
                // Task #187 — file-scope free function. Free functions are
                // implicitly internal (Solidity rejects `public`/`external`
                // at file scope). Normalize visibility to `Internal` so the
                // merged function behaves like any other internal helper in
                // the consuming contract, and mark the type as `Function`
                // regardless of what solang-parser surfaces.
                let mut fn_ir = convert_function(*def, &comment_map);
                fn_ir.visibility = VisibilityKind::Internal;
                // File-scope free functions are internal by language rule
                // (Solidity rejects an explicit visibility here), so their
                // omitted specifier is not the error the visibility check looks
                // for.
                fn_ir.explicit_visibility = true;
                fn_ir.ty = FunctionTy::Function;
                file_level_free_functions.push(fn_ir);
            }
            SourceUnitPart::Using(using) => {
                // Task #188 — capture file-level `using` directives; merged
                // into each contract below once all parts have been parsed.
                file_level_usings.push(*using);
            }
            SourceUnitPart::ImportDirective(_) => {
                // Intentionally ignored. Import resolution happens in a
                // separate pass that scans source for import directives
                // independently and merges imported files into the source
                // unit before/around this function. Any ImportDirective
                // node still present here is therefore a no-op.
            }
            SourceUnitPart::StraySemicolon(_) => {
                // Parser artifact (a stray `;` at file scope) — drop silently.
            }
            SourceUnitPart::EventDefinition(_)
            | SourceUnitPart::VariableDefinition(_)
            | SourceUnitPart::Annotation(_) => {
                // File-level events / constants / annotations are not yet
                // lowered into the IR. Listed explicitly — never via `_` —
                // so a future solang-parser grammar addition cannot silently
                // fall through and produce an empty contract (audit L-FE1).
            }
            // L-FE1 safety net — any variant not explicitly handled above is
            // a construct this compiler does not know about. Unreachable for
            // the parser version pinned in Cargo.lock (hence the allow below);
            // if it fires after a parser upgrade, the user sees a clear "file
            // a bug" message instead of a silent empty contract. Keep this arm
            // even though it is currently unreachable — it is the forward-
            // compatibility guard L-FE1 requires.
            #[allow(unreachable_patterns)]
            other => {
                return Err(FrontendError::UnsupportedConstruct(format!("{other:?}")));
            }
        }
    }

    // Enforce per-feature pragma gates (solc emits a hard error when a feature
    // is used outside its declared minimum version). POC: `string.concat` /
    // `bytes.concat`. See `FEATURE_*_MIN` constants for the registry.
    enforce_feature_version_gates(source, pragma_min_version)?;

    // Inject file-level type aliases into every contract in the file.
    if !file_level_type_aliases.is_empty() {
        for contract in &mut contracts {
            for (name, underlying) in &file_level_type_aliases {
                contract
                    .type_aliases
                    .entry(name.clone())
                    .or_insert_with(|| underlying.clone());
            }
        }
    }

    if !file_level_structs.is_empty() {
        for contract in &mut contracts {
            for file_struct in &file_level_structs {
                if !contract
                    .structs
                    .iter()
                    .any(|existing| existing.name == file_struct.name)
                {
                    contract.structs.push(file_struct.clone());
                }
            }
        }
    }

    if !file_level_enums.is_empty() {
        for contract in &mut contracts {
            for file_enum in &file_level_enums {
                if !contract
                    .enums
                    .iter()
                    .any(|existing| existing.name == file_enum.name)
                {
                    contract.enums.push(file_enum.clone());
                }
            }
        }
    }

    // Inject file-level custom errors into every contract in the file.
    // Contract-scope declarations shadow same-named file-scope ones.
    if !file_level_errors.is_empty() {
        for contract in &mut contracts {
            for file_error in &file_level_errors {
                if !contract
                    .errors
                    .iter()
                    .any(|existing| existing.name == file_error.name)
                {
                    contract.errors.push(file_error.clone());
                }
            }
        }
    }

    // Task #187 — inject file-scope free functions into every contract in the
    // source unit. Mirrors how the library-merge pass in `analyse_all_sources`
    // pulls sibling library bodies into primary contracts so the IR lowering
    // stage (`function_names` symbol table) can dispatch free-function calls
    // as regular internal calls instead of falling through to the
    // unresolved-call compatibility path that silently drops arguments and
    // pushes a zero return value. Contracts where a same-named method already
    // exists keep their own definition (contract-scope wins).
    if !file_level_free_functions.is_empty() {
        for contract in &mut contracts {
            for free_fn in &file_level_free_functions {
                // Dedup by (name, arity), not bare name: free-function
                // OVERLOADS (`pick(uint)` / `pick(uint,uint)`) are distinct
                // callables, and a bare-name check dropped every overload
                // after the first — the call site then aborted with
                // "'pick'/2 has no compiled body" (feature audit).
                // Contract-scope still wins per arity.
                if !contract.functions.iter().any(|existing| {
                    existing.name == free_fn.name
                        && existing.parameters.len() == free_fn.parameters.len()
                }) {
                    contract.functions.push(free_fn.clone());
                }
            }
        }
    }

    // Task #188 — merge every file-level `using` directive into each contract
    // in the source unit. The IR-lowering stage consumes `ContractIR`'s
    // `using_directives` / `using_for_libraries` / `has_using_function_list`
    // fields to build the `using_target_types`, `using_function_list_targets`,
    // and `using_function_list_scope_targets` symbol tables that drive
    // `ctx.has_using_directives()` and the member-style call resolver. Without
    // this merge, the file-level form `using { L.f1, L.f2 } for T;` is
    // completely invisible to lowering (both library-form `using L for T;`
    // and function-list form are affected). Libraries don't participate in
    // `using`-for dispatch, so skip them here — mirroring the
    // `normalize_library_for_neo` treatment downstream.
    if !file_level_usings.is_empty() {
        for contract in &mut contracts {
            if matches!(contract.kind, ContractKind::Library) {
                continue;
            }
            for using in &file_level_usings {
                apply_file_level_using(contract, using);
            }
        }
    }

    Ok(contracts)
}
