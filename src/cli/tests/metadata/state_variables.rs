fn contract_with_state_variables(state_variables: Vec<StateVariableMetadata>) -> ContractMetadata {
    ContractMetadata {
        name: "StateVariableDiagnostics".to_string(),
        is_abstract: false,
            is_interface: false,
        is_library: false,
        methods: vec![],
        events: vec![],
        errors: vec![],
        uses_storage: state_variables.iter().any(|state| !state.is_constant),
        state_variables,
        structs: vec![],
        enums: vec![],
        contract_types: vec![],
        selector_registry: std::sync::Arc::new(SelectorRegistry::default()),
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
    }
}

fn constant_address_state(name: &str) -> StateVariableMetadata {
    StateVariableMetadata {
        name: Some(name.to_string()),
        ty: "address".to_string(),
        is_transient: false,
        is_constant: true,
        is_immutable: false,
        visibility: Some("internal".to_string()),
        neo_type: Some(NeoType::Address),
        has_initializer: true,
        initializer: None,
    }
}

fn constant_uint_state(name: &str) -> StateVariableMetadata {
    StateVariableMetadata {
        name: Some(name.to_string()),
        ty: "uint256".to_string(),
        is_transient: false,
        is_constant: true,
        is_immutable: false,
        visibility: Some("internal".to_string()),
        neo_type: Some(NeoType::Integer {
            signed: false,
            bits: 256,
        }),
        has_initializer: true,
        initializer: None,
    }
}

fn storage_uint_state(name: &str) -> StateVariableMetadata {
    StateVariableMetadata {
        name: Some(name.to_string()),
        ty: "uint256".to_string(),
        is_transient: false,
        is_constant: false,
        is_immutable: false,
        visibility: Some("internal".to_string()),
        neo_type: Some(NeoType::Integer {
            signed: false,
            bits: 256,
        }),
        has_initializer: false,
        initializer: None,
    }
}

#[test]
fn identical_duplicate_constant_state_variables_are_silent() {
    let metadata = contract_with_state_variables(vec![
        constant_address_state("NEO_CONTRACT"),
        constant_address_state("NEO_CONTRACT"),
    ]);

    let diagnostics = validate_contract(&metadata);
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code.as_deref() != Some("W121")),
        "identical constant duplicates should not emit W121, got: {diagnostics:?}",
    );
}

#[test]
fn conflicting_duplicate_constant_state_variables_emit_single_warning() {
    let metadata = contract_with_state_variables(vec![
        constant_address_state("NEO_CONTRACT"),
        constant_uint_state("NEO_CONTRACT"),
        constant_uint_state("NEO_CONTRACT"),
    ]);

    let diagnostics = validate_contract(&metadata);
    let warnings: Vec<&Diagnostic> = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code.as_deref() == Some("W121"))
        .collect();
    assert_eq!(
        warnings.len(),
        1,
        "conflicting constant duplicates should emit exactly one W121, got: {diagnostics:?}",
    );
    assert!(
        warnings[0]
            .message
            .contains("conflicting duplicate constant state variable 'NEO_CONTRACT'"),
        "W121 message should explain conflict semantics, got: {:?}",
        warnings[0].message,
    );
}

#[test]
fn non_constant_duplicate_state_variables_still_emit_w122() {
    let metadata = contract_with_state_variables(vec![
        constant_uint_state("totalSupply"),
        storage_uint_state("totalSupply"),
        storage_uint_state("totalSupply"),
    ]);

    let diagnostics = validate_contract(&metadata);
    let non_constant_warnings: Vec<&Diagnostic> = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code.as_deref() == Some("W122"))
        .collect();
    assert_eq!(
        non_constant_warnings.len(),
        1,
        "non-constant duplicates should emit exactly one W122, got: {diagnostics:?}",
    );

    let constant_conflict_warnings: Vec<&Diagnostic> = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code.as_deref() == Some("W121"))
        .collect();
    assert_eq!(
        constant_conflict_warnings.len(),
        0,
        "non-constant duplicates should not emit W121, got: {diagnostics:?}",
    );
}
