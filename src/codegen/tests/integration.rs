#[test]
fn mapping_code_generation_emits_storage_ops() {
    let source = r#"
        pragma solidity ^0.8.19;

        contract MappingExample {
            mapping(address => uint256) public balances;

            function setBalance(address owner, uint256 amount) public {
                balances[owner] = amount;
            }

            function getBalance(address owner) public view returns (uint256) {
                return balances[owner];
            }
        }
        "#;

    let mut metadata = analyse_source(source).expect("analysis failed");
    let ir_module = ir::Module::from_contract(&metadata).expect("IR lowering failed");
    let bytecode = generate_contract_bytecode(&mut metadata, &ir_module, false, 2, false)
        .expect("bytecode generation")
        .script;

    assert!(!bytecode.is_empty());

    // Mapping slots are hashed via CryptoLib.keccak256 through System.Contract.Call.
    let call_id = interop_id_bytes("System.Contract.Call");
    assert!(bytecode.windows(4).any(|window| window == call_id));

    const CRYPTOLIB_HASH_LE: [u8; 20] = [
        0x1B, 0xF5, 0x75, 0xAB, 0x11, 0x89, 0x68, 0x84, 0x13, 0x61, 0x0A, 0x35, 0xA1, 0x28, 0x86,
        0xCD, 0xE0, 0xB6, 0x6C, 0x72,
    ];
    let mut cryptolib_push = vec![0x0C, 0x14];
    cryptolib_push.extend_from_slice(&CRYPTOLIB_HASH_LE);
    assert!(
        bytecode
            .windows(cryptolib_push.len())
            .any(|window| window == cryptolib_push.as_slice()),
        "expected CryptoLib script hash to be pushed for keccak256"
    );

    let put_id = interop_id_bytes("System.Storage.Put");
    assert!(bytecode.windows(4).any(|window| window == put_id));
    let get_id = interop_id_bytes("System.Storage.Get");
    assert!(bytecode.windows(4).any(|window| window == get_id));
}

#[test]
fn event_emission_places_name_first_in_payload() {
    // `emit Ping(1, 2)` lowers to an EVM-canonical state shape while keeping
    // the `Runtime.Notify` eventName as the raw UTF-8 `"Ping"` string. The
    // signature keccak is carried as state[0], followed by ABI data.
    let source = r#"
        pragma solidity ^0.8.19;

        contract EventOrder {
            event Ping(uint256 a, uint256 b);

            function fire() public {
                emit Ping(1, 2);
            }
        }
        "#;

    let mut metadata = analyse_source(source).expect("analysis failed");
    let ir_module = ir::Module::from_contract(&metadata).expect("IR lowering failed");
    let bytecode = generate_contract_bytecode(&mut metadata, &ir_module, false, 2, false)
        .expect("bytecode generation")
        .script;

    let notify_id = interop_id_bytes("System.Runtime.Notify");
    let notify_sequence: Vec<u8> = std::iter::once(0x41u8).chain(notify_id).collect();
    assert!(
        bytecode
            .windows(notify_sequence.len())
            .any(|window| window == notify_sequence),
        "expected Runtime.Notify syscall"
    );

    // The event name "Ping" is pushed as the eventName arg of Runtime.Notify
    // (UTF-8 string, matching the manifest declaration).
    let mut expected_name_push = vec![0x0C, 0x04];
    expected_name_push.extend_from_slice(b"Ping");
    assert!(
        bytecode
            .windows(expected_name_push.len())
            .any(|window| window == expected_name_push.as_slice()),
        "expected event name 'Ping' to be pushed as eventName arg"
    );

    // The keccak hash appears in bytecode as the first state-array item, not
    // as the Runtime.Notify eventName.
    use sha3::{Digest, Keccak256};
    let topic0 = Keccak256::digest(b"Ping(uint256,uint256)");
    let mut keccak_push = vec![0x0C, 0x20];
    keccak_push.extend_from_slice(&topic0);
    assert!(
        bytecode
            .windows(keccak_push.len())
            .any(|window| window == keccak_push.as_slice()),
        "keccak256(\"Ping(...)\") must appear as the state-array topic0"
    );
}

#[test]
fn zero_arg_abi_encode_skips_pseudo_native_stdlib_call() {
    let source = r#"
        pragma solidity ^0.8.19;

        contract EmptyEncode {
            function f() public pure returns (bytes memory) {
                return abi.encode();
            }
        }
        "#;

    let mut metadata = analyse_source(source).expect("analysis failed");
    let ir_module = ir::Module::from_contract(&metadata).expect("IR lowering failed");
    let bytecode = generate_contract_bytecode(&mut metadata, &ir_module, false, 2, false)
        .expect("bytecode generation")
        .script;

    assert!(
        !bytecode
            .windows(b"abiEncode".len())
            .any(|window| window == b"abiEncode"),
        "zero-arg abi.encode() should not lower to StdLib.abiEncode"
    );
}

#[test]
fn nonzero_abi_helpers_do_not_emit_pseudo_native_stdlib_calls() {
    let source = r#"
        pragma solidity ^0.8.19;

        contract EncodeDecode {
            function encode(uint256 value) public pure returns (bytes memory) {
                return abi.encode(value);
            }

            function packed(uint256 value) public pure returns (bytes memory) {
                return abi.encodePacked(value);
            }

            function decode(bytes memory data) public pure returns (uint256) {
                return abi.decode(data, (uint256));
            }
        }
        "#;

    let mut metadata = analyse_source(source).expect("analysis failed");
    let ir_module = ir::Module::from_contract(&metadata).expect("IR lowering failed");
    let bytecode = generate_contract_bytecode(&mut metadata, &ir_module, false, 2, false)
        .expect("bytecode generation")
        .script;

    for pseudo_method in [b"abiEncode".as_slice(), b"abiEncodePacked", b"abiDecode"] {
        assert!(
            !bytecode
                .windows(pseudo_method.len())
                .any(|window| window == pseudo_method),
            "ABI helpers must not lower to nonexistent StdLib.{}",
            std::str::from_utf8(pseudo_method).unwrap()
        );
    }
}

