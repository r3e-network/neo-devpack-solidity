#[test]
fn contract_output_prefix_sanitizes_and_indexes() {
    // Single contract preserves base name
    assert_eq!(
        contract_output_prefix("out.nef", "MyContract", 0, 1),
        "out.nef"
    );

    // Directory output places files inside the directory using contract name
    let dir = tempfile::tempdir().expect("tempdir");
    let dir_path = dir.path().to_str().expect("dir path utf-8");
    assert_eq!(
        contract_output_prefix(dir_path, "My Contract!", 0, 1),
        dir.path().join("My_Contract").to_string_lossy().to_string()
    );

    // Multiple contracts append sanitized name and keep extension
    let prefixed = contract_output_prefix("bundle.nef", "My Contract!", 1, 3);
    assert_eq!(prefixed, "bundle-My_Contract.nef");
}

#[test]
fn contract_output_prefix_ignores_dots_in_parent_directories() {
    let dir = tempfile::tempdir().expect("tempdir");
    let dotted = dir.path().join("parent.with.dots");
    std::fs::create_dir_all(&dotted).expect("create dotted dir");

    // When the output base has NO extension, dots in parent directories must not
    // be treated as a file extension separator.
    let base_no_ext = dotted.join("out").to_string_lossy().to_string();
    let prefixed = contract_output_prefix(&base_no_ext, "My Contract!", 1, 3);
    assert_eq!(
        prefixed,
        dotted.join("out-My_Contract").to_string_lossy().to_string()
    );

    // When the output base HAS an extension, preserve it and still ignore dots
    // in parent directories.
    let base_with_ext = dotted.join("bundle.nef").to_string_lossy().to_string();
    let prefixed = contract_output_prefix(&base_with_ext, "My Contract!", 1, 3);
    assert_eq!(
        prefixed,
        dotted
            .join("bundle-My_Contract.nef")
            .to_string_lossy()
            .to_string()
    );
}

#[test]
fn storage_map_assigns_slots_and_names() {
    let metadata = ContractMetadata {
        name: "StorageExample".to_string(),
        is_abstract: false,
        is_interface: false,
        is_library: false,
        methods: vec![],
        events: vec![],
        errors: vec![],
        uses_storage: true,
        structs: vec![],
        enums: vec![],
        state_variables: vec![
            StateVariableMetadata {
                name: Some("alpha".to_string()),
                ty: "uint256".to_string(),
                is_transient: false,
        is_constant: false,
                is_immutable: false,
                visibility: Some("public".to_string()),
                neo_type: None,
                has_initializer: false,
                initializer: None,
            },
            StateVariableMetadata {
                name: None,
                ty: "address".to_string(),
                is_transient: false,
        is_constant: false,
                is_immutable: false,
                visibility: Some("internal".to_string()),
                neo_type: None,
                has_initializer: false,
                initializer: None,
            },
            StateVariableMetadata {
                name: Some("gamma".to_string()),
                ty: "bool".to_string(),
                is_transient: false,
        is_constant: false,
                is_immutable: false,
                visibility: None,
                neo_type: None,
                has_initializer: false,
                initializer: None,
            },
        ],
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

    let map = build_storage_map(&metadata);
    let obj = map.as_object().expect("storage map object");

    assert_eq!(obj["alpha"]["slot"], Value::from(0));
    assert_eq!(obj["alpha"]["type"], Value::from("uint256"));
    assert_eq!(obj["slot_1"]["slot"], Value::from(1));
    assert_eq!(
        obj["slot_1"]["description"],
        Value::from("internal".to_string())
    );
    assert_eq!(obj["gamma"]["slot"], Value::from(2));
}

#[test]
fn standard_abi_includes_constructor_and_event() {
    let constructor = FunctionMetadata {
        name: "constructor".to_string(),
        neo_name: "constructor".to_string(),
        kind: FunctionKind::Constructor,
        parameters: vec![
            ParameterMetadata {
                name: Some("owner".to_string()),
                ty: "address".to_string(),
                neo_type: Some(NeoType::Address),
                storage: None,
            },
            ParameterMetadata {
                name: Some("initialSupply".to_string()),
                ty: "uint256".to_string(),
                neo_type: Some(NeoType::Integer {
                    signed: false,
                    bits: 256,
                }),
                storage: None,
            },
        ],
        return_parameters: vec![],
        state_mutability: StateMutability::NonPayable,
        visibility: VisibilityKind::Public,
        offset: 0,
        body: None,
        selector: [0u8; 4],
        is_virtual: false,
        is_override: false,
        documentation: NatspecDoc::default(),
        had_modifier_epilogue: false,
    };

    let regular = FunctionMetadata {
        name: "balanceOf".to_string(),
        neo_name: "balanceOf".to_string(),
        kind: FunctionKind::Regular,
        parameters: vec![ParameterMetadata {
            name: Some("addr".to_string()),
            ty: "address".to_string(),
            neo_type: Some(NeoType::Address),
            storage: None,
        }],
        return_parameters: vec![ParameterMetadata {
            name: None,
            ty: "uint256".to_string(),
            neo_type: Some(NeoType::Integer {
                signed: false,
                bits: 256,
            }),
            storage: None,
        }],
        state_mutability: StateMutability::View,
        visibility: VisibilityKind::Public,
        offset: 0,
        body: None,
        selector: [0u8; 4],
        is_virtual: false,
        is_override: false,
        documentation: NatspecDoc::default(),
        had_modifier_epilogue: false,
    };

    let metadata = ContractMetadata {
        name: "Token".to_string(),
        is_abstract: false,
        is_interface: false,
        is_library: false,
        methods: vec![constructor, regular],
        events: vec![EventMetadata {
            name: "Mint".to_string(),
            normalized_name: "Mint".to_string(),
            parameters: vec![
                EventParameter {
                    name: Some("to".to_string()),
                    ty: "address".to_string(),
                    indexed: true,
                    neo_type: Some(NeoType::Address),
                },
                EventParameter {
                    name: Some("amount".to_string()),
                    ty: "uint256".to_string(),
                    indexed: false,
                    neo_type: Some(NeoType::Integer {
                        signed: false,
                        bits: 256,
                    }),
                },
            ],
            anonymous: false,
        }],
        errors: vec![],
        uses_storage: true,
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

    let abi = build_standard_abi(&metadata);
    let ctor_entry = abi
        .iter()
        .find(|entry| entry["type"] == "constructor")
        .expect("constructor entry");
    assert_eq!(
        ctor_entry["inputs"]
            .as_array()
            .map(|a| a.len())
            .unwrap_or(0),
        2
    );
    assert_eq!(ctor_entry["stateMutability"], Value::from("nonpayable"));

    let function_entry = abi
        .iter()
        .find(|entry| entry["name"] == "balanceOf")
        .expect("balanceOf");
    assert_eq!(function_entry["outputs"][0]["type"], Value::from("uint256"));

    let event_entry = abi
        .iter()
        .find(|entry| entry["type"] == "event" && entry["name"] == "Mint")
        .expect("Mint event");
    assert_eq!(
        event_entry["inputs"][0]["indexed"],
        Value::Bool(true),
        "first parameter should be indexed"
    );
}
