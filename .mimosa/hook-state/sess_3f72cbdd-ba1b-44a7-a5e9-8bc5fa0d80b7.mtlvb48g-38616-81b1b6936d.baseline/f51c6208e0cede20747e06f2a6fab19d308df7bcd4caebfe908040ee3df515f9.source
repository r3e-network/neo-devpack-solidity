#[test]
fn example_contract_compiles_and_manifest_is_populated() {
    let source = include_str!("../../../../examples/TestContract.sol");
    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let artifact = &artifacts[0];
    assert_eq!(artifact.metadata.name, "TestContract");

    let methods = artifact.manifest["abi"]["methods"]
        .as_array()
        .expect("methods array");
    assert!(
        methods.len() >= 2,
        "expected at least setter/getter methods in manifest"
    );

    let get_value = methods
        .iter()
        .find(|m| m.get("name").and_then(Value::as_str) == Some("getValue"))
        .expect("getValue method present");
    assert!(
        get_value
            .get("safe")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        "view function should be marked safe"
    );
}

#[test]
fn counter_example_compiles_and_has_events() {
    let source = include_str!("../../../../examples/new/Counter.sol");
    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let artifact = &artifacts[0];
    assert_eq!(artifact.metadata.name, "Counter");

    let manifest = &artifact.manifest;
    let methods = manifest["abi"]["methods"]
        .as_array()
        .expect("methods array");
    assert!(
        methods
            .iter()
            .any(|m| m.get("name").and_then(Value::as_str) == Some("get")),
        "getter should be present in manifest"
    );

    let events = manifest["abi"]["events"].as_array().expect("events array");
    let event_names: Vec<_> = events
        .iter()
        .filter_map(|e| e.get("name").and_then(Value::as_str))
        .collect();
    assert!(
        event_names.contains(&"Incremented") && event_names.contains(&"Decremented"),
        "expected Incremented and Decremented events"
    );
}

#[test]
fn nft_example_advertises_nep11() {
    let source = include_str!("../../../../examples/new/NFT.sol");
    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);
    let artifact = &artifacts[0];
    assert_eq!(artifact.metadata.name, "SimpleNFT");

    let manifest = &artifact.manifest;
    let standards = manifest["supportedstandards"]
        .as_array()
        .expect("supportedstandards array");
    assert!(
        standards.iter().any(|s| s.as_str() == Some("NEP-11")),
        "SimpleNFT should advertise NEP-11"
    );

    let methods = manifest["abi"]["methods"]
        .as_array()
        .expect("methods array");
    for required in ["ownerOf", "transferFrom"] {
        assert!(
            methods
                .iter()
                .any(|m| m.get("name").and_then(Value::as_str) == Some(required)),
            "expected method '{required}' in manifest"
        );
    }

    // Public state variables should be exposed as Solidity-style getters.
    for required in ["name", "symbol"] {
        assert!(
            methods
                .iter()
                .any(|m| m.get("name").and_then(Value::as_str) == Some(required)),
            "expected public getter '{required}' in manifest"
        );
    }
}

#[test]
fn vault_example_compiles_with_external_calls() {
    let source = include_str!("../../../../examples/new/Vault.sol");
    let artifacts = compile_contracts(source, false, 2).expect("vault compilation failed");
    assert_eq!(artifacts.len(), 1);

    let methods = artifacts[0].manifest["abi"]["methods"]
        .as_array()
        .expect("methods array");
    for required in ["deposit", "withdraw"] {
        assert!(
            methods
                .iter()
                .any(|m| m.get("name").and_then(Value::as_str) == Some(required)),
            "expected method '{required}' in manifest"
        );
    }
}

#[test]
fn tx_origin_example_emits_structured_warning() {
    let source = include_str!("../../../../examples/new/EvmCompatTxOrigin.sol");
    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    assert!(
        artifacts[0].warnings.iter().any(|warning| warning
            .message
            .contains("tx.origin has different semantics on Neo N3")),
        "expected tx.origin warning to be captured in compilation artifacts"
    );
}

