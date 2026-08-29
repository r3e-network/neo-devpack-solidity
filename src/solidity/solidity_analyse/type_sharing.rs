//! Stage 4 — Cross-contract struct/enum namespace sharing.
//!
//! Makes non-inherited type definitions visible across compilation units so
//! expressions like `Enum.Operation.DelegateCall` resolve even when the
//! defining type lives in another contract file.

use super::*;

/// Share non-inherited enum/struct definitions across all primary contracts.
pub(crate) fn share_type_definitions(
    has_primary: bool,
    primary: &mut [ContractIR],
    pre_merge_contract_map: &std::collections::HashMap<String, ContractIR>,
) {
    if !has_primary {
        return;
    }

    // Make non-inherited enum/struct namespaces visible across compilation
    // units so expressions like `Enum.Operation.DelegateCall` can resolve even
    // when the defining type lives in another top-level contract/library file.
    let shared_type_defs: Vec<(String, Vec<StructIR>, Vec<EnumIR>)> = pre_merge_contract_map
        .values()
        .filter(|contract| {
            !matches!(contract.kind, ContractKind::Library)
                || !is_builtin_library_name(contract.name.as_str())
        })
        .map(|contract| {
            (
                contract.name.clone(),
                contract.structs.clone(),
                contract.enums.clone(),
            )
        })
        .collect();

    for contract in primary.iter_mut() {
        let mut seen_structs: std::collections::HashSet<String> = contract
            .structs
            .iter()
            .map(|item| item.name.to_ascii_lowercase())
            .collect();
        let mut seen_enums: std::collections::HashSet<String> = contract
            .enums
            .iter()
            .map(|item| item.name.to_ascii_lowercase())
            .collect();

        for (owner_name, structs, enums) in &shared_type_defs {
            if owner_name == &contract.name {
                continue;
            }
            for item in structs {
                let key = item.name.to_ascii_lowercase();
                if seen_structs.insert(key) {
                    contract.structs.push(item.clone());
                }
            }
            for item in enums {
                let key = item.name.to_ascii_lowercase();
                if seen_enums.insert(key) {
                    contract.enums.push(item.clone());
                }
            }
        }
    }
}
