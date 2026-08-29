#[test]
fn standard_json_handles_multiple_contracts_per_source() {
    let temp = tempdir().expect("tempdir");
    let input_path = temp.path().join("input.json");
    let output_path = temp.path().join("out.json");

    let source = r#"
    pragma solidity ^0.8.19;

    contract A {
        function foo() public pure returns (uint256) {
            return 1;
        }
    }

    contract B {
        function bar(uint256 n) public pure returns (uint256) {
            return n * 2;
        }
    }
    "#;

    let input_json = json!({
        "language": "Solidity",
        "sources": {
            "Both.sol": { "content": source }
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

    let expected_keccak = hex_prefixed(&keccak256_hex(source));
    assert_eq!(
        output["sources"]["Both.sol"]["keccak256"],
        Value::String(expected_keccak)
    );

    let contracts = &output["contracts"]["Both.sol"];
    let a_method = contracts["A"]["evm"]["methodIdentifiers"]["foo()"]
        .as_str()
        .expect("A foo method id");
    let b_method = contracts["B"]["evm"]["methodIdentifiers"]["bar(uint256)"]
        .as_str()
        .expect("B bar method id");

    let selector_a = {
        let mut hasher = Keccak256::new();
        hasher.update("foo()".as_bytes());
        let digest = hasher.finalize();
        hex_prefixed(&hex::encode(&digest[..4]))
    };
    let selector_b = {
        let mut hasher = Keccak256::new();
        hasher.update("bar(uint256)".as_bytes());
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
