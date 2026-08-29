#[test]
fn standard_json_emits_codes_for_validation_errors() {
    let temp = tempdir().expect("tempdir");
    let input_path = temp.path().join("input.json");
    let output_path = temp.path().join("out.json");

    let source = r#"
    pragma solidity ^0.8.19;
    contract D {
        function foo(uint256 a) public {}
        function foo(uint256 a) public {}
    }
    "#;

    let input_json = json!({
        "language": "Solidity",
        "sources": {
            "D.sol": { "content": source }
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
    .expect("standard-json processing should succeed with validation errors");

    let output: Value =
        serde_json::from_str(&fs::read_to_string(&output_path).expect("read output"))
            .expect("output json");

    let errors = output["errors"].as_array().unwrap_or(&Vec::new()).clone();
    assert!(
        errors
            .iter()
            .any(|err| err["code"] == "NSH-2000" && err["message"].as_str().unwrap_or("").to_ascii_lowercase().contains("duplicate")),
        "expected duplicate signature error code"
    );
}

#[test]
fn standard_json_codes_for_invalid_storage_param() {
    let temp = tempdir().expect("tempdir");
    let input_path = temp.path().join("input.json");
    let output_path = temp.path().join("out.json");

    let source = r#"
    pragma solidity ^0.8.19;
    contract E {
        function foo(uint256[] storage x) public {}
    }
    "#;

    let input_json = json!({
        "language": "Solidity",
        "sources": {
            "E.sol": { "content": source }
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
    .expect("standard-json processing should succeed with validation errors");

    let output: Value =
        serde_json::from_str(&fs::read_to_string(&output_path).expect("read output"))
            .expect("output json");

    let errors = output["errors"].as_array().unwrap_or(&Vec::new()).clone();
    assert!(
        errors
            .iter()
            .any(|err| err["code"] == "NSH-3000" && err["message"].as_str().unwrap_or("").to_ascii_lowercase().contains("storage")),
        "expected invalid storage param code"
    );
}

#[test]
fn standard_json_codes_for_unsafe_external_type() {
    let temp = tempdir().expect("tempdir");
    let input_path = temp.path().join("input.json");
    let output_path = temp.path().join("out.json");

    let source = r#"
    pragma solidity ^0.8.19;
    contract Unsafe {
        function foo(string storage x) external {}
    }
    "#;

    let input_json = json!({
        "language": "Solidity",
        "sources": {
            "Unsafe.sol": { "content": source }
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
    .expect("standard-json processing should surface validation errors");

    let output: Value =
        serde_json::from_str(&fs::read_to_string(&output_path).expect("read output"))
            .expect("output json");

    let errors = output["errors"].as_array().unwrap_or(&Vec::new()).clone();
    assert!(
        errors
            .iter()
            .any(|err| err["code"] == "NSH-3000" && err["message"].as_str().unwrap_or("").to_ascii_lowercase().contains("storage")),
        "expected invalid storage param code"
    );
}
