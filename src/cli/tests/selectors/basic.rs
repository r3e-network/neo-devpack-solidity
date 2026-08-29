// Basic selector and type bounds tests
//
// Tests for:
// - Function selector literals (IFoo.bar.selector)
// - Type bounds (type(uint256).max)
// - Hex number literal coercion to bytes4

#[test]
fn interface_method_selector_lowers_to_bytes4_literal() {
    let source = r#"
    pragma solidity ^0.8.20;

    interface IFoo {
        function bar(uint256 x) external returns (bool);
    }

    contract SelectorHarness {
        function sel() public pure returns (bytes4) {
            return IFoo.bar.selector;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 0).expect("compile");
    let artifact = artifacts
        .iter()
        .find(|a| a.metadata.name == "SelectorHarness")
        .expect("contract artifact");

    let ir_module = ir::Module::from_contract(&artifact.metadata).expect("build IR");
    let sel_function = ir_module
        .functions
        .iter()
        .find(|function| function.name == "sel")
        .expect("sel function");

    let mut hasher = Keccak256::new();
    hasher.update("bar(uint256)".as_bytes());
    let digest = hasher.finalize();
    let expected = digest[..4].to_vec();

    let instrs: Vec<_> = sel_function
        .basic_blocks
        .iter()
        .flat_map(|block| block.instructions.iter())
        .collect();

    assert!(
        instrs.iter().any(|instr| matches!(
            instr,
            ir::Instruction::PushLiteral(ir::LiteralValue::ByteArray(bytes)) if *bytes == expected
        )),
        "expected selector bytes4 literal push in sel() IR"
    );
}

#[test]
fn msg_sig_lowers_to_current_function_selector_literal() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract MsgSigHarness {
        function getSelector() public pure returns (bytes4) {
            return msg.sig;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 0).expect("compile");
    let artifact = artifacts
        .iter()
        .find(|a| a.metadata.name == "MsgSigHarness")
        .expect("contract artifact");

    let ir_module = ir::Module::from_contract(&artifact.metadata).expect("build IR");
    let selector_function = ir_module
        .functions
        .iter()
        .find(|function| function.name == "getSelector")
        .expect("getSelector function");

    let mut hasher = Keccak256::new();
    hasher.update("getSelector()".as_bytes());
    let digest = hasher.finalize();
    let expected = digest[..4].to_vec();

    let instrs: Vec<_> = selector_function
        .basic_blocks
        .iter()
        .flat_map(|block| block.instructions.iter())
        .collect();

    assert!(
        instrs.iter().any(|instr| matches!(
            instr,
            ir::Instruction::PushLiteral(ir::LiteralValue::ByteArray(bytes)) if *bytes == expected
        )),
        "expected msg.sig lowering to push the current function selector bytes"
    );
}

#[test]
fn type_uint256_max_lowers_to_integer_literal() {
    let source = r#"
    pragma solidity ^0.8.20;

    contract MaxHarness {
        function max() public pure returns (uint256) {
            return type(uint256).max;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 0).expect("compile");
    let artifact = artifacts
        .iter()
        .find(|a| a.metadata.name == "MaxHarness")
        .expect("contract artifact");

    let ir_module = ir::Module::from_contract(&artifact.metadata).expect("build IR");
    let max_function = ir_module
        .functions
        .iter()
        .find(|function| function.name == "max")
        .expect("max function");

    let mut expected = BigInt::one();
    expected <<= 256usize;
    expected -= BigInt::one();

    let instrs: Vec<_> = max_function
        .basic_blocks
        .iter()
        .flat_map(|block| block.instructions.iter())
        .collect();

    assert!(
        instrs.iter().any(|instr| matches!(
            instr,
            ir::Instruction::PushLiteral(ir::LiteralValue::Integer(value)) if value == &expected
        )),
        "expected `type(uint256).max` lowering to push the correct BigInt literal"
    );
}

#[test]
fn hex_number_literal_coerces_to_fixed_bytes_for_eq() {
    let source = r#"
    pragma solidity ^0.8.20;

    contract Bytes4EqHarness {
        function ok(bytes4 interfaceId) public pure returns (bool) {
            return interfaceId == 0x01ffc9a7;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 0).expect("compile");
    let artifact = artifacts
        .iter()
        .find(|a| a.metadata.name == "Bytes4EqHarness")
        .expect("contract artifact");

    let ir_module = ir::Module::from_contract(&artifact.metadata).expect("build IR");
    let ok_function = ir_module
        .functions
        .iter()
        .find(|function| function.name == "ok")
        .expect("ok function");

    let expected = vec![0x01, 0xFF, 0xC9, 0xA7];
    let instrs: Vec<_> = ok_function
        .basic_blocks
        .iter()
        .flat_map(|block| block.instructions.iter())
        .collect();

    assert!(
        instrs.iter().any(|instr| matches!(
            instr,
            ir::Instruction::PushLiteral(ir::LiteralValue::ByteArray(bytes)) if *bytes == expected
        )),
        "expected bytes4 equality against hex number to push a fixed-length ByteArray literal"
    );
}

#[test]
fn type_of_call_is_rejected_as_value() {
    let source = r#"
    pragma solidity ^0.8.20;

    contract TypeOfHarness {
        function bad() public pure returns (uint256) {
            // Solidity does not allow using `type(T)` as a runtime value.
            return uint256(type(uint256));
        }
    }
    "#;

    assert!(
        compile_contracts(source, false, 0).is_err(),
        "expected `type(uint256)` used as a value to be rejected"
    );
}
