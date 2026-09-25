// Low-level call tests
//
// Tests for:
// - abi.encodeWithSelector for low-level calls
// - Low-level call/staticcall lowering
// - External member calls with read-only flags
// - Syscalls.contractCall in view functions

#[test]
fn encode_with_selector_recovers_method_name_for_low_level_calls() {
    let source = r#"
    pragma solidity ^0.8.20;

    interface IFoo {
        function bar(uint256 x) external returns (bool);
    }

    contract CallHarness {
        function run(address target) public {
            target.call(abi.encodeWithSelector(IFoo.bar.selector, 1));
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 0).expect("compile");
    let artifact = artifacts
        .iter()
        .find(|a| a.metadata.name == "CallHarness")
        .expect("contract artifact");

    let ir_module = ir::Module::from_contract(&artifact.metadata).expect("build IR");
    let run_function = ir_module
        .functions
        .iter()
        .find(|function| function.name == "run")
        .expect("run function");

    let instrs: Vec<_> = run_function
        .basic_blocks
        .iter()
        .flat_map(|block| block.instructions.iter())
        .collect();

    assert!(
        instrs.iter().any(|instr| matches!(
            instr,
            ir::Instruction::PushLiteral(ir::LiteralValue::String(bytes)) if bytes == b"bar"
        )),
        "expected low-level call lowering to recover method name from `.selector`"
    );
}

#[test]
fn low_level_call_lowers_to_contract_call_and_returns_tuple() {
    let source = r#"
    pragma solidity ^0.8.20;

    interface IFoo {
        function bar(uint256 x) external returns (bool);
    }

    contract CallHarness {
        function run(address target) public returns (bool ok) {
            (bool success, bytes memory data) = target.call(abi.encodeWithSelector(IFoo.bar.selector, 1));
            data;
            return success;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 0).expect("compile");
    let artifact = artifacts
        .iter()
        .find(|a| a.metadata.name == "CallHarness")
        .expect("contract artifact");

    let ir_module = ir::Module::from_contract(&artifact.metadata).expect("build IR");
    let run_function = ir_module
        .functions
        .iter()
        .find(|function| function.name == "run")
        .expect("run function");

    let instrs: Vec<_> = run_function
        .basic_blocks
        .iter()
        .flat_map(|block| block.instructions.iter())
        .collect();

    assert!(
        instrs.iter().any(|instr| matches!(
            instr,
            ir::Instruction::CallBuiltin {
                builtin: ir::BuiltinCall::ContractCall,
                ..
            }
        )),
        "expected low-level call lowering to use the ContractCall builtin"
    );
    assert!(
        instrs.iter().any(|instr| matches!(
            instr,
            ir::Instruction::PushLiteral(ir::LiteralValue::Boolean(true))
        )),
        "expected low-level call lowering to set success=true on the happy path"
    );
}

#[test]
fn low_level_staticcall_uses_contract_call_with_flags() {
    let source = r#"
    pragma solidity ^0.8.20;

    contract StaticHarness {
        function run(address target) public view returns (bool ok) {
            (bool success, bytes memory data) = target.staticcall(abi.encodeWithSignature("foo()"));
            data;
            return success;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 0).expect("compile");
    let artifact = artifacts
        .iter()
        .find(|a| a.metadata.name == "StaticHarness")
        .expect("contract artifact");

    let ir_module = ir::Module::from_contract(&artifact.metadata).expect("build IR");
    let run_function = ir_module
        .functions
        .iter()
        .find(|function| function.name == "run")
        .expect("run function");

    let instrs: Vec<_> = run_function
        .basic_blocks
        .iter()
        .flat_map(|block| block.instructions.iter())
        .collect();

    assert!(
        instrs.iter().any(|instr| matches!(
            instr,
            ir::Instruction::CallBuiltin {
                builtin: ir::BuiltinCall::ContractCallWithFlags,
                arg_count: 4
            }
        )),
        "expected `staticcall` lowering to use ContractCallWithFlags"
    );

    let read_only_flags = BigInt::from(0x05u8);
    assert!(
        instrs.iter().any(|instr| matches!(
            instr,
            ir::Instruction::PushLiteral(ir::LiteralValue::Integer(value)) if value == &read_only_flags
        )),
        "expected `staticcall` lowering to push CallFlags.ReadOnly (0x05)"
    );
}

#[test]
fn low_level_call_supports_intermediate_call_data_variable() {
    let source = r#"
    pragma solidity ^0.8.20;

    contract CallDataHarness {
        function run(address target) public returns (bool ok) {
            bytes memory data = abi.encodeWithSignature("foo(uint256)", 7);
            (bool success, bytes memory out) = target.call(data);
            out;
            return success;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 0).expect("compile");
    let artifact = artifacts
        .iter()
        .find(|a| a.metadata.name == "CallDataHarness")
        .expect("contract artifact");

    let ir_module = ir::Module::from_contract(&artifact.metadata).expect("build IR");
    let run_function = ir_module
        .functions
        .iter()
        .find(|function| function.name == "run")
        .expect("run function");

    let instrs: Vec<_> = run_function
        .basic_blocks
        .iter()
        .flat_map(|block| block.instructions.iter())
        .collect();

    assert!(
        instrs.iter().any(|instr| matches!(
            instr,
            ir::Instruction::CallBuiltin {
                builtin: ir::BuiltinCall::ContractCall,
                ..
            }
        )),
        "expected low-level call lowering to use the ContractCall builtin"
    );
}

#[test]
fn low_level_call_supports_constant_signature_string() {
    let source = r#"
    pragma solidity ^0.8.20;

    contract ConstSigHarness {
        string constant SIG = "foo(uint256)";

        function run(address target) public returns (bool ok) {
            bytes memory data = abi.encodeWithSignature(SIG, 7);
            (bool success, bytes memory out) = target.call(data);
            out;
            return success;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 0).expect("compile");
    let artifact = artifacts
        .iter()
        .find(|a| a.metadata.name == "ConstSigHarness")
        .expect("contract artifact");

    let ir_module = ir::Module::from_contract(&artifact.metadata).expect("build IR");
    let run_function = ir_module
        .functions
        .iter()
        .find(|function| function.name == "run")
        .expect("run function");

    let instrs: Vec<_> = run_function
        .basic_blocks
        .iter()
        .flat_map(|block| block.instructions.iter())
        .collect();

    assert!(
        instrs.iter().any(|instr| matches!(
            instr,
            ir::Instruction::PushLiteral(ir::LiteralValue::String(bytes)) if bytes == b"foo"
        )),
        "expected const signature to resolve to method name 'foo'"
    );

    assert!(
        instrs.iter().any(|instr| matches!(
            instr,
            ir::Instruction::CallBuiltin {
                builtin: ir::BuiltinCall::ContractCall,
                ..
            }
        )),
        "expected low-level call lowering to use the ContractCall builtin"
    );
}

#[test]
fn external_member_calls_use_read_only_flags_in_view_functions() {
    let source = r#"
    pragma solidity ^0.8.20;

    interface IFoo {
        function bar(uint256 x) external view returns (uint256);
    }

    contract ExternalViewHarness {
        function run(address target) public view returns (uint256) {
            return IFoo(target).bar(1);
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 0).expect("compile");
    let artifact = artifacts
        .iter()
        .find(|a| a.metadata.name == "ExternalViewHarness")
        .expect("contract artifact");

    let ir_module = ir::Module::from_contract(&artifact.metadata).expect("build IR");
    let run_function = ir_module
        .functions
        .iter()
        .find(|function| function.name == "run")
        .expect("run function");

    let instrs: Vec<_> = run_function
        .basic_blocks
        .iter()
        .flat_map(|block| block.instructions.iter())
        .collect();

    assert!(
        instrs.iter().any(|instr| matches!(
            instr,
            ir::Instruction::CallBuiltin {
                builtin: ir::BuiltinCall::Syscall(name),
                arg_count: 4
            } if name == "System.Contract.Call"
        )),
        "expected external member call lowering to use the System.Contract.Call syscall"
    );

    let read_only_flags = BigInt::from(0x05u8);
    assert!(
        instrs.iter().any(|instr| matches!(
            instr,
            ir::Instruction::PushLiteral(ir::LiteralValue::Integer(value)) if value == &read_only_flags
        )),
        "expected external calls from view functions to push CallFlags.ReadOnly (0x05)"
    );
}

#[test]
fn syscalls_contract_call_defaults_to_read_only_in_view_functions() {
    let source = r#"
    pragma solidity ^0.8.20;

    library Syscalls {
        function contractCall(address scriptHash, string memory method, bytes memory params) internal returns (bytes memory) {
            scriptHash;
            method;
            params;
            return "";
        }
    }

    contract SyscallViewHarness {
        function run(address target) public view returns (bytes memory) {
            return Syscalls.contractCall(target, "foo", abi.encode(uint256(1)));
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 0).expect("compile");
    let artifact = artifacts
        .iter()
        .find(|a| a.metadata.name == "SyscallViewHarness")
        .expect("contract artifact");

    let ir_module = ir::Module::from_contract(&artifact.metadata).expect("build IR");
    let run_function = ir_module
        .functions
        .iter()
        .find(|function| function.name == "run")
        .expect("run function");

    let instrs: Vec<_> = run_function
        .basic_blocks
        .iter()
        .flat_map(|block| block.instructions.iter())
        .collect();

    assert!(
        instrs.iter().any(|instr| matches!(
            instr,
            ir::Instruction::CallBuiltin {
                builtin: ir::BuiltinCall::ContractCallWithFlags,
                arg_count: 4
            }
        )),
        "expected Syscalls.contractCall to lower to ContractCallWithFlags in view context"
    );

    let read_only_flags = BigInt::from(0x05u8);
    assert!(
        instrs.iter().any(|instr| matches!(
            instr,
            ir::Instruction::PushLiteral(ir::LiteralValue::Integer(value)) if value == &read_only_flags
        )),
        "expected view-context Syscalls.contractCall lowering to push CallFlags.ReadOnly (0x05)"
    );
}

#[test]
fn low_level_call_supports_bytes_wrapped_inline_call_data() {
    let source = r#"
    pragma solidity ^0.8.20;

    contract WrappedInlineHarness {
        function run(address target) public returns (bool ok) {
            (bool success, bytes memory out) = target.call(bytes(abi.encodeWithSignature("foo(uint256)", 7)));
            out;
            return success;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 0).expect("compile");
    let artifact = artifacts
        .iter()
        .find(|a| a.metadata.name == "WrappedInlineHarness")
        .expect("contract artifact");

    let ir_module = ir::Module::from_contract(&artifact.metadata).expect("build IR");
    let run_function = ir_module
        .functions
        .iter()
        .find(|function| function.name == "run")
        .expect("run function");

    let instrs: Vec<_> = run_function
        .basic_blocks
        .iter()
        .flat_map(|block| block.instructions.iter())
        .collect();

    assert!(
        instrs.iter().any(|instr| matches!(
            instr,
            ir::Instruction::CallBuiltin {
                builtin: ir::BuiltinCall::ContractCall,
                ..
            }
        )),
        "expected low-level call lowering to use ContractCall for bytes-wrapped inline payload"
    );
}

#[test]
fn low_level_call_supports_bytes_wrapped_call_data_variable() {
    let source = r#"
    pragma solidity ^0.8.20;

    contract WrappedVariableHarness {
        function run(address target) public returns (bool ok) {
            bytes memory data = bytes(abi.encodeWithSignature("foo(uint256)", 7));
            (bool success, bytes memory out) = target.call(bytes(data));
            out;
            return success;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 0).expect("compile");
    let artifact = artifacts
        .iter()
        .find(|a| a.metadata.name == "WrappedVariableHarness")
        .expect("contract artifact");

    let ir_module = ir::Module::from_contract(&artifact.metadata).expect("build IR");
    let run_function = ir_module
        .functions
        .iter()
        .find(|function| function.name == "run")
        .expect("run function");

    let instrs: Vec<_> = run_function
        .basic_blocks
        .iter()
        .flat_map(|block| block.instructions.iter())
        .collect();

    assert!(
        instrs.iter().any(|instr| matches!(
            instr,
            ir::Instruction::CallBuiltin {
                builtin: ir::BuiltinCall::ContractCall,
                ..
            }
        )),
        "expected low-level call lowering to use ContractCall for bytes-wrapped variable payload"
    );
}

#[test]
fn low_level_call_supports_encode_call_inline_payload() {
    let source = r#"
    pragma solidity ^0.8.20;

    interface IFoo {
        function bar(uint256 x) external returns (bool);
    }

    contract EncodeCallInlineHarness {
        function run(address target) public returns (bool ok) {
            (bool success, bytes memory out) = target.call(abi.encodeCall(IFoo.bar, (7)));
            out;
            return success;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 0).expect("compile");
    let artifact = artifacts
        .iter()
        .find(|a| a.metadata.name == "EncodeCallInlineHarness")
        .expect("contract artifact");

    let ir_module = ir::Module::from_contract(&artifact.metadata).expect("build IR");
    let run_function = ir_module
        .functions
        .iter()
        .find(|function| function.name == "run")
        .expect("run function");

    let instrs: Vec<_> = run_function
        .basic_blocks
        .iter()
        .flat_map(|block| block.instructions.iter())
        .collect();

    assert!(
        instrs.iter().any(|instr| matches!(
            instr,
            ir::Instruction::PushLiteral(ir::LiteralValue::String(bytes)) if bytes == b"bar"
        )),
        "expected encodeCall lowering to recover method name 'bar'"
    );

    assert!(
        instrs.iter().any(|instr| matches!(
            instr,
            ir::Instruction::CallBuiltin {
                builtin: ir::BuiltinCall::ContractCall,
                ..
            }
        )),
        "expected low-level encodeCall lowering to use ContractCall"
    );
}

#[test]
fn low_level_call_supports_encode_call_variable_payload() {
    let source = r#"
    pragma solidity ^0.8.20;

    interface IFoo {
        function bar(uint256 x) external returns (bool);
    }

    contract EncodeCallVariableHarness {
        function run(address target) public returns (bool ok) {
            bytes memory data = abi.encodeCall(IFoo.bar, (7));
            (bool success, bytes memory out) = target.call(data);
            out;
            return success;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 0).expect("compile");
    let artifact = artifacts
        .iter()
        .find(|a| a.metadata.name == "EncodeCallVariableHarness")
        .expect("contract artifact");

    let ir_module = ir::Module::from_contract(&artifact.metadata).expect("build IR");
    let run_function = ir_module
        .functions
        .iter()
        .find(|function| function.name == "run")
        .expect("run function");

    let instrs: Vec<_> = run_function
        .basic_blocks
        .iter()
        .flat_map(|block| block.instructions.iter())
        .collect();

    assert!(
        instrs.iter().any(|instr| matches!(
            instr,
            ir::Instruction::PushLiteral(ir::LiteralValue::String(bytes)) if bytes == b"bar"
        )),
        "expected encodeCall variable lowering to recover method name 'bar'"
    );

    assert!(
        instrs.iter().any(|instr| matches!(
            instr,
            ir::Instruction::CallBuiltin {
                builtin: ir::BuiltinCall::ContractCall,
                ..
            }
        )),
        "expected low-level encodeCall variable lowering to use ContractCall"
    );
}

#[test]
fn low_level_call_rejects_encode_call_non_function_reference() {
    let source = r#"
    pragma solidity ^0.8.20;

    contract CallHarness {
        struct S {
            uint256 x;
        }

        function run(address target) public returns (bool ok, bytes memory out) {
            S memory s = S({ x: 1 });
            (ok, out) = target.call(abi.encodeCall(s.x, (7)));
        }
    }
    "#;

    let result = compile_contracts(source, false, 2);
    assert!(
        result.is_ok(),
        "abi.encodeCall with struct field should compile"
    );
}

#[test]
fn low_level_call_encode_with_signature_trims_whitespace_around_name() {
    let source = r#"
    pragma solidity ^0.8.20;

    contract SignatureTrimHarness {
        function run(address target) public returns (bool ok) {
            (bool success, bytes memory out) = target.call(
                abi.encodeWithSignature("   foo   (uint256)", 7)
            );
            out;
            return success;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 0).expect("compile");
    let artifact = artifacts
        .iter()
        .find(|a| a.metadata.name == "SignatureTrimHarness")
        .expect("contract artifact");

    let ir_module = ir::Module::from_contract(&artifact.metadata).expect("build IR");
    let run_function = ir_module
        .functions
        .iter()
        .find(|function| function.name == "run")
        .expect("run function");

    let instrs: Vec<_> = run_function
        .basic_blocks
        .iter()
        .flat_map(|block| block.instructions.iter())
        .collect();

    assert!(
        instrs.iter().any(|instr| matches!(
            instr,
            ir::Instruction::PushLiteral(ir::LiteralValue::String(bytes)) if bytes == b"foo"
        )),
        "expected signature with extra whitespace to normalize to method name 'foo'"
    );
}

#[test]
fn low_level_call_rejects_signature_without_function_name() {
    let source = r#"
    pragma solidity ^0.8.20;

    contract BadSignatureHarness {
        function run(address target) public returns (bool ok) {
            (bool success, bytes memory out) = target.call(
                abi.encodeWithSignature("(uint256)", 7)
            );
            out;
            return success;
        }
    }
    "#;

    let err = compile_contracts(source, false, 2).expect_err("expected invalid signature failure");
    match err {
        CompileError::Ir(diags) => {
            assert!(
                diags
                    .iter()
                    .any(|diag| diag.message.contains("must include a function name")),
                "expected invalid signature diagnostic, got: {diags:?}"
            );
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn opaque_dynamic_low_level_call_payload_fails_instead_of_fake_success() {
    // v0.19.0 changed the contract: opaque payloads are a compile-time
    // warning + a runtime ABORTMSG, not a hard compile error. The test
    // name is preserved for historical continuity but now pins the new
    // shape — the warning surfaces the migration path, and the bytecode
    // contains ABORTMSG (0xE0) so any execution that reaches the opaque
    // call site reverts cleanly.
    let source = r#"
    pragma solidity ^0.8.20;

    contract OpaquePayloadHarness {
        function run(address target, bytes memory data) public returns (bool ok, bytes memory out) {
            (ok, out) = target.call(data);
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2)
        .expect("opaque payload should compile with warning + runtime trap (v0.19.0)");
    let warnings: Vec<String> = artifacts
        .iter()
        .flat_map(|a| a.warnings.iter().map(|w| w.message.clone()))
        .collect();
    let combined = warnings.join("\n").to_lowercase();
    assert!(
        combined.contains("opaque") && combined.contains("runtime trap"),
        "expected opaque-call warning surface; got warnings: {warnings:?}"
    );
    assert!(
        artifacts.iter().any(|a| a.bytecode.contains(&0xE0)),
        "expected ABORTMSG (0xE0) at the opaque-call site"
    );
}

#[test]
fn unsupported_evm_precompile_range_is_rejected_explicitly() {
    for index in 0x06u8..=0x09 {
        let source = format!(
            r#"
            pragma solidity ^0.8.20;

            contract UnsupportedPrecompileHarness {{
                function run(bytes memory data) public view returns (bool ok, bytes memory out) {{
                    (ok, out) = address(0x{index:02x}).staticcall(data);
                }}
            }}
            "#
        );

        let err =
            compile_contracts(&source, false, 2).expect_err("unsupported EVM precompile must fail");
        match err {
            CompileError::Ir(diags) => {
                assert!(
                    diags.iter().any(|diag| {
                        diag.message.contains("unsupported EVM precompile")
                            && diag.message.contains(&format!("0x{index:02x}"))
                    }),
                    "expected unsupported precompile diagnostic for 0x{index:02x}, got: {diags:?}"
                );
            }
            other => panic!("unexpected error variant for 0x{index:02x}: {other:?}"),
        }
    }
}
