#[test]
fn neo_hash_helpers_lower_to_cryptolib_native_calls() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract NeoHashHarness {
        function sha(bytes memory data) public view returns (bytes32) {
            return Neo.sha256Hash(data);
        }

        function ripemd(bytes memory data) public view returns (bytes20) {
            return Neo.ripemd160Hash(data);
        }
    }
    "#;

    let metadata = analyse_source(source).expect("analysis failed");
    let module = ir::Module::from_contract(&metadata).expect("IR lowering failed");

    let sha_fn = module
        .functions
        .iter()
        .find(|function| function.name == "sha")
        .expect("expected sha function");
    assert!(
        sha_fn.basic_blocks.iter().any(|block| {
            block.instructions.iter().any(|instr| {
                matches!(
                    instr,
                    ir::Instruction::CallBuiltin {
                        builtin: ir::BuiltinCall::NativeCall { contract, method },
                        arg_count: 1,
                    } if *contract == ir::NativeContract::CryptoLib && method == "sha256"
                )
            })
        }),
        "expected Neo.sha256Hash to lower to CryptoLib.sha256 native call"
    );

    let ripemd_fn = module
        .functions
        .iter()
        .find(|function| function.name == "ripemd")
        .expect("expected ripemd function");
    assert!(
        ripemd_fn.basic_blocks.iter().any(|block| {
            block.instructions.iter().any(|instr| {
                matches!(
                    instr,
                    ir::Instruction::CallBuiltin {
                        builtin: ir::BuiltinCall::NativeCall { contract, method },
                        arg_count: 1,
                    } if *contract == ir::NativeContract::CryptoLib && method == "ripemd160"
                )
            })
        }),
        "expected Neo.ripemd160Hash to lower to CryptoLib.ripemd160 native call"
    );

    assert!(
        module.functions.iter().all(|function| {
            function.basic_blocks.iter().all(|block| {
                block.instructions.iter().all(|instr| {
                    !matches!(
                        instr,
                        ir::Instruction::CallBuiltin {
                            builtin: ir::BuiltinCall::Syscall(name),
                            ..
                        } if name == "System.Crypto.SHA256"
                            || name == "System.Crypto.RIPEMD160"
                    )
                })
            })
        }),
        "Neo hash helpers must not lower to nonexistent System.Crypto hash syscalls"
    );
}
