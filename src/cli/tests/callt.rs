use super::common::execute_bytecode_with_tokens;
use super::*;

#[test]
fn callt_emits_method_tokens_for_native_calls() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract CallTHarness {
        function supply() public view returns (uint256) {
            return NativeCalls.gasTotalSupply();
        }
    }
    "#;

    let artifacts = compile_contracts_with_options(
        source,
        false,
        CompileOptions {
            optimizer_level: 2,
            use_callt: true,
            deny_wildcard_permissions: false,
            deny_wildcard_contracts: false,
            deny_wildcard_methods: false,
            manifest_permissions: None,
        },
    )
    .expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let artifact = &artifacts[0];
    assert_eq!(
        artifact.tokens.len(),
        1,
        "expected exactly one method token for a single native call"
    );

    let token = &artifact.tokens[0];
    let expected_hash =
        crate::codegen::native_contract_hash(neo_devpack_solidity::ir::NativeContract::Gas);
    assert_eq!(token.hash, expected_hash);
    assert_eq!(token.method, "totalSupply");
    assert_eq!(token.parameters_count, 0);
    assert!(token.has_return_value);
    assert_eq!(token.call_flags, 0x05);

    assert!(
        artifact
            .bytecode
            .windows(3)
            .any(|w| w == [0x37, 0x00, 0x00]),
        "expected CALLT opcode with token index 0"
    );
}

#[test]
fn callt_emits_cryptolib_tokens_for_neo_hash_helpers() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract NeoHashCallTHarness {
        function sha() public view returns (bytes32) {
            return Neo.sha256Hash(hex"616263");
        }

        function ripemd() public view returns (bytes20) {
            return Neo.ripemd160Hash(hex"616263");
        }
    }
    "#;

    let artifacts = compile_contracts_with_options(
        source,
        false,
        CompileOptions {
            optimizer_level: 2,
            use_callt: true,
            deny_wildcard_permissions: false,
            deny_wildcard_contracts: false,
            deny_wildcard_methods: false,
            manifest_permissions: None,
        },
    )
    .expect("compilation failed");
    assert_eq!(artifacts.len(), 1);

    let artifact = &artifacts[0];
    let cryptolib_hash =
        crate::codegen::native_contract_hash(neo_devpack_solidity::ir::NativeContract::CryptoLib);

    let sha_token = artifact
        .tokens
        .iter()
        .find(|token| token.hash == cryptolib_hash && token.method == "sha256")
        .expect("expected CryptoLib.sha256 method token");
    assert_eq!(sha_token.parameters_count, 1);
    assert!(sha_token.has_return_value);
    assert_eq!(sha_token.call_flags, 0x05);

    let ripemd_token = artifact
        .tokens
        .iter()
        .find(|token| token.hash == cryptolib_hash && token.method == "ripemd160")
        .expect("expected CryptoLib.ripemd160 method token");
    assert_eq!(ripemd_token.parameters_count, 1);
    assert!(ripemd_token.has_return_value);
    assert_eq!(ripemd_token.call_flags, 0x05);

    assert!(
        artifact.tokens.iter().all(|token| {
            token.method != "System.Crypto.SHA256" && token.method != "System.Crypto.RIPEMD160"
        }),
        "Neo hash helpers must not emit nonexistent System.Crypto hash method tokens"
    );
}

#[test]
fn callt_executes_neo_hash_helpers_through_cryptolib() {
    fn assert_hash_helper_executes(
        return_type: &str,
        helper: &str,
        native_method: &str,
        expected_hex: &str,
    ) {
        let source = format!(
            r#"
            pragma solidity ^0.8.19;

            contract NeoHashRuntimeHarness {{
                function run() public view returns ({return_type}) {{
                    return Neo.{helper}(hex"616263");
                }}
            }}
            "#
        );

        let artifacts = compile_contracts_with_options(
            &source,
            false,
            CompileOptions {
                optimizer_level: 2,
                use_callt: true,
                deny_wildcard_permissions: false,
                deny_wildcard_contracts: false,
                deny_wildcard_methods: false,
                manifest_permissions: None,
            },
        )
        .expect("compilation failed");
        assert_eq!(artifacts.len(), 1);

        let artifact = &artifacts[0];
        let cryptolib_hash = crate::codegen::native_contract_hash(
            neo_devpack_solidity::ir::NativeContract::CryptoLib,
        );
        assert!(
            artifact.tokens.iter().any(|token| {
                token.hash == cryptolib_hash
                    && token.method == native_method
                    && token.parameters_count == 1
                    && token.has_return_value
                    && token.call_flags == 0x05
            }),
            "expected CryptoLib.{native_method} method token"
        );

        let result = execute_bytecode_with_tokens(&artifact.bytecode, &artifact.tokens);
        assert!(
            result.is_success(),
            "expected CryptoLib.{native_method} CALLT execution to succeed: {:?}",
            result.exception
        );
        assert_eq!(
            result.return_data,
            hex::decode(expected_hex).expect("expected test vector hex"),
            "unexpected CryptoLib.{native_method} digest"
        );
    }

    assert_hash_helper_executes(
        "bytes32",
        "sha256Hash",
        "sha256",
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
    );
    assert_hash_helper_executes(
        "bytes20",
        "ripemd160Hash",
        "ripemd160",
        "8eb208f7e05d987a9b044a8e98c6b087f15a0bfc",
    );
}
