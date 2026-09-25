use super::*;

pub(crate) fn collect_interface_events_recursive(
    contract: &ContractIR,
    contract_map: &std::collections::HashMap<String, ContractIR>,
    seen: &mut std::collections::HashSet<String>,
    out: &mut Vec<EventIR>,
) {
    for base in &contract.bases {
        let Some(base_name) = base_last_name(base) else {
            continue;
        };

        let Some(base_contract) = contract_map.get(&base_name) else {
            continue;
        };

        match base_contract.kind {
            ContractKind::Interface if seen.insert(base_name.clone()) => {
                out.extend(base_contract.events.clone());
                collect_interface_events_recursive(base_contract, contract_map, seen, out);
            }
            ContractKind::Contract | ContractKind::AbstractContract => {
                collect_interface_events_recursive(base_contract, contract_map, seen, out);
            }
            _ => {}
        }
    }
}

pub(crate) fn collect_interface_events(
    contract: &ContractIR,
    contract_map: &std::collections::HashMap<String, ContractIR>,
) -> Vec<EventIR> {
    let mut events = Vec::new();
    let mut seen = std::collections::HashSet::new();
    collect_interface_events_recursive(contract, contract_map, &mut seen, &mut events);
    events
}

pub(crate) fn collect_interface_types_recursive(
    contract: &ContractIR,
    contract_map: &std::collections::HashMap<String, ContractIR>,
    seen: &mut std::collections::HashSet<String>,
    structs: &mut Vec<StructIR>,
    enums: &mut Vec<EnumIR>,
) {
    for base in &contract.bases {
        let Some(base_name) = base_last_name(base) else {
            continue;
        };

        let Some(base_contract) = contract_map.get(&base_name) else {
            continue;
        };

        match base_contract.kind {
            ContractKind::Interface if seen.insert(base_name.clone()) => {
                structs.extend(base_contract.structs.clone());
                enums.extend(base_contract.enums.clone());
                collect_interface_types_recursive(
                    base_contract,
                    contract_map,
                    seen,
                    structs,
                    enums,
                );
            }
            ContractKind::Contract | ContractKind::AbstractContract => {
                collect_interface_types_recursive(
                    base_contract,
                    contract_map,
                    seen,
                    structs,
                    enums,
                );
            }
            _ => {}
        }
    }
}

pub(crate) fn collect_interface_types(
    contract: &ContractIR,
    contract_map: &std::collections::HashMap<String, ContractIR>,
) -> (Vec<StructIR>, Vec<EnumIR>) {
    let mut structs = Vec::new();
    let mut enums = Vec::new();
    let mut seen = std::collections::HashSet::new();
    collect_interface_types_recursive(contract, contract_map, &mut seen, &mut structs, &mut enums);
    (structs, enums)
}
