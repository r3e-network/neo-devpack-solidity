use super::*;

pub(crate) fn c3_merge(
    contract_name: &str,
    mut sequences: Vec<Vec<String>>,
) -> Result<Vec<String>, SolidityError> {
    let mut result = Vec::new();

    loop {
        sequences.retain(|seq| !seq.is_empty());
        if sequences.is_empty() {
            break;
        }

        let mut candidate: Option<String> = None;
        for seq in &sequences {
            let head = &seq[0];
            let is_in_tail = sequences
                .iter()
                .any(|other| other.iter().skip(1).any(|v| v == head));
            if !is_in_tail {
                candidate = Some(head.clone());
                break;
            }
        }

        let Some(candidate) = candidate else {
            return Err(SolidityError::Analysis(format!(
                "inheritance linearization failed for '{contract_name}': inconsistent base order"
            )));
        };

        result.push(candidate.clone());
        for seq in sequences.iter_mut() {
            if seq.first().map(|v| v == &candidate).unwrap_or(false) {
                seq.remove(0);
            }
        }
    }

    Ok(result)
}

pub(crate) fn contract_linearization_mro(
    contract_name: &str,
    contract_map: &std::collections::HashMap<String, ContractIR>,
    visiting: &mut std::collections::HashSet<String>,
    cache: &mut std::collections::HashMap<String, Vec<String>>,
) -> Result<Vec<String>, SolidityError> {
    if let Some(cached) = cache.get(contract_name) {
        return Ok(cached.clone());
    }

    if !visiting.insert(contract_name.to_string()) {
        return Err(SolidityError::Analysis(format!(
            "inheritance cycle detected at '{contract_name}'"
        )));
    }

    let contract = contract_map.get(contract_name).ok_or_else(|| {
        SolidityError::Analysis(format!(
            "internal error: contract '{contract_name}' missing from analysis map"
        ))
    })?;

    let mut direct_bases: Vec<String> = Vec::new();
    for base in &contract.bases {
        let Some(base_name) = base_last_name(base) else {
            continue;
        };

        let Some(base_contract) = contract_map.get(&base_name) else {
            return Err(SolidityError::Analysis(format!(
                "contract '{}' inherits from unknown base '{}'",
                contract.name, base_name
            )));
        };

        match (contract.kind, base_contract.kind) {
            (ContractKind::Contract | ContractKind::AbstractContract, ContractKind::Interface) => {
                // Interfaces do not influence contract storage layout or constructor order.
            }
            (
                ContractKind::Contract | ContractKind::AbstractContract,
                ContractKind::Contract | ContractKind::AbstractContract,
            ) => {
                direct_bases.push(base_name);
            }
            (ContractKind::Interface, ContractKind::Interface) => {
                direct_bases.push(base_name);
            }
            (ContractKind::Interface, ContractKind::Contract | ContractKind::AbstractContract) => {
                return Err(SolidityError::Analysis(format!(
                    "interface '{}' cannot inherit from contract '{}'",
                    contract.name, base_name
                )));
            }
            (_, ContractKind::Library) => {
                return Err(SolidityError::Analysis(format!(
                    "contract '{}' cannot inherit from library '{}'",
                    contract.name, base_name
                )));
            }
            (ContractKind::Library, _) => {
                return Err(SolidityError::Analysis(format!(
                    "library '{}' cannot use inheritance",
                    contract.name
                )));
            }
        }
    }

    // Solidity gives precedence to right-most bases (`contract C is A, B` => B
    // overrides A). Feed C3 with bases in that precedence order.
    let precedence_bases: Vec<String> = direct_bases.iter().rev().cloned().collect();

    let mut sequences: Vec<Vec<String>> = Vec::new();
    for base_name in &precedence_bases {
        sequences.push(contract_linearization_mro(
            base_name,
            contract_map,
            visiting,
            cache,
        )?);
    }
    sequences.push(precedence_bases);

    let merged = c3_merge(contract_name, sequences)?;
    let mut linearization = Vec::with_capacity(1 + merged.len());
    linearization.push(contract_name.to_string());
    linearization.extend(merged);

    visiting.remove(contract_name);
    cache.insert(contract_name.to_string(), linearization.clone());
    Ok(linearization)
}

pub(crate) fn contract_linearization_base_to_derived(
    contract_name: &str,
    contract_map: &std::collections::HashMap<String, ContractIR>,
) -> Result<Vec<String>, SolidityError> {
    let mut cache = std::collections::HashMap::new();
    let mut visiting = std::collections::HashSet::new();
    let mro = contract_linearization_mro(contract_name, contract_map, &mut visiting, &mut cache)?;
    Ok(mro.into_iter().rev().collect())
}