#[test]
fn static_custom_error_revert_skips_pseudo_native_abi_encode() {
    let source = r#"
        pragma solidity ^0.8.19;

        contract StaticError {
            error Unauthorized(address who, uint256 code);

            function boom() public pure {
                revert Unauthorized(address(0x1234), 7);
            }
        }
        "#;

    let mut metadata = analyse_source(source).expect("analysis failed");
    let ir_module = ir::Module::from_contract(&metadata).expect("IR lowering failed");
    let bytecode = generate_contract_bytecode(&mut metadata, &ir_module, false, 2, false)
        .expect("bytecode generation")
        .script;

    assert!(
        !bytecode
            .windows(b"abiEncode".len())
            .any(|window| window == b"abiEncode"),
        "static custom-error revert should not lower to StdLib.abiEncode"
    );
}

#[test]
fn empty_contract_emits_ret_instruction() {
    // No functions other than constructors -> should produce a single RET (0x40)
    let mut metadata = ContractMetadata {
        name: "Empty".to_string(),
        is_abstract: false,
        is_interface: false,
        is_library: false,
        methods: vec![FunctionMetadata {
            name: "constructor".to_string(),
            neo_name: "constructor".to_string(),
            kind: FunctionKind::Constructor,
            parameters: vec![],
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
        }],
        events: vec![],
        errors: vec![],
        uses_storage: false,
        state_variables: vec![],
        structs: vec![],
        enums: vec![],
        contract_types: vec![],
        selector_registry: std::sync::Arc::new(crate::solidity::SelectorRegistry::default()),
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

    let ir_module = ir::Module {
        functions: vec![],
        state_variables: vec![],
        events: vec![],
    };

    let bytecode = generate_contract_bytecode(&mut metadata, &ir_module, false, 2, false)
        .expect("bytecode generation")
        .script;
    assert_eq!(bytecode, vec![0x40], "should emit a lone RET");
}

#[test]
fn call_fixups_patch_offsets() {
    let mut metadata = ContractMetadata {
        name: "Callers".to_string(),
        is_abstract: false,
        is_interface: false,
        is_library: false,
        methods: vec![
            FunctionMetadata {
                name: "foo".to_string(),
                neo_name: "foo".to_string(),
                kind: FunctionKind::Regular,
                parameters: vec![],
                return_parameters: vec![],
                state_mutability: StateMutability::View,
                visibility: VisibilityKind::Public,
                offset: 0,
                body: None,
                selector: [0u8; 4],
                is_virtual: false,
                is_override: false,
                documentation: NatspecDoc::default(),
                had_modifier_epilogue: false,
            },
            FunctionMetadata {
                name: "bar".to_string(),
                neo_name: "bar".to_string(),
                kind: FunctionKind::Regular,
                parameters: vec![],
                return_parameters: vec![],
                state_mutability: StateMutability::View,
                visibility: VisibilityKind::Public,
                offset: 0,
                body: None,
                selector: [0u8; 4],
                is_virtual: false,
                is_override: false,
                documentation: NatspecDoc::default(),
                had_modifier_epilogue: false,
            },
        ],
        events: vec![],
        errors: vec![],
        uses_storage: false,
        state_variables: vec![],
        structs: vec![],
        enums: vec![],
        contract_types: vec![],
        selector_registry: std::sync::Arc::new(crate::solidity::SelectorRegistry::default()),
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

    let mut module = ir::Module {
        functions: vec![],
        state_variables: vec![],
        events: vec![],
    };

    module.functions.push(ir::Function {
        name: "foo".to_string(),
        kind: ir::FunctionKind::Regular,
        parameters: vec![],
        returns: vec![],
        basic_blocks: vec![ir::BasicBlock {
            instructions: vec![
                ir::Instruction::CallFunction {
                    name: "bar".to_string(),
                    arg_count: 0,
                },
                ir::Instruction::ReturnVoid,
            ],
        }],
        local_count: 0,
    });
    module.functions.push(ir::Function {
        name: "bar".to_string(),
        kind: ir::FunctionKind::Regular,
        parameters: vec![],
        returns: vec![],
        basic_blocks: vec![ir::BasicBlock {
            instructions: vec![ir::Instruction::ReturnVoid],
        }],
        local_count: 0,
    });

    let bytecode = generate_contract_bytecode(&mut metadata, &module, false, 2, false)
        .expect("bytecode generation")
        .script;

    let foo_offset = metadata
        .methods
        .iter()
        .find(|m| m.name == "foo")
        .map(|m| m.offset)
        .unwrap();
    assert_eq!(foo_offset, 0, "foo should start at offset 0");

    let bar_offset = metadata
        .methods
        .iter()
        .find(|m| m.name == "bar")
        .map(|m| m.offset)
        .unwrap();
    assert!(
        bar_offset > 0,
        "bar should be placed after foo in bytecode sequence"
    );

    // Locate CALL_L opcode and patched relative offset (little endian immediately following)
    let call_pos = bytecode
        .iter()
        .position(|b| *b == 0x35)
        .expect("CALL_L opcode should be present");
    let patched = i32::from_le_bytes([
        bytecode[call_pos + 1],
        bytecode[call_pos + 2],
        bytecode[call_pos + 3],
        bytecode[call_pos + 4],
    ]);
    let expected = bar_offset as i32 - call_pos as i32;
    assert_eq!(
        patched, expected,
        "CALL_L target should be patched with relative offset to bar"
    );
}
