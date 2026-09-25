#[test]
fn devpack_nep17_standard_compiles() {
    let source = [
        include_str!("../../../../devpack/contracts/Syscalls.sol"),
        include_str!("../../../../devpack/contracts/NativeContracts.sol"),
        include_str!("../../../../devpack/contracts/NativeCalls.sol"),
        include_str!("../../../../devpack/libraries/Storage.sol"),
        include_str!("../../../../devpack/libraries/Runtime.sol"),
        include_str!("../../../../devpack/libraries/Neo.sol"),
        include_str!("../../../../devpack/contracts/FrameworkBase.sol"),
        include_str!("../../../../devpack/standards/NEP17.sol"),
    ]
    .join("\n");

    let artifacts = compile_contracts(&source, false, 2).expect("devpack NEP17 compilation failed");
    let names: Vec<_> = artifacts.iter().map(|a| a.metadata.name.as_str()).collect();
    assert_eq!(artifacts.len(), 2, "expected FrameworkBase + NEP17 outputs");
    assert!(names.contains(&"FrameworkBase"));
    assert!(names.contains(&"NEP17"));

    let nep17 = artifacts
        .iter()
        .find(|a| a.metadata.name == "NEP17")
        .expect("NEP17 artifact");
    let permissions = nep17.manifest["permissions"]
        .as_array()
        .expect("permissions array");
    assert!(
        permissions.iter().all(|entry| {
            !(entry["contract"] == Value::String("*".into())
                && entry["methods"] == Value::String("*".into()))
        }),
        "NEP17 should not require full wildcard permissions"
    );
}

#[test]
fn devpack_nep11_standard_compiles() {
    let source = [
        include_str!("../../../../devpack/contracts/Syscalls.sol"),
        include_str!("../../../../devpack/contracts/NativeContracts.sol"),
        include_str!("../../../../devpack/contracts/NativeCalls.sol"),
        include_str!("../../../../devpack/libraries/Storage.sol"),
        include_str!("../../../../devpack/libraries/Runtime.sol"),
        include_str!("../../../../devpack/libraries/Neo.sol"),
        include_str!("../../../../devpack/contracts/FrameworkBase.sol"),
        include_str!("../../../../devpack/standards/NEP11.sol"),
    ]
    .join("\n");

    let artifacts = compile_contracts(&source, false, 2).expect("devpack NEP11 compilation failed");
    let names: Vec<_> = artifacts.iter().map(|a| a.metadata.name.as_str()).collect();
    assert_eq!(artifacts.len(), 2, "expected FrameworkBase + NEP11 outputs");
    assert!(names.contains(&"FrameworkBase"));
    assert!(names.contains(&"NEP11"));

    let nep11 = artifacts
        .iter()
        .find(|a| a.metadata.name == "NEP11")
        .expect("NEP11 artifact");
    let permissions = nep11.manifest["permissions"]
        .as_array()
        .expect("permissions array");
    assert!(
        permissions.iter().all(|entry| {
            !(entry["contract"] == Value::String("*".into())
                && entry["methods"] == Value::String("*".into()))
        }),
        "NEP11 should not require full wildcard permissions"
    );
}

#[test]
fn devpack_nep24_standard_compiles() {
    let source = [
        include_str!("../../../../devpack/contracts/Syscalls.sol"),
        include_str!("../../../../devpack/contracts/NativeContracts.sol"),
        include_str!("../../../../devpack/contracts/NativeCalls.sol"),
        include_str!("../../../../devpack/libraries/Storage.sol"),
        include_str!("../../../../devpack/libraries/Runtime.sol"),
        include_str!("../../../../devpack/libraries/Neo.sol"),
        include_str!("../../../../devpack/contracts/FrameworkBase.sol"),
        include_str!("../../../../devpack/standards/NEP24.sol"),
    ]
    .join("\n");

    let artifacts = compile_contracts(&source, false, 2).expect("devpack NEP24 compilation failed");
    let names: Vec<_> = artifacts.iter().map(|a| a.metadata.name.as_str()).collect();
    assert_eq!(artifacts.len(), 2, "expected FrameworkBase + NEP24 outputs");
    assert!(names.contains(&"FrameworkBase"));
    assert!(names.contains(&"NEP24Royalty"));

    let nep24 = artifacts
        .iter()
        .find(|a| a.metadata.name == "NEP24Royalty")
        .expect("NEP24Royalty artifact");
    let permissions = nep24.manifest["permissions"]
        .as_array()
        .expect("permissions array");
    assert!(
        permissions.iter().all(|entry| {
            !(entry["contract"] == Value::String("*".into())
                && entry["methods"] == Value::String("*".into()))
        }),
        "NEP24Royalty should not require full wildcard permissions"
    );
}

