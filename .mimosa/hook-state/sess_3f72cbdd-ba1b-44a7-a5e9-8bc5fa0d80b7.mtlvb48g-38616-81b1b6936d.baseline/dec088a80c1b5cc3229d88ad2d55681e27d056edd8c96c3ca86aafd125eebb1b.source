#[test]
fn self_contract_calls_do_not_require_wildcard_permissions() {
    let source = r#"
    pragma solidity ^0.8.20;

    contract SelfCallHarness {
        function dispatch(string calldata methodName) public {
            bytes memory params = abi.encode(uint256(1));
            // Self-calls are allowed on Neo N3 and should not force wildcard permissions.
            Syscalls.contractCall(address(this), methodName, params);
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let manifest = &artifacts[0].manifest;
    let permissions = manifest["permissions"]
        .as_array()
        .expect("permissions array");

    assert!(
        permissions
            .iter()
            .all(|entry| entry["contract"] != Value::String("*".into())),
        "self contract calls should not require wildcard permissions"
    );
}

#[test]
fn self_contract_calls_preserve_state_for_following_calls() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract SelfThenNative {
        function run() public returns (bytes memory) {
            bytes memory params = abi.encode(uint256(1));
            bytes memory ignored = Syscalls.contractCall(address(this), "localPing", params);
            return Syscalls.contractCall(NativeCalls.GAS_CONTRACT, "totalSupply", ignored);
        }

        function localPing(uint256 value) public pure returns (uint256) {
            return value + 1;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let permissions = artifacts[0].manifest["permissions"]
        .as_array()
        .expect("permissions array");

    let gas_hash_le = crate::codegen::native_contract_hash(neo_devpack_solidity::ir::NativeContract::Gas);
    let gas_hash_be: Vec<u8> = gas_hash_le.iter().rev().copied().collect();
    let gas_contract = format!("0x{}", hex::encode(gas_hash_be));

    assert!(
        permissions.iter().any(|entry| {
            entry["contract"] == Value::String(gas_contract.clone())
                && entry["methods"]
                    .as_array()
                    .is_some_and(|methods| methods.iter().any(|m| m == "totalSupply"))
        }),
        "expected explicit permission for the native GAS totalSupply call"
    );

    assert!(
        permissions
            .iter()
            .all(|entry| !(entry["contract"] == Value::String("*".into())
                && entry["methods"] == "*")),
        "self call followed by native call should not degrade to full wildcard permissions"
    );
}

#[test]
fn syscalls_contract_call_dynamic_method_restricts_to_contract() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract CallsNativeViaContractCall {
        function callGas(address /*ignored*/, string memory method) public returns (bytes memory) {
            return Syscalls.contractCall(
                NativeCalls.GAS_CONTRACT,
                method,
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

    assert!(
        permissions
            .iter()
            .any(|entry| entry["contract"] == gas_contract && entry["methods"] == "*"),
        "expected GAS wildcard methods (methods='*') when the method name is dynamic"
    );

    assert!(
        permissions
            .iter()
            .all(|entry| entry["contract"] != Value::String("*".into())),
        "dynamic method name alone should not force wildcard contract permissions"
    );
}

#[test]
fn external_contract_call_dynamic_target_restricts_to_method() {
    let source = r#"
    pragma solidity ^0.8.19;

    interface IFace {
        function ping(uint256 value) external returns (uint256);
    }

    contract ExternalCallPermissionHarness {
        function callPing(address target, uint256 value) public returns (uint256) {
            return IFace(target).ping(value);
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let manifest = &artifacts[0].manifest;
    let permissions = manifest["permissions"]
        .as_array()
        .expect("permissions array");

    assert!(
        permissions.iter().any(|entry| {
            entry["contract"] == Value::String("*".into())
                && entry["methods"]
                    .as_array()
                    .is_some_and(|methods| methods.iter().any(|m| m == "ping"))
        }),
        "expected wildcard contract permission restricted to ping() method"
    );

    assert!(
        permissions
            .iter()
            .all(|entry| !(entry["contract"] == Value::String("*".into())
                && entry["methods"] == "*")),
        "expected no full wildcard permissions for external call with static method name"
    );
}

#[test]
fn deny_wildcard_contracts_rejects_dynamic_target_permissions() {
    let source = r#"
    pragma solidity ^0.8.19;

    interface IFace {
        function ping(uint256 value) external returns (uint256);
    }

    contract ExternalCallPermissionHarness {
        function callPing(address target, uint256 value) public returns (uint256) {
            return IFace(target).ping(value);
        }
    }
    "#;

    let err = compile_contracts_with_options(
        source,
        false,
        CompileOptions {
            optimizer_level: 2,
            use_callt: false,
            deny_wildcard_permissions: false,
            deny_wildcard_contracts: true,
            deny_wildcard_methods: false,
            manifest_permissions: None,
        },
    )
    .expect_err("expected manifest wildcard contract error");

    match err {
        CompileError::Manifest(diag) => {
            let message = diag.message;

            assert!(
                message
                    .to_ascii_lowercase()
                    .contains("wildcard contract manifest permissions"),
                "unexpected error message: {message}"
            );
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn manifest_permissions_replace_wildcards_allows_deny_wildcard_contracts() {
    let source = r#"
    pragma solidity ^0.8.19;

    interface IFace {
        function ping(uint256 value) external returns (uint256);
    }

    contract ExternalCallPermissionHarness {
        function callPing(address target, uint256 value) public returns (uint256) {
            return IFace(target).ping(value);
        }
    }
    "#;

    let temp = tempdir().expect("tempdir");
    let permissions_path = temp.path().join("permissions.json");
    std::fs::write(
        &permissions_path,
        r#"[{"contract":"0x0102030405060708090a0b0c0d0e0f1011121314","methods":["ping"]}]"#,
    )
    .expect("write permissions");

    let override_permissions = load_manifest_permissions_override(
        permissions_path.to_str().expect("permissions path"),
        "replace-wildcards",
    )
    .expect("permissions override parse");

    let artifacts = compile_contracts_with_options(
        source,
        false,
        CompileOptions {
            optimizer_level: 2,
            use_callt: false,
            deny_wildcard_permissions: false,
            deny_wildcard_contracts: true,
            deny_wildcard_methods: false,
            manifest_permissions: Some(override_permissions),
        },
    )
    .expect("expected compilation to succeed with wildcard replacement");
    assert_eq!(artifacts.len(), 1);

    let permissions = artifacts[0].manifest["permissions"]
        .as_array()
        .expect("permissions array");

    assert!(
        permissions
            .iter()
            .all(|entry| entry["contract"] != Value::String("*".into())),
        "expected wildcard contract permissions to be replaced"
    );

    assert!(
        permissions.iter().any(|entry| {
            entry["contract"] == Value::String("0x0102030405060708090a0b0c0d0e0f1011121314".into())
                && entry["methods"]
                    .as_array()
                    .is_some_and(|methods| methods.iter().any(|m| m == "ping"))
        }),
        "expected explicit allowlist entry for the target contract"
    );
}

#[test]
fn branch_selected_known_targets_do_not_require_wildcard_contract_permissions() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract BranchSelectedTargets {
        function callEither(bool useGas, address account) public returns (bytes memory) {
            address target;
            if (useGas) {
                target = NativeCalls.GAS_CONTRACT;
            } else {
                target = NativeCalls.NEO_CONTRACT;
            }
            return Syscalls.contractCall(target, "balanceOf", abi.encode(account));
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let permissions = artifacts[0].manifest["permissions"]
        .as_array()
        .expect("permissions array");

    let gas_hash_le = crate::codegen::native_contract_hash(neo_devpack_solidity::ir::NativeContract::Gas);
    let gas_hash_be: Vec<u8> = gas_hash_le.iter().rev().copied().collect();
    let gas_contract = Value::String(format!("0x{}", hex::encode(gas_hash_be)));

    let neo_hash_le = crate::codegen::native_contract_hash(neo_devpack_solidity::ir::NativeContract::Neo);
    let neo_hash_be: Vec<u8> = neo_hash_le.iter().rev().copied().collect();
    let neo_contract = Value::String(format!("0x{}", hex::encode(neo_hash_be)));

    assert!(
        permissions.iter().any(|entry| {
            entry["contract"] == gas_contract
                && entry["methods"]
                    .as_array()
                    .is_some_and(|methods| methods.iter().any(|m| m == "balanceOf"))
        }),
        "expected explicit GAS permission"
    );
    assert!(
        permissions.iter().any(|entry| {
            entry["contract"] == neo_contract
                && entry["methods"]
                    .as_array()
                    .is_some_and(|methods| methods.iter().any(|m| m == "balanceOf"))
        }),
        "expected explicit NEO permission"
    );
    assert!(
        permissions
            .iter()
            .all(|entry| entry["contract"] != Value::String("*".into())),
        "branch-selected known targets should not require wildcard contract permissions"
    );
}

#[test]
fn correlated_branch_selected_target_and_method_do_not_cross_product_permissions() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract CorrelatedTargetAndMethod {
        function callEither(bool useGas) public returns (bytes memory) {
            address target;
            string memory method;
            if (useGas) {
                target = NativeCalls.GAS_CONTRACT;
                method = "totalSupply";
            } else {
                target = NativeCalls.NEO_CONTRACT;
                method = "getCommittee";
            }
            return Syscalls.contractCall(target, method, abi.encode());
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let permissions = artifacts[0].manifest["permissions"]
        .as_array()
        .expect("permissions array");

    let gas_hash_le = crate::codegen::native_contract_hash(neo_devpack_solidity::ir::NativeContract::Gas);
    let gas_hash_be: Vec<u8> = gas_hash_le.iter().rev().copied().collect();
    let gas_contract = Value::String(format!("0x{}", hex::encode(gas_hash_be)));

    let neo_hash_le = crate::codegen::native_contract_hash(neo_devpack_solidity::ir::NativeContract::Neo);
    let neo_hash_be: Vec<u8> = neo_hash_le.iter().rev().copied().collect();
    let neo_contract = Value::String(format!("0x{}", hex::encode(neo_hash_be)));

    let gas_entry = permissions
        .iter()
        .find(|entry| entry["contract"] == gas_contract)
        .expect("gas permission entry");
    let neo_entry = permissions
        .iter()
        .find(|entry| entry["contract"] == neo_contract)
        .expect("neo permission entry");

    let gas_methods = gas_entry["methods"].as_array().expect("gas methods array");
    let neo_methods = neo_entry["methods"].as_array().expect("neo methods array");

    assert!(gas_methods.iter().any(|method| method == "totalSupply"));
    assert!(!gas_methods.iter().any(|method| method == "getCommittee"));
    assert!(neo_methods.iter().any(|method| method == "getCommittee"));
    assert!(!neo_methods.iter().any(|method| method == "totalSupply"));
}

#[test]
fn natspec_manifest_permissions_replace_dynamic_wildcards_under_strict_flags() {
    let source = r#"
    pragma solidity ^0.8.19;

    /**
     * @custom:neo.manifest.permissions [
     *   {"contract":"0xd2a4cff31913016155e38e474a2c06d08be276cf","methods":["balanceOf"]}
     * ]
     */
    contract DynamicPermissionOverride {
        function callAny(address target, string memory method, address account) public returns (bytes memory) {
            return Syscalls.contractCall(target, method, abi.encode(account));
        }
    }
    "#;

    let artifacts = compile_contracts_with_options(
        source,
        false,
        CompileOptions {
            optimizer_level: 2,
            use_callt: false,
            deny_wildcard_permissions: true,
            deny_wildcard_contracts: true,
            deny_wildcard_methods: true,
            manifest_permissions: None,
        },
    )
    .expect("expected NatSpec permissions override to satisfy strict flags");
    assert_eq!(artifacts.len(), 1);

    let permissions = artifacts[0].manifest["permissions"]
        .as_array()
        .expect("permissions array");

    let gas_contract = Value::String("0xd2a4cff31913016155e38e474a2c06d08be276cf".into());
    assert!(
        permissions.iter().any(|entry| {
            entry["contract"] == gas_contract
                && entry["methods"]
                    .as_array()
                    .is_some_and(|methods| methods.iter().any(|m| m == "balanceOf"))
        }),
        "expected explicit NatSpec allowlist entry"
    );
    assert!(
        permissions
            .iter()
            .all(|entry| entry["contract"] != Value::String("*".into()) && entry["methods"] != "*"),
        "expected NatSpec override to remove wildcard permissions"
    );
}

#[test]
fn natspec_manifest_permissions_mode_merge_preserves_wildcards() {
    let source = r#"
    pragma solidity ^0.8.19;

    /**
     * @custom:neo.manifest.permissions [{"contract":"0xd2a4cff31913016155e38e474a2c06d08be276cf","methods":["balanceOf"]}]
     * @custom:neo.manifest.permissionsmode "merge"
     */
    contract DynamicPermissionMerge {
        function callAny(address target, string memory method, address account) public returns (bytes memory) {
            return Syscalls.contractCall(target, method, abi.encode(account));
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let permissions = artifacts[0].manifest["permissions"]
        .as_array()
        .expect("permissions array");

    assert!(
        permissions.iter().any(|entry| {
            entry["contract"] == Value::String("*".into()) && entry["methods"] == "*"
        }),
        "merge mode should preserve inferred wildcard permissions"
    );
}

#[test]
fn address_constants_in_contract_calls_infer_explicit_contract_permissions() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract ConstantTargetCallHarness {
        address constant TARGET = 0xd2a4cff31913016155e38e474a2c06d08be276cf;

        function callSupply() public view returns (bytes memory) {
            return Syscalls.contractCall(TARGET, "totalSupply", abi.encode());
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let permissions = artifacts[0].manifest["permissions"]
        .as_array()
        .expect("permissions array");

    assert!(
        permissions.iter().any(|entry| {
            entry["contract"] == Value::String("0xd2a4cff31913016155e38e474a2c06d08be276cf".into())
                && entry["methods"]
                    .as_array()
                    .is_some_and(|methods| methods.iter().any(|m| m == "totalSupply"))
        }),
        "expected explicit GAS contract permission for totalSupply"
    );

    assert!(
        permissions
            .iter()
            .all(|entry| entry["contract"] != Value::String("*".into())),
        "address constants should not degrade into wildcard contract permissions"
    );
}
