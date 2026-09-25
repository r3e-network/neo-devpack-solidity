#[test]
fn abi_encode_preserves_argument_order() {
    let source = r#"
    pragma solidity ^0.8.20;

    contract EncodeOrder {
        function encode() public pure returns (bytes memory) {
            return abi.encode(uint256(1), uint256(2));
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let result = crate::cli::tests::common::execute_bytecode(&artifacts[0].bytecode);
    assert!(result.is_success(), "expected encode() to succeed");

    let mut expected = vec![0u8; 64];
    expected[24..32].copy_from_slice(&1u64.to_be_bytes());
    expected[56..64].copy_from_slice(&2u64.to_be_bytes());
    assert_eq!(
        result.return_data, expected,
        "abi.encode(uint256(1), uint256(2)) must preserve argument order"
    );
}

#[test]
fn msg_data_lowers_to_runtime_input_data_with_arguments() {
    // msg.data now lowers to `RuntimeValue::MsgData`, which the bytecode emitter
    // resolves via `System.Runtime.GetScriptContainer.Script` (index 7 —
    // the raw input_data bytes passed to `execute`). The IR should therefore
    // contain exactly one `LoadRuntimeValue(MsgData)` and no argument-synthesis
    // (no selector push, no AbiEncode, no BytesConcat for this access).
    let source = r#"
    pragma solidity ^0.8.19;

    contract MsgDataHarness {
        function payload(uint256 a, address b) public pure returns (bytes memory) {
            return msg.data;
        }
    }
    "#;

    let metadata = analyse_source(source).expect("analysis failed");
    let ir_module = ir::Module::from_contract(&metadata).expect("IR lowering failed");
    let payload_function = ir_module
        .functions
        .iter()
        .find(|function| function.name == "payload")
        .expect("payload function");

    let instrs: Vec<_> = payload_function
        .basic_blocks
        .iter()
        .flat_map(|block| block.instructions.iter())
        .collect();

    let msg_data_loads = instrs
        .iter()
        .filter(|instr| {
            matches!(
                instr,
                ir::Instruction::LoadRuntimeValue(ir::RuntimeValue::MsgData)
            )
        })
        .count();
    assert_eq!(
        msg_data_loads, 1,
        "expected msg.data to lower to a single LoadRuntimeValue(MsgData) — found {msg_data_loads}",
    );

    // The old selector-prefix synthesis must be gone: no AbiEncode + BytesConcat
    // pair attributable to msg.data (function has no other abi.encode calls).
    assert!(
        !instrs.iter().any(|instr| matches!(
            instr,
            ir::Instruction::CallBuiltin {
                builtin: ir::BuiltinCall::AbiEncode,
                ..
            }
        )),
        "expected no AbiEncode for the new msg.data lowering",
    );
    // Keccak256 is still imported elsewhere in the module; keep the use-site
    // alive so the import doesn't go stale.
    let _ = Keccak256::new();
}

#[test]
fn msg_data_with_no_params_lowers_to_runtime_input_data() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract MsgDataHarness {
        function payload() public pure returns (bytes memory) {
            return msg.data;
        }
    }
    "#;

    let metadata = analyse_source(source).expect("analysis failed");
    let ir_module = ir::Module::from_contract(&metadata).expect("IR lowering failed");
    let payload_function = ir_module
        .functions
        .iter()
        .find(|function| function.name == "payload")
        .expect("payload function");

    let instrs: Vec<_> = payload_function
        .basic_blocks
        .iter()
        .flat_map(|block| block.instructions.iter())
        .collect();

    assert!(
        instrs.iter().any(|instr| matches!(
            instr,
            ir::Instruction::LoadRuntimeValue(ir::RuntimeValue::MsgData)
        )),
        "expected msg.data with no params to lower to LoadRuntimeValue(MsgData)",
    );
    assert!(
        !instrs.iter().any(|instr| matches!(
            instr,
            ir::Instruction::CallBuiltin {
                builtin: ir::BuiltinCall::BytesConcat,
                ..
            }
        )),
        "expected msg.data with no params not to call BytesConcat",
    );
}

#[test]
fn on_nep11_payment_msg_data_uses_param_3() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract Nep11PaymentHarness {
        function onNEP11Payment(address from, uint256 amount, bytes32 tokenId, bytes memory data) external {
            bytes memory d = msg.data;
        }
    }
    "#;

    let metadata = analyse_source(source).expect("analysis failed");
    let ir_module = ir::Module::from_contract(&metadata).expect("IR lowering failed");
    let payment_function = ir_module
        .functions
        .iter()
        .find(|function| function.name == "onNEP11Payment")
        .expect("onNEP11Payment function");

    let instrs: Vec<_> = payment_function
        .basic_blocks
        .iter()
        .flat_map(|block| block.instructions.iter())
        .collect();

    // onNEP11Payment has data at param index 3 (after from, amount, tokenId)
    assert!(
        instrs
            .iter()
            .any(|instr| matches!(instr, ir::Instruction::LoadParameter(3))),
        "expected msg.data in onNEP11Payment to load parameter 3 (data), got: {:?}",
        instrs
            .iter()
            .filter(|i| matches!(i, ir::Instruction::LoadParameter(_)))
            .collect::<Vec<_>>()
    );
}

