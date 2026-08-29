#[test]
fn require_allows_execution_when_condition_is_true() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract RequireHarness {
        function ok() public pure returns (uint256) {
            require(true, "should not abort");
            return 1;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let result = execute_bytecode(&artifacts[0].bytecode);
    assert!(result.is_success(), "expected require(true) to succeed");
}

#[test]
fn require_throws_when_condition_is_false() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract RequireHarness {
        function fail() public pure returns (uint256) {
            require(false, "should abort");
            return 2;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let result = execute_bytecode(&artifacts[0].bytecode);
    assert!(
        !result.is_success(),
        "expected require(false) to abort execution"
    );
    let message = result
        .exception
        .as_ref()
        .map(|ex| ex.message.as_str())
        .unwrap_or_default();
    assert!(
        message.contains("THROW"),
        "expected failure message to mention THROW, got: {message}"
    );
}

#[test]
fn assert_panics_when_condition_is_false() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract AssertOk {
        function ok() public pure returns (uint256) {
            assert(true);
            return 1;
        }
    }

    contract AssertFail {
        function fail() public pure returns (uint256) {
            assert(false);
            return 2;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");

    let ok_artifact = artifacts
        .iter()
        .find(|artifact| artifact.metadata.name == "AssertOk")
        .expect("expected AssertOk artifact");
    let ok_result = execute_bytecode(&ok_artifact.bytecode);
    assert!(ok_result.is_success(), "expected assert(true) to succeed");

    let fail_artifact = artifacts
        .iter()
        .find(|artifact| artifact.metadata.name == "AssertFail")
        .expect("expected AssertFail artifact");
    let fail_result = execute_bytecode(&fail_artifact.bytecode);
    assert!(
        !fail_result.is_success(),
        "expected assert(false) to fail execution"
    );
    // Task #27 (compiler slice) — `assert(false)` now emits the EVM-canonical
    // Panic payload `keccak256("Panic(uint256)")[..4] || abi.encode(0x01)`
    // (36 bytes total) instead of a bare ASCII marker string. The machine-
    // readable shape lives in `return_data`; the exception `message` still
    // carries the THROW marker (for compatibility with the revert-path
    // discriminator) but is now a lossy UTF-8 decoding of the binary payload.
    let message = fail_result
        .exception
        .as_ref()
        .map(|ex| ex.message.as_str())
        .unwrap_or_default();
    assert!(
        message.contains("THROW"),
        "expected panic message to carry the THROW marker, got: {message}"
    );
    // 4-byte selector + 32-byte uint256 = 36 bytes.
    assert_eq!(
        fail_result.return_data.len(),
        36,
        "expected Panic(uint256) payload to be 36 bytes; got {} bytes",
        fail_result.return_data.len()
    );
    use sha3::{Digest, Keccak256};
    let mut hasher = Keccak256::new();
    hasher.update(b"Panic(uint256)");
    let expected_selector = hasher.finalize();
    assert_eq!(
        &fail_result.return_data[..4],
        &expected_selector[..4],
        "expected Panic(uint256) selector prefix"
    );
    let mut expected_code = [0u8; 32];
    expected_code[31] = 0x01;
    assert_eq!(
        &fail_result.return_data[4..36],
        &expected_code[..],
        "expected Panic code 0x01 (assertion failed) in abi.encode tail"
    );
}

#[test]
fn compound_assignment_subtraction_preserves_operand_order_and_returns_value() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract CompoundOrder {
        function run() public pure returns (uint256) {
            uint256 x = 10;
            x -= 3;
            return x;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let result = execute_bytecode(&artifacts[0].bytecode);
    assert!(result.is_success(), "expected run() to succeed");
    assert_eq!(result.return_data, 7i64.to_le_bytes().to_vec());
}

#[test]
fn compound_assignment_mapping_evaluates_key_once() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract KeyOnce {
        uint256 public counter;
        mapping(uint256 => uint256) public m;

        function key() internal returns (uint256) {
            counter = counter + 1;
            return 1;
        }

        function run() public returns (uint256) {
            m[1] = 10;
            m[key()] -= 3;
            return counter;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let result = execute_bytecode(&artifacts[0].bytecode);
    let failure = result
        .exception
        .as_ref()
        .map(|ex| ex.message.as_str())
        .unwrap_or("<no exception>");
    assert!(
        result.is_success(),
        "expected run() to succeed, got: {failure}"
    );
    assert_eq!(result.return_data, 1i64.to_le_bytes().to_vec());
}

#[test]
fn compound_assignment_mapping_subtraction_updates_value() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract MapCompound {
        mapping(uint256 => uint256) public m;

        function run() public returns (uint256) {
            m[1] = 10;
            m[1] -= 3;
            return m[1];
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let result = execute_bytecode(&artifacts[0].bytecode);
    assert!(result.is_success(), "expected run() to succeed");
    assert_eq!(result.return_data, 7i64.to_le_bytes().to_vec());
}

#[test]
fn division_by_zero_panics_with_0x12() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract DivZero {
        uint256 private x;

        function run() public returns (uint256) {
            return 1 / x;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let result = execute_bytecode(&artifacts[0].bytecode);
    assert!(!result.is_success(), "expected division by zero to fail");
    // Task #103 — div-by-zero now emits the canonical EVM envelope:
    //   keccak256("Panic(uint256)")[..4] || abi.encode(0x12)
    // Check the 36-byte structured payload on return_data.
    let rd = &result.return_data;
    assert!(
        rd.len() >= 36 && rd[..4] == [0x4eu8, 0x48, 0x7b, 0x71] && rd[35] == 0x12,
        "expected canonical Panic(0x12) envelope; got rd_len={} rd={:?}",
        rd.len(),
        rd
    );
}

#[test]
fn modulo_by_zero_panics_with_0x12() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract ModZero {
        uint256 private x;

        function run() public returns (uint256) {
            return 1 % x;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let result = execute_bytecode(&artifacts[0].bytecode);
    assert!(!result.is_success(), "expected modulo by zero to fail");
    // Task #103 — same canonical EVM envelope as div-by-zero.
    let rd = &result.return_data;
    assert!(
        rd.len() >= 36 && rd[..4] == [0x4eu8, 0x48, 0x7b, 0x71] && rd[35] == 0x12,
        "expected canonical Panic(0x12) envelope; got rd_len={} rd={:?}",
        rd.len(),
        rd
    );
}

#[test]
fn ternary_operator_evaluates_correct_branch() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract TernaryHarness {
        function value() public pure returns (uint256) {
            return (1 < 2) ? 1 : 2;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let result = execute_bytecode(&artifacts[0].bytecode);
    assert!(result.is_success(), "expected ternary execution to succeed");
    assert_eq!(result.return_data, 1i64.to_le_bytes().to_vec());
}

#[test]
fn power_operator_executes_exponentiation_loop() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract PowHarness {
        function value() public pure returns (uint256) {
            return 2 ** 3;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let result = execute_bytecode(&artifacts[0].bytecode);
    assert!(result.is_success(), "expected exponentiation to succeed");
    assert_eq!(result.return_data, 8i64.to_le_bytes().to_vec());
}

#[test]
fn deploy_update_flag_branches_to_skip_initializers() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract DeployHarness {
        uint256 public value = 7;
        constructor() {}
    }
    "#;

    let mut metadata = analyse_source(source).expect("analysis failed");
    ensure_deploy_stub(&mut metadata).expect("deploy stub");

    let module = ir::Module::from_contract(&metadata).expect("IR lowering failed");
    let deploy = module
        .functions
        .iter()
        .find(|function| function.name == "_deploy")
        .expect("expected _deploy function");
    let instrs = &deploy.basic_blocks[0].instructions;

    let (jump_if_target, jump_target) = match instrs.get(0..3) {
        Some(
            [ir::Instruction::LoadParameter(1), ir::Instruction::JumpIf { target }, ir::Instruction::Jump {
                target: jump_target,
            }],
        ) => (*target, *jump_target),
        other => panic!("unexpected deploy prologue: {other:?}"),
    };

    match instrs.get(3) {
        Some(ir::Instruction::Label(label)) if *label == jump_if_target => {}
        other => panic!("expected deploy to label run block, got: {other:?}"),
    }

    assert!(
        instrs
            .iter()
            .any(|instr| matches!(instr, ir::Instruction::Label(label) if *label == jump_target)),
        "expected deploy to include end label target"
    );
}