#[test]
fn devpack_additional_nep_standards_compile() {
    let source = [
        include_str!("../../../../devpack/standards/NEP22.sol"),
        include_str!("../../../../devpack/standards/NEP26.sol"),
        include_str!("../../../../devpack/standards/NEP27.sol"),
        include_str!("../../../../devpack/standards/NEP29.sol"),
        include_str!("../../../../devpack/standards/NEP30.sol"),
        include_str!("../../../../devpack/standards/NEP31.sol"),
        "pragma solidity ^0.8.19; contract _NEPCompileAnchor {}",
    ]
    .join("\n");

    let artifacts =
        compile_contracts(&source, false, 2).expect("additional NEP standards compilation failed");
    assert!(
        artifacts
            .iter()
            .any(|artifact| artifact.metadata.name == "_NEPCompileAnchor"),
        "expected anchor contract artifact"
    );
}

#[test]
fn devpack_nativecontracts_library_compiles_standalone() {
    let source = include_str!("../../../../devpack/contracts/NativeContracts.sol");
    let artifacts =
        compile_contracts(source, false, 2).expect("NativeContracts compilation failed");

    assert!(
        artifacts
            .iter()
            .any(|artifact| artifact.metadata.name == "NativeContracts"),
        "expected NativeContracts artifact"
    );
}

#[test]
fn devpack_syscalls_library_compiles_standalone() {
    let source = [
        include_str!("../../../../devpack/contracts/NativeContracts.sol"),
        include_str!("../../../../devpack/contracts/Syscalls.sol"),
    ]
    .join("\n");
    let artifacts = compile_contracts(&source, false, 2).expect("Syscalls compilation failed");

    assert!(
        artifacts
            .iter()
            .any(|artifact| artifact.metadata.name == "Syscalls"),
        "expected Syscalls artifact"
    );
}

#[test]
fn devpack_nativecalls_library_compiles_with_syscalls() {
    let source = [
        include_str!("../../../../devpack/contracts/Syscalls.sol"),
        include_str!("../../../../devpack/contracts/NativeContracts.sol"),
        include_str!("../../../../devpack/contracts/NativeCalls.sol"),
    ]
    .join("\n");

    let artifacts = compile_contracts(&source, false, 2).expect("NativeCalls compilation failed");

    assert!(
        artifacts
            .iter()
            .any(|artifact| artifact.metadata.name == "NativeCalls"),
        "expected NativeCalls artifact"
    );
}

#[test]
fn devpack_runtime_library_compiles_with_dependencies() {
    let source = [
        include_str!("../../../../devpack/contracts/Syscalls.sol"),
        include_str!("../../../../devpack/contracts/NativeContracts.sol"),
        include_str!("../../../../devpack/contracts/NativeCalls.sol"),
        include_str!("../../../../devpack/libraries/Runtime.sol"),
    ]
    .join("\n");

    let artifacts = compile_contracts(&source, false, 2).expect("Runtime compilation failed");

    assert!(
        artifacts
            .iter()
            .any(|artifact| artifact.metadata.name == "Runtime"),
        "expected Runtime artifact"
    );
}

#[test]
fn devpack_storage_library_compiles_with_dependencies() {
    let source = [
        include_str!("../../../../devpack/contracts/Syscalls.sol"),
        include_str!("../../../../devpack/contracts/NativeContracts.sol"),
        include_str!("../../../../devpack/contracts/NativeCalls.sol"),
        include_str!("../../../../devpack/libraries/Runtime.sol"),
        include_str!("../../../../devpack/libraries/Storage.sol"),
    ]
    .join("\n");

    let artifacts = compile_contracts(&source, false, 2).expect("Storage compilation failed");

    assert!(
        artifacts
            .iter()
            .any(|artifact| artifact.metadata.name == "Storage"),
        "expected Storage artifact"
    );
}


