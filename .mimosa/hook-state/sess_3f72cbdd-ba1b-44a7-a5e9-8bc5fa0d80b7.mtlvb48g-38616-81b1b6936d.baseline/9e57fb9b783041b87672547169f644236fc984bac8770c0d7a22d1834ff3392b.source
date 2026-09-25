#[test]
fn fixed_size_new_array_is_lowered() {
    let source = r#"
    pragma solidity ^0.8.20;

    contract FixedArrayNew {
        function run() public pure returns (uint256) {
            uint256[3] memory arr = new uint256[3];
            arr[1] = 9;
            return arr[1];
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
    assert!(
        instrs
            .iter()
            .any(|instr| matches!(instr, ir::Instruction::NewArray { .. })),
        "expected fixed-size new array allocation to emit NewArray"
    );
}
