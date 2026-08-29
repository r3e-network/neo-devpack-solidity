#[test]
fn standard_json_carries_features_and_permissions() {
    let temp = tempdir().expect("tempdir");
    let input_path = temp.path().join("input.json");
    let output_path = temp.path().join("out.json");

    let source = r#"
    pragma solidity ^0.8.19;

    contract Bank {
        mapping(address => uint256) public balances;

        function deposit() public payable {
            balances[msg.sender] += msg.value;
        }
    }
    "#;

    let input_json = json!({
        "language": "Solidity",
        "sources": {
            "Bank.sol": { "content": source }
        },
        "settings": {}
    });
    fs::write(
        &input_path,
        serde_json::to_string_pretty(&input_json).unwrap(),
    )
    .expect("write input");

    process_standard_json(
        input_path.to_str().unwrap(),
        Some(output_path.to_str().unwrap()),
        StandardJsonOptions {
            optimizer_level: 2,
            use_callt: false,
            deny_wildcard_permissions: false,
            deny_wildcard_contracts: false,
            deny_wildcard_methods: false,
            nef_source: None,
            manifest_permissions: None,
            contract_names: Vec::new(),
        },
    )
    .expect("standard-json processing should succeed");

    let output: Value =
        serde_json::from_str(&fs::read_to_string(&output_path).expect("read output"))
            .expect("output json");

    let manifest = &output["contracts"]["Bank.sol"]["Bank"]["neo"]["manifest"];
    assert!(
        manifest
            .get("features")
            .and_then(Value::as_object)
            .is_some_and(|features| features.is_empty()),
        "expected `manifest.features` to be an empty object for Neo N3 compatibility"
    );

    let permissions = manifest["permissions"]
        .as_array()
        .expect("permissions array");
    assert!(
        permissions
            .iter()
            .all(|entry| entry["contract"] != Value::String("*".into())),
        "native-only contracts should not require wildcard permissions"
    );

    let mut has_serialize = false;
    let mut has_keccak = false;
    for entry in permissions {
        if let Some(methods) = entry["methods"].as_array() {
            for method in methods {
                if method == "serialize" {
                    has_serialize = true;
                }
                if method == "keccak256" {
                    has_keccak = true;
                }
            }
        }
    }
    assert!(has_serialize, "expected StdLib.serialize permission");
    assert!(has_keccak, "expected CryptoLib.keccak256 permission");
}

#[test]
fn standard_json_abi_preserves_event_anonymous_flag() {
    let temp = tempdir().expect("tempdir");
    let input_path = temp.path().join("input.json");
    let output_path = temp.path().join("out.json");

    let source = r#"
    pragma solidity ^0.8.19;

    contract EventAbi {
        event Visible(address indexed account);
        event Hidden(address indexed account) anonymous;
    }
    "#;

    let input_json = json!({
        "language": "Solidity",
        "sources": {
            "EventAbi.sol": { "content": source }
        },
        "settings": {}
    });
    fs::write(
        &input_path,
        serde_json::to_string_pretty(&input_json).unwrap(),
    )
    .expect("write input");

    process_standard_json(
        input_path.to_str().unwrap(),
        Some(output_path.to_str().unwrap()),
        StandardJsonOptions {
            optimizer_level: 2,
            use_callt: false,
            deny_wildcard_permissions: false,
            deny_wildcard_contracts: false,
            deny_wildcard_methods: false,
            nef_source: None,
            manifest_permissions: None,
            contract_names: Vec::new(),
        },
    )
    .expect("standard-json processing should succeed");

    let output: Value =
        serde_json::from_str(&fs::read_to_string(&output_path).expect("read output"))
            .expect("output json");
    let abi = output["contracts"]["EventAbi.sol"]["EventAbi"]["abi"]
        .as_array()
        .expect("abi array");

    let visible = abi
        .iter()
        .find(|entry| entry["type"] == "event" && entry["name"] == "Visible")
        .expect("Visible event");
    let hidden = abi
        .iter()
        .find(|entry| entry["type"] == "event" && entry["name"] == "Hidden")
        .expect("Hidden event");

    assert_eq!(visible["anonymous"], Value::Bool(false));
    assert_eq!(hidden["anonymous"], Value::Bool(true));
}

