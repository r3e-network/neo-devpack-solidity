#[test]
fn deploy_function_calls_constructor_when_present() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract ConstructorHarness {
        uint256 public stored;

        constructor() {
            stored = 42;
        }
    }
    "#;

    let mut metadata = analyse_source(source).expect("analysis failed");
    ensure_deploy_stub(&mut metadata).expect("deploy stub");
    let module = ir::Module::from_contract(&metadata).expect("IR lowering");

    let deploy = module
        .functions
        .iter()
        .find(|function| function.name == "_deploy")
        .expect("expected _deploy function");

    let mut calls_constructor = false;
    for instruction in &deploy.basic_blocks[0].instructions {
        if let ir::Instruction::CallFunction { name, .. } = instruction {
            if name == "constructor" {
                calls_constructor = true;
                break;
            }
        }
    }

    assert!(
        calls_constructor,
        "_deploy should call constructor when present"
    );
}

#[test]
fn deploy_function_supports_parameterised_constructors_via_json_deserialize_fallback() {
    let source = r#"
    pragma solidity ^0.8.20;

    contract ConstructorArgsHarness {
        uint256 public stored;

        constructor(uint256 value) {
            stored = value;
        }
    }
    "#;

    let mut metadata = analyse_source(source).expect("analysis failed");
    ensure_deploy_stub(&mut metadata).expect("deploy stub");
    let module = ir::Module::from_contract(&metadata).expect("IR lowering");

    let deploy = module
        .functions
        .iter()
        .find(|function| function.name == "_deploy")
        .expect("expected _deploy function");

    let instrs = &deploy.basic_blocks[0].instructions;

    let json_calls: Vec<_> = instrs
        .iter()
        .filter(|instr| {
            matches!(
                instr,
                ir::Instruction::CallBuiltin {
                    builtin: ir::BuiltinCall::NativeCall { contract, method },
                    arg_count: 1,
                } if *contract == ir::NativeContract::StdLib && method == "jsonDeserialize"
            )
        })
        .collect();
    assert_eq!(
        json_calls.len(),
        1,
        "expected exactly one StdLib.jsonDeserialize call in _deploy"
    );

    let deserialize_calls: Vec<_> = instrs
        .iter()
        .filter(|instr| {
            matches!(
                instr,
                ir::Instruction::CallBuiltin {
                    builtin: ir::BuiltinCall::NativeCall { contract, method },
                    arg_count: 1,
                } if *contract == ir::NativeContract::StdLib && method == "deserialize"
            )
        })
        .collect();
    assert_eq!(
        deserialize_calls.len(),
        1,
        "expected exactly one StdLib.deserialize call in _deploy"
    );

    let tries: Vec<_> = instrs
        .iter()
        .filter_map(|instr| match instr {
            ir::Instruction::Try { catch_target } => Some(*catch_target),
            _ => None,
        })
        .collect();
    assert_eq!(
        tries.len(),
        2,
        "expected TRY blocks for jsonDeserialize + deserialize fallbacks"
    );

    let endtrys: Vec<_> = instrs
        .iter()
        .filter(|instr| matches!(*instr, ir::Instruction::EndTry { .. }))
        .collect();
    assert_eq!(
        endtrys.len(),
        4,
        "expected success+catch ENDTRY for jsonDeserialize + deserialize"
    );

    for catch_label in tries {
        let catch_index = instrs
            .iter()
            .position(|instr| matches!(instr, ir::Instruction::Label(id) if *id == catch_label))
            .expect("catch label should exist");
        assert!(
            matches!(instrs.get(catch_index + 1), Some(ir::Instruction::Drop(_))),
            "expected catch handler to drop NeoVM exception value"
        );
    }
}

#[test]
fn deploy_function_initializes_state_variables() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract InitHarness {
        uint256 public a = 7;
        bool public b = true;
        uint256 public c;

        constructor() {}
    }
    "#;

    let mut metadata = analyse_source(source).expect("analysis failed");
    ensure_deploy_stub(&mut metadata).expect("deploy stub");
    let module = ir::Module::from_contract(&metadata).expect("IR lowering");

    let deploy = module
        .functions
        .iter()
        .find(|function| function.name == "_deploy")
        .expect("expected _deploy function");

    let instrs = &deploy.basic_blocks[0].instructions;
    assert!(
        instrs
            .iter()
            .any(|instr| matches!(instr, ir::Instruction::StoreState(idx) if *idx == 0)),
        "expected deploy prologue to store initializer for a"
    );
    assert!(
        instrs
            .iter()
            .any(|instr| matches!(instr, ir::Instruction::StoreState(idx) if *idx == 1)),
        "expected deploy prologue to store initializer for b"
    );
    assert!(
        !instrs
            .iter()
            .any(|instr| matches!(instr, ir::Instruction::StoreState(idx) if *idx == 2)),
        "c has no initializer and should not be stored"
    );
}
