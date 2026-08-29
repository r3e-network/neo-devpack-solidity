use super::*;
use solang_parser::pt::Identifier;

#[test]
fn lower_emit_pushes_event_name_before_args() {
    let state_index_map = HashMap::new();
    let state_variables: Vec<StateVariableMetadata> = Vec::new();
    let event_index_map = HashMap::new();
    let event_signature_map: HashMap<String, Vec<ManifestType>> = HashMap::new();
    // Register the declaration: `event MyEvent(uint256, uint256)` (both
    // non-indexed). The emit must lower to the EVM wire shape
    // `Notify("MyEvent", [topic0, data])`.
    let mut event_params_map: HashMap<String, EventSignature> = HashMap::new();
    let error_signature_map: HashMap<String, ErrorAbiSignature> = HashMap::new();
    let canonical = "MyEvent(uint256,uint256)".to_string();
    let mut hasher = sha3::Keccak256::new();
    hasher.update(canonical.as_bytes());
    let digest = hasher.finalize();
    let mut topic0 = [0u8; 32];
    topic0.copy_from_slice(&digest);
    event_params_map.insert(
        "MyEvent".to_string(),
        EventSignature {
            canonical,
            topic0,
            params: vec![
                EventParamInfo {
                    canonical_type: "uint256".to_string(),
                    indexed: false,
                    is_dynamic: false,
                },
                EventParamInfo {
                    canonical_type: "uint256".to_string(),
                    indexed: false,
                    is_dynamic: false,
                },
            ],
            is_anonymous: false,
        },
    );
    let enum_variant_map: HashMap<String, HashMap<String, u64>> = HashMap::new();
    let contract_types = HashSet::new();
    let selector_registry = SelectorRegistry::default();
    let function_names = HashSet::new();
    let function_overloads: FunctionOverloadTable = HashMap::new();
    let function_first_param_types: HashMap<(String, usize), Vec<ValueType>> = HashMap::new();
    let function_return_types: HashMap<(String, usize), ValueType> = HashMap::new();
    let using_target_types: Vec<Option<String>> = Vec::new();
    let using_function_list_targets: HashMap<String, Vec<Option<String>>> = HashMap::new();
    let using_function_list_scope_targets: Vec<Option<String>> = Vec::new();
    let function_param_names: HashMap<(String, usize), Vec<String>> = HashMap::new();
    let void_functions = HashSet::new();
    let externally_callable_functions: HashSet<(String, usize)> = HashSet::new();
    let state_types: Vec<ValueType> = Vec::new();
    let defined_struct_types: Vec<ValueType> = Vec::new();
    let struct_fixed_array_bounds: HashMap<(String, String), u64> = HashMap::new();
    let super_method_map: HashMap<String, String> = HashMap::new();
    let library_storage_bodies: HashMap<(String, usize), LibraryStorageBody> = HashMap::new();
    let storage_pointer_returning_fns: HashMap<String, String> = HashMap::new();

    let mut ctx = LoweringContext::new(
        "test_emit",
        "TestContract",
        [0u8; 4],
        false,
        false,
        HashMap::new(),
        &[],
        &state_variables,
        &state_index_map,
        &state_types,
        &defined_struct_types,
        &struct_fixed_array_bounds,
        &event_index_map,
        &event_signature_map,
        &event_params_map,
        &error_signature_map,
        &enum_variant_map,
        &contract_types,
        &selector_registry,
        &function_names,
        &function_overloads,
        &function_first_param_types,
        &function_return_types,
        &using_target_types,
        &using_function_list_targets,
        &using_function_list_scope_targets,
        &function_param_names,
        &void_functions,
        &externally_callable_functions,
        &super_method_map,
        &library_storage_bodies,
        &storage_pointer_returning_fns,
    );

    let expr = Expression::FunctionCall(
        Default::default(),
        Box::new(Expression::Variable(Identifier {
            loc: Default::default(),
            name: "MyEvent".to_string(),
        })),
        vec![
            Expression::NumberLiteral(Default::default(), "1".to_string(), "".to_string(), None),
            Expression::NumberLiteral(Default::default(), "2".to_string(), "".to_string(), None),
        ],
    );

    let mut instructions = Vec::new();
    lower_emit(&expr, &mut ctx, &mut instructions);

    assert!(
        ctx.errors.is_empty(),
        "lowering produced errors: {:?}",
        ctx.errors
    );
    assert!(
        matches!(
            instructions.first(),
            Some(Instruction::PushLiteral(LiteralValue::String(bytes))) if bytes == b"MyEvent"
        ),
        "expected event name literal to be pushed first, got {:?}",
        instructions.first()
    );
    assert!(
        instructions.iter().any(|instr| matches!(
            instr,
            Instruction::EmitEventByName { name, arg_count }
                if name == "MyEvent" && *arg_count == 2
        )),
        "expected EmitEventByName with the EVM wire count ([topic0, data] = 2)"
    );

    // Emitting an event with NO registered declaration must record a compile
    // error (an undeclared notification faults on Neo nodes >= 3.6) and must
    // not lower any instructions.
    let undeclared = Expression::FunctionCall(
        Default::default(),
        Box::new(Expression::Variable(Identifier {
            loc: Default::default(),
            name: "Undeclared".to_string(),
        })),
        vec![Expression::NumberLiteral(
            Default::default(),
            "1".to_string(),
            "".to_string(),
            None,
        )],
    );
    let mut undeclared_instructions = Vec::new();
    lower_emit(&undeclared, &mut ctx, &mut undeclared_instructions);
    assert!(
        undeclared_instructions.is_empty(),
        "undeclared event emit must not lower instructions, got {undeclared_instructions:?}"
    );
    assert!(
        ctx.errors
            .iter()
            .any(|err| err.message.contains("Undeclared")),
        "expected a compile error for the undeclared event, got {:?}",
        ctx.errors
    );
}