#[test]
fn abi_encode_with_signature_lowers_to_selector_prefix_plus_encoded_args() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract EncodeSigHarness {
        function payload() public pure returns (bytes memory) {
            return abi.encodeWithSignature("foo(uint256)", 7);
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 0).expect("compile");
    let artifact = artifacts
        .iter()
        .find(|a| a.metadata.name == "EncodeSigHarness")
        .expect("contract artifact");

    let ir_module = ir::Module::from_contract(&artifact.metadata).expect("build IR");
    let payload_function = ir_module
        .functions
        .iter()
        .find(|function| function.name == "payload")
        .expect("payload function");

    let mut hasher = Keccak256::new();
    hasher.update("foo(uint256)".as_bytes());
    let digest = hasher.finalize();
    let expected_selector = digest[..4].to_vec();

    let instrs: Vec<_> = payload_function
        .basic_blocks
        .iter()
        .flat_map(|block| block.instructions.iter())
        .collect();

    assert!(
        instrs.iter().any(|instr| matches!(
            instr,
            ir::Instruction::PushLiteral(ir::LiteralValue::ByteArray(bytes)) if *bytes == expected_selector
        )),
        "expected abi.encodeWithSignature to push the selector prefix"
    );
    assert!(
        !instrs.iter().any(|instr| matches!(
            instr,
            ir::Instruction::CallBuiltin {
                builtin: ir::BuiltinCall::AbiEncode,
                ..
            }
        )),
        "static abi.encodeWithSignature payload should lower without pseudo-native AbiEncode"
    );
    assert!(
        instrs.iter().any(|instr| matches!(
            instr,
            ir::Instruction::CallBuiltin {
                builtin: ir::BuiltinCall::BytesConcat,
                arg_count: 2,
            }
        )),
        "expected abi.encodeWithSignature to concatenate selector and encoded args"
    );
}

#[test]
fn abi_encode_with_selector_lowers_to_selector_prefix_plus_encoded_args() {
    let source = r#"
    pragma solidity ^0.8.19;

    interface IFoo {
        function bar(uint256 x) external returns (bool);
    }

    contract EncodeSelectorHarness {
        function payload() public pure returns (bytes memory) {
            return abi.encodeWithSelector(IFoo.bar.selector, 7);
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 0).expect("compile");
    let artifact = artifacts
        .iter()
        .find(|a| a.metadata.name == "EncodeSelectorHarness")
        .expect("contract artifact");

    let ir_module = ir::Module::from_contract(&artifact.metadata).expect("build IR");
    let payload_function = ir_module
        .functions
        .iter()
        .find(|function| function.name == "payload")
        .expect("payload function");

    let mut hasher = Keccak256::new();
    hasher.update("bar(uint256)".as_bytes());
    let digest = hasher.finalize();
    let expected_selector = digest[..4].to_vec();

    let instrs: Vec<_> = payload_function
        .basic_blocks
        .iter()
        .flat_map(|block| block.instructions.iter())
        .collect();

    assert!(
        instrs.iter().any(|instr| matches!(
            instr,
            ir::Instruction::PushLiteral(ir::LiteralValue::ByteArray(bytes)) if *bytes == expected_selector
        )),
        "expected abi.encodeWithSelector to push the selector prefix"
    );
    assert!(
        !instrs.iter().any(|instr| matches!(
            instr,
            ir::Instruction::CallBuiltin {
                builtin: ir::BuiltinCall::AbiEncode,
                ..
            }
        )),
        "static abi.encodeWithSelector payload should lower without pseudo-native AbiEncode"
    );
    assert!(
        instrs.iter().any(|instr| matches!(
            instr,
            ir::Instruction::CallBuiltin {
                builtin: ir::BuiltinCall::BytesConcat,
                arg_count: 2,
            }
        )),
        "expected abi.encodeWithSelector to concatenate selector and encoded args"
    );
}
