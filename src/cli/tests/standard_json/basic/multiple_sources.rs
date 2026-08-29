#[test]
fn standard_json_handles_multiple_sources() {
    let temp = tempdir().expect("tempdir");
    let input_path = temp.path().join("input.json");
    let output_path = temp.path().join("out.json");

    let source_a = r#"
    pragma solidity ^0.8.19;

    contract A {
        function foo(uint256 a) public pure returns (uint256) {
            return a + 1;
        }
    }
    "#;

    let source_b = r#"
    pragma solidity ^0.8.19;

    contract B {
        function bar() public pure returns (uint256) {
            return 42;
        }
    }
    "#;

    let input_json = json!({
        "language": "Solidity",
        "sources": {
            "A.sol": { "content": source_a },
            "B.sol": { "content": source_b }
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

    let expected_keccak_a = hex_prefixed(&keccak256_hex(source_a));
    let expected_keccak_b = hex_prefixed(&keccak256_hex(source_b));

    assert_eq!(
        output["sources"]["A.sol"]["keccak256"],
        Value::String(expected_keccak_a)
    );
    assert_eq!(
        output["sources"]["B.sol"]["keccak256"],
        Value::String(expected_keccak_b)
    );

    let contracts = &output["contracts"];
    let a_contract = &contracts["A.sol"]["A"];
    let b_contract = &contracts["B.sol"]["B"];
    let a_method = a_contract["evm"]["methodIdentifiers"]["foo(uint256)"]
        .as_str()
        .expect("A.foo method id");
    let b_method = b_contract["evm"]["methodIdentifiers"]["bar()"]
        .as_str()
        .expect("B.bar method id");

    let selector_a = {
        let mut hasher = Keccak256::new();
        hasher.update("foo(uint256)".as_bytes());
        let digest = hasher.finalize();
        hex_prefixed(&hex::encode(&digest[..4]))
    };
    let selector_b = {
        let mut hasher = Keccak256::new();
        hasher.update("bar()".as_bytes());
        let digest = hasher.finalize();
        hex_prefixed(&hex::encode(&digest[..4]))
    };

    assert_eq!(a_method, selector_a);
    assert_eq!(b_method, selector_b);

    assert!(output
        .get("errors")
        .map(|v| v.as_array().map(|arr| arr.is_empty()).unwrap_or(true))
        .unwrap_or(true));
}

#[test]
fn standard_json_keeps_duplicate_contract_names_under_each_source() {
    let temp = tempdir().expect("tempdir");
    let input_path = temp.path().join("input.json");
    let output_path = temp.path().join("out.json");

    let source_a = r#"
    pragma solidity ^0.8.19;

    contract Duplicate {
        function fromA() public pure returns (uint256) {
            return 1;
        }
    }
    "#;

    let source_b = r#"
    pragma solidity ^0.8.19;

    contract Duplicate {
        function fromB() public pure returns (uint256) {
            return 2;
        }
    }
    "#;

    let input_json = json!({
        "language": "Solidity",
        "sources": {
            "A.sol": { "content": source_a },
            "B.sol": { "content": source_b }
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

    let a_contract = &output["contracts"]["A.sol"]["Duplicate"];
    let b_contract = &output["contracts"]["B.sol"]["Duplicate"];

    assert!(
        a_contract["evm"]["methodIdentifiers"]["fromA()"].is_string(),
        "expected A.sol Duplicate artifact to contain fromA(): {output}"
    );
    assert!(
        a_contract["evm"]["methodIdentifiers"]["fromB()"].is_null(),
        "A.sol Duplicate must not receive B.sol methods: {output}"
    );
    assert!(
        b_contract["evm"]["methodIdentifiers"]["fromB()"].is_string(),
        "expected B.sol Duplicate artifact to contain fromB(): {output}"
    );
    assert!(
        b_contract["evm"]["methodIdentifiers"]["fromA()"].is_null(),
        "B.sol Duplicate must not receive A.sol methods: {output}"
    );

    assert!(output
        .get("errors")
        .map(|v| v.as_array().map(|arr| arr.is_empty()).unwrap_or(true))
        .unwrap_or(true));
}
