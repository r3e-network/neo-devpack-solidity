#[test]
fn deny_wildcard_methods_rejects_dynamic_method_permissions() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract CallsNativeViaContractCall {
        function callGas(string memory method) public returns (bytes memory) {
            return Syscalls.contractCall(
                NativeCalls.GAS_CONTRACT,
                method,
                abi.encode()
            );
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
            deny_wildcard_contracts: false,
            deny_wildcard_methods: true,
            manifest_permissions: None,
        },
    )
    .expect_err("expected manifest wildcard method error");

    match err {
        CompileError::Manifest(diag) => {
            let message = diag.message;

            assert!(
                message
                    .to_ascii_lowercase()
                    .contains("wildcard method manifest permissions"),
                "unexpected error message: {message}"
            );
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn manifest_permissions_replace_wildcards_allows_deny_wildcard_methods() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract CallsNativeViaContractCall {
        function callGas(string memory method) public returns (bytes memory) {
            return Syscalls.contractCall(
                NativeCalls.GAS_CONTRACT,
                method,
                abi.encode()
            );
        }
    }
    "#;

    let temp = tempdir().expect("tempdir");
    let permissions_path = temp.path().join("permissions.json");
    std::fs::write(
        &permissions_path,
        r#"[{"contract":"0xd2a4cff31913016155e38e474a2c06d08be276cf","methods":["balanceOf"]}]"#,
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
            deny_wildcard_contracts: false,
            deny_wildcard_methods: true,
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
            .all(|entry| entry["methods"] != Value::String("*".into())),
        "expected wildcard method permissions to be replaced"
    );

    let gas_contract = Value::String("0xd2a4cff31913016155e38e474a2c06d08be276cf".into());
    assert!(
        permissions.iter().any(|entry| {
            entry["contract"] == gas_contract
                && entry["methods"]
                    .as_array()
                    .is_some_and(|methods| methods.iter().any(|m| m == "balanceOf"))
        }),
        "expected explicit allowlist entry for GAS contract"
    );
}

#[test]
fn manifest_permissions_replace_wildcards_rejects_empty_override_for_dynamic_calls() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract FullyDynamicCalls {
        function callAny(address target, string memory method) public returns (bytes memory) {
            return Syscalls.contractCall(target, method, abi.encode());
        }
    }
    "#;

    let temp = tempdir().expect("tempdir");
    let permissions_path = temp.path().join("permissions.json");
    std::fs::write(&permissions_path, "[]").expect("write empty permissions");

    let override_permissions = load_manifest_permissions_override(
        permissions_path.to_str().expect("permissions path"),
        "replace-wildcards",
    )
    .expect("permissions override parse");

    let err = compile_contracts_with_options(
        source,
        false,
        CompileOptions {
            optimizer_level: 2,
            use_callt: false,
            deny_wildcard_permissions: false,
            deny_wildcard_contracts: false,
            deny_wildcard_methods: false,
            manifest_permissions: Some(override_permissions),
        },
    )
    .expect_err("expected empty replace-wildcards override to fail");

    match err {
        CompileError::Manifest(diag) => {
            let message = diag.message;

            assert!(
                message
                    .to_ascii_lowercase()
                    .contains("replace-wildcards"),
                "unexpected error message: {message}"
            );
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn deny_full_wildcard_permissions_rejects_fully_dynamic_calls() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract FullyDynamicCalls {
        function callAny(address target, string memory method) public returns (bytes memory) {
            return Syscalls.contractCall(target, method, abi.encode());
        }
    }
    "#;

    let err = compile_contracts_with_options(
        source,
        false,
        CompileOptions {
            optimizer_level: 2,
            use_callt: false,
            deny_wildcard_permissions: true,
            deny_wildcard_contracts: false,
            deny_wildcard_methods: false,
            manifest_permissions: None,
        },
    )
    .expect_err("expected full wildcard permission error");

    match err {
        CompileError::Manifest(diag) => {
            let message = diag.message;

            assert!(
                message
                    .to_ascii_lowercase()
                    .contains("full wildcard manifest permissions"),
                "unexpected error message: {message}"
            );
            assert!(
                message.contains("@custom:neo.manifest.permissions"),
                "expected NatSpec permission override guidance: {message}"
            );
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn manifest_permissions_merge_normalizes_and_deduplicates_override_entries() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract CallsNativeViaContractCall {
        function callGas() public returns (bytes memory) {
            return Syscalls.contractCall(
                NativeCalls.GAS_CONTRACT,
                "totalSupply",
                abi.encode()
            );
        }
    }
    "#;

    let temp = tempdir().expect("tempdir");
    let permissions_path = temp.path().join("permissions.json");
    std::fs::write(
        &permissions_path,
        r#"{
            "permissions": [
                {"contract":"0xD2A4CFF31913016155E38E474A2C06D08BE276CF","methods":["balanceOf","balanceOf"]},
                {"contract":"d2a4cff31913016155e38e474a2c06d08be276cf","methods":["totalSupply"]}
            ]
        }"#,
    )
    .expect("write permissions");

    let override_permissions = load_manifest_permissions_override(
        permissions_path.to_str().expect("permissions path"),
        "merge",
    )
    .expect("permissions override parse");

    let artifacts = compile_contracts_with_options(
        source,
        false,
        CompileOptions {
            optimizer_level: 2,
            use_callt: false,
            deny_wildcard_permissions: false,
            deny_wildcard_contracts: false,
            deny_wildcard_methods: false,
            manifest_permissions: Some(override_permissions),
        },
    )
    .expect("expected compilation to succeed with merged override");
    assert_eq!(artifacts.len(), 1);

    let permissions = artifacts[0].manifest["permissions"]
        .as_array()
        .expect("permissions array");

    let gas_contract = Value::String("0xd2a4cff31913016155e38e474a2c06d08be276cf".into());
    let gas_entries: Vec<&Value> = permissions
        .iter()
        .filter(|entry| entry["contract"] == gas_contract)
        .collect();
    assert_eq!(gas_entries.len(), 1, "expected normalized merged gas entry");

    let gas_methods = gas_entries[0]["methods"].as_array().expect("methods array");
    let names: Vec<&str> = gas_methods.iter().filter_map(|m| m.as_str()).collect();
    assert!(
        names.contains(&"balanceOf") && names.contains(&"totalSupply"),
        "expected merged method set, got: {names:?}"
    );
    let mut unique = std::collections::BTreeSet::new();
    for name in &names {
        unique.insert(*name);
    }
    assert_eq!(unique.len(), names.len(), "methods should be deduplicated");
}

#[test]
fn branch_selected_known_methods_do_not_require_wildcard_method_permissions() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract BranchSelectedMethods {
        function callEither(bool supply) public returns (bytes memory) {
            string memory method;
            if (supply) {
                method = "totalSupply";
            } else {
                method = "symbol";
            }
            return Syscalls.contractCall(NativeCalls.GAS_CONTRACT, method, abi.encode());
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

    let gas_entry = permissions
        .iter()
        .find(|entry| entry["contract"] == gas_contract)
        .expect("gas permission entry");
    let methods = gas_entry["methods"].as_array().expect("methods array");

    assert!(methods.iter().any(|method| method == "totalSupply"));
    assert!(methods.iter().any(|method| method == "symbol"));
    assert_ne!(
        gas_entry["methods"],
        Value::String("*".into()),
        "branch-selected known methods should not require wildcard method permissions"
    );
}

#[test]
fn deny_wildcard_methods_rejects_wildcard_from_merge_override() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract StaticCallHarness {
        function callGas() public returns (bytes memory) {
            return Syscalls.contractCall(
                NativeCalls.GAS_CONTRACT,
                "totalSupply",
                abi.encode()
            );
        }
    }
    "#;

    let temp = tempdir().expect("tempdir");
    let permissions_path = temp.path().join("permissions.json");
    std::fs::write(
        &permissions_path,
        r#"[{"contract":"0xd2a4cff31913016155e38e474a2c06d08be276cf","methods":"*"}]"#,
    )
    .expect("write permissions");

    let override_permissions = load_manifest_permissions_override(
        permissions_path.to_str().expect("permissions path"),
        "merge",
    )
    .expect("permissions override parse");

    let err = compile_contracts_with_options(
        source,
        false,
        CompileOptions {
            optimizer_level: 2,
            use_callt: false,
            deny_wildcard_permissions: false,
            deny_wildcard_contracts: false,
            deny_wildcard_methods: true,
            manifest_permissions: Some(override_permissions),
        },
    )
    .expect_err("expected wildcard methods in override to be rejected");

    match err {
        CompileError::Manifest(diag) => {
            let message = diag.message;

            assert!(
                message
                    .to_ascii_lowercase()
                    .contains("wildcard method manifest permissions"),
                "unexpected error message: {message}"
            );
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}