#[test]
fn standard_json_exposes_neo_method_map_for_overloads() {
    let temp = tempdir().expect("tempdir");
    let input_path = temp.path().join("input.json");
    let output_path = temp.path().join("out.json");

    let source = r#"
    pragma solidity ^0.8.19;

    contract OverloadedApi {
        function foo(uint256 value) public pure returns (uint256) {
            return value;
        }

        function foo(uint256 value, uint256 extra) public pure returns (uint256) {
            return value + extra;
        }
    }
    "#;

    let input_json = json!({
        "language": "Solidity",
        "sources": {
            "OverloadedApi.sol": { "content": source }
        },
        "settings": {}
    });
    fs::write(
        &input_path,
        serde_json::to_string_pretty(&input_json).unwrap(),
    )
    .expect("write input");

    process_standard_json(
        input_path.to_str().unwrap(),
        Some(output_path.to_str().unwrap()),
        StandardJsonOptions {
            optimizer_level: 2,
            use_callt: false,
            deny_wildcard_permissions: false,
            deny_wildcard_contracts: false,
            deny_wildcard_methods: false,
            nef_source: None,
            manifest_permissions: None,
            contract_names: Vec::new(),
        },
    )
    .expect("standard-json processing should succeed");

    let output: Value =
        serde_json::from_str(&fs::read_to_string(&output_path).expect("read output"))
            .expect("output json");

    // Distinct-arity overloads are legal in Neo manifests (dispatch is keyed
    // on name + parameter count), so both keep the original Solidity name.
    let method_map = &output["contracts"]["OverloadedApi.sol"]["OverloadedApi"]["neo"]["methodMap"];
    assert_eq!(method_map["foo(uint256)"], "foo");
    assert_eq!(method_map["foo(uint256,uint256)"], "foo");
}

#[test]
fn standard_json_exposes_neo_method_map_for_same_arity_overloads() {
    let temp = tempdir().expect("tempdir");
    let input_path = temp.path().join("input.json");
    let output_path = temp.path().join("out.json");

    let source = r#"
    pragma solidity ^0.8.19;

    contract OverloadedApi {
        function foo(uint256 value) public pure returns (uint256) {
            return value;
        }

        function foo(address account) public pure returns (address) {
            return account;
        }
    }
    "#;

    let input_json = json!({
        "language": "Solidity",
        "sources": {
            "OverloadedApi.sol": { "content": source }
        },
        "settings": {}
    });
    fs::write(
        &input_path,
        serde_json::to_string_pretty(&input_json).unwrap(),
    )
    .expect("write input");

    process_standard_json(
        input_path.to_str().unwrap(),
        Some(output_path.to_str().unwrap()),
        StandardJsonOptions {
            optimizer_level: 2,
            use_callt: false,
            deny_wildcard_permissions: false,
            deny_wildcard_contracts: false,
            deny_wildcard_methods: false,
            nef_source: None,
            manifest_permissions: None,
            contract_names: Vec::new(),
        },
    )
    .expect("standard-json processing should succeed");

    let output: Value =
        serde_json::from_str(&fs::read_to_string(&output_path).expect("read output"))
            .expect("output json");

    // Same-arity collision: Neo cannot disambiguate by parameter count, so
    // one deterministic primary (smallest neo_name) keeps the original name
    // and the other keeps the mangled name. The methodMap reports the
    // manifest-visible (callable) name in both cases.
    let method_map = &output["contracts"]["OverloadedApi.sol"]["OverloadedApi"]["neo"]["methodMap"];
    assert_eq!(method_map["foo(uint256)"], "foo(uint256)");
    assert_eq!(method_map["foo(address)"], "foo");

    let methods = output["contracts"]["OverloadedApi.sol"]["OverloadedApi"]["neo"]["manifest"]
        ["abi"]["methods"]
        .as_array()
        .expect("methods array");
    let method_names: Vec<_> = methods
        .iter()
        .filter_map(|method| method["name"].as_str())
        .collect();

    assert!(
        method_names.contains(&"foo(uint256)") || method_names.contains(&"foo(address)"),
        "expected at least one mangled same-arity overload name, got {method_names:?}"
    );
}

