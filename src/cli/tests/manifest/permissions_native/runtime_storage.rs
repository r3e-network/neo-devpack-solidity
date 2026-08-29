#[test]
fn syscalls_contract_exists_uses_contract_management_is_contract() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract ExistsHarness {
        function exists(address hash) public view returns (bool) {
            return Syscalls.contractExists(hash);
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
                    .is_some_and(|methods| methods.iter().any(|m| m == "isContract"))
        }),
        "expected ContractManagement.isContract permission in manifest"
    );
}

#[test]
fn runtime_notify_requires_stdlib_serialize_and_deserialize_permissions() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract UsesRuntimeNotify {
        event Ping(uint256 a, uint256 b);

        function ping() public {
            Runtime.notify("Ping", abi.encode(uint256(1), uint256(2)));
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let manifest = &artifacts[0].manifest;
    let permissions = manifest["permissions"]
        .as_array()
        .expect("permissions array");

    let stdlib_hash_le =
        crate::codegen::native_contract_hash(neo_devpack_solidity::ir::NativeContract::StdLib);
    let stdlib_hash_be: Vec<u8> = stdlib_hash_le.iter().rev().copied().collect();
    let stdlib_contract = format!("0x{}", hex::encode(stdlib_hash_be));

    assert!(
        permissions.iter().any(|entry| {
            entry["contract"] == stdlib_contract
                && entry["methods"].as_array().is_some_and(|methods| {
                    methods.iter().any(|m| m == "serialize")
                        && methods.iter().any(|m| m == "deserialize")
                })
        }),
        "expected StdLib.serialize + StdLib.deserialize permissions for Runtime.notify"
    );

    assert!(
        permissions
            .iter()
            .all(|entry| entry["contract"] != Value::String("*".into())),
        "Runtime.notify should not require wildcard permissions"
    );
}

#[test]
fn runtime_witness_helpers_compile_as_intrinsics() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract RuntimeWitnessHelpers {
        function requireOne(address account) public view {
            Runtime.requireWitness(account);
        }

        function anyOf(address[] memory accounts) public view returns (bool) {
            return Runtime.checkAnyWitness(accounts);
        }

        function allOf(address[] memory accounts) public view returns (bool) {
            return Runtime.checkAllWitnesses(accounts);
        }

        function threshold(address[] memory signers, uint256 minSigners)
            public
            view
            returns (bool)
        {
            return Runtime.checkMultiSigWitness(signers, minSigners);
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);
}

#[test]
fn mapping_storage_requires_crypto_keccak_and_stdlib_serialize_permissions() {
    let source = r#"
    pragma solidity ^0.8.20;

    contract MappingStoragePermissions {
        mapping(address => uint256) private balances;

        function set(uint256 v) public {
            balances[msg.sender] = v;
        }

        function get() public view returns (uint256) {
            return balances[msg.sender];
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let manifest = &artifacts[0].manifest;
    let permissions = manifest["permissions"]
        .as_array()
        .expect("permissions array");

    let stdlib_hash_le =
        crate::codegen::native_contract_hash(neo_devpack_solidity::ir::NativeContract::StdLib);
    let stdlib_hash_be: Vec<u8> = stdlib_hash_le.iter().rev().copied().collect();
    let stdlib_contract = format!("0x{}", hex::encode(stdlib_hash_be));

    let crypto_hash_le =
        crate::codegen::native_contract_hash(neo_devpack_solidity::ir::NativeContract::CryptoLib);
    let crypto_hash_be: Vec<u8> = crypto_hash_le.iter().rev().copied().collect();
    let crypto_contract = format!("0x{}", hex::encode(crypto_hash_be));

    assert!(
        permissions.iter().any(|entry| {
            entry["contract"] == stdlib_contract
                && entry["methods"]
                    .as_array()
                    .is_some_and(|methods| methods.iter().any(|m| m == "serialize"))
        }),
        "expected StdLib.serialize permission for mapping storage"
    );

    assert!(
        permissions.iter().any(|entry| {
            entry["contract"] == crypto_contract
                && entry["methods"]
                    .as_array()
                    .is_some_and(|methods| methods.iter().any(|m| m == "keccak256"))
        }),
        "expected CryptoLib.keccak256 permission for mapping storage"
    );

    assert!(
        permissions
            .iter()
            .all(|entry| entry["contract"] != Value::String("*".into())),
        "mapping storage should not require wildcard permissions"
    );
}

#[test]
fn parameterised_constructor_requires_stdlib_json_deserialize_permission() {
    let source = r#"
    pragma solidity ^0.8.20;

    contract CtorArgs {
        uint256 private stored;

        constructor(uint256 initialValue) {
            stored = initialValue;
        }

        function get() public view returns (uint256) {
            return stored;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let manifest = &artifacts[0].manifest;
    let permissions = manifest["permissions"]
        .as_array()
        .expect("permissions array");

    let stdlib_hash_le =
        crate::codegen::native_contract_hash(neo_devpack_solidity::ir::NativeContract::StdLib);
    let stdlib_hash_be: Vec<u8> = stdlib_hash_le.iter().rev().copied().collect();
    let stdlib_contract = format!("0x{}", hex::encode(stdlib_hash_be));

    assert!(
        permissions.iter().any(|entry| {
            entry["contract"] == stdlib_contract
                && entry["methods"].as_array().is_some_and(|methods| {
                    methods.iter().any(|m| m == "jsonDeserialize")
                        && methods.iter().any(|m| m == "deserialize")
                })
        }),
        "expected StdLib.jsonDeserialize + StdLib.deserialize permission in manifest for parameterised constructors"
    );

    assert!(
        permissions
            .iter()
            .all(|entry| entry["contract"] != Value::String("*".into())),
        "parameterised constructors should not require wildcard permissions"
    );
}

#[test]
fn storage_put_contract_metadata_sets_storage_feature_and_stdlib_permissions() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract ContractMetadataHarness {
        function set() public {
            Storage.putContractMetadata(
                "MyContract",
                "1.0.0",
                "Neo DevPack for Solidity",
                abi.encode(uint256(1))
            );
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

    let stdlib_hash_le =
        crate::codegen::native_contract_hash(neo_devpack_solidity::ir::NativeContract::StdLib);
    let stdlib_hash_be: Vec<u8> = stdlib_hash_le.iter().rev().copied().collect();
    let stdlib_contract = format!("0x{}", hex::encode(stdlib_hash_be));

    assert!(
        permissions.iter().any(|entry| {
            entry["contract"] == stdlib_contract
                && entry["methods"]
                    .as_array()
                    .is_some_and(|methods| methods.iter().any(|m| m == "serialize"))
        }),
        "expected StdLib.serialize permission for Storage.putContractMetadata"
    );

    assert!(
        permissions
            .iter()
            .all(|entry| entry["contract"] != Value::String("*".into())),
        "Storage.putContractMetadata should not require wildcard permissions"
    );
}
