use super::*;

use std::collections::BTreeSet;

pub(crate) fn parse_manifest_permissions_mode(
    value: &str,
) -> Result<ManifestPermissionsMode, String> {
    match value.trim() {
        "merge" => Ok(ManifestPermissionsMode::Merge),
        "replace-wildcards" => Ok(ManifestPermissionsMode::ReplaceWildcards),
        other => Err(format!(
            "invalid manifest permissions mode '{other}' (expected 'merge' or 'replace-wildcards')"
        )),
    }
}

pub(crate) fn load_manifest_permissions_override(
    path: &str,
    mode: &str,
) -> Result<ManifestPermissionsOverride, String> {
    let mode = parse_manifest_permissions_mode(mode)?;
    let content = fs::read_to_string(path)
        .map_err(|err| format!("Failed to read manifest permissions file '{path}': {err}"))?;
    let root: Value = serde_json::from_str(&content)
        .map_err(|err| format!("Failed to parse manifest permissions JSON: {err}"))?;
    let permissions_value = match root {
        Value::Array(_) => root,
        Value::Object(mut map) => map
            .remove("permissions")
            .ok_or_else(|| "manifest permissions JSON object must contain a 'permissions' array".to_string())?,
        _ => {
            return Err(
                "manifest permissions JSON must be either an array or an object containing a 'permissions' array"
                    .to_string(),
            )
        }
    };

    let permissions = parse_manifest_permissions_array(&permissions_value)?;

    Ok(ManifestPermissionsOverride { mode, permissions })
}

pub(crate) fn load_manifest_permissions_override_from_natspec(
    metadata: &ContractMetadata,
) -> Result<Option<ManifestPermissionsOverride>, String> {
    let mut raw_permissions: Option<&str> = None;
    let mut mode = ManifestPermissionsMode::ReplaceWildcards;

    for (tag, raw_value) in &metadata.documentation.custom {
        let Some(field) = tag
            .strip_prefix("neo.manifest.")
            .or_else(|| tag.strip_prefix("manifest."))
        else {
            continue;
        };

        match field {
            "permissions" => raw_permissions = Some(raw_value.trim()),
            "permissionsmode" => {
                let parsed = serde_json::from_str::<Value>(raw_value)
                    .ok()
                    .and_then(|value| value.as_str().map(str::to_string))
                    .unwrap_or_else(|| raw_value.trim().to_string());
                mode = parse_manifest_permissions_mode(parsed.trim())?;
            }
            _ => {}
        }
    }

    let Some(raw_permissions) = raw_permissions else {
        return Ok(None);
    };

    let root: Value = serde_json::from_str(raw_permissions).map_err(|err| {
        format!(
            "Failed to parse NatSpec manifest permissions JSON for contract '{}': {err}",
            metadata.name
        )
    })?;

    let permissions_value = match root {
        Value::Array(_) => root,
        Value::Object(mut map) => map
            .remove("permissions")
            .ok_or_else(|| {
                format!(
                    "NatSpec manifest permissions for contract '{}' must be an array or an object containing 'permissions'",
                    metadata.name
                )
            })?,
        _ => {
            return Err(format!(
                "NatSpec manifest permissions for contract '{}' must be either an array or an object containing 'permissions'",
                metadata.name
            ))
        }
    };

    let permissions = parse_manifest_permissions_array(&permissions_value)?;

    Ok(Some(ManifestPermissionsOverride { mode, permissions }))
}

