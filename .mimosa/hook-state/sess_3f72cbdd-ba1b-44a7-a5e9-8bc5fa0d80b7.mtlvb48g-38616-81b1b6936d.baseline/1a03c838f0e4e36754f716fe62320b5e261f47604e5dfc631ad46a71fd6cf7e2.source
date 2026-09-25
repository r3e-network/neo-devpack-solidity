#[test]
fn method_identifiers_include_selectors() {
    let selector_bytes = {
        let mut hasher = Keccak256::new();
        hasher.update("foo(uint256)".as_bytes());
        let digest = hasher.finalize();
        let mut selector = [0u8; 4];
        selector.copy_from_slice(&digest[..4]);
        selector
    };

    let metadata = ContractMetadata {
        name: "Test".to_string(),
        is_abstract: false,
            is_interface: false,
        is_library: false,
        methods: vec![FunctionMetadata {
            name: "foo".to_string(),
            neo_name: "foo".to_string(),
            kind: FunctionKind::Regular,
            parameters: vec![ParameterMetadata {
                name: Some("a".to_string()),
                ty: "uint256 memory".to_string(),
                neo_type: Some(NeoType::Integer {
                    signed: false,
                    bits: 256,
                }),
                storage: None,
            }],
            return_parameters: vec![],
            state_mutability: StateMutability::View,
            visibility: VisibilityKind::Public,
            offset: 0,
            body: None,
            selector: selector_bytes,
            is_virtual: false,
            is_override: false,
            documentation: NatspecDoc::default(),
            had_modifier_epilogue: false,
        }],
        events: vec![],
        errors: vec![],
        uses_storage: false,
        state_variables: vec![],
        structs: vec![],
        enums: vec![],
        contract_types: vec![],
        selector_registry: std::sync::Arc::new(neo_devpack_solidity::solidity::SelectorRegistry::default()),
        documentation: NatspecDoc::default(),
        has_using_for_star: false,
        has_using_function_list: false,
        using_for_libraries: vec![],
        using_directives: vec![],
        has_type_definitions: false,
        type_aliases: std::collections::HashMap::new(),
        flatten_warnings: Vec::new(),
        functions_missing_visibility: Vec::new(),
        super_method_map: std::collections::HashMap::new(),
    };

    let identifiers = build_method_identifiers(&metadata);
    assert_eq!(identifiers.len(), 1);
    assert_eq!(
        identifiers
            .get("foo(uint256)")
            .and_then(|v| v.as_str())
            .unwrap_or_default(),
        &hex_prefixed(&hex::encode(selector_bytes))
    );
}

#[test]
fn metadata_blob_populates_keccak() {
    let metadata = build_metadata_blob(
        "Example",
        &[],
        "Contract.sol",
        &json!({"optimizer": true}),
        Some("deadbeef"),
    );

    let value: Value = serde_json::from_str(&metadata).expect("metadata should be valid JSON");
    assert_eq!(
        value["sources"]["Contract.sol"]["keccak256"],
        Value::String("0xdeadbeef".to_string())
    );
    assert_eq!(value["compiler"]["name"], COMPILER_ID);
}
