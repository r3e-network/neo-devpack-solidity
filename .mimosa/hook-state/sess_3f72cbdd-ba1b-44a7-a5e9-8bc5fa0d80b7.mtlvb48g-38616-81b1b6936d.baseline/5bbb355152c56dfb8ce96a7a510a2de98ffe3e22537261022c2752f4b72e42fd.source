#[test]
fn base_constructor_is_inlined_into_derived_constructor() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract Base {
        uint256 public x;

        constructor() {
            x = 1;
        }
    }

    contract Derived is Base {
        uint256 public y;

        constructor() Base() {
            y = 2;
        }
    }
    "#;

    let metadatas = analyse_all_sources(source).expect("analysis failed");
    let derived = metadatas
        .into_iter()
        .find(|m| m.name == "Derived")
        .expect("expected Derived metadata");

    let x_index = derived
        .state_variables
        .iter()
        .position(|v| v.name.as_deref() == Some("x"))
        .expect("expected state variable x");
    let y_index = derived
        .state_variables
        .iter()
        .position(|v| v.name.as_deref() == Some("y"))
        .expect("expected state variable y");
    assert!(
        x_index < y_index,
        "expected base state variable to precede derived state variable"
    );

    let module = ir::Module::from_contract(&derived).expect("IR lowering failed");
    let ctor = module
        .functions
        .iter()
        .find(|function| matches!(function.kind, ir::FunctionKind::Constructor))
        .expect("expected constructor function");

    let instrs = &ctor.basic_blocks[0].instructions;
    assert!(
        instrs.iter().any(|instr| matches!(instr, ir::Instruction::StoreState(idx) if *idx == x_index)),
        "expected base constructor assignment to store state variable 'x'; instrs: {instrs:?}"
    );
    assert!(
        instrs.iter().any(|instr| matches!(instr, ir::Instruction::StoreState(idx) if *idx == y_index)),
        "expected derived constructor assignment to store state variable 'y'; instrs: {instrs:?}"
    );
}

#[test]
fn base_constructor_args_from_intermediate_contract_are_applied() {
    use num_bigint::BigInt;

    let source = r#"
    pragma solidity ^0.8.19;

    contract A {
        uint256 public x;

        constructor(uint256 v) {
            x = v;
        }
    }

    contract B is A {
        constructor() A(7) {}
    }

    contract C is B {
        constructor() {}
    }
    "#;

    let metadatas = analyse_all_sources(source).expect("analysis failed");
    let c = metadatas
        .into_iter()
        .find(|m| m.name == "C")
        .expect("expected C metadata");

    let x_index = c
        .state_variables
        .iter()
        .position(|v| v.name.as_deref() == Some("x"))
        .expect("expected x in state variables");

    let module = ir::Module::from_contract(&c).expect("IR lowering failed");
    let ctor = module
        .functions
        .iter()
        .find(|function| matches!(function.kind, ir::FunctionKind::Constructor))
        .expect("expected constructor");

    let instrs = &ctor.basic_blocks[0].instructions;
    let mut found = false;
    for window in instrs.windows(2) {
        if matches!(
            window,
            [
                ir::Instruction::PushLiteral(ir::LiteralValue::Integer(value)),
                ir::Instruction::StoreState(state_index)
            ] if *state_index == x_index
                && value == &BigInt::from(7u8)
        ) {
            found = true;
            break;
        }
    }

    assert!(
        found,
        "expected base constructor arg 7 to be stored into 'x'; instrs: {instrs:?}"
    );
}