pub(crate) fn parse_manifest_permissions_array(
    value: &Value,
) -> Result<ManifestPermissionMap, String> {
    let arr = value.as_array().ok_or_else(|| {
        "manifest permissions must be an array of objects like {\"contract\":\"0x...\",\"methods\":[...]}"
            .to_string()
    })?;

    let mut out: ManifestPermissionMap = ManifestPermissionMap::new();
    for (index, entry) in arr.iter().enumerate() {
        let obj = entry
            .as_object()
            .ok_or_else(|| format!("manifest permission entry #{index} must be an object"))?;

        let contract_value = obj
            .get("contract")
            .ok_or_else(|| format!("manifest permission entry #{index} is missing 'contract'"))?;
        let contract_raw = contract_value.as_str().ok_or_else(|| {
            format!("manifest permission entry #{index} field 'contract' must be a string")
        })?;
        let contract = if contract_raw.trim() == "*" {
            "*".to_string()
        } else {
            let parsed =
                neo_devpack_solidity::neo::parse_uint160_hex_be(contract_raw).map_err(|err| {
                    format!("manifest permission entry #{index} has invalid 'contract': {err}")
                })?;
            neo_devpack_solidity::neo::format_uint160_hex_be(&parsed)
        };

        let methods_value = obj
            .get("methods")
            .ok_or_else(|| format!("manifest permission entry #{index} is missing 'methods'"))?;
        let methods = match methods_value {
            Value::String(s) if s.trim() == "*" => ManifestPermissionMethods::All,
            Value::Array(list) => {
                let mut set = BTreeSet::new();
                for (method_index, method) in list.iter().enumerate() {
                    let method_str = method.as_str().ok_or_else(|| {
                        format!(
                            "manifest permission entry #{index} methods[{method_index}] must be a string"
                        )
                    })?;
                    let trimmed = method_str.trim();
                    if trimmed.is_empty() {
                        return Err(format!(
                            "manifest permission entry #{index} methods[{method_index}] must not be empty"
                        ));
                    }
                    set.insert(trimmed.to_string());
                }
                ManifestPermissionMethods::Some(set)
            }
            _ => {
                return Err(format!(
                    "manifest permission entry #{index} field 'methods' must be \"*\" or an array of strings"
                ))
            }
        };

        out.entry(contract)
            .and_modify(|existing| existing.merge_in(methods.clone()))
            .or_insert(methods);
    }

    Ok(out)
}

pub(crate) fn manifest_permissions_to_json(permissions: ManifestPermissionMap) -> Value {
    Value::Array(
        permissions
            .into_iter()
            .map(|(contract, methods)| {
                let methods_json = match methods {
                    ManifestPermissionMethods::All => json!("*"),
                    ManifestPermissionMethods::Some(set) => {
                        json!(set.into_iter().collect::<Vec<_>>())
                    }
                };
                json!({
                    "contract": contract,
                    "methods": methods_json,
                })
            })
            .collect(),
    )
}

pub(crate) fn parse_manifest_permissions_from_manifest(
    manifest: &Value,
) -> Result<ManifestPermissionMap, String> {
    match manifest.get("permissions") {
        Some(Value::Array(_)) => parse_manifest_permissions_array(&manifest["permissions"]),
        Some(other) => Err(format!(
            "manifest 'permissions' must be an array (got {other})"
        )),
        None => Ok(ManifestPermissionMap::new()),
    }
}

pub(crate) fn merge_manifest_permissions(
    into: &mut ManifestPermissionMap,
    other: &ManifestPermissionMap,
) {
    for (contract, methods) in other {
        into.entry(contract.clone())
            .and_modify(|existing| existing.merge_in(methods.clone()))
            .or_insert_with(|| methods.clone());
    }
}

#[cfg(test)]
mod permissions_parser_tests {
    use super::*;

    use proptest::prelude::*;
    use serde_json::json;

    fn build_permission_map(entries: Vec<(String, bool, Vec<String>)>) -> ManifestPermissionMap {
        let mut map = ManifestPermissionMap::new();
        for (contract, wildcard, methods) in entries {
            let methods = if wildcard {
                ManifestPermissionMethods::All
            } else {
                ManifestPermissionMethods::Some(methods.into_iter().collect())
            };
            map.entry(contract)
                .and_modify(|existing| existing.merge_in(methods.clone()))
                .or_insert(methods);
        }
        map
    }

    fn method_name_strategy() -> impl Strategy<Value = String> {
        proptest::string::string_regex("[a-z][a-z0-9_]{0,11}").expect("valid regex")
    }

    fn permission_entry_strategy() -> impl Strategy<Value = (String, bool, Vec<String>)> {
        (
            prop::array::uniform20(any::<u8>()),
            any::<bool>(),
            prop::collection::vec(method_name_strategy(), 1..=4),
        )
            .prop_map(|(contract_bytes, wildcard, methods)| {
                (
                    format!("0x{}", hex::encode(contract_bytes)),
                    wildcard,
                    methods,
                )
            })
    }

