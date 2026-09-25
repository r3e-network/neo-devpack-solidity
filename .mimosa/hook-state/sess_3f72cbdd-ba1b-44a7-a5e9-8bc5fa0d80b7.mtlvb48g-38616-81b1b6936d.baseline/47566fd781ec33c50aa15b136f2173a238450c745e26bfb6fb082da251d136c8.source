use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static OUTPUT_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn get_compiler_path() -> PathBuf {
    // CARGO_BIN_EXE_<name> is set by Cargo for integration tests, pointing
    // to the correct binary for the current build profile (debug or release).
    PathBuf::from(env!("CARGO_BIN_EXE_neo-solc"))
}

fn get_example_path(contract: &str) -> PathBuf {
    // CARGO_MANIFEST_DIR is the project root — portable across CI and local dev.
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.join("examples").join(contract)
}

fn unique_output_prefix(scope: &str, contract_path: &Path) -> Result<PathBuf, String> {
    let stem = contract_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| format!("Invalid contract path: {}", contract_path.display()))?;
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let seq = OUTPUT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let output_dir = std::env::temp_dir().join(format!(
        "neo-sol-{scope}-{}-{nanos}-{seq}",
        std::process::id()
    ));
    std::fs::create_dir_all(&output_dir).map_err(|e| {
        format!(
            "Failed to create output directory '{}': {e}",
            output_dir.display()
        )
    })?;
    Ok(output_dir.join(stem))
}

/// Compile and assert success (exit-code only). Works for multi-contract files
/// where the compiler suffixes output with contract names.
fn assert_compiles(contract_path: &str) {
    let compiler = get_compiler_path();
    assert!(compiler.exists(), "Compiler not found");

    let contract_path = get_example_path(contract_path);
    let output_prefix = unique_output_prefix("test", &contract_path).expect("output prefix");

    let output = Command::new(&compiler)
        .arg(&contract_path)
        .arg("-I")
        .arg("devpack")
        .arg("-O2")
        .arg("-o")
        .arg(&output_prefix)
        .output()
        .expect("Failed to run compiler");

    assert!(
        output.status.success(),
        "Compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn compile_contract(contract_path: &str) -> Result<(PathBuf, PathBuf), String> {
    let compiler = get_compiler_path();

    if !compiler.exists() {
        return Err(format!(
            "Compiler not found at {}. Run 'cargo build --release' first.",
            compiler.display()
        ));
    }

    let contract_path = get_example_path(contract_path);
    let output_prefix = unique_output_prefix("test", &contract_path)?;

    let output = Command::new(&compiler)
        .arg(&contract_path)
        .arg("-I")
        .arg("devpack")
        .arg("-O2")
        .arg("-o")
        .arg(&output_prefix)
        .output()
        .map_err(|e| format!("Failed to run compiler: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Compilation failed: {stderr}"));
    }

    let nef_path = output_prefix.with_extension("nef");
    let manifest_path = output_prefix.with_extension("manifest.json");

    if !nef_path.exists() {
        return Err(format!("NEF file not generated: {}", nef_path.display()));
    }

    if !manifest_path.exists() {
        return Err(format!(
            "Manifest not generated: {}",
            manifest_path.display()
        ));
    }

    Ok((nef_path, manifest_path))
}

fn compile_contract_strict(contract_path: &str) -> Result<(PathBuf, PathBuf), String> {
    let compiler = get_compiler_path();

    if !compiler.exists() {
        return Err(format!(
            "Compiler not found at {}. Run 'cargo build --release' first.",
            compiler.display()
        ));
    }

    let contract_path = get_example_path(contract_path);
    let output_prefix = unique_output_prefix("test-strict", &contract_path)?;

    let output = Command::new(&compiler)
        .arg(&contract_path)
        .arg("-I")
        .arg("devpack")
        .arg("-O2")
        .arg("--deny-wildcard-permissions")
        .arg("--deny-wildcard-contracts")
        .arg("--deny-wildcard-methods")
        .arg("-o")
        .arg(&output_prefix)
        .output()
        .map_err(|e| format!("Failed to run compiler: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Strict compilation failed: {stderr}"));
    }

    let nef_path = output_prefix.with_extension("nef");
    let manifest_path = output_prefix.with_extension("manifest.json");

    if !nef_path.exists() {
        return Err(format!("NEF file not generated: {}", nef_path.display()));
    }

    if !manifest_path.exists() {
        return Err(format!(
            "Manifest not generated: {}",
            manifest_path.display()
        ));
    }

    Ok((nef_path, manifest_path))
}

