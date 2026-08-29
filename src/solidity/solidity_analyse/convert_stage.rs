//! Stage 5 — Inheritance flattening, selector registry, and metadata conversion.
//!
//! Builds the shared selector registry from the contract map, flattens
//! inheritance, applies modifiers/base constructors, and converts each
//! selected contract into `ContractMetadata`.

use super::*;

/// Build a lookup map from struct name to its (name, type) field pairs.
///
/// Task #106 — gather struct fields across all contracts so canonical
/// signatures can expand struct params into their `(field1,field2,...)` tuple
/// form per the EVM ABI spec.
pub(crate) fn build_struct_fields_map(
    contract_map: &std::collections::HashMap<String, ContractIR>,
) -> std::collections::HashMap<String, Vec<(String, String)>> {
    let mut struct_fields_map: std::collections::HashMap<String, Vec<(String, String)>> =
        std::collections::HashMap::new();
    for contract in contract_map.values() {
        for struct_def in &contract.structs {
            let entries: Vec<(String, String)> = struct_def
                .fields
                .iter()
                .map(|f| (f.name.clone(), f.ty.clone()))
                .collect();
            struct_fields_map
                .entry(struct_def.name.clone())
                .or_insert(entries);
        }
    }
    struct_fields_map
}

/// Build a shared selector registry so `.selector` expressions can resolve
/// against any contract/interface visible to this compilation unit (including
/// those defined after the primary contract in the same file).
pub(crate) fn build_selector_registry(
    contract_map: &std::collections::HashMap<String, ContractIR>,
) -> std::sync::Arc<SelectorRegistry> {
    // Every visible type name (contract/interface/library) — contract-typed
    // params resolve to `address` for ABI canonicalization.
    let registry_contract_types: Vec<String> =
        contract_map.values().map(|c| c.name.clone()).collect();
    let mut type_method_selectors: std::collections::HashMap<
        String,
        std::collections::HashMap<String, Vec<[u8; 4]>>,
    > = std::collections::HashMap::new();
    let mut interface_types: std::collections::HashSet<String> = std::collections::HashSet::new();
    for contract in contract_map.values() {
        if matches!(contract.kind, ContractKind::Interface) {
            interface_types.insert(contract.name.clone());
        }

        // When building selector lookups for `.selector` / `.interfaceId`, include inherited
        // interface methods as part of the derived interface. This matches Solidity behavior
        // and supports patterns like `type(IChild).interfaceId` where `IChild is IParent`.
        let selector_contract = match contract.kind {
            ContractKind::Contract | ContractKind::AbstractContract | ContractKind::Interface => {
                flatten_contract_inheritance(contract.clone(), contract_map)
                    .map(|(ir, _warnings)| ir)
                    .unwrap_or_else(|_| contract.clone())
            }
            ContractKind::Library => contract.clone(),
        };

        let mut per_type: std::collections::HashMap<String, Vec<[u8; 4]>> =
            std::collections::HashMap::new();

        // Resolve each `.selector` parameter through the SAME canonicalization as
        // the manifest selector (`FunctionMetadata.selector`, built via
        // `NeoType::canonical_abi_type` in convert/functions.rs): structs expand to
        // tuples, enums render as `uint8`, integer widths are explicit. The two
        // paths must produce identical selectors — both drive on-chain dispatch and
        // a contract's `this.f.selector` must match what external callers compute.
        let sel_struct_types: Vec<StructTypeMetadata> = selector_contract
            .structs
            .iter()
            .map(|s| StructTypeMetadata {
                name: s.name.clone(),
                fields: s
                    .fields
                    .iter()
                    .map(|f| NeoStructFieldMetadata {
                        name: f.name.clone(),
                        ty: f.ty.clone(),
                    })
                    .collect(),
            })
            .collect();
        let sel_enum_types: Vec<EnumTypeMetadata> = selector_contract
            .enums
            .iter()
            .map(|e| EnumTypeMetadata {
                name: e.name.clone(),
                variants: e.values.len(),
            })
            .collect();

        for function in &selector_contract.functions {
            if !matches!(function.ty, FunctionTy::Function) {
                continue;
            }

            if !matches!(
                function.visibility,
                VisibilityKind::External | VisibilityKind::Public
            ) {
                continue;
            }

            let param_signatures: Vec<String> = function
                .parameters
                .iter()
                .map(|param| {
                    // Resolve user-defined value-type aliases (`type X is Y`) to
                    // their underlying ABI type, exactly as the authoritative
                    // manifest selector path does (convert/functions.rs). Without
                    // this, a UDVT-typed param canonicalized to the bare alias
                    // name, so `Type.method.selector` / `type(I).interfaceId`
                    // diverged from the manifest (and on-chain) selector.
                    match NeoType::from_solidity_with_aliases(
                        &param.ty,
                        &sel_struct_types,
                        &sel_enum_types,
                        &registry_contract_types,
                        &selector_contract.type_aliases,
                    ) {
                        Ok(neo_type) => neo_type.canonical_abi_type(),
                        // Fall back to the struct-aware string canonicalizer only
                        // when the type cannot be resolved (keeps prior behavior).
                        Err(_) => crate::utils::canonical_param_type_with_structs(
                            &param.ty,
                            &build_struct_fields_map(contract_map),
                        ),
                    }
                })
                .collect();
            let selector = compute_function_selector(&function.name, &param_signatures);
            per_type
                .entry(function.name.clone())
                .or_default()
                .push(selector);
        }

        type_method_selectors.insert(contract.name.clone(), per_type);
    }
    std::sync::Arc::new(SelectorRegistry {
        type_method_selectors,
        interface_types,
    })
}