#[test]
fn sanitize_contract_name_handles_invalid_chars() {
    assert_eq!(
        sanitize_contract_name("My Contract!").as_deref(),
        Some("My_Contract")
    );
    assert_eq!(
        sanitize_contract_name("___invalid***").as_deref(),
        Some("invalid")
    );
    assert_eq!(
        sanitize_contract_name("token-1").as_deref(),
        Some("token-1")
    );
}

#[test]
fn solidity_types_map_to_manifest_types() {
    assert_eq!(solidity_to_manifest_type("uint256"), "Integer");
    assert_eq!(solidity_to_manifest_type("int8"), "Integer");
    assert_eq!(solidity_to_manifest_type("bool"), "Boolean");
    assert_eq!(solidity_to_manifest_type("address"), "Hash160");
    assert_eq!(solidity_to_manifest_type("bytes32"), "Hash256");
    assert_eq!(solidity_to_manifest_type("string"), "String");
    // Arrays should correctly return "Array" regardless of element type
    assert_eq!(solidity_to_manifest_type("uint256[]"), "Array");
    assert_eq!(solidity_to_manifest_type("address[]"), "Array");
    assert_eq!(solidity_to_manifest_type("bool[]"), "Array");
    // Mappings should return "Map"
    assert_eq!(
        solidity_to_manifest_type("mapping(address => uint256)"),
        "Map"
    );
    assert_eq!(solidity_to_manifest_type("customStruct"), "Any");
}

#[test]
fn devpack_iterator_maps_to_interop_interface() {
    // Gap `nep11` — the devpack NeoVM iterator handle must surface as the
    // manifest `InteropInterface` type (NEP-11 `tokensOf`/`tokens`).
    assert_eq!(solidity_to_manifest_type("Syscalls.Iterator"), "InteropInterface");
    assert_eq!(solidity_to_manifest_type("Storage.Iterator"), "InteropInterface");
    assert_eq!(solidity_to_manifest_type("Iterator"), "InteropInterface");
    // But iterator-ish user types keep their existing mapping.
    assert_eq!(solidity_to_manifest_type("MyLib.IteratorState"), "Any");
    assert_eq!(solidity_to_manifest_type("Iterator[]"), "Array");
}

#[test]
fn hex_prefixed_is_idempotent() {
    assert_eq!(hex_prefixed("deadbeef"), "0xdeadbeef");
    assert_eq!(hex_prefixed("0xdeadbeef"), "0xdeadbeef");
}

#[test]
fn metadata_blob_defaults_keccak_to_empty_hash() {
    let blob = build_metadata_blob(
        "Sample",
        &[],
        "Sample.sol",
        &json!({"optimizer": {"enabled": true}}),
        None,
    );
    let value: Value = serde_json::from_str(&blob).expect("metadata json");
    assert_eq!(
        value["sources"]["Sample.sol"]["keccak256"],
        Value::String(String::new()),
        "keccak field should be empty string when not provided"
    );
}

#[test]
fn state_mutability_label_maps_all_variants() {
    assert_eq!(state_mutability_label(StateMutability::Pure), "pure");
    assert_eq!(state_mutability_label(StateMutability::View), "view");
    assert_eq!(state_mutability_label(StateMutability::Payable), "payable");
    assert_eq!(
        state_mutability_label(StateMutability::NonPayable),
        "nonpayable"
    );
}
