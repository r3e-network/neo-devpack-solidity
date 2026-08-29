#[test]
fn native_calls_contract_constants_lower_to_native_hash_bytes() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract NativeCallsConstHarness {
        function value() public pure returns (address) {
            return NativeCalls.GAS_CONTRACT;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let result = execute_bytecode(&artifacts[0].bytecode);
    assert!(result.is_success(), "expected constant-return to succeed");

    let expected = crate::codegen::native_contract_hash(ir::NativeContract::Gas);
    assert_eq!(
        result.return_data,
        expected.to_vec(),
        "expected GAS native contract hash (UInt160 little-endian) on the VM stack"
    );
}

#[test]
fn native_contracts_constants_match_nativecalls_and_syscalls() {
    let checks = [
        "NativeContracts.NEO_CONTRACT == NativeCalls.NEO_CONTRACT",
        "NativeContracts.GAS_CONTRACT == NativeCalls.GAS_CONTRACT",
        "NativeContracts.CONTRACT_MANAGEMENT == NativeCalls.CONTRACT_MANAGEMENT",
        "NativeContracts.POLICY_CONTRACT == NativeCalls.POLICY_CONTRACT",
        "NativeContracts.ORACLE_CONTRACT == NativeCalls.ORACLE_CONTRACT",
        "NativeContracts.ROLE_MANAGEMENT == NativeCalls.ROLE_MANAGEMENT",
        "NativeContracts.NOTARY_CONTRACT == NativeCalls.NOTARY_CONTRACT",
        "NativeContracts.TREASURY_CONTRACT == NativeCalls.TREASURY_CONTRACT",
        "NativeContracts.LEDGER_CONTRACT == NativeCalls.LEDGER_CONTRACT",
        "NativeContracts.CRYPTO_LIB == NativeCalls.CRYPTO_LIB",
        "NativeContracts.STD_LIB == NativeCalls.STD_LIB",
        "NativeContracts.CONTRACT_MANAGEMENT == Syscalls.CONTRACT_MANAGEMENT",
        "NativeContracts.POLICY_CONTRACT == Syscalls.POLICY_CONTRACT",
        "NativeContracts.ORACLE_CONTRACT == Syscalls.ORACLE_CONTRACT",
        "NativeContracts.ROLE_MANAGEMENT == Syscalls.ROLE_MANAGEMENT",
        "NativeContracts.LEDGER_CONTRACT == Syscalls.LEDGER_CONTRACT",
        "NativeContracts.CRYPTO_LIB == Syscalls.CRYPTO_LIB",
        "NativeContracts.STD_LIB == Syscalls.STD_LIB",
    ];

    for check in checks {
        let source = [
            include_str!("../../../../devpack/contracts/NativeContracts.sol"),
            &format!(
                r#"
                pragma solidity ^0.8.19;

                contract NativeContractsParityHarness {{
                    function ok() public pure returns (bool) {{
                        return {check};
                    }}
                }}
                "#
            ),
        ]
        .join("\n");

        let artifacts = compile_contracts(&source, false, 2).expect("compilation failed");
        let harness = artifacts
            .iter()
            .find(|artifact| artifact.metadata.name == "NativeContractsParityHarness")
            .expect("expected NativeContractsParityHarness artifact");

        let result = execute_bytecode(&harness.bytecode);
        assert!(
            result.is_success(),
            "expected parity check execution to succeed: {check}"
        );
        assert_eq!(result.return_data, vec![1u8], "parity mismatch: {check}");
    }
}

#[test]
fn native_contracts_gas_constant_matches_native_hash_bytes() {
    let source = [
        include_str!("../../../../devpack/contracts/NativeContracts.sol"),
        r#"
        pragma solidity ^0.8.19;

        contract NativeContractsConstHarness {
            function value() public pure returns (address) {
                return NativeContracts.GAS_CONTRACT;
            }
        }
        "#,
    ]
    .join("\n");

    let artifacts = compile_contracts(&source, false, 2).expect("compilation failed");
    let harness = artifacts
        .iter()
        .find(|artifact| artifact.metadata.name == "NativeContractsConstHarness")
        .expect("expected NativeContractsConstHarness artifact");

    let result = execute_bytecode(&harness.bytecode);
    assert!(
        result.is_success(),
        "expected NativeContracts constant-return to succeed"
    );

    let expected = crate::codegen::native_contract_hash(ir::NativeContract::Gas);
    assert_eq!(
        result.return_data,
        expected.to_vec(),
        "expected GAS native contract hash bytes for NativeContracts.GAS_CONTRACT"
    );
}

#[test]
fn block_chainid_maps_to_neo_network_magic() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract ChainIdHarness {
        function id() public view returns (uint256) {
            return block.chainid;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let result = execute_bytecode(&artifacts[0].bytecode);
    assert!(result.is_success(), "expected execution to succeed");
    assert_eq!(result.return_data, (0x4F454Eu64).to_le_bytes().to_vec());
}

#[test]
fn external_member_call_returns_native_stack_item() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract ExternalCallHarness {
        function supply() public view returns (uint256) {
            return NativeCalls.GAS_CONTRACT.totalSupply();
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let result = execute_bytecode(&artifacts[0].bytecode);
    assert!(result.is_success(), "expected contract call to succeed");

    assert_eq!(
        result.return_data,
        30_000_000_000u64.to_le_bytes().to_vec(),
        "expected GAS.totalSupply result (u64 little-endian) to be returned directly"
    );
}

#[test]
fn native_contracts_member_call_returns_native_stack_item() {
    let source = [
        include_str!("../../../../devpack/contracts/NativeContracts.sol"),
        r#"
        pragma solidity ^0.8.19;

        contract NativeContractsCallHarness {
            function supply() public view returns (uint256) {
                return NativeContracts.GAS_CONTRACT.totalSupply();
            }
        }
        "#,
    ]
    .join("\n");

    let artifacts = compile_contracts(&source, false, 2).expect("compilation failed");
    let harness = artifacts
        .iter()
        .find(|artifact| artifact.metadata.name == "NativeContractsCallHarness")
        .expect("expected NativeContractsCallHarness artifact");

    let result = execute_bytecode(&harness.bytecode);
    assert!(
        result.is_success(),
        "expected NativeContracts member call to succeed"
    );

    assert_eq!(
        result.return_data,
        30_000_000_000u64.to_le_bytes().to_vec(),
        "expected GAS.totalSupply result (u64 little-endian) for NativeContracts target"
    );
}

#[test]
fn neo_account_state_defaults_when_native_returns_null() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract AccountStateHarness {
        function isDefault() public view returns (bool) {
            NativeCalls.AccountState memory state = NativeCalls.getAccountState(address(1));
            return state.balance == 0
                && state.balanceHeight == 0
                && state.lastGasPerVote == 0
                && state.voteTo.length == 0;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let result = execute_bytecode(&artifacts[0].bytecode);
    let failure = result
        .exception
        .as_ref()
        .map(|ex| ex.message.as_str())
        .unwrap_or("<no exception>");
    assert!(
        result.is_success(),
        "expected account state helper to succeed, got: {failure}"
    );
    assert_eq!(result.return_data, vec![1u8]);
}

#[test]
fn neo_account_state_non_null_path_normalizes_vote_to_bytes() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract AccountStateNonNullHarness {
        function ok() public view returns (bool) {
            address selfHash = Syscalls.getExecutingScriptHash();
            NativeCalls.AccountState memory state = NativeCalls.getAccountState(selfHash);
            // The embedded runtime seeds NEO balance for the executing script hash.
            // The NeoToken.getAccountState response may contain a null voteTo; the compiler
            // helper normalizes that to an empty byte string.
            return state.balance > 0 && state.voteTo.length == 0;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let result = execute_bytecode(&artifacts[0].bytecode);
    let failure = result
        .exception
        .as_ref()
        .map(|ex| ex.message.as_str())
        .unwrap_or("<no exception>");
    assert!(
        result.is_success(),
        "expected non-null account state helper to succeed, got: {failure}"
    );
    assert_eq!(result.return_data, vec![1u8]);
}

#[test]
fn neo_get_candidates_returns_expected_struct_shape() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract CandidatesHarness {
        function ok() public view returns (bool) {
            NativeCalls.NeoCandidate[] memory candidates = NativeCalls.getCandidates();
            return candidates.length > 0
                && candidates[0].votes == 1000
                && candidates[0].publicKey.length == 33;
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let result = execute_bytecode(&artifacts[0].bytecode);
    let failure = result
        .exception
        .as_ref()
        .map(|ex| ex.message.as_str())
        .unwrap_or("<no exception>");
    assert!(
        result.is_success(),
        "expected candidates helper to succeed, got: {failure}"
    );
    assert_eq!(result.return_data, vec![1u8]);
}

#[test]
fn neo_committee_helpers_return_addresses_and_membership() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract CommitteeHarness {
        function firstIsCommittee() public view returns (bool) {
            address[] memory committee = Neo.getCommittee();
            return Neo.isCommittee(committee[0]);
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let result = execute_bytecode(&artifacts[0].bytecode);
    assert!(result.is_success(), "expected committee helper to succeed");
    assert_eq!(result.return_data, vec![1u8]);
}

#[test]
fn neo_validator_helpers_return_addresses_and_membership() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract ValidatorHarness {
        function firstIsValidator() public view returns (bool) {
            address[] memory validators = Neo.getValidators();
            return Neo.isValidator(validators[0]);
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let result = execute_bytecode(&artifacts[0].bytecode);
    assert!(result.is_success(), "expected validator helper to succeed");
    assert_eq!(result.return_data, vec![1u8]);
}

#[test]
fn native_calls_committee_matches_neo_committee_conversion() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract NativeCommitteeHarness {
        function matches() public view returns (bool) {
            address[] memory a = NativeCalls.getCommittee();
            address[] memory b = Neo.getCommittee();
            return a[0] == b[0];
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let result = execute_bytecode(&artifacts[0].bytecode);
    assert!(
        result.is_success(),
        "expected committee conversion to succeed"
    );
    assert_eq!(result.return_data, vec![1u8]);
}

#[test]
fn native_calls_validators_match_neo_validator_conversion() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract NativeValidatorHarness {
        function matches() public view returns (bool) {
            address[] memory a = NativeCalls.getNextBlockValidators();
            address[] memory b = Neo.getValidators();
            return a[0] == b[0];
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let result = execute_bytecode(&artifacts[0].bytecode);
    assert!(
        result.is_success(),
        "expected validator conversion to succeed"
    );
    assert_eq!(result.return_data, vec![1u8]);
}

#[test]
fn runtime_check_multisig_rejects_duplicate_signers() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract RuntimeMultisigDuplicateHarness {
        function run(address[] memory signers) public view returns (bool) {
            return Runtime.checkMultiSigWitness(signers, 1);
        }
    }
    "#;

    let artifacts = compile_contracts(source, false, 2).expect("compilation failed");
    let message = b"Runtime: duplicate signer";
    assert!(
        artifacts[0]
            .bytecode
            .windows(message.len())
            .any(|window| window == message),
        "expected bytecode to include duplicate-signer guard message"
    );
}

#[test]
fn runtime_executes_callt_bytecode_when_tokens_are_loaded() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract CalltRuntimeHarness {
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

    let artifact = &artifacts[0];
    assert!(
        !artifact.tokens.is_empty(),
        "expected method tokens to be emitted when --callt is enabled"
    );

    let result = execute_bytecode_with_tokens(&artifact.bytecode, &artifact.tokens);
    assert!(result.is_success(), "expected CALLT execution to succeed");
    assert_eq!(result.return_data, 30_000_000_000u64.to_le_bytes().to_vec());
}