/// Flatten inheritance, merge user libraries, apply modifiers/base
/// constructors, and convert each selected contract to `ContractMetadata`.
pub(crate) fn convert_all_contracts(
    selected: Vec<ContractIR>,
    contract_map: &std::collections::HashMap<String, ContractIR>,
    contract_types: &[String],
    libraries: &[ContractIR],
    has_primary: bool,
) -> Result<Vec<ContractMetadata>, SolidityError> {
    if selected.is_empty() {
        return Ok(Vec::new());
    }

    let selector_registry = build_selector_registry(contract_map);
    let mut metadatas = Vec::new();
    for contract in selected {
        let (mut flattened, flatten_warnings) =
            flatten_contract_inheritance(contract, contract_map)?;
        // Merge user-defined libraries AFTER inheritance flattening so the
        // flattener doesn't mistake cloned library helpers for inheritance
        // overrides. The final flattened contract still needs the library
        // helpers/types present before `convert_contract` so direct library
        // calls and `using for` member-style calls lower correctly.
        if has_primary && !libraries.is_empty() {
            for lib in libraries {
                flattened.functions.extend(lib.functions.clone());
                flattened
                    .state_variables
                    .extend(lib.state_variables.clone());
                flattened.structs.extend(lib.structs.clone());
                flattened.enums.extend(lib.enums.clone());
                // Carry the library's OWN event declarations so an inlined
                // library body's `emit Foo(...)` resolves against the host's
                // event table. Real-world case: Aave v3 `library ReserveLogic`
                // declares `event ReserveDataUpdated` (a copy of `IPool`'s) and
                // emits it unqualified; once the helper is inlined into `Pool`
                // the emit resolves `ReserveDataUpdated` against `Pool`'s events,
                // which previously lacked it -> "emit references event ... which
                // has no resolved declaration" and the whole compile failed.
                // Dedup by name: the host may already declare/inherit the same
                // event. Mirrors the sibling-contract event merge (Task #23).
                for lib_event in &lib.events {
                    if !flattened.events.iter().any(|e| e.name == lib_event.name) {
                        flattened.events.push(lib_event.clone());
                    }
                }
                // Merge the library's own `using` directives into the host.
                // Library function bodies are inlined verbatim above, so any
                // member-style call resolved by a library-scope `using` (e.g.
                // OZ Strings.sol declares `using SafeCast for *;` then calls
                // `someBool.toUint()` inside its own helpers) must continue to
                // resolve after the body lives inside the host contract.
                // Without this, the IR-lowering pass at
                // `src/ir/expressions/calls/member_calls.rs:432` reports
                // "member-style call '...' requires an explicit `using`
                // directive" for the inlined library code.
                for directive in &lib.using_directives {
                    if !flattened.using_directives.iter().any(|existing| {
                        existing.target_type == directive.target_type
                            && existing.function_names == directive.function_names
                    }) {
                        flattened.using_directives.push(directive.clone());
                    }
                }
                for lib_name in &lib.using_for_libraries {
                    if !flattened.using_for_libraries.contains(lib_name) {
                        flattened.using_for_libraries.push(lib_name.clone());
                    }
                }
                flattened.has_using_for_star =
                    flattened.has_using_for_star || lib.has_using_for_star;
                flattened.has_using_function_list =
                    flattened.has_using_function_list || lib.has_using_function_list;
            }
        }
        apply_modifiers_and_base_constructors(&mut flattened, contract_map)?;
        let mut metadata =
            convert_contract(flattened, &[], contract_types, selector_registry.clone());
        metadata.flatten_warnings = flatten_warnings;
        metadatas.push(metadata);
    }

    Ok(metadatas)
}
