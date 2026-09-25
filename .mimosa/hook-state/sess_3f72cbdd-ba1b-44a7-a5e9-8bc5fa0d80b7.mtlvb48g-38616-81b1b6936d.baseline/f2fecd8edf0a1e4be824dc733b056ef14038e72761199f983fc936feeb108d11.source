#[test]
fn syscalls_script_hash_to_address_keeps_precise_contract_permissions() {
    let source = r#"
    pragma solidity ^0.8.20;

    interface ICallee {
        function foo(uint256 x) external view returns (uint256);
    }

    contract UsesScriptHashConversions {
        bytes20 constant CALLEE_HASH_LE = hex"14131211100f0e0d0c0b0a090807060504030201";

        function run(uint256 x) public view returns (uint256) {
            return ICallee(Syscalls.scriptHashToAddress(CALLEE_HASH_LE)).foo(x);
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let manifest = &artifacts[0].manifest;
    let permissions = manifest["permissions"]
        .as_array()
        .expect("permissions array");
    let contract = "0x0102030405060708090a0b0c0d0e0f1011121314";

    assert!(
        permissions.iter().any(|entry| {
            entry["contract"] == Value::String(contract.into())
                && entry["methods"]
                    .as_array()
                    .is_some_and(|methods| methods.iter().any(|m| m == "foo"))
        }),
        "expected exact contract permission for scriptHashToAddress static target"
    );

    assert!(
        permissions
            .iter()
            .all(|entry| entry["contract"] != Value::String("*".into())),
        "scriptHashToAddress static target should not require wildcard permissions"
    );
}

#[test]
fn syscalls_contract_call_static_target_emits_exact_permissions() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract CallsNativeViaContractCall {
        function callGasSupply() public returns (bytes memory) {
            return Syscalls.contractCall(
                NativeCalls.GAS_CONTRACT,
                "totalSupply",
                abi.encode()
            );
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let manifest = &artifacts[0].manifest;
    let permissions = manifest["permissions"]
        .as_array()
        .expect("permissions array");

    let gas_hash_le = crate::codegen::native_contract_hash(neo_devpack_solidity::ir::NativeContract::Gas);
    let gas_hash_be: Vec<u8> = gas_hash_le.iter().rev().copied().collect();
    let gas_contract = format!("0x{}", hex::encode(gas_hash_be));

    let stdlib_hash_le =
        crate::codegen::native_contract_hash(neo_devpack_solidity::ir::NativeContract::StdLib);
    let stdlib_hash_be: Vec<u8> = stdlib_hash_le.iter().rev().copied().collect();
    let stdlib_contract = format!("0x{}", hex::encode(stdlib_hash_be));

    assert!(
        permissions.iter().any(|entry| {
            entry["contract"] == gas_contract
                && entry["methods"]
                    .as_array()
                    .is_some_and(|methods| methods.iter().any(|m| m == "totalSupply"))
        }),
        "expected GAS.totalSupply permission in manifest"
    );

    assert!(
        permissions.iter().any(|entry| {
            entry["contract"] == stdlib_contract
                && entry["methods"].as_array().is_some_and(|methods| {
                    methods.iter().any(|m| m == "serialize")
                        && methods.iter().any(|m| m == "deserialize")
                })
        }),
        "expected StdLib.serialize + StdLib.deserialize permissions for Syscalls.contractCall"
    );

    assert!(
        permissions
            .iter()
            .all(|entry| entry["contract"] != Value::String("*".into())),
        "static Syscalls.contractCall should not require wildcard permissions"
    );
}

#[test]
fn external_contract_call_static_target_emits_exact_permissions() {
    let source = r#"
    pragma solidity ^0.8.19;

    interface IFace {
        function ping(uint256 value) external returns (uint256);
    }

    contract ExternalCallPermissionHarness {
        function callPing(uint256 value) public returns (uint256) {
            return IFace(0x0102030405060708090a0b0c0d0e0f1011121314).ping(value);
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let manifest = &artifacts[0].manifest;
    let permissions = manifest["permissions"]
        .as_array()
        .expect("permissions array");

    let contract = "0x0102030405060708090a0b0c0d0e0f1011121314";

    assert!(
        permissions.iter().any(|entry| {
            entry["contract"] == Value::String(contract.into())
                && entry["methods"]
                    .as_array()
                    .is_some_and(|methods| methods.iter().any(|m| m == "ping"))
        }),
        "expected exact contract permission for static target address"
    );

    assert!(
        permissions
            .iter()
            .all(|entry| entry["contract"] != Value::String("*".into())),
        "static target should not require wildcard contract permissions"
    );
}

#[test]
fn native_contracts_member_call_emits_exact_native_permission() {
    let source = [
        include_str!("../../../../devpack/contracts/NativeContracts.sol"),
        r#"
        pragma solidity ^0.8.19;

        contract NativeContractsPermissionHarness {
            function callGasSupply() public view returns (uint256) {
                return NativeContracts.GAS_CONTRACT.totalSupply();
            }
        }
        "#,
    ]
    .join("\n");

    let artifacts = compile_contracts(&source, false, 2).expect("compilation failed");
    let harness = artifacts
        .iter()
        .find(|artifact| artifact.metadata.name == "NativeContractsPermissionHarness")
        .expect("expected NativeContractsPermissionHarness artifact");

    let permissions = harness.manifest["permissions"]
        .as_array()
        .expect("permissions array");

    let gas_hash_le = crate::codegen::native_contract_hash(neo_devpack_solidity::ir::NativeContract::Gas);
    let gas_hash_be: Vec<u8> = gas_hash_le.iter().rev().copied().collect();
    let gas_contract = format!("0x{}", hex::encode(gas_hash_be));

    assert!(
        permissions.iter().any(|entry| {
            entry["contract"] == gas_contract
                && entry["methods"]
                    .as_array()
                    .is_some_and(|methods| methods.iter().any(|m| m == "totalSupply"))
        }),
        "expected GAS.totalSupply permission in manifest for NativeContracts member call"
    );

    assert!(
        permissions
            .iter()
            .all(|entry| entry["contract"] != Value::String("*".into())),
        "NativeContracts member call should not require wildcard contract permissions"
    );
}

#[test]
fn repeated_static_native_contract_calls_emit_single_deduplicated_permission_entry() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract PermissionDedupHarness {
        function run() public returns (bytes memory, bytes memory, bytes memory) {
            bytes memory a = Syscalls.contractCall(
                NativeCalls.GAS_CONTRACT,
                "totalSupply",
                abi.encode()
            );
            bytes memory b = Syscalls.contractCall(
                NativeCalls.GAS_CONTRACT,
                "balanceOf",
                abi.encode(address(this))
            );
            bytes memory c = Syscalls.contractCall(
                NativeCalls.GAS_CONTRACT,
                "totalSupply",
                abi.encode()
            );
            return (a, b, c);
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let manifest = &artifacts[0].manifest;
    let permissions = manifest["permissions"]
        .as_array()
        .expect("permissions array");

    let gas_hash_le = crate::codegen::native_contract_hash(neo_devpack_solidity::ir::NativeContract::Gas);
    let gas_hash_be: Vec<u8> = gas_hash_le.iter().rev().copied().collect();
    let gas_contract = format!("0x{}", hex::encode(gas_hash_be));

    let gas_entries: Vec<&Value> = permissions
        .iter()
        .filter(|entry| entry["contract"] == Value::String(gas_contract.clone()))
        .collect();
    assert_eq!(
        gas_entries.len(),
        1,
        "expected exactly one GAS permission entry, got: {gas_entries:?}"
    );

    let gas_methods = gas_entries[0]["methods"].as_array().expect("GAS methods array");
    let gas_method_names: Vec<&str> = gas_methods.iter().filter_map(|m| m.as_str()).collect();
    assert_eq!(
        gas_method_names,
        vec!["balanceOf", "totalSupply"],
        "expected deduplicated, deterministic method set"
    );

    assert!(
        permissions
            .iter()
            .all(|entry| entry["contract"] != Value::String("*".into())),
        "static native calls should not require wildcard contract permissions"
    );
}
