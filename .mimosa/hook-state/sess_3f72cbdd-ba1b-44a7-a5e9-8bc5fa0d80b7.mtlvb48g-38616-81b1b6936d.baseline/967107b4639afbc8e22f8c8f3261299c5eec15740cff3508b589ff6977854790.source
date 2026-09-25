use super::*;

pub(crate) fn keccak256_hex(input: &str) -> String {
    let mut hasher = Keccak256::new();
    hasher.update(input.as_bytes());
    let digest = hasher.finalize();
    hex::encode(digest)
}

pub(crate) fn hex_prefixed(value: &str) -> String {
    if value.starts_with("0x") || value.starts_with("0X") {
        value.to_string()
    } else {
        format!("0x{value}")
    }
}

pub(crate) fn canonical_param_type(ty: &str) -> String {
    ty.split_whitespace().next().unwrap_or_default().to_string()
}

pub(crate) fn take_next_contract_source(
    contract_sources: &mut VecDeque<StandardJsonContractSource>,
    contract_name: &str,
) -> Option<String> {
    while let Some(contract_source) = contract_sources.pop_front() {
        if contract_source.contract_name == contract_name {
            return Some(contract_source.file_name);
        }
    }

    None
}

pub(crate) fn has_duplicate_contract_names(
    contract_sources: &[StandardJsonContractSource],
) -> bool {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for contract_source in contract_sources {
        let count = counts
            .entry(contract_source.contract_name.as_str())
            .or_insert(0);
        *count += 1;
        if *count > 1 {
            return true;
        }
    }

    false
}

pub(crate) fn standard_json_manual_code(typ: &str) -> &'static str {
    match typ {
        "UnsupportedSettings" => "UNSUPPORTED_SETTINGS",
        "MissingSourceContent" => "MISSING_SOURCE_CONTENT",
        "NoContracts" => "NO_CONTRACTS",
        "ContractNotFound" => "CONTRACT_NOT_FOUND",
        "ImportParseError" => "IMPORT_PARSE_ERROR",
        "UnsupportedImportPath" => "UNSUPPORTED_IMPORT_PATH",
        "MissingImport" => "MISSING_IMPORT",
        "NefSourceTruncated" => "NEF_SOURCE_TRUNCATED",
        "StandardJsonOutput" => "STANDARD_JSON_OUTPUT_ERROR",
        _ => "STANDARD_JSON_ERROR",
    }
}

pub(crate) fn unsupported_settings_warning(settings: &Value) -> Option<Value> {
    let unsupported_keys: Vec<_> = match settings {
        Value::Null => return None,
        Value::Object(map) if map.is_empty() => return None,
        Value::Object(map) => map
            .keys()
            .filter(|k| k.as_str() != "optimizer")
            .cloned()
            .collect(),
        _ => vec!["<non-object settings>".to_string()],
    };

    if unsupported_keys.is_empty() {
        None
    } else {
        Some(json!({
            "component": "neo-devpack-solidity",
            "severity": "warning",
            "type": "UnsupportedSettings",
            "code": standard_json_manual_code("UnsupportedSettings"),
            "formattedMessage": format!(
                "Standard JSON settings contain unsupported keys: {:?}",
                unsupported_keys
            ),
            "message": "Standard JSON settings are present but not fully supported; unsupported keys were ignored.",
        }))
    }
}

pub(crate) fn read_optimizer_level(settings: &Value) -> Option<u8> {
    let optimizer = match settings {
        Value::Object(map) => map.get("optimizer"),
        _ => None,
    };

    match optimizer {
        Some(Value::Bool(enabled)) => {
            if *enabled {
                Some(3)
            } else {
                Some(0)
            }
        }
        Some(Value::Object(opt_map)) => {
            if let Some(Value::Bool(enabled)) = opt_map.get("enabled") {
                if !enabled {
                    return Some(0);
                }
            }
            if let Some(Value::Number(level_num)) = opt_map.get("level") {
                if let Some(level) = level_num.as_u64() {
                    return Some((level as u8).min(3));
                }
            }
            if let Some(Value::Number(runs)) = opt_map.get("runs") {
                if let Some(runs_val) = runs.as_u64() {
                    return Some(if runs_val > 200 { 3 } else { 2 });
                }
            }
            Some(3)
        }
        _ => None,
    }
}
