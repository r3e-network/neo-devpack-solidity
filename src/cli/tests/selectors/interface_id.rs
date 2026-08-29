// Interface ID tests
//
// Tests for:
// - Interface ID calculation (XOR of function selectors)
// - Inherited interface functions in interface ID

#[test]
fn interface_id_lowers_to_xor_of_function_selectors() {
    let source = r#"
    pragma solidity ^0.8.20;

    interface IFoo {
        function a(uint256 x) external;
        function b(address y) external;
    }

    contract InterfaceIdHarness {
        function id() public pure returns (bytes4) {
            return type(IFoo).interfaceId;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 0).expect("compile");
    let artifact = artifacts
        .iter()
        .find(|a| a.metadata.name == "InterfaceIdHarness")
        .expect("contract artifact");

    let ir_module = ir::Module::from_contract(&artifact.metadata).expect("build IR");
    let id_function = ir_module
        .functions
        .iter()
        .find(|function| function.name == "id")
        .expect("id function");

    fn selector(signature: &str) -> [u8; 4] {
        let mut hasher = Keccak256::new();
        hasher.update(signature.as_bytes());
        let digest = hasher.finalize();
        let mut out = [0u8; 4];
        out.copy_from_slice(&digest[..4]);
        out
    }

    let sel_a = selector("a(uint256)");
    let sel_b = selector("b(address)");
    let expected: Vec<u8> = sel_a.iter().zip(sel_b.iter()).map(|(a, b)| a ^ b).collect();

    let instrs: Vec<_> = id_function
        .basic_blocks
        .iter()
        .flat_map(|block| block.instructions.iter())
        .collect();

    assert!(
        instrs.iter().any(|instr| matches!(
            instr,
            ir::Instruction::PushLiteral(ir::LiteralValue::ByteArray(bytes)) if *bytes == expected
        )),
        "expected `type(IFoo).interfaceId` lowering to push the correct bytes4 literal"
    );
}

#[test]
fn interface_id_includes_inherited_interface_functions() {
    let source = r#"
    pragma solidity ^0.8.20;

    interface IParent {
        function foo(uint256 x) external;
    }

    interface IChild is IParent {
        function bar(address y) external;
    }

    contract InterfaceIdInheritHarness {
        function id() public pure returns (bytes4) {
            return type(IChild).interfaceId;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 0).expect("compile");
    let artifact = artifacts
        .iter()
        .find(|a| a.metadata.name == "InterfaceIdInheritHarness")
        .expect("contract artifact");

    let ir_module = ir::Module::from_contract(&artifact.metadata).expect("build IR");
    let id_function = ir_module
        .functions
        .iter()
        .find(|function| function.name == "id")
        .expect("id function");

    fn selector(signature: &str) -> [u8; 4] {
        let mut hasher = Keccak256::new();
        hasher.update(signature.as_bytes());
        let digest = hasher.finalize();
        let mut out = [0u8; 4];
        out.copy_from_slice(&digest[..4]);
        out
    }

    let sel_foo = selector("foo(uint256)");
    let sel_bar = selector("bar(address)");
    let expected: Vec<u8> = sel_foo
        .iter()
        .zip(sel_bar.iter())
        .map(|(a, b)| a ^ b)
        .collect();

    let instrs: Vec<_> = id_function
        .basic_blocks
        .iter()
        .flat_map(|block| block.instructions.iter())
        .collect();

    assert!(
        instrs.iter().any(|instr| matches!(
            instr,
            ir::Instruction::PushLiteral(ir::LiteralValue::ByteArray(bytes)) if *bytes == expected
        )),
        "expected inherited interfaceId to XOR parent + child selectors"
    );
}