#[test]
fn vault_example_compiles_under_strict_manifest_flags() {
    let source = include_str!("../../../../examples/new/Vault.sol");
    let artifacts = compile_contracts_with_options(
        source,
        false,
        CompileOptions {
            optimizer_level: 2,
            use_callt: false,
            deny_wildcard_permissions: true,
            deny_wildcard_contracts: true,
            deny_wildcard_methods: true,
            manifest_permissions: None,
        },
    )
    .expect("vault strict compilation failed");

    assert_eq!(artifacts.len(), 1);
}

#[test]
fn neo_interop_showcase_compiles_with_expected_methods() {
    let source = include_str!("../../../../examples/new/NeoInteropShowcase.sol");
    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let methods = artifacts[0].manifest["abi"]["methods"]
        .as_array()
        .expect("methods array");

    for required in [
        "credit",
        "creditOf",
        "transferGasFromSelf",
        "gasBalanceViaSyscall",
    ] {
        assert!(
            methods
                .iter()
                .any(|m| m.get("name").and_then(Value::as_str) == Some(required)),
            "expected method '{required}' in manifest"
        );
    }

    let permissions = artifacts[0].manifest["permissions"]
        .as_array()
        .expect("permissions array");
    assert!(
        permissions
            .iter()
            .all(|entry| entry["contract"].as_str() != Some("*")),
        "neo interop showcase should avoid wildcard contract permissions"
    );
}

#[test]
fn low_level_call_showcase_compiles_with_manifest_methods() {
    let source = include_str!("../../../../examples/new/LowLevelCallShowcase.sol");
    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let methods = artifacts[0].manifest["abi"]["methods"]
        .as_array()
        .expect("methods array");
    for required in ["readViaSignature", "readViaSelector", "writeViaSignature"] {
        assert!(
            methods
                .iter()
                .any(|m| m.get("name").and_then(Value::as_str) == Some(required)),
            "expected method '{required}' in manifest"
        );
    }
}

#[test]
fn low_level_call_showcase_compiles_under_strict_manifest_flags() {
    let source = include_str!("../../../../examples/new/LowLevelCallShowcase.sol");
    let artifacts = compile_contracts_with_options(
        source,
        false,
        CompileOptions {
            optimizer_level: 2,
            use_callt: false,
            deny_wildcard_permissions: true,
            deny_wildcard_contracts: true,
            deny_wildcard_methods: true,
            manifest_permissions: None,
        },
    )
    .expect("low-level strict compilation failed");

    assert_eq!(artifacts.len(), 1);
}

#[test]
fn enum_array_showcase_compiles_and_returns_array() {
    let source = include_str!("../../../../examples/new/EnumArrayShowcase.sol");
    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let methods = artifacts[0].manifest["abi"]["methods"]
        .as_array()
        .expect("methods array");
    let method = methods
        .iter()
        .find(|m| m.get("name").and_then(Value::as_str) == Some("statesForReview"))
        .expect("statesForReview method");

    // Externally-callable array returns are abi-encoded into a single
    // ByteString (offset || length || elements) by the return lowering,
    // so the manifest advertises ByteArray (verified at runtime).
    assert_eq!(
        method.get("returntype").and_then(Value::as_str),
        Some("ByteArray"),
        "enum array return is abi-encoded bytes in the manifest"
    );
}

#[test]
fn nep17_is_detected_when_core_methods_are_public_getters() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract AutoGetterToken {
        string public symbol = "AUTO";
        uint8 public decimals = 8;
        uint256 public totalSupply = 100;
        mapping(address => uint256) public balanceOf;

        event Transfer(address indexed from, address indexed to, uint256 amount);

        constructor() {
            balanceOf[msg.sender] = totalSupply;
        }

        function transfer(address from, address to, uint256 amount, bytes memory data) public returns (bool) {
            data;
            require(from == msg.sender, "bad from");
            require(to != address(0), "bad to");
            require(balanceOf[from] >= amount, "insufficient");
            balanceOf[from] -= amount;
            balanceOf[to] += amount;
            emit Transfer(from, to, amount);
            return true;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);
    let manifest = &artifacts[0].manifest;
    let standards = manifest["supportedstandards"]
        .as_array()
        .expect("supportedstandards array");
    assert!(
        standards.iter().any(|s| s.as_str() == Some("NEP-17")),
        "AutoGetterToken should advertise NEP-17 when getters are present"
    );
}

