#[test]
fn uint_cast_from_string_byte_materializes_buffer_before_reverse() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract CastHarness {
        function f(string memory raw) external pure returns (uint8) {
            return uint8(bytes(raw)[0]);
        }
    }
    "#;

    let metadata = analyse_source(source).expect("analysis failed");
    let module = ir::Module::from_contract(&metadata).expect("IR lowering failed");
    let function = module
        .functions
        .iter()
        .find(|function| function.name == "f")
        .expect("expected f function");
    let instrs = &function.basic_blocks[0].instructions;

    let has_old_pattern = instrs.windows(3).any(|window| {
        matches!(
            window,
            [
                ir::Instruction::Convert {
                    target: ir::ConvertTarget::ByteArray
                },
                ir::Instruction::Dup,
                ir::Instruction::ReverseItems
            ]
        )
    });
    assert!(
        !has_old_pattern,
        "bytes-to-int cast must not emit CONVERT(ByteArray) -> DUP -> REVERSEITEMS directly"
    );

    if let Some(reverse_index) = instrs
        .iter()
        .position(|instr| matches!(instr, ir::Instruction::ReverseItems))
    {
        assert!(
            instrs[..reverse_index]
                .iter()
                .any(|instr| matches!(instr, ir::Instruction::NewBuffer)),
            "expected any REVERSEITEMS used for bytes-to-int casting to be preceded by buffer materialization"
        );
    }
}
