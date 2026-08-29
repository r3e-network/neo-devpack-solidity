#[test]
fn standard_json_includes_keccak_and_identifiers() {
    let temp = tempdir().expect("tempdir");
    let input_path = temp.path().join("input.json");
    let output_path = temp.path().join("out.json");

    let source = r#"
    pragma solidity ^0.8.19;

    contract C {
        function foo(uint256 a) public view returns (uint256) {
            return a;
        }
    }
    "#;
    let input_json = json!({
        "language": "Solidity",
        "sources": {
            "C.sol": { "content": source }
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
        output["sources"]["C.sol"]["keccak256"],
        Value::String(expected_keccak)
    );

    let method_id = output["contracts"]["C.sol"]["C"]["evm"]["methodIdentifiers"]["foo(uint256)"]
        .as_str()
        .expect("method identifier string");
    let selector = {
        let mut hasher = Keccak256::new();
        hasher.update("foo(uint256)".as_bytes());
        let digest = hasher.finalize();
        hex_prefixed(&hex::encode(&digest[..4]))
    };
    assert_eq!(method_id, selector);

    assert!(output["contracts"]["C.sol"]["C"]["metadata"]
        .as_str()
        .map(|s| !s.is_empty())
        .unwrap_or(false));
}
