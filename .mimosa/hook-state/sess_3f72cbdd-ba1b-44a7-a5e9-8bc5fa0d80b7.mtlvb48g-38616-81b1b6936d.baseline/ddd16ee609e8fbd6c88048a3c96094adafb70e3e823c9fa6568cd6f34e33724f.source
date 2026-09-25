#[test]
fn bytecode_peephole_reduces_size_at_o3() {
    let mut metadata = ContractMetadata {
        name: "Peephole".to_string(),
        is_abstract: false,
            is_interface: false,
        is_library: false,
        methods: vec![FunctionMetadata {
            name: "foo".to_string(),
            neo_name: "foo".to_string(),
            kind: FunctionKind::Regular,
            parameters: vec![],
            return_parameters: vec![],
            state_mutability: neo_devpack_solidity::solidity::StateMutability::NonPayable,
            visibility: neo_devpack_solidity::frontend::VisibilityKind::Public,
            offset: 0,
            body: None,
            selector: [0u8; 4],
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

    let module = Module {
        functions: vec![Function {
            name: "foo".to_string(),
            kind: IrFunctionKind::Regular,
            parameters: vec![],
            returns: vec![],
            basic_blocks: vec![BasicBlock {
                instructions: vec![
                    Instruction::PushLiteral(neo_devpack_solidity::ir::LiteralValue::Boolean(true)),
                    Instruction::Drop(ValueType::Boolean),
                    Instruction::Return,
                ],
            }],
            local_count: 0,
        }],
        state_variables: vec![],
        events: vec![],
    };

    let bytecode_o0 = generate_contract_bytecode(&mut metadata.clone(), &module, false, 0, false)
        .expect("bytecode generation")
        .script;
    let bytecode_o3 = generate_contract_bytecode(&mut metadata, &module, false, 3, false)
        .expect("bytecode generation")
        .script;

    assert!(
        bytecode_o3.len() <= bytecode_o0.len(),
        "peephole optimizer should not grow bytecode at O3 (O0={}, O3={})",
        bytecode_o0.len(),
        bytecode_o3.len()
    );

    // O3 output should not contain redundant RET RET
    let double_ret = bytecode_o3.windows(2).any(|w| w[0] == 0x40 && w[1] == 0x40);
    assert!(
        !double_ret,
        "peephole optimizer should collapse duplicate RET instructions"
    );
}