    #[test]
    fn parse_manifest_permissions_array_normalizes_and_merges_duplicate_contract_entries() {
        let mixed_case = "0xD2A4CFF31913016155E38E474A2C06D08BE276CF";
        let without_prefix = "d2a4cff31913016155e38e474a2c06d08be276cf";
        let value = json!([
            {
                "contract": mixed_case,
                "methods": ["totalSupply", "balanceOf", "totalSupply"]
            },
            {
                "contract": without_prefix,
                "methods": ["decimals", "balanceOf"]
            }
        ]);

        let parsed = parse_manifest_permissions_array(&value).expect("permissions parsed");
        let methods = parsed
            .get("0xd2a4cff31913016155e38e474a2c06d08be276cf")
            .expect("normalized contract key");

        match methods {
            ManifestPermissionMethods::All => {
                panic!("expected method set, got wildcard")
            }
            ManifestPermissionMethods::Some(set) => {
                assert_eq!(set.len(), 3, "duplicate methods should be deduplicated");
                assert!(set.contains("totalSupply"));
                assert!(set.contains("balanceOf"));
                assert!(set.contains("decimals"));
            }
        }
    }

    #[test]
    fn parse_manifest_permissions_array_wildcard_methods_dominate_specific_entries() {
        let value = json!([
            {
                "contract": "0xd2a4cff31913016155e38e474a2c06d08be276cf",
                "methods": ["totalSupply"]
            },
            {
                "contract": "0xd2a4cff31913016155e38e474a2c06d08be276cf",
                "methods": "*"
            },
            {
                "contract": "0xd2a4cff31913016155e38e474a2c06d08be276cf",
                "methods": ["balanceOf"]
            }
        ]);

        let parsed = parse_manifest_permissions_array(&value).expect("permissions parsed");
        let methods = parsed
            .get("0xd2a4cff31913016155e38e474a2c06d08be276cf")
            .expect("normalized contract key");
        assert!(matches!(methods, ManifestPermissionMethods::All));
    }

    proptest! {
        #[test]
        fn manifest_permissions_json_roundtrip_preserves_semantics(
            entries in prop::collection::vec(permission_entry_strategy(), 0..8)
        ) {
            let baseline = build_permission_map(entries);
            let encoded = manifest_permissions_to_json(baseline.clone());
            let decoded = parse_manifest_permissions_array(&encoded)
                .expect("roundtrip decode should succeed");
            prop_assert_eq!(decoded, baseline);
        }

        #[test]
        fn merge_manifest_permissions_is_commutative(
            left_entries in prop::collection::vec(permission_entry_strategy(), 0..6),
            right_entries in prop::collection::vec(permission_entry_strategy(), 0..6),
        ) {
            let left = build_permission_map(left_entries);
            let right = build_permission_map(right_entries);

            let mut left_then_right = left.clone();
            merge_manifest_permissions(&mut left_then_right, &right);

            let mut right_then_left = right.clone();
            merge_manifest_permissions(&mut right_then_left, &left);

            prop_assert_eq!(left_then_right, right_then_left);
        }

        #[test]
        fn merge_manifest_permissions_is_associative(
            a_entries in prop::collection::vec(permission_entry_strategy(), 0..5),
            b_entries in prop::collection::vec(permission_entry_strategy(), 0..5),
            c_entries in prop::collection::vec(permission_entry_strategy(), 0..5),
        ) {
            let a = build_permission_map(a_entries);
            let b = build_permission_map(b_entries);
            let c = build_permission_map(c_entries);

            let mut left = a.clone();
            merge_manifest_permissions(&mut left, &b);
            merge_manifest_permissions(&mut left, &c);

            let mut bc = b.clone();
            merge_manifest_permissions(&mut bc, &c);
            let mut right = a.clone();
            merge_manifest_permissions(&mut right, &bc);

            prop_assert_eq!(left, right);
        }
    }
}
