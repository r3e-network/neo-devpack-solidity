#[test]
fn tuple_return_is_lowered_to_static_abi_slots() {
    // Multi-value static `return (a, b)` must produce BE-packed 32-byte
    // slots without routing through pseudo-native StdLib ABI helpers that do
    // not exist on real Neo N3.
    let source = r#"
    pragma solidity ^0.8.19;

    contract TupleReturn {
        function foo() public pure returns (uint256, bool) {
            return (1, true);
        }
    }
    "#;

    let metadata = analyse_source(source).expect("analysis failed");
    let module = ir::Module::from_contract(&metadata).expect("IR lowering failed");

    let foo = module
        .functions
        .iter()
        .find(|function| function.name == "foo")
        .expect("expected foo");

    let instrs = &foo.basic_blocks[0].instructions;
    let concat_call = instrs.iter().find(|instr| {
        matches!(
            instr,
            ir::Instruction::CallBuiltin {
                builtin: ir::BuiltinCall::BytesConcat,
                arg_count: 2,
            }
        )
    });
    assert!(
        concat_call.is_some(),
        "expected tuple return to emit static ABI slot BytesConcat with arg_count=2; \
         got instructions: {instrs:?}",
    );
    assert!(
        !instrs.iter().any(|instr| {
            matches!(
                instr,
                ir::Instruction::CallBuiltin {
                    builtin: ir::BuiltinCall::AbiEncode,
                    ..
                }
            )
        }),
        "static tuple return must not depend on pseudo-native AbiEncode"
    );
    // No NewArray should be emitted for the return value — that was the
    // pre-Task-#64 shape that leaked as JSON at main-frame RET.
    assert!(
        !instrs
            .iter()
            .any(|instr| matches!(instr, ir::Instruction::NewArray { .. })),
        "tuple return must NOT build a StackItem::Array post-Task-#64"
    );
}

#[test]
fn tuple_assignment_picks_items_from_returned_array() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract TupleAssign {
        function foo() internal pure returns (uint256, uint256) {
            return (1, 2);
        }

        function bar() public pure returns (uint256) {
            (uint256 x, uint256 y) = foo();
            return x + y;
        }
    }
    "#;

    let metadata = analyse_source(source).expect("analysis failed");
    let module = ir::Module::from_contract(&metadata).expect("IR lowering failed");

    let bar = module
        .functions
        .iter()
        .find(|function| function.name == "bar")
        .expect("expected bar");

    let instrs = &bar.basic_blocks[0].instructions;
    assert!(
        instrs
            .iter()
            .any(|instr| matches!(instr, ir::Instruction::ArrayGet)),
        "expected tuple assignment to pick items from returned array"
    );
}
#[test]
fn nested_tuple_destructuring_assignment_is_lowered() {
    let source = r#"
    pragma solidity ^0.8.20;

    contract NestedTupleAssign {
        function make() internal pure returns ((uint256, uint256), uint256) {
            return ((1, 2), 3);
        }

        function run() public pure returns (uint256) {
            ((uint256 a, uint256 b), uint256 c) = make();
            return a + b + c;
        }
    }
    "#;

    let metadata = analyse_source(source).expect("analysis failed");
    let module = ir::Module::from_contract(&metadata).expect("IR lowering failed");

    let run = module
        .functions
        .iter()
        .find(|function| function.name == "run")
        .expect("expected run");

    let instrs = &run.basic_blocks[0].instructions;
    let array_get_count = instrs
        .iter()
        .filter(|instr| matches!(instr, ir::Instruction::ArrayGet))
        .count();

    assert!(
        array_get_count >= 3,
        "expected nested tuple destructuring to perform nested ArrayGet operations"
    );
}
