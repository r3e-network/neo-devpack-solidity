#[test]
fn manifest_methods_have_offsets_and_safe_flag() {
    let source = include_str!("../../../../examples/TestContract.sol");
    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let manifest = &artifacts[0].manifest;

    let methods = manifest["abi"]["methods"]
        .as_array()
        .expect("methods array");
    assert!(
        methods
            .iter()
            .all(|m| m.get("offset").and_then(Value::as_u64).is_some()),
        "all methods should have numeric offsets"
    );
    for method in methods {
        let name = method
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let safe = method.get("safe").and_then(Value::as_bool).unwrap_or(false);
        if name == "getValue" {
            assert!(safe, "view getter should be marked safe");
        }
    }
}

#[test]
fn manifest_storage_feature_set_for_storage_builtins() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract UsesStorageBuiltins {
        function put(bytes memory key, bytes memory value) public {
            Storage.put(key, value);
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let manifest = &artifacts[0].manifest;
    assert!(
        manifest
            .get("features")
            .and_then(Value::as_object)
            .is_some_and(|features| features.is_empty()),
        "expected `manifest.features` to be an empty object for Neo N3 compatibility"
    );
}

#[test]
fn manifest_events_do_not_include_indexed_fields() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract Events {
        event Transfer(address indexed from, address indexed to, uint256 amount);

        function emitIt(address to, uint256 amount) public {
            emit Transfer(msg.sender, to, amount);
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let manifest = &artifacts[0].manifest;
    let events = manifest["abi"]["events"].as_array().expect("events array");
    assert!(!events.is_empty(), "expected at least one event");

    for event in events {
        let params = event["parameters"]
            .as_array()
            .expect("event parameters array");
        for param in params {
            assert!(
                param.get("indexed").is_none(),
                "Neo N3 manifest event parameters must not include an 'indexed' field"
            );
        }
    }
}

#[test]
fn manifest_event_declares_truthful_evm_wire_shape() {
    // Non-NEP events are emitted in EVM log shape
    // (`Notify(name, [topic0, indexed..., data])`), and post-Basilisk Neo
    // nodes fault notifications whose state-item count mismatches the
    // manifest declaration — so the manifest must declare exactly that
    // wire shape: `[topic0, <indexed names>..., data]`, all ByteArray.
    let source = r#"
    pragma solidity ^0.8.19;

    contract EventsWithStruct {
        struct Payload {
            uint256 amount;
            address from;
        }

        event Snapshot(Payload payload);

        function emitIt() public {
            emit Snapshot(Payload({ amount: 7, from: msg.sender }));
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let manifest = &artifacts[0].manifest;
    let events = manifest["abi"]["events"].as_array().expect("events array");
    let snapshot = events
        .iter()
        .find(|event| event["name"] == "Snapshot")
        .expect("Snapshot event");
    let params = snapshot["parameters"]
        .as_array()
        .expect("event parameters array");
    // One non-indexed struct param => wire shape is [topic0, data].
    assert_eq!(params.len(), 2, "expected [topic0, data] declaration");
    assert_eq!(params[0]["name"], Value::String("topic0".to_string()));
    assert_eq!(params[0]["type"], Value::String("ByteArray".to_string()));
    assert_eq!(params[1]["name"], Value::String("data".to_string()));
    assert_eq!(params[1]["type"], Value::String("ByteArray".to_string()));
}

#[test]
fn manifest_contains_all_required_top_level_fields() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract MinimalManifest {
        function ping() public {}
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let manifest = &artifacts[0].manifest;
    assert!(
        manifest.get("name").and_then(Value::as_str).is_some(),
        "manifest must include 'name'"
    );
    assert!(
        manifest.get("groups").and_then(Value::as_array).is_some(),
        "manifest must include 'groups' array"
    );
    assert!(
        manifest
            .get("features")
            .and_then(Value::as_object)
            .is_some(),
        "manifest must include 'features' object"
    );
    assert!(
        manifest
            .get("supportedstandards")
            .and_then(Value::as_array)
            .is_some(),
        "manifest must include 'supportedstandards' array"
    );
    assert!(
        manifest.get("abi").and_then(Value::as_object).is_some(),
        "manifest must include 'abi' object"
    );
    assert!(
        manifest
            .get("permissions")
            .and_then(Value::as_array)
            .is_some(),
        "manifest must include 'permissions' array"
    );
    assert!(
        manifest.get("trusts").is_some(),
        "manifest must include 'trusts'"
    );
    assert!(
        manifest.get("extra").and_then(Value::as_object).is_some(),
        "manifest must include 'extra' object"
    );
}

#[test]
fn manifest_supports_natspec_manifest_field_overrides() {
    // Task #28: declaring `NEP-17` in supportedstandards now requires the
    // full method set plus a 3-param `Transfer` event, otherwise compilation
    // fails with CompileError::Manifest. Provide them so the test continues
    // to focus on natspec override propagation rather than NEP compliance.
    let source = r#"
    pragma solidity ^0.8.19;

    /**
     * @custom:neo.manifest.features {"storage":true}
     * @custom:neo.manifest.supportedstandards ["NEP-17","NEP-26"]
     * @custom:neo.manifest.groups [{"pubkey":"03aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","signature":"AQID"}]
     * @custom:neo.manifest.trusts ["0x1111111111111111111111111111111111111111"]
     * @custom:neo.manifest.extra.Repository "https://github.com/r3e-network/neo-devpack-solidity"
     * @custom:neo.manifest.extra.Build {"commit":"abc123"}
     */
    contract ManifestOverrides {
        event Transfer(address indexed from, address indexed to, uint256 amount);
        function symbol() external pure returns (string memory) { return "MOV"; }
        function decimals() external pure returns (uint8) { return 8; }
        function totalSupply() external view returns (uint256) { return 0; }
        function balanceOf(address) external view returns (uint256) { return 0; }
        function transfer(address from, address to, uint256 amount, bytes calldata) external returns (bool) {
            emit Transfer(from, to, amount);
            return true;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let manifest = &artifacts[0].manifest;
    assert_eq!(
        manifest["features"],
        Value::Object(serde_json::Map::new()),
        "Neo N3 requires manifest.features to remain empty"
    );

    let standards = manifest["supportedstandards"]
        .as_array()
        .expect("supportedstandards array");
    assert_eq!(standards.len(), 2);
    assert!(standards.iter().any(|v| v == "NEP-17"));
    assert!(standards.iter().any(|v| v == "NEP-26"));

    let groups = manifest["groups"].as_array().expect("groups array");
    assert_eq!(groups.len(), 1, "expected one custom group");
    assert_eq!(
        groups[0]["pubkey"].as_str(),
        Some("03aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
    );

    let trusts = manifest["trusts"].as_array().expect("trusts array");
    assert_eq!(trusts.len(), 1, "expected one custom trust");
    assert_eq!(
        trusts[0].as_str(),
        Some("0x1111111111111111111111111111111111111111")
    );

    let extra = manifest["extra"].as_object().expect("extra object");
    assert_eq!(
        extra.get("Repository").and_then(Value::as_str),
        Some("https://github.com/r3e-network/neo-devpack-solidity")
    );
    assert_eq!(
        extra
            .get("Build")
            .and_then(|v| v.get("commit"))
            .and_then(Value::as_str),
        Some("abc123")
    );
}

#[test]
fn manifest_non_empty_features_override_is_ignored_for_neo_compatibility() {
    let source = r#"
    pragma solidity ^0.8.19;

    /**
     * @custom:neo.manifest.features {"storage":true}
     */
    contract FeaturesMustStayEmpty {
        function ping() public {}
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let manifest = &artifacts[0].manifest;
    assert_eq!(
        manifest["features"],
        Value::Object(serde_json::Map::new()),
        "Neo N3 rejects populated manifest.features; compiler must keep it empty"
    );
}
