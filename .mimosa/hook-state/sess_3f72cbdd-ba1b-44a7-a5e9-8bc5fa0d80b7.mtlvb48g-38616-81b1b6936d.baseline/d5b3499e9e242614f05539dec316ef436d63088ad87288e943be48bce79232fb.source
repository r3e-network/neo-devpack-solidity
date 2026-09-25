//! Stage 1 — Parse and classify contracts.
//!
//! Helpers that split parsed `ContractIR` values into primary (contract /
//! abstract) and fallback (library / interface) buckets, plus normalization
//! for built-in libraries and the contract-type list used by later stages.

use super::*;

/// Returns true for the built-in helper libraries that are lowered directly
/// during IR generation rather than merged into user contracts.
pub(crate) fn is_builtin_library_name(name: &str) -> bool {
    crate::ir::ir_context::BUILTIN_LIBRARY_BASES.contains(&name)
}

/// Normalize a library for Neo N3: externally visible functions become
/// internal, and state variables become internal so they don't leak into the
/// consuming contract's ABI or collide with auto-generated getters.
pub(crate) fn normalize_library_for_neo(mut contract: ContractIR) -> ContractIR {
    if !matches!(contract.kind, ContractKind::Library) {
        return contract;
    }

    // Neo N3 libraries are inlined into contracts; treat externally visible
    // library functions as internal helper functions to avoid exposing them
    // through the contract ABI.
    for function in &mut contract.functions {
        if !matches!(function.ty, FunctionTy::Function) {
            continue;
        }
        if matches!(
            function.visibility,
            VisibilityKind::External | VisibilityKind::Public
        ) {
            function.visibility = VisibilityKind::Internal;
        }
    }

    // Keep merged library state as internal implementation detail.
    // Public library constants would otherwise synthesize contract-level
    // getters and create ABI/name collisions in the consuming contract.
    for state in &mut contract.state_variables {
        state.visibility = Some("internal".to_string());
    }

    contract
}

/// The set of method names on `contract` that are part of its public
/// ABI surface — free `function`s with `external`/`public` visibility.
/// Used for both the interface→primary implementation lookup and the
/// reverse primary→interface map; centralising it here keeps the two
/// sides in lockstep so a future visibility tweak can't desync them.
pub(crate) fn public_external_method_names(
    contract: &ContractIR,
) -> std::collections::HashSet<String> {
    contract
        .functions
        .iter()
        .filter(|f| {
            matches!(f.ty, FunctionTy::Function)
                && matches!(
                    f.visibility,
                    VisibilityKind::External | VisibilityKind::Public
                )
        })
        .map(|f| f.name.clone())
        .collect()
}

/// Collect the set of user-visible contract/interface names (excluding built-in
/// libraries) used by the selector registry and conversion passes.
pub(crate) fn collect_contract_types(
    contract_map: &std::collections::HashMap<String, ContractIR>,
) -> Vec<String> {
    let mut contract_types: Vec<String> = Vec::new();
    let mut seen_contract_types = std::collections::HashSet::new();

    for contract in contract_map.values() {
        let include_as_contract_type = match contract.kind {
            ContractKind::Contract | ContractKind::AbstractContract | ContractKind::Interface => {
                true
            }
            ContractKind::Library => !is_builtin_library_name(contract.name.as_str()),
        };

        if include_as_contract_type
            && seen_contract_types.insert(contract.name.to_ascii_lowercase())
        {
            contract_types.push(contract.name.clone());
        }
    }

    contract_types
}

/// Parse raw Solidity source and separate primary (contract/abstract) from
/// fallback (library/interface) contracts.
///
/// Returns `(primary, fallback, has_primary, pre_merge_contract_map, contract_types)`.
#[allow(clippy::type_complexity)]
pub(crate) fn classify_contracts(
    source: &str,
) -> Result<
    (
        Vec<ContractIR>,
        Vec<ContractIR>,
        bool,
        std::collections::HashMap<String, ContractIR>,
        Vec<String>,
    ),
    SolidityError,
> {
    let mut primary = Vec::new();
    let mut fallback = Vec::new();

    let contracts = parse_source(source)?;
    for contract in contracts {
        if matches!(
            contract.kind,
            ContractKind::Contract | ContractKind::AbstractContract
        ) {
            primary.push(contract);
        } else {
            fallback.push(contract);
        }
    }

    let has_primary = !primary.is_empty();
    let pre_merge_contract_map: std::collections::HashMap<String, ContractIR> = primary
        .iter()
        .chain(fallback.iter())
        .map(|contract| (contract.name.clone(), contract.clone()))
        .collect();
    let contract_types = collect_contract_types(&pre_merge_contract_map);

    Ok((
        primary,
        fallback,
        has_primary,
        pre_merge_contract_map,
        contract_types,
    ))
}