#[test]
fn devpack_complete_nep17_example_compiles_under_strict_manifest_flags() {
    let source = [
        include_str!("../../../../devpack/contracts/Syscalls.sol"),
        include_str!("../../../../devpack/contracts/NativeContracts.sol"),
        include_str!("../../../../devpack/contracts/NativeCalls.sol"),
        include_str!("../../../../devpack/libraries/Storage.sol"),
        include_str!("../../../../devpack/libraries/Runtime.sol"),
        include_str!("../../../../devpack/libraries/Neo.sol"),
        include_str!("../../../../devpack/contracts/FrameworkBase.sol"),
        include_str!("../../../../devpack/standards/NEP17.sol"),
        include_str!("../../../../devpack/examples/CompleteNEP17Token.sol"),
    ]
    .join("\n");

    let artifacts = compile_contracts_with_options(
        &source,
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
    .expect("strict CompleteNEP17 compilation failed");

    assert!(
        artifacts
            .iter()
            .any(|artifact| artifact.metadata.name == "CompleteNEP17Token"),
        "expected CompleteNEP17Token artifact"
    );
}

#[test]
fn devpack_complete_nep11_example_compiles_under_strict_manifest_flags() {
    let source = [
        include_str!("../../../../devpack/contracts/Syscalls.sol"),
        include_str!("../../../../devpack/contracts/NativeContracts.sol"),
        include_str!("../../../../devpack/contracts/NativeCalls.sol"),
        include_str!("../../../../devpack/libraries/Storage.sol"),
        include_str!("../../../../devpack/libraries/Runtime.sol"),
        include_str!("../../../../devpack/libraries/Neo.sol"),
        include_str!("../../../../devpack/contracts/FrameworkBase.sol"),
        include_str!("../../../../devpack/standards/NEP11.sol"),
        include_str!("../../../../devpack/examples/CompleteNEP11NFT.sol"),
    ]
    .join("\n");

    let artifacts = compile_contracts_with_options(
        &source,
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
    .expect("strict CompleteNEP11 compilation failed");

    assert!(
        artifacts
            .iter()
            .any(|artifact| artifact.metadata.name == "CompleteNEP11NFT"),
        "expected CompleteNEP11NFT artifact"
    );
}

#[test]
fn devpack_nativecalls_contract_compiles_under_strict_manifest_flags() {
    let source = [
        include_str!("../../../../devpack/contracts/Syscalls.sol"),
        include_str!("../../../../devpack/contracts/NativeContracts.sol"),
        include_str!("../../../../devpack/contracts/NativeCalls.sol"),
    ]
    .join("\n");

    let artifacts = compile_contracts_with_options(
        &source,
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
    .expect("strict NativeCalls compilation failed");

    assert!(
        artifacts
            .iter()
            .any(|artifact| artifact.metadata.name == "NativeCalls"),
        "expected NativeCalls artifact"
    );
}

#[test]
fn devpack_framework_contract_compiles_under_strict_manifest_flags() {
    let source = [
        include_str!("../../../../devpack/contracts/Syscalls.sol"),
        include_str!("../../../../devpack/contracts/NativeContracts.sol"),
        include_str!("../../../../devpack/contracts/NativeCalls.sol"),
        include_str!("../../../../devpack/libraries/Storage.sol"),
        include_str!("../../../../devpack/libraries/Runtime.sol"),
        include_str!("../../../../devpack/libraries/Neo.sol"),
        include_str!("../../../../devpack/contracts/FrameworkBase.sol"),
        include_str!("../../../../devpack/contracts/Framework.sol"),
    ]
    .join("\n");

    let artifacts = compile_contracts_with_options(
        &source,
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
    .expect("strict Framework compilation failed");

    assert!(
        artifacts
            .iter()
            .any(|artifact| artifact.metadata.name == "Framework"),
        "expected Framework artifact"
    );
}

#[test]
fn devpack_nep17_rescue_contract_compiles_under_strict_manifest_flags() {
    let source = [
        include_str!("../../../../devpack/contracts/Syscalls.sol"),
        include_str!("../../../../devpack/contracts/NativeContracts.sol"),
        include_str!("../../../../devpack/contracts/NativeCalls.sol"),
        include_str!("../../../../devpack/libraries/Storage.sol"),
        include_str!("../../../../devpack/libraries/Runtime.sol"),
        include_str!("../../../../devpack/libraries/Neo.sol"),
        include_str!("../../../../devpack/contracts/FrameworkBase.sol"),
        include_str!("../../../../devpack/standards/NEP17.sol"),
        include_str!("../../../../devpack/contracts/NEP17Rescue.sol"),
    ]
    .join("\n");

    let artifacts = compile_contracts_with_options(
        &source,
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
    .expect("strict NEP17Rescue compilation failed");

    assert!(
        artifacts
            .iter()
            .any(|artifact| artifact.metadata.name == "NEP17Rescue"),
        "expected NEP17Rescue artifact"
    );
}

#[test]
fn devpack_oracle_service_compiles_under_strict_manifest_flags() {
    let source = [
        include_str!("../../../../devpack/contracts/Syscalls.sol"),
        include_str!("../../../../devpack/contracts/NativeContracts.sol"),
        include_str!("../../../../devpack/contracts/NativeCalls.sol"),
        include_str!("../../../../devpack/libraries/Storage.sol"),
        include_str!("../../../../devpack/libraries/Runtime.sol"),
        include_str!("../../../../devpack/libraries/Neo.sol"),
        include_str!("../../../../devpack/contracts/FrameworkBase.sol"),
        include_str!("../../../../devpack/contracts/OracleService.sol"),
    ]
    .join("\n");

    let artifacts = compile_contracts_with_options(
        &source,
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
    .expect("strict OracleService compilation failed");

    assert!(
        artifacts
            .iter()
            .any(|artifact| artifact.metadata.name == "OracleService"),
        "expected OracleService artifact"
    );
}
