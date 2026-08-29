#[test]
fn standard_json_accepts_inline_content() {
    let temp = tempdir().expect("tempdir");
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
            contract_names: Vec::new(),
        },
    )
    .expect("standard-json processing should succeed");

    let output: Value =
        serde_json::from_str(&fs::read_to_string(&output_path).expect("read output"))
            .expect("output json");
    assert!(output["contracts"]["C.sol"]["C"].is_object());
}
