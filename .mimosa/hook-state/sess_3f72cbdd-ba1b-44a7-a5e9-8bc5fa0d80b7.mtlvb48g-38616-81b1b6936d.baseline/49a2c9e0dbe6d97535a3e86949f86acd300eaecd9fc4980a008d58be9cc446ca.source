//! Stage 2 — Library validation and normalization.
//!
//! Validates user-defined libraries before they are merged into primary
//! contracts. Pre-populates a cross-library struct/enum pool so
//! cross-references between libraries resolve correctly.

use super::*;

/// Validate and normalize user libraries before merging them into primary
/// contracts. Pre-populates a cross-library struct/enum pool so cross-
/// references between libraries resolve correctly.
///
/// Returns the normalized library contracts.
pub(crate) fn validate_libraries(
    has_primary: bool,
    fallback: &[ContractIR],
    contract_types: &[String],
) -> Result<Vec<ContractIR>, SolidityError> {
    if !has_primary {
        return Ok(Vec::new());
    }

    let raw_libraries: Vec<ContractIR> = fallback
        .iter()
        .filter(|contract| matches!(contract.kind, ContractKind::Library))
        // Built-in helper libraries (Runtime/Storage/Syscalls/Neo) are lowered directly during
        // IR generation. Avoid merging their Solidity bodies into user contracts since they
        // may contain EVM-only stubs or unsupported constructs, and they would bloat bytecode.
        .filter(|contract| !is_builtin_library_name(contract.name.as_str()))
        .cloned()
        .collect();

    // Validate user libraries before merging. Convert each library to metadata
    // and run the standard validation pipeline to catch library-specific errors
    // (state variables, constructors, external functions) early.
    //
    // Cross-library struct references — e.g. `function executeInitReserve(
    // ConfiguratorInputTypes.InitReserveInput calldata input)` declared in
    // library `ConfiguratorLogic` and referencing a struct from library
    // `ConfiguratorInputTypes` (both shipped in @aave/core-v3) — require that
    // each library's validation pass see the structs declared in its peers.
    // Otherwise `NeoType::from_solidity` can't resolve the qualified type,
    // `param.neo_type` stays `None`, and the external-function check fires a
    // spurious "uses unsupported type" error.
    //
    // We solve this by pre-merging every other library's structs (and enums,
    // for symmetry) into each library's struct table before running its
    // validation. Doing the merge here (instead of at flatten time) keeps the
    // mutation scoped to a clone and means downstream stages still see the
    // original, un-merged library tree.
    let library_struct_pool: Vec<StructIR> = raw_libraries
        .iter()
        .flat_map(|lib| lib.structs.iter().cloned())
        .collect();
    let library_enum_pool: Vec<EnumIR> = raw_libraries
        .iter()
        .flat_map(|lib| lib.enums.iter().cloned())
        .collect();
    for lib in &raw_libraries {
        let mut lib_with_peers = lib.clone();
        for s in &library_struct_pool {
            if !lib_with_peers.structs.iter().any(|own| own.name == s.name) {
                lib_with_peers.structs.push(s.clone());
            }
        }
        for e in &library_enum_pool {
            if !lib_with_peers.enums.iter().any(|own| own.name == e.name) {
                lib_with_peers.enums.push(e.clone());
            }
        }
        // Run normalize first so the validation sees the post-merge
        // semantics — library external functions get converted to internal
        // BEFORE validate enforces "no storage parameter on external
        // functions". Otherwise the validator rejects legitimate library
        // patterns like `EModeLogic.executeSetUserEMode(mapping storage, ...)`
        // (Aave) where the function is `external` in source but operates as
        // an internal helper on Neo (libraries inline into their callers).
        let normalized_lib = normalize_library_for_neo(lib_with_peers);
        let lib_metadata = convert_contract(
            normalized_lib,
            &[],
            contract_types,
            std::sync::Arc::new(SelectorRegistry::default()),
        );
        let lib_diagnostics = validate_contract(&lib_metadata);
        let lib_errors: Vec<Diagnostic> = lib_diagnostics
            .into_iter()
            .filter(|d| matches!(d.severity, DiagnosticSeverity::Error))
            .collect();
        if !lib_errors.is_empty() {
            let messages: Vec<String> = lib_errors
                .iter()
                .map(|d| {
                    let mut msg = d.message.clone();
                    if let Some(suggestion) = &d.suggestion {
                        msg.push_str(&format!("\n  suggestion: {suggestion}"));
                    }
                    msg
                })
                .collect();
            return Err(SolidityError::analysis(messages.join("\n")));
        }
    }

    Ok(raw_libraries
        .into_iter()
        .map(normalize_library_for_neo)
        .collect())
}
