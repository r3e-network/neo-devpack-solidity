#[test]
fn standard_json_can_filter_contract_outputs() {
    let temp = tempdir().expect("tempdir");
    let output_path = temp.path().join("out.json");

    let source = r#"
    pragma solidity ^0.8.19;

    contract A {
        function a() public pure returns (uint256) {
            return 1;
        }
    }

    contract B {
        function b() public pure returns (uint256) {
            return 2;
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
    let input_content = serde_json::to_string_pretty(&input_json).unwrap();

    process_standard_json_content(
        &input_content,
        Some(output_path.to_str().unwrap()),
        StandardJsonOptions {
            optimizer_level: 2,
            use_callt: false,
            deny_wildcard_permissions: false,
            deny_wildcard_contracts: false,
            deny_wildcard_methods: false,
            nef_source: None,
            manifest_permissions: None,
            contract_names: vec!["B".to_string()],
        },
    )
    .expect("standard-json processing should succeed");

    let output: Value =
        serde_json::from_str(&fs::read_to_string(&output_path).expect("read output"))
            .expect("output json");

    let per_file = output["contracts"]["C.sol"]
        .as_object()
        .expect("contracts for C.sol should be an object");
    assert!(
        per_file.contains_key("B"),
        "expected filtered contract B output"
    );
    assert!(
        !per_file.contains_key("A"),
        "did not expect contract A output when filtering"
    );
}

#[test]
fn standard_json_reports_contract_not_found_when_filter_matches_none() {
    let temp = tempdir().expect("tempdir");
    let output_path = temp.path().join("out.json");

    let source = r#"
    pragma solidity ^0.8.19;
    contract A { function a() public pure returns (uint256) { return 1; } }
    "#;

    let input_json = json!({
        "language": "Solidity",
        "sources": {
            "C.sol": { "content": source }
        },
        "settings": {}
    });
    let input_content = serde_json::to_string_pretty(&input_json).unwrap();

    process_standard_json_content(
        &input_content,
        Some(output_path.to_str().unwrap()),
        StandardJsonOptions {
            optimizer_level: 2,
            use_callt: false,
            deny_wildcard_permissions: false,
            deny_wildcard_contracts: false,
            deny_wildcard_methods: false,
            nef_source: None,
            manifest_permissions: None,
            contract_names: vec!["Missing".to_string()],
        },
    )
    .expect("standard-json processing should surface errors but not fail");

    let output: Value =
        serde_json::from_str(&fs::read_to_string(&output_path).expect("read output"))
            .expect("output json");

    let errors = output["errors"].as_array().expect("errors array expected");
    assert!(
        errors.iter().any(|err| {
            err.get("type").and_then(Value::as_str) == Some("ContractNotFound")
                && err["code"] == "CONTRACT_NOT_FOUND"
        }),
        "expected ContractNotFound error"
    );
}