#[test]
fn custom_errors_showcase_compiles_with_expected_methods() {
    let source = include_str!("../../../../examples/new/CustomErrorsShowcase.sol");
    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let methods = artifacts[0].manifest["abi"]["methods"]
        .as_array()
        .expect("methods array");
    for required in ["deposit", "withdraw", "adminReset"] {
        assert!(
            methods
                .iter()
                .any(|m| m.get("name").and_then(Value::as_str) == Some(required)),
            "expected method '{required}' in manifest"
        );
    }
}

#[test]
fn modifier_showcase_compiles_with_expected_methods() {
    let source = include_str!("../../../../examples/new/ModifierShowcase.sol");
    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let methods = artifacts[0].manifest["abi"]["methods"]
        .as_array()
        .expect("methods array");
    for required in ["pause", "unpause", "setValue", "getCallCount"] {
        assert!(
            methods
                .iter()
                .any(|m| m.get("name").and_then(Value::as_str) == Some(required)),
            "expected method '{required}' in ModifierShowcase manifest"
        );
    }
}

#[test]
fn struct_mapping_showcase_compiles_with_expected_methods() {
    let source = include_str!("../../../../examples/new/StructMappingShowcase.sol");
    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let methods = artifacts[0].manifest["abi"]["methods"]
        .as_array()
        .expect("methods array");
    for required in ["createProfile", "getProfile", "createOrg", "getOrg"] {
        assert!(
            methods
                .iter()
                .any(|m| m.get("name").and_then(Value::as_str) == Some(required)),
            "expected method '{required}' in StructMappingShowcase manifest"
        );
    }
}

#[test]
fn type_casting_showcase_compiles_with_expected_methods() {
    let source = include_str!("../../../../examples/new/TypeCastingShowcase.sol");
    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let methods = artifacts[0].manifest["abi"]["methods"]
        .as_array()
        .expect("methods array");
    for required in ["widenUint", "narrowUint", "addressToUint", "signedCast"] {
        assert!(
            methods
                .iter()
                .any(|m| m.get("name").and_then(Value::as_str) == Some(required)),
            "expected method '{required}' in TypeCastingShowcase manifest"
        );
    }
}

#[test]
fn bitwise_showcase_compiles_with_expected_methods() {
    let source = include_str!("../../../../examples/new/BitwiseShowcase.sol");
    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let methods = artifacts[0].manifest["abi"]["methods"]
        .as_array()
        .expect("methods array");
    for required in ["grantFlag", "hasFlag", "pack", "unpack"] {
        assert!(
            methods
                .iter()
                .any(|m| m.get("name").and_then(Value::as_str) == Some(required)),
            "expected method '{required}' in BitwiseShowcase manifest"
        );
    }
}

#[test]
fn constants_immutable_showcase_compiles_with_expected_methods() {
    let source = include_str!("../../../../examples/new/ConstantsImmutableShowcase.sol");
    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let methods = artifacts[0].manifest["abi"]["methods"]
        .as_array()
        .expect("methods array");
    for required in ["mint", "getInfo"] {
        assert!(
            methods
                .iter()
                .any(|m| m.get("name").and_then(Value::as_str) == Some(required)),
            "expected method '{required}' in ConstantsImmutableShowcase manifest"
        );
    }
}

#[test]
fn event_indexed_showcase_compiles_with_events() {
    let source = include_str!("../../../../examples/new/EventIndexedShowcase.sol");
    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let manifest = &artifacts[0].manifest;
    let events = manifest["abi"]["events"].as_array().expect("events array");
    let event_names: Vec<_> = events
        .iter()
        .filter_map(|e| e.get("name").and_then(Value::as_str))
        .collect();
    assert!(
        event_names.contains(&"Transfer"),
        "expected Transfer event in EventIndexedShowcase"
    );
    assert!(
        event_names.contains(&"Approval"),
        "expected Approval event in EventIndexedShowcase"
    );
}

#[test]
fn try_catch_showcase_compiles_with_expected_methods() {
    let source = include_str!("../../../../examples/new/TryCatchShowcase.sol");
    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let methods = artifacts[0].manifest["abi"]["methods"]
        .as_array()
        .expect("methods array");
    for required in ["tryCatchSuccess", "tryCatchRevert", "tryCatchPanic"] {
        assert!(
            methods
                .iter()
                .any(|m| m.get("name").and_then(Value::as_str) == Some(required)),
            "expected method '{required}' in TryCatchShowcase manifest"
        );
    }
}