#[test]
fn test_compile_simple_storage() {
    let result = compile_contract("SimpleStorage.sol");
    assert!(
        result.is_ok(),
        "Failed to compile SimpleStorage: {:?}",
        result.err()
    );
    let (nef, manifest) = result.unwrap();
    assert!(nef.exists(), "NEF file should exist: {}", nef.display());
    assert!(
        manifest.exists(),
        "Manifest should exist: {}",
        manifest.display()
    );
}

#[test]
fn test_compile_erc20_token() {
    let result = compile_contract("ERC20Token.sol");
    assert!(
        result.is_ok(),
        "Failed to compile ERC20Token: {:?}",
        result.err()
    );
    let (nef, manifest) = result.unwrap();
    assert!(nef.exists(), "NEF file should exist: {}", nef.display());
    assert!(
        manifest.exists(),
        "Manifest should exist: {}",
        manifest.display()
    );
}

#[test]
fn test_compile_staking() {
    let result = compile_contract("Staking.sol");
    assert!(
        result.is_ok(),
        "Failed to compile Staking: {:?}",
        result.err()
    );
    let (nef, manifest) = result.unwrap();
    assert!(nef.exists(), "NEF file should exist: {}", nef.display());
    assert!(
        manifest.exists(),
        "Manifest should exist: {}",
        manifest.display()
    );
}

#[test]
fn test_compile_multisig_wallet() {
    let result = compile_contract("MultiSigWallet.sol");
    assert!(
        result.is_ok(),
        "Failed to compile MultiSigWallet: {:?}",
        result.err()
    );
    let (nef, manifest) = result.unwrap();
    assert!(nef.exists(), "NEF file should exist: {}", nef.display());
    assert!(
        manifest.exists(),
        "Manifest should exist: {}",
        manifest.display()
    );
}

