#[test]
fn address_transfer_and_send_emit_gas_permissions() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract Payments {
        function sendTo(address payable to, uint256 amount) public returns (bool) {
            return to.send(amount);
        }

        function transferTo(address payable to, uint256 amount) public {
            to.transfer(amount);
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let manifest = &artifacts[0].manifest;
    assert!(
        manifest
            .get("features")
            .and_then(Value::as_object)
            .is_some_and(|features| features.is_empty()),
        "expected `manifest.features` to be an empty object for Neo N3 compatibility"
    );

    let permissions = manifest["permissions"]
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
                    .is_some_and(|methods| methods.iter().any(|m| m == "transfer"))
        }),
        "expected GAS.transfer permission in manifest"
    );

    assert!(
        permissions
            .iter()
            .all(|entry| entry["contract"] != Value::String("*".into())),
        "transfer/send should not require wildcard permissions"
    );
}

#[test]
fn native_calls_emit_exact_permissions() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract UsesNativeCalls {
        function transferGas(address from, address to, uint256 amount, bytes memory data) public returns (bool) {
            return NativeCalls.gasTransfer(from, to, amount, data);
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

    assert!(
        permissions.iter().any(|entry| {
            entry["contract"] == gas_contract
                && entry["methods"]
                    .as_array()
                    .is_some_and(|methods| methods.iter().any(|m| m == "transfer"))
        }),
        "expected GAS.transfer permission in manifest"
    );

    assert!(
        permissions
            .iter()
            .all(|entry| entry["contract"] != Value::String("*".into())),
        "NativeCalls should not require wildcard permissions"
    );
}

#[test]
fn native_contract_constant_member_calls_do_not_require_wildcard_permissions() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract UsesNativeConstants {
        function supply() public view returns (uint256) {
            return NativeCalls.GAS_CONTRACT.totalSupply();
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
        permissions
            .iter()
            .all(|entry| entry["contract"] != Value::String("*".into())),
        "native contract calls with known hashes should not require wildcard permissions"
    );
}

#[test]
fn contract_management_deploy_contract_requires_explicit_permission() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract DeployHarness {
        function deploy(bytes memory nef, bytes memory manifest) public returns (address) {
            return NativeCalls.deployContract(nef, manifest);
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let manifest = &artifacts[0].manifest;
    let permissions = manifest["permissions"]
        .as_array()
        .expect("permissions array");

    let cm_hash_le =
        crate::codegen::native_contract_hash(neo_devpack_solidity::ir::NativeContract::ContractManagement);
    let cm_hash_be: Vec<u8> = cm_hash_le.iter().rev().copied().collect();
    let cm_contract = format!("0x{}", hex::encode(cm_hash_be));

    assert!(
        permissions.iter().any(|entry| {
            entry["contract"] == cm_contract
                && entry["methods"]
                    .as_array()
                    .is_some_and(|methods| methods.iter().any(|m| m == "deploy"))
        }),
        "expected ContractManagement.deploy permission in manifest"
    );

    assert!(
        permissions
            .iter()
            .all(|entry| entry["contract"] != Value::String("*".into())),
        "NativeCalls.deployContract should not require wildcard permissions"
    );
}

#[test]
fn contract_management_get_contract_requires_stdlib_serialize_permission() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract GetContractHarness {
        function manifestBytes(address hash) public view returns (bytes memory) {
            NativeCalls.ContractState memory state = NativeCalls.getContract(hash);
            return state.manifest;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let manifest = &artifacts[0].manifest;
    let permissions = manifest["permissions"]
        .as_array()
        .expect("permissions array");

    let cm_hash_le =
        crate::codegen::native_contract_hash(neo_devpack_solidity::ir::NativeContract::ContractManagement);
    let cm_hash_be: Vec<u8> = cm_hash_le.iter().rev().copied().collect();
    let cm_contract = format!("0x{}", hex::encode(cm_hash_be));

    let stdlib_hash_le =
        crate::codegen::native_contract_hash(neo_devpack_solidity::ir::NativeContract::StdLib);
    let stdlib_hash_be: Vec<u8> = stdlib_hash_le.iter().rev().copied().collect();
    let stdlib_contract = format!("0x{}", hex::encode(stdlib_hash_be));

    assert!(
        permissions.iter().any(|entry| {
            entry["contract"] == cm_contract
                && entry["methods"]
                    .as_array()
                    .is_some_and(|methods| methods.iter().any(|m| m == "getContract"))
        }),
        "expected ContractManagement.getContract permission in manifest"
    );

    assert!(
        permissions.iter().any(|entry| {
            entry["contract"] == stdlib_contract
                && entry["methods"]
                    .as_array()
                    .is_some_and(|methods| methods.iter().any(|m| m == "serialize"))
        }),
        "expected StdLib.serialize permission in manifest"
    );

    assert!(
        permissions
            .iter()
            .all(|entry| entry["contract"] != Value::String("*".into())),
        "NativeCalls.getContract should not require wildcard permissions"
    );
}