#[test]
fn interface_showcase_compiles_with_expected_methods() {
    let source = include_str!("../../../../examples/new/InterfaceShowcase.sol");
    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert!(
        !artifacts.is_empty(),
        "should produce at least one artifact"
    );

    let token = artifacts
        .iter()
        .find(|a| a.metadata.name == "SimpleToken")
        .expect("SimpleToken artifact");
    let methods = token.manifest["abi"]["methods"]
        .as_array()
        .expect("methods array");
    for required in ["name", "symbol", "totalSupply", "balanceOf", "transfer"] {
        assert!(
            methods
                .iter()
                .any(|m| m.get("name").and_then(Value::as_str) == Some(required)),
            "expected method '{required}' in SimpleToken manifest"
        );
    }
}

#[test]
fn upgrade_lifecycle_showcase_compiles_under_strict_manifest_flags() {
    let source = include_str!("../../../../examples/new/UpgradeLifecycleShowcase.sol");
    let artifacts = compile_contracts_with_options(
        source,
        false,
        CompileOptions {
            optimizer_level: 2,
            use_callt: false,
            deny_wildcard_permissions: true,
            deny_wildcard_contracts: true,
            deny_wildcard_methods: true,
            manifest_permissions: None,
        },
    )
    .expect("UpgradeLifecycleShowcase strict compilation failed");

    assert_eq!(artifacts.len(), 1);
    let methods = artifacts[0].manifest["abi"]["methods"]
        .as_array()
        .expect("methods array");
    for required in [
        "transferOwnership",
        "upgrade",
        "destroyContract",
        "gasBalance",
    ] {
        assert!(
            methods
                .iter()
                .any(|m| m.get("name").and_then(Value::as_str) == Some(required)),
            "expected method '{required}' in UpgradeLifecycleShowcase manifest"
        );
    }
}

#[test]
fn witness_guard_showcase_compiles_under_strict_manifest_flags() {
    let source = include_str!("../../../../examples/new/WitnessGuardShowcase.sol");
    let artifacts = compile_contracts_with_options(
        source,
        false,
        CompileOptions {
            optimizer_level: 2,
            use_callt: false,
            deny_wildcard_permissions: true,
            deny_wildcard_contracts: true,
            deny_wildcard_methods: true,
            manifest_permissions: None,
        },
    )
    .expect("WitnessGuardShowcase strict compilation failed");

    assert_eq!(artifacts.len(), 1);
    let methods = artifacts[0].manifest["abi"]["methods"]
        .as_array()
        .expect("methods array");
    for required in [
        "setGuardian",
        "lockAccount",
        "unlockAccount",
        "privilegedAction",
        "isLocked",
    ] {
        assert!(
            methods
                .iter()
                .any(|m| m.get("name").and_then(Value::as_str) == Some(required)),
            "expected method '{required}' in WitnessGuardShowcase manifest"
        );
    }
}

#[test]
fn oracle_relay_showcase_compiles_under_strict_manifest_flags() {
    let source = include_str!("../../../../examples/new/OracleRelayStrictShowcase.sol");
    let artifacts = compile_contracts_with_options(
        source,
        false,
        CompileOptions {
            optimizer_level: 2,
            use_callt: false,
            deny_wildcard_permissions: true,
            deny_wildcard_contracts: true,
            deny_wildcard_methods: true,
            manifest_permissions: None,
        },
    )
    .expect("OracleRelayStrictShowcase strict compilation failed");

    assert_eq!(artifacts.len(), 1);

    let methods = artifacts[0].manifest["abi"]["methods"]
        .as_array()
        .expect("methods array");
    for required in ["request", "onOracleResponse", "getResult"] {
        assert!(
            methods
                .iter()
                .any(|m| m.get("name").and_then(Value::as_str) == Some(required)),
            "expected method '{required}' in OracleRelayStrictShowcase manifest"
        );
    }

    let permissions = artifacts[0].manifest["permissions"]
        .as_array()
        .expect("permissions array");
    assert!(
        permissions
            .iter()
            .all(|entry| entry["contract"].as_str() != Some("*")),
        "OracleRelayStrictShowcase should avoid wildcard contract permissions"
    );
}