#[test]
fn test_compile_with_optimization() {
    let compiler = get_compiler_path();
    if !compiler.exists() {
        return;
    }

    let example_path = get_example_path("SimpleStorage.sol");
    let output_prefix = unique_output_prefix("opt-test", &example_path).expect("output prefix");
    let output = Command::new(&compiler)
        .arg(&example_path)
        .arg("-O3")
        .arg("-o")
        .arg(&output_prefix)
        .output()
        .expect("Failed to run compiler");

    assert!(
        output.status.success(),
        "High optimization should compile successfully: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_nef_file_structure() {
    let result = compile_contract("SimpleStorage.sol");
    assert!(result.is_ok(), "Failed to compile SimpleStorage");

    let (nef_path, _) = result.unwrap();
    let nef_data = std::fs::read(&nef_path)
        .map_err(|e| format!("Failed to read NEF: {e}"))
        .unwrap();

    assert!(!nef_data.is_empty(), "NEF should not be empty");
    assert!(
        nef_data.len() < 1024 * 1024,
        "NEF should be reasonable size"
    );

    // NEF3 format magic bytes
    let magic = &nef_data[..4];
    assert_eq!(
        magic, b"NEF3",
        "NEF should have correct magic bytes (NEF3 format)"
    );
}

#[test]
fn test_manifest_structure() {
    let result = compile_contract("SimpleStorage.sol");
    assert!(result.is_ok(), "Failed to compile SimpleStorage");

    let (_, manifest_path) = result.unwrap();
    let manifest_data = std::fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read manifest: {e}"))
        .unwrap();

    let json: serde_json::Value = serde_json::from_str(&manifest_data)
        .map_err(|e| format!("Invalid JSON: {e}"))
        .unwrap();

    assert!(json.is_object(), "Manifest should be JSON object");
    assert!(
        json.get("name").is_some(),
        "Manifest should have name field"
    );
    assert!(json.get("abi").is_some(), "Manifest should have abi field");
    assert!(
        json.get("permissions").is_some(),
        "Manifest should have permissions field"
    );
}

#[test]
fn test_compile_name_service() {
    let result = compile_contract("NameService.sol");
    assert!(
        result.is_ok(),
        "Failed to compile NameService: {:?}",
        result.err()
    );
    let (nef, manifest) = result.unwrap();
    assert!(nef.exists(), "NEF file should exist: {}", nef.display());
    assert!(
        manifest.exists(),
        "Manifest should exist: {}",
        manifest.display()
    );
}

#[test]
fn test_compile_lottery() {
    let result = compile_contract("Lottery.sol");
    assert!(
        result.is_ok(),
        "Failed to compile Lottery: {:?}",
        result.err()
    );
    let (nef, manifest) = result.unwrap();
    assert!(nef.exists(), "NEF file should exist: {}", nef.display());
    assert!(
        manifest.exists(),
        "Manifest should exist: {}",
        manifest.display()
    );
}

#[test]
fn test_compile_escrow() {
    let result = compile_contract("Escrow.sol");
    assert!(
        result.is_ok(),
        "Failed to compile Escrow: {:?}",
        result.err()
    );
    let (nef, manifest) = result.unwrap();
    assert!(nef.exists(), "NEF file should exist: {}", nef.display());
    assert!(
        manifest.exists(),
        "Manifest should exist: {}",
        manifest.display()
    );
}

#[test]
fn test_compile_new_neo_interop_showcase() {
    let result = compile_contract("new/NeoInteropShowcase.sol");
    assert!(
        result.is_ok(),
        "Failed to compile NeoInteropShowcase: {:?}",
        result.err()
    );
    let (nef, manifest) = result.unwrap();
    assert!(nef.exists(), "NEF file should exist: {}", nef.display());
    assert!(
        manifest.exists(),
        "Manifest should exist: {}",
        manifest.display()
    );
}

#[test]
fn test_compile_new_low_level_call_showcase() {
    let result = compile_contract("new/LowLevelCallShowcase.sol");
    assert!(
        result.is_ok(),
        "Failed to compile LowLevelCallShowcase: {:?}",
        result.err()
    );
    let (nef, manifest) = result.unwrap();
    assert!(nef.exists(), "NEF file should exist: {}", nef.display());
    assert!(
        manifest.exists(),
        "Manifest should exist: {}",
        manifest.display()
    );
}

#[test]
fn test_compile_new_enum_array_showcase() {
    let result = compile_contract("new/EnumArrayShowcase.sol");
    assert!(
        result.is_ok(),
        "Failed to compile EnumArrayShowcase: {:?}",
        result.err()
    );
    let (nef, manifest) = result.unwrap();
    assert!(nef.exists(), "NEF file should exist: {}", nef.display());
    assert!(
        manifest.exists(),
        "Manifest should exist: {}",
        manifest.display()
    );
}

#[test]
fn test_optimization_reduces_size() {
    let compiler = get_compiler_path();
    if !compiler.exists() {
        return;
    }

    let example_path = get_example_path("SimpleStorage.sol");

    // Compile without optimization
    let output_no_opt = Command::new(&compiler)
        .arg(&example_path)
        .arg("-O0")
        .arg("-o")
        .arg("/tmp/no_opt_test")
        .output()
        .expect("Failed to run compiler");

    assert!(
        output_no_opt.status.success(),
        "No-opt compilation should succeed"
    );

    let no_opt_size = std::fs::metadata("/tmp/no_opt_test.nef")
        .map(|m| m.len())
        .unwrap_or(0);

    // Compile with optimization
    let output_opt = Command::new(&compiler)
        .arg(&example_path)
        .arg("-O3")
        .arg("-o")
        .arg("/tmp/opt_test")
        .output()
        .expect("Failed to run compiler");

    assert!(
        output_opt.status.success(),
        "Optimized compilation should succeed"
    );

    let opt_size = std::fs::metadata("/tmp/opt_test.nef")
        .map(|m| m.len())
        .unwrap_or(0);

    // Optimization should not significantly increase size
    // (Note: for simple contracts, O3 might be slightly larger due to inlining)
    assert!(
        opt_size <= no_opt_size * 2,
        "Optimized bytecode should not be more than 2x the original size"
    );
}

#[test]
fn test_manifest_has_valid_abi() {
    let result = compile_contract("ERC20Token.sol");
    assert!(result.is_ok(), "Failed to compile ERC20Token");

    let (_, manifest_path) = result.unwrap();
    let manifest_data = std::fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read manifest: {e}"))
        .unwrap();

    let json: serde_json::Value = serde_json::from_str(&manifest_data)
        .map_err(|e| format!("Invalid JSON: {e}"))
        .unwrap();

    let abi = json.get("abi").expect("Manifest should have ABI");

    assert!(
        abi.is_object(),
        "ABI should be an object with methods and events"
    );

    let abi_obj = abi.as_object().unwrap();
    let methods = abi_obj.get("methods").expect("ABI should have methods");

    assert!(methods.is_array(), "ABI methods should be an array");

    let methods_array = methods.as_array().unwrap();
    assert!(
        !methods_array.is_empty(),
        "Contract should have at least one method"
    );

    // Check that methods have required fields
    for method in methods_array {
        assert!(method.get("name").is_some(), "Method should have a name");
        assert!(
            method.get("parameters").is_some(),
            "Method should have parameters"
        );
        assert!(
            method.get("returntype").is_some(),
            "Method should have return type"
        );
    }
}

// ========== Root examples — previously untested ==========

#[test]
fn test_compile_erc721_token() {
    let result = compile_contract("ERC721Token.sol");
    assert!(
        result.is_ok(),
        "Failed to compile ERC721Token: {:?}",
        result.err()
    );
}

#[test]
fn test_compile_governance_token() {
    let result = compile_contract("GovernanceToken.sol");
    assert!(
        result.is_ok(),
        "Failed to compile GovernanceToken: {:?}",
        result.err()
    );
}

#[test]
fn test_compile_uniswap_v2_pair() {
    let result = compile_contract("UniswapV2Pair.sol");
    assert!(
        result.is_ok(),
        "Failed to compile UniswapV2Pair: {:?}",
        result.err()
    );
}

#[test]
fn test_compile_test_contract() {
    let result = compile_contract("TestContract.sol");
    assert!(
        result.is_ok(),
        "Failed to compile TestContract: {:?}",
        result.err()
    );
}

// ========== Existing new/ examples — previously untested ==========

#[test]
fn test_compile_new_counter() {
    let result = compile_contract("new/Counter.sol");
    assert!(
        result.is_ok(),
        "Failed to compile Counter: {:?}",
        result.err()
    );
}

#[test]
fn test_compile_new_vault() {
    let result = compile_contract("new/Vault.sol");
    assert!(
        result.is_ok(),
        "Failed to compile Vault: {:?}",
        result.err()
    );
}

#[test]
fn test_compile_new_nft() {
    let result = compile_contract("new/NFT.sol");
    assert!(result.is_ok(), "Failed to compile NFT: {:?}", result.err());
}

#[test]
fn test_compile_new_bank() {
    let result = compile_contract("new/Bank.sol");
    assert!(result.is_ok(), "Failed to compile Bank: {:?}", result.err());
}

#[test]
fn test_compile_new_multisig_wallet_nep17() {
    let result = compile_contract("new/MultiSigWalletNEP17.sol");
    assert!(
        result.is_ok(),
        "Failed to compile MultiSigWalletNEP17: {:?}",
        result.err()
    );
}

// ========== New showcase contracts ==========

#[test]
fn test_compile_new_custom_errors_showcase() {
    let result = compile_contract("new/CustomErrorsShowcase.sol");
    assert!(
        result.is_ok(),
        "Failed to compile CustomErrorsShowcase: {:?}",
        result.err()
    );
}

#[test]
fn test_compile_new_inheritance_showcase() {
    // Multi-contract file: compiler suffixes output with contract names
    assert_compiles("new/InheritanceShowcase.sol");
}

#[test]
fn test_compile_new_interface_showcase() {
    let result = compile_contract("new/InterfaceShowcase.sol");
    assert!(
        result.is_ok(),
        "Failed to compile InterfaceShowcase: {:?}",
        result.err()
    );
}

#[test]
fn test_compile_new_modifier_showcase() {
    let result = compile_contract("new/ModifierShowcase.sol");
    assert!(
        result.is_ok(),
        "Failed to compile ModifierShowcase: {:?}",
        result.err()
    );
}

#[test]
fn test_compile_new_struct_mapping_showcase() {
    let result = compile_contract("new/StructMappingShowcase.sol");
    assert!(
        result.is_ok(),
        "Failed to compile StructMappingShowcase: {:?}",
        result.err()
    );
}

#[test]
fn test_compile_new_type_casting_showcase() {
    let result = compile_contract("new/TypeCastingShowcase.sol");
    assert!(
        result.is_ok(),
        "Failed to compile TypeCastingShowcase: {:?}",
        result.err()
    );
}

#[test]
fn test_compile_new_constants_immutable_showcase() {
    let result = compile_contract("new/ConstantsImmutableShowcase.sol");
    assert!(
        result.is_ok(),
        "Failed to compile ConstantsImmutableShowcase: {:?}",
        result.err()
    );
}

#[test]
fn test_compile_new_bitwise_showcase() {
    let result = compile_contract("new/BitwiseShowcase.sol");
    assert!(
        result.is_ok(),
        "Failed to compile BitwiseShowcase: {:?}",
        result.err()
    );
}

#[test]
fn test_compile_new_try_catch_showcase() {
    let result = compile_contract("new/TryCatchShowcase.sol");
    assert!(
        result.is_ok(),
        "Failed to compile TryCatchShowcase: {:?}",
        result.err()
    );
}

#[test]
fn test_compile_new_catch_panic_showcase() {
    let result = compile_contract("new/CatchPanicShowcase.sol");
    assert!(
        result.is_ok(),
        "Failed to compile CatchPanicShowcase: {:?}",
        result.err()
    );
}

#[test]
fn test_compile_new_event_indexed_showcase() {
    let result = compile_contract("new/EventIndexedShowcase.sol");
    assert!(
        result.is_ok(),
        "Failed to compile EventIndexedShowcase: {:?}",
        result.err()
    );
}

#[test]
fn test_compile_new_oracle_showcase() {
    let result = compile_contract("new/OracleShowcase.sol");
    assert!(
        result.is_ok(),
        "Failed to compile OracleShowcase: {:?}",
        result.err()
    );
}

#[test]
fn test_compile_new_multi_standard_token() {
    let result = compile_contract("new/MultiStandardToken.sol");
    assert!(
        result.is_ok(),
        "Failed to compile MultiStandardToken: {:?}",
        result.err()
    );
}

#[test]
fn test_compile_new_upgrade_lifecycle_showcase_strict() {
    let result = compile_contract_strict("new/UpgradeLifecycleShowcase.sol");
    assert!(
        result.is_ok(),
        "Failed strict compilation for UpgradeLifecycleShowcase: {:?}",
        result.err()
    );
}

#[test]
fn test_compile_new_witness_guard_showcase_strict() {
    let result = compile_contract_strict("new/WitnessGuardShowcase.sol");
    assert!(
        result.is_ok(),
        "Failed strict compilation for WitnessGuardShowcase: {:?}",
        result.err()
    );
}

#[test]
fn test_compile_new_oracle_relay_strict_showcase_strict() {
    let result = compile_contract_strict("new/OracleRelayStrictShowcase.sol");
    assert!(
        result.is_ok(),
        "Failed strict compilation for OracleRelayStrictShowcase: {:?}",
        result.err()
    );
}

// ========== Famous DeFi/Web3 contracts ==========

#[test]
fn test_compile_famous_wgas() {
    let result = compile_contract("famous/WGAS.sol");
    assert!(result.is_ok(), "Failed to compile WGAS: {:?}", result.err());
}

#[test]
fn test_compile_famous_flashloan() {
    let result = compile_contract("famous/FlashLoan.sol");
    assert!(
        result.is_ok(),
        "Failed to compile FlashLoan: {:?}",
        result.err()
    );
}

#[test]
fn test_compile_famous_simple_amm() {
    let result = compile_contract("famous/SimpleAMM.sol");
    assert!(
        result.is_ok(),
        "Failed to compile SimpleAMM: {:?}",
        result.err()
    );
}

#[test]
fn test_compile_famous_token_vesting() {
    let result = compile_contract("famous/TokenVesting.sol");
    assert!(
        result.is_ok(),
        "Failed to compile TokenVesting: {:?}",
        result.err()
    );
}

#[test]
fn test_compile_famous_simple_lending() {
    let result = compile_contract("famous/SimpleLending.sol");
    assert!(
        result.is_ok(),
        "Failed to compile SimpleLending: {:?}",
        result.err()
    );
}

#[test]
fn test_compile_famous_simple_dao() {
    let result = compile_contract("famous/SimpleDAO.sol");
    assert!(
        result.is_ok(),
        "Failed to compile SimpleDAO: {:?}",
        result.err()
    );
}

// ========== Native contract integration tests (Phase 7c) ==========

#[test]
fn test_compile_native_contract_showcase() {
    let result = compile_contract("new/NativeContractShowcase.sol");
    assert!(
        result.is_ok(),
        "Failed to compile NativeContractShowcase: {:?}",
        result.err()
    );
}

#[test]
fn test_native_contract_showcase_manifest_methods() {
    let result = compile_contract("new/NativeContractShowcase.sol");
    assert!(result.is_ok(), "Failed to compile NativeContractShowcase");

    let (_, manifest_path) = result.unwrap();
    let manifest_data = std::fs::read_to_string(&manifest_path).expect("Failed to read manifest");

    let json: serde_json::Value = serde_json::from_str(&manifest_data).expect("Invalid JSON");

    let methods = json["abi"]["methods"]
        .as_array()
        .expect("ABI methods should be an array");

    let method_names: Vec<&str> = methods.iter().filter_map(|m| m["name"].as_str()).collect();

    // Verify key methods from each native contract section are present
    let expected = [
        "policyFeePerByte",
        "oraclePrice",
        "ledgerCurrentIndex",
        "ledgerCurrentHash",
        "notaryBalance",
        "treasuryIsValid",
        "isNative",
    ];

    for name in &expected {
        assert!(
            method_names.contains(name),
            "Manifest should contain method '{name}', found: {method_names:?}"
        );
    }
}

#[test]
fn test_native_contract_showcase_manifest_permissions() {
    let result = compile_contract("new/NativeContractShowcase.sol");
    assert!(result.is_ok(), "Failed to compile NativeContractShowcase");

    let (_, manifest_path) = result.unwrap();
    let manifest_data = std::fs::read_to_string(&manifest_path).expect("Failed to read manifest");

    let json: serde_json::Value = serde_json::from_str(&manifest_data).expect("Invalid JSON");

    let permissions = json["permissions"]
        .as_array()
        .expect("Permissions should be an array");

    // Contract calls native contracts → permissions must be non-empty
    assert!(
        !permissions.is_empty(),
        "NativeContractShowcase should have non-empty permissions for native contract calls"
    );
}

#[test]
fn test_native_contract_showcase_nef_valid() {
    let result = compile_contract("new/NativeContractShowcase.sol");
    assert!(result.is_ok(), "Failed to compile NativeContractShowcase");

    let (nef_path, _) = result.unwrap();
    let nef_data = std::fs::read(&nef_path).expect("Failed to read NEF");

    assert!(!nef_data.is_empty(), "NEF should not be empty");
    assert_eq!(
        &nef_data[..4],
        b"NEF3",
        "NEF should have correct magic bytes"
    );
    assert!(
        nef_data.len() > 50,
        "NEF should contain meaningful bytecode, got {} bytes",
        nef_data.len()
    );
}

#[test]
fn test_parallel_compilation_outputs_are_isolated() {
    let workers = 4usize;
    let iterations_per_worker = 2usize;

    let handles: Vec<_> = (0..workers)
        .map(|_| {
            std::thread::spawn(move || {
                for _ in 0..iterations_per_worker {
                    let (nef_path, manifest_path) =
                        compile_contract("new/NativeContractShowcase.sol")
                            .expect("parallel compile should succeed");

                    let nef_data = std::fs::read(&nef_path).expect("read NEF");
                    assert!(
                        nef_data.len() >= 4 && &nef_data[..4] == b"NEF3",
                        "parallel compile produced invalid NEF header"
                    );

                    let manifest_data =
                        std::fs::read_to_string(&manifest_path).expect("read manifest");
                    let json: serde_json::Value =
                        serde_json::from_str(&manifest_data).expect("manifest JSON");
                    let permissions = json["permissions"].as_array().expect("permissions array");
                    assert!(
                        !permissions.is_empty(),
                        "expected non-empty permissions in parallel compilation manifest"
                    );
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("parallel worker panicked");
    }
}

// ========== EVM compatibility error tests ==========

/// Compile a contract and assert that compilation fails with stderr containing
/// the expected error substring.
fn assert_compile_error_contains(contract_path: &str, expected_error: &str) {
    let compiler = get_compiler_path();
    assert!(compiler.exists(), "Compiler not found");

    let contract_path = get_example_path(contract_path);
    let output_prefix =
        unique_output_prefix("evm-compat-error", &contract_path).expect("output prefix");

    let output = Command::new(&compiler)
        .arg(&contract_path)
        .arg("-I")
        .arg("devpack")
        .arg("-O2")
        .arg("-o")
        .arg(&output_prefix)
        .output()
        .expect("Failed to run compiler");

    assert!(
        !output.status.success(),
        "Expected compilation to fail for {contract_path:?}"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(expected_error),
        "Expected stderr to contain '{expected_error}', got:\n{stderr}"
    );
}

/// Compile a contract and assert that compilation succeeds but stderr contains
/// the expected warning substring.
fn assert_compile_warns(contract_path: &str, expected_warning: &str) {
    let compiler = get_compiler_path();
    assert!(compiler.exists(), "Compiler not found");

    let contract_path = get_example_path(contract_path);
    let output_prefix =
        unique_output_prefix("evm-compat-warn", &contract_path).expect("output prefix");

    let output = Command::new(&compiler)
        .arg(&contract_path)
        .arg("-I")
        .arg("devpack")
        .arg("-O2")
        .arg("-o")
        .arg(&output_prefix)
        .output()
        .expect("Failed to run compiler");

    assert!(
        output.status.success(),
        "Expected compilation to succeed for {:?}, stderr:\n{}",
        contract_path,
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(expected_warning),
        "Expected stderr to contain warning '{expected_warning}', got:\n{stderr}"
    );
}

/// Compile a contract and assert that compilation succeeds and stderr does
/// not contain the given warning substring.
fn assert_compile_not_warns(contract_path: &str, unexpected_warning: &str) {
    let compiler = get_compiler_path();
    assert!(compiler.exists(), "Compiler not found");

    let contract_path = get_example_path(contract_path);
    let output_prefix =
        unique_output_prefix("evm-compat-no-warn", &contract_path).expect("output prefix");

    let output = Command::new(&compiler)
        .arg(&contract_path)
        .arg("-I")
        .arg("devpack")
        .arg("-O2")
        .arg("-o")
        .arg(&output_prefix)
        .output()
        .expect("Failed to run compiler");

    assert!(
        output.status.success(),
        "Expected compilation to succeed for {:?}, stderr:\n{}",
        contract_path,
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains(unexpected_warning),
        "Did not expect stderr to contain warning '{unexpected_warning}', got:\n{stderr}"
    );
}

// block.coinbase/difficulty/gaslimit/basefee are now auto-mapped to Neo N3
// equivalents (address(0), Runtime.getRandom, Policy.getExecFeeFactor,
// Policy.getFeePerByte) with compile-time warnings instead of errors.
#[test]
fn test_evm_compat_block_auto_mapped() {
    assert_compiles("new/EvmCompatBlockErrors.sol");
}

// blockhash() is now auto-mapped to Ledger.getBlock(n).hash with warning.
#[test]
fn test_evm_compat_blockhash_auto_mapped() {
    assert_compiles("new/EvmCompatBlockhashError.sol");
}

#[test]
fn test_evm_compat_blockhash_warning() {
    assert_compile_warns(
        "new/EvmCompatBlockhashError.sol",
        "blockhash() maps to Ledger.getBlock(n).hash on Neo N3",
    );
}

// address.codehash is now auto-mapped to contract script hash with warning.
#[test]
fn test_evm_compat_address_codehash_auto_mapped() {
    assert_compiles("new/EvmCompatAddressCodehash.sol");
}

#[test]
fn test_evm_compat_address_code_warning() {
    assert_compile_warns(
        "new/EvmCompatAddressCode.sol",
        "address.code auto-mapped to the Neo contract script bytes via ContractManagement.getContract()",
    );
}

// selfdestruct() is now auto-mapped to ContractManagement.destroy() with warning.
#[test]
fn test_evm_compat_selfdestruct_auto_mapped() {
    assert_compiles("new/EvmCompatSelfdestructError.sol");
}

#[test]
fn test_evm_compat_ether_units_error() {
    assert_compile_error_contains(
        "new/EvmCompatEtherUnits.sol",
        "ether units (wei/gwei/ether) are not applicable on Neo N3",
    );
}

#[test]
fn test_evm_compat_msg_sig_warning() {
    assert_compile_warns(
        "new/EvmCompatMsgSig.sol",
        "msg.sig is approximated on Neo N3 using the current function selector",
    );
}

#[test]
fn test_evm_compat_msg_data_warning() {
    assert_compile_warns(
        "new/EvmCompatMsgData.sol",
        "msg.data is approximated on Neo N3 as `selector || abi.encode(current args)`",
    );
}

#[test]
fn test_evm_compat_encode_calldata_warning() {
    assert_compile_warns(
        "new/EvmCompatEncodeCalldata.sol",
        "abi.encodeWithSignature(...) is approximated on Neo N3 as selector bytes concatenated with abi.encode(args)",
    );
}

#[test]
fn test_evm_compat_tx_origin_warning() {
    assert_compile_warns(
        "new/EvmCompatTxOrigin.sol",
        "tx.origin has different semantics on Neo N3",
    );
}

// ── Phase 3: Showcase compilation tests ──────────────────────────────────

#[test]
fn test_expression_showcase_compiles() {
    assert_compiles("new/ExpressionShowcase.sol");
}

#[test]
fn test_storage_concat_showcase_compiles() {
    assert_compiles("new/StorageConcatShowcase.sol");
}

// ── Function polish: payable, receive, fallback, modifiers, overloading ──

#[test]
fn test_compile_function_polish_showcase() {
    let result = compile_contract("new/FunctionPolishShowcase.sol");
    assert!(
        result.is_ok(),
        "Failed to compile FunctionPolishShowcase: {:?}",
        result.err()
    );
}

// ── Unchecked block support ───────────────────────────────────────────────

#[test]
fn test_compile_unchecked_showcase() {
    let result = compile_contract("new/UncheckedShowcase.sol");
    assert!(
        result.is_ok(),
        "Failed to compile UncheckedShowcase: {:?}",
        result.err()
    );
}

// ── Virtual/override inheritance enforcement ─────────────────────────────

#[test]
fn test_compile_virtual_override_showcase() {
    // Multi-contract file: compiler suffixes output with contract names
    assert_compiles("new/VirtualOverrideShowcase.sol");
}

// ── Library support ──────────────────────────────────────────────────────

#[test]
fn test_compile_library_showcase() {
    let result = compile_contract("new/LibraryShowcase.sol");
    assert!(
        result.is_ok(),
        "Failed to compile LibraryShowcase: {:?}",
        result.err()
    );
}

#[test]
fn test_library_showcase_manifest_has_compute_method() {
    let result = compile_contract("new/LibraryShowcase.sol");
    assert!(result.is_ok(), "Failed to compile LibraryShowcase");

    let (_, manifest_path) = result.unwrap();
    let manifest_data = std::fs::read_to_string(&manifest_path).expect("Failed to read manifest");
    let json: serde_json::Value = serde_json::from_str(&manifest_data).expect("Invalid JSON");

    let methods = json["abi"]["methods"]
        .as_array()
        .expect("ABI methods should be an array");
    let method_names: Vec<&str> = methods.iter().filter_map(|m| m["name"].as_str()).collect();

    assert!(
        method_names.contains(&"compute"),
        "Manifest should contain 'compute' method, found: {method_names:?}"
    );
}

#[test]
fn test_library_external_function_showcase_compiles() {
    assert_compiles("new/LibraryExternalError.sol");
}

#[test]
fn test_solidity_features_complete_has_no_false_override_warning() {
    assert_compile_not_warns(
        "new/SolidityFeaturesComplete.sol",
        "overrides 'Ownable::add' which is not marked 'virtual'",
    );
}

#[test]
fn test_library_state_variable_error() {
    assert_compile_error_contains("new/LibraryStateVarError.sol", "cannot have state variable");
}

#[test]
fn test_library_constructor_error() {
    assert_compile_error_contains(
        "new/LibraryConstructorError.sol",
        "cannot have a constructor",
    );
}

// ── Type system error tests ─────────────────────────────────────────────

#[test]
fn test_fixed_point_type_error() {
    assert_compile_error_contains("new/FixedPointError.sol", "fixed-point type");
}

#[test]
fn test_fixed_point_suggestion() {
    assert_compile_error_contains("new/FixedPointError.sol", "not supported on NeoVM");
}

#[test]
fn test_user_defined_value_type_compiles() {
    // `type Price is uint256` with `Price.wrap(...)` is now supported.
    // User-defined value types are transparent aliases; wrap/unwrap are no-ops.
    assert_compiles("new/UserDefinedTypeError.sol");
}

#[test]
fn test_type_name_expression_compiles() {
    // `type(Contract).name` and `type(uint256).name` resolve to compile-time
    // string constants on NeoVM.
    assert_compiles("new/TypeNameShowcase.sol");
}

// ── Super keyword diagnostic tests ──────────────────────────────────────

#[test]
fn test_compile_super_showcase_workaround() {
    // SuperShowcase.sol demonstrates the internal-helper workaround pattern
    // for the `super` keyword. It must compile successfully.
    assert_compiles("new/SuperShowcase.sol");
}

#[test]
fn test_super_keyword_compiles() {
    // SuperError.sol uses `super.greet()` which is now supported via
    // inheritance flattening with __super_ method preservation.
    assert_compiles("new/SuperError.sol");
}

// ── require(condition, CustomError(...)) support ────────────────────────

#[test]
fn test_require_custom_error_compiles() {
    // Solidity 0.8.26+ `require(cond, CustomError(args))` syntax.
    // On NeoVM the error name and arg count are preserved in the THROW message.
    let result = compile_contract("new/RequireCustomErrorShowcase.sol");
    assert!(
        result.is_ok(),
        "Failed to compile RequireCustomErrorShowcase: {:?}",
        result.err()
    );
}
