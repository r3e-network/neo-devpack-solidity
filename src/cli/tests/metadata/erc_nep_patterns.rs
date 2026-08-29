// ── Helper: build minimal ContractMetadata for validation tests ───────────

fn build_test_contract(name: &str, methods: Vec<FunctionMetadata>) -> ContractMetadata {
    ContractMetadata {
        name: name.to_string(),
        is_abstract: false,
            is_interface: false,
        is_library: false,
        methods,
        events: vec![],
        errors: vec![],
        uses_storage: false,
        state_variables: vec![],
        structs: vec![],
        enums: vec![],
        contract_types: vec![],
        selector_registry: std::sync::Arc::new(SelectorRegistry::default()),
        documentation: NatspecDoc::default(),
        has_using_for_star: false,
        has_using_function_list: false,
        using_for_libraries: vec![],
        using_directives: vec![],
        has_type_definitions: false,
        type_aliases: std::collections::HashMap::new(),
        flatten_warnings: Vec::new(),
        functions_missing_visibility: Vec::new(),
        super_method_map: std::collections::HashMap::new(),
    }
}

fn build_public_method(name: &str, param_count: usize) -> FunctionMetadata {
    let parameters: Vec<ParameterMetadata> = (0..param_count)
        .map(|i| ParameterMetadata {
            name: Some(format!("arg{i}")),
            ty: "uint256".to_string(),
            neo_type: Some(NeoType::Integer {
                signed: false,
                bits: 256,
            }),
            storage: None,
        })
        .collect();

    FunctionMetadata {
        name: name.to_string(),
        neo_name: name.to_string(),
        kind: FunctionKind::Regular,
        parameters,
        return_parameters: vec![],
        state_mutability: StateMutability::NonPayable,
        visibility: VisibilityKind::Public,
        offset: 0,
        body: None,
        selector: [0u8; 4],
        is_virtual: false,
        is_override: false,
        documentation: NatspecDoc::default(),
        had_modifier_epilogue: false,
    }
}

fn warnings_containing<'a>(diagnostics: &'a [Diagnostic], substring: &str) -> Vec<&'a Diagnostic> {
    diagnostics
        .iter()
        .filter(|d| {
            d.severity == DiagnosticSeverity::Warning && d.message.contains(substring)
        })
        .collect()
}

// ── ERC-20 transfer(to, amount) → NEP-17 transfer(from, to, amount, data) ──

#[test]
fn erc20_transfer_2_params_warns_nep17() {
    let methods = vec![
        build_public_method("transfer", 2),
        build_public_method("balanceOf", 1),
        build_public_method("totalSupply", 0),
    ];
    let metadata = build_test_contract("ERC20Token", methods);
    let diagnostics = validate_contract(&metadata);

    let transfer_warns = warnings_containing(&diagnostics, "NEP-17 requires 4 parameters");
    assert!(
        !transfer_warns.is_empty(),
        "expected warning about ERC-20 transfer(to, amount) needing 4 params for NEP-17, got: {:?}",
        diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn erc20_transfer_3_params_warns_missing_data() {
    let methods = vec![
        build_public_method("transfer", 3),
        build_public_method("balanceOf", 1),
    ];
    let metadata = build_test_contract("PartialNEP17", methods);
    let diagnostics = validate_contract(&metadata);

    let warns = warnings_containing(&diagnostics, "NEP-17 requires 4");
    assert!(
        !warns.is_empty(),
        "expected warning about 3-param transfer missing `data`, got: {:?}",
        diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn nep17_transfer_4_params_no_erc_warning() {
    let methods = vec![
        build_public_method("transfer", 4),
        build_public_method("balanceOf", 1),
        build_public_method("totalSupply", 0),
        build_public_method("symbol", 0),
        build_public_method("decimals", 0),
    ];
    let metadata = build_test_contract("GoodNEP17", methods);
    let diagnostics = validate_contract(&metadata);

    let erc_warns = warnings_containing(&diagnostics, "ERC-20 pattern");
    assert!(
        erc_warns.is_empty(),
        "NEP-17 compliant transfer should not trigger ERC-20 warning, got: {:?}",
        erc_warns.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

// ── ERC-20 approve/allowance/transferFrom → not in NEP-17 ──────────────────

#[test]
fn erc20_approve_pattern_warns_with_erc20_transfer_shape() {
    let methods = vec![
        build_public_method("transfer", 2),
        build_public_method("balanceOf", 1),
        build_public_method("approve", 2),
        build_public_method("allowance", 2),
    ];
    let metadata = build_test_contract("ERC20WithApprove", methods);
    let diagnostics = validate_contract(&metadata);

    let warns = warnings_containing(&diagnostics, "not part of the NEP-17 spec");
    assert!(
        !warns.is_empty(),
        "expected warning about approve/allowance not being in NEP-17, got: {:?}",
        diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn approve_with_nep17_transfer_no_warning() {
    // Keep approve/allowance as optional extensions on top of canonical NEP-17.
    let methods = vec![
        build_public_method("transfer", 4),
        build_public_method("balanceOf", 1),
        build_public_method("approve", 2),
        build_public_method("allowance", 2),
    ];
    let metadata = build_test_contract("NEP17WithApproveExtension", methods);
    let diagnostics = validate_contract(&metadata);

    let warns = warnings_containing(&diagnostics, "not part of the NEP-17 spec");
    assert!(
        warns.is_empty(),
        "approve/allowance with canonical NEP-17 transfer should not warn, got: {:?}",
        warns.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn approve_without_token_signal_no_warning() {
    // Contract with approve but no balanceOf/transfer → not a token, no warning
    let methods = vec![
        build_public_method("approve", 2),
        build_public_method("doSomething", 1),
    ];
    let metadata = build_test_contract("NotAToken", methods);
    let diagnostics = validate_contract(&metadata);

    let warns = warnings_containing(&diagnostics, "NEP-17 spec");
    assert!(
        warns.is_empty(),
        "approve without token signals should not warn, got: {:?}",
        warns.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

// ── ERC-721 transferFrom → NEP-11 transfer ─────────────────────────────────

#[test]
fn erc721_transferfrom_warns_nep11() {
    let methods = vec![
        build_public_method("ownerOf", 1),
        build_public_method("balanceOf", 1),
        build_public_method("transferFrom", 3),
    ];
    let metadata = build_test_contract("ERC721Token", methods);
    let diagnostics = validate_contract(&metadata);

    let warns = warnings_containing(&diagnostics, "NEP-11 uses transfer");
    assert!(
        !warns.is_empty(),
        "expected warning about ERC-721 transferFrom → NEP-11 transfer, got: {:?}",
        diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn transferfrom_with_nep11_transfer_no_warning() {
    let methods = vec![
        build_public_method("ownerOf", 1),
        build_public_method("balanceOf", 1),
        build_public_method("transfer", 3),
        build_public_method("transferFrom", 3),
    ];
    let metadata = build_test_contract("NEP11WithCompatTransferFrom", methods);
    let diagnostics = validate_contract(&metadata);

    let warns = warnings_containing(&diagnostics, "NEP-11 uses transfer");
    assert!(
        warns.is_empty(),
        "NEP-11 transfer + transferFrom compatibility should not warn, got: {:?}",
        warns.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn transferfrom_without_ownerof_no_nft_warning() {
    // transferFrom without ownerOf → not an NFT contract, no NFT-specific warning
    let methods = vec![
        build_public_method("transferFrom", 3),
        build_public_method("balanceOf", 1),
    ];
    let metadata = build_test_contract("NotNFT", methods);
    let diagnostics = validate_contract(&metadata);

    let warns = warnings_containing(&diagnostics, "NEP-11 uses transfer");
    assert!(
        warns.is_empty(),
        "transferFrom without ownerOf should not trigger NFT warning, got: {:?}",
        warns.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

// ── receive() / fallback() → onNEP17Payment() ─────────────────────────────

#[test]
fn receive_function_warns_onnep17payment() {
    let methods = vec![build_public_method("receive", 0)];
    let metadata = build_test_contract("ReceiverContract", methods);
    let diagnostics = validate_contract(&metadata);

    let warns = warnings_containing(&diagnostics, "onNEP17Payment");
    assert!(
        !warns.is_empty(),
        "expected warning about receive() → onNEP17Payment, got: {:?}",
        diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn fallback_function_warns_onnep17payment() {
    let methods = vec![build_public_method("fallback", 0)];
    let metadata = build_test_contract("FallbackContract", methods);
    let diagnostics = validate_contract(&metadata);

    let warns = warnings_containing(&diagnostics, "onNEP17Payment");
    assert!(
        !warns.is_empty(),
        "expected warning about fallback() → onNEP17Payment, got: {:?}",
        diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn receive_with_existing_onnep17payment_is_a_hard_error() {
    // M-FE1 fix: defining both receive() and an explicit onNEP17Payment makes
    // receive() dead code (Neo N3 only invokes onNEP17Payment). The validator
    // must emit a hard ERROR, not a warning.
    let methods = vec![
        build_public_method("receive", 0),
        build_public_method("onNEP17Payment", 3),
    ];
    let metadata = build_test_contract("DualCallback", methods);
    let diagnostics = validate_contract(&metadata);

    let errors: Vec<&Diagnostic> = diagnostics
        .iter()
        .filter(|d| {
            d.severity == DiagnosticSeverity::Error && d.message.contains("dead code")
        })
        .collect();
    assert!(
        !errors.is_empty(),
        "M-FE1: expected a 'dead code' ERROR when both receive and onNEP17Payment exist, got: {:?}",
        diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

// ── supportsInterface(bytes4) → manifest-based ─────────────────────────────

#[test]
fn supports_interface_warns_manifest() {
    let methods = vec![build_public_method("supportsInterface", 1)];
    let metadata = build_test_contract("EIP165Contract", methods);
    let diagnostics = validate_contract(&metadata);

    let warns = warnings_containing(&diagnostics, "unnecessary on Neo N3");
    assert!(
        !warns.is_empty(),
        "expected warning about supportsInterface being unnecessary, got: {:?}",
        diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

// ── Clean NEP-17 contract produces no ERC pattern warnings ─────────────────

#[test]
fn clean_nep17_contract_no_erc_pattern_warnings() {
    let methods = vec![
        build_public_method("symbol", 0),
        build_public_method("decimals", 0),
        build_public_method("totalSupply", 0),
        build_public_method("balanceOf", 1),
        build_public_method("transfer", 4),
        build_public_method("onNEP17Payment", 3),
    ];
    let metadata = build_test_contract("CleanNEP17", methods);
    let diagnostics = validate_contract(&metadata);

    let erc_warns: Vec<_> = diagnostics
        .iter()
        .filter(|d| {
            d.severity == DiagnosticSeverity::Warning
                && (d.message.contains("ERC-20")
                    || d.message.contains("ERC-721")
                    || d.message.contains("EIP-165")
                    || d.message.contains("no effect"))
        })
        .collect();

    assert!(
        erc_warns.is_empty(),
        "clean NEP-17 contract should have no ERC pattern warnings, got: {:?}",
        erc_warns.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

// ── ERC-1155 multi-token pattern detection ──────────────────────────────

#[test]
fn erc1155_safe_transfer_from_warns() {
    let methods = vec![
        build_public_method("safeTransferFrom", 5),
        build_public_method("balanceOf", 2),
    ];
    let metadata = build_test_contract("ERC1155Token", methods);
    let diagnostics = validate_contract(&metadata);

    let warns = warnings_containing(&diagnostics, "ERC-1155");
    assert!(
        !warns.is_empty(),
        "expected ERC-1155 warning for safeTransferFrom(5 params), got: {:?}",
        diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn erc1155_safe_batch_transfer_warns() {
    let methods = vec![
        build_public_method("safeBatchTransferFrom", 5),
        build_public_method("balanceOfBatch", 2),
    ];
    let metadata = build_test_contract("ERC1155Batch", methods);
    let diagnostics = validate_contract(&metadata);

    let warns = warnings_containing(&diagnostics, "ERC-1155");
    assert!(
        !warns.is_empty(),
        "expected ERC-1155 warning for safeBatchTransferFrom(5 params), got: {:?}",
        diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn erc1155_wrong_param_count_no_warning() {
    // safeTransferFrom with 3 params is ERC-721 style, not ERC-1155
    let methods = vec![build_public_method("safeTransferFrom", 3)];
    let metadata = build_test_contract("NotERC1155", methods);
    let diagnostics = validate_contract(&metadata);

    let warns = warnings_containing(&diagnostics, "ERC-1155");
    assert!(
        warns.is_empty(),
        "safeTransferFrom with 3 params should not trigger ERC-1155 warning, got: {:?}",
        warns.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

// ── ERC-2612 permit pattern detection ───────────────────────────────────

#[test]
fn erc2612_permit_7_params_warns() {
    let methods = vec![
        build_public_method("permit", 7),
        build_public_method("transfer", 2),
        build_public_method("balanceOf", 1),
    ];
    let metadata = build_test_contract("ERC2612Token", methods);
    let diagnostics = validate_contract(&metadata);

    let warns = warnings_containing(&diagnostics, "ERC-2612");
    assert!(
        !warns.is_empty(),
        "expected ERC-2612 warning for permit(7 params), got: {:?}",
        diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn permit_wrong_param_count_no_erc2612_warning() {
    // permit with 2 params is not ERC-2612
    let methods = vec![build_public_method("permit", 2)];
    let metadata = build_test_contract("CustomPermit", methods);
    let diagnostics = validate_contract(&metadata);

    let warns = warnings_containing(&diagnostics, "ERC-2612");
    assert!(
        warns.is_empty(),
        "permit with 2 params should not trigger ERC-2612 warning, got: {:?}",
        warns.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

// ── ERC-4626 vault pattern detection ────────────────────────────────────

#[test]
fn erc4626_vault_pattern_warns() {
    let methods = vec![
        build_public_method("deposit", 2),
        build_public_method("withdraw", 3),
        build_public_method("convertToShares", 1),
        build_public_method("convertToAssets", 1),
        build_public_method("totalAssets", 0),
    ];
    let metadata = build_test_contract("ERC4626Vault", methods);
    let diagnostics = validate_contract(&metadata);

    let warns = warnings_containing(&diagnostics, "ERC-4626");
    assert!(
        !warns.is_empty(),
        "expected ERC-4626 warning for vault pattern, got: {:?}",
        diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn deposit_without_vault_methods_no_erc4626_warning() {
    // deposit alone without convertToShares/convertToAssets is not ERC-4626
    let methods = vec![
        build_public_method("deposit", 2),
        build_public_method("withdraw", 3),
    ];
    let metadata = build_test_contract("SimpleBank", methods);
    let diagnostics = validate_contract(&metadata);

    let warns = warnings_containing(&diagnostics, "ERC-4626");
    assert!(
        warns.is_empty(),
        "deposit+withdraw without convert methods should not trigger ERC-4626, got: {:?}",
        warns.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

// ── BN254 precompile usage detection ────────────────────────────────────

#[test]
fn bn254_ecadd_function_warns() {
    let methods = vec![build_public_method("ecAdd", 4)];
    let metadata = build_test_contract("BN254User", methods);
    let diagnostics = validate_contract(&metadata);

    let warns = warnings_containing(&diagnostics, "BN254");
    assert!(
        !warns.is_empty(),
        "expected BN254 warning for ecAdd function, got: {:?}",
        diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn bn254_ecpairing_function_warns() {
    let methods = vec![build_public_method("ecPairing", 1)];
    let metadata = build_test_contract("PairingCheck", methods);
    let diagnostics = validate_contract(&metadata);

    let warns = warnings_containing(&diagnostics, "BN254");
    assert!(
        !warns.is_empty(),
        "expected BN254 warning for ecPairing function, got: {:?}",
        diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn normal_function_no_bn254_warning() {
    let methods = vec![
        build_public_method("transfer", 4),
        build_public_method("verify", 2),
    ];
    let metadata = build_test_contract("NormalContract", methods);
    let diagnostics = validate_contract(&metadata);

    let warns = warnings_containing(&diagnostics, "BN254");
    assert!(
        warns.is_empty(),
        "normal functions should not trigger BN254 warning, got: {:?}",
        warns.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

// ── Phase 5: Diagnostic code and suggestion field tests ─────────────────────

#[test]
fn erc20_transfer_diagnostic_has_code_and_suggestion() {
    let methods = vec![build_public_method("transfer", 2)];
    let metadata = build_test_contract("ERC20", methods);
    let diagnostics = validate_contract(&metadata);

    let warns = warnings_containing(&diagnostics, "ERC-20 pattern");
    assert!(!warns.is_empty(), "expected ERC-20 transfer warning");
    let diag = warns[0];
    assert!(
        diag.code.is_some(),
        "diagnostic should have a code, got None"
    );
    assert_eq!(diag.code.as_deref(), Some("W101"));
    assert!(
        diag.suggestion.is_some(),
        "diagnostic should have a suggestion, got None"
    );
}

#[test]
fn erc20_approve_diagnostic_has_code_w103() {
    // W103 requires: token signal (balanceof/transfer), no ownerOf, no NEP-17 transfer(4 params)
    let methods = vec![
        build_public_method("transfer", 2),
        build_public_method("balanceOf", 1),
        build_public_method("approve", 2),
        build_public_method("allowance", 2),
    ];
    let metadata = build_test_contract("ERC20Approve", methods);
    let diagnostics = validate_contract(&metadata);

    let warns = warnings_containing(&diagnostics, "approve");
    assert!(!warns.is_empty(), "expected approve/allowance warning");
    let diag = warns[0];
    assert_eq!(diag.code.as_deref(), Some("W103"));
    assert!(diag.suggestion.is_some());
}

#[test]
fn erc721_transfer_from_diagnostic_has_code_w104() {
    // W104 requires: ownerOf present (NFT signal), no NEP-11 transfer(3 params)
    let methods = vec![
        build_public_method("ownerOf", 1),
        build_public_method("transferFrom", 3),
    ];
    let metadata = build_test_contract("ERC721", methods);
    let diagnostics = validate_contract(&metadata);

    let warns = warnings_containing(&diagnostics, "transferFrom");
    assert!(!warns.is_empty(), "expected ERC-721 transferFrom warning");
    let diag = warns[0];
    assert_eq!(diag.code.as_deref(), Some("W104"));
    assert!(diag.suggestion.is_some());
}

#[test]
fn receive_fallback_diagnostic_has_code_w105() {
    let methods = vec![build_public_method("receive", 0)];
    let metadata = build_test_contract("Receiver", methods);
    let diagnostics = validate_contract(&metadata);

    let warns = warnings_containing(&diagnostics, "receive");
    assert!(!warns.is_empty(), "expected receive/fallback warning");
    let diag = warns[0];
    assert_eq!(diag.code.as_deref(), Some("W105"));
    assert!(diag.suggestion.is_some());
}

#[test]
fn supports_interface_diagnostic_has_code_w106() {
    let methods = vec![build_public_method("supportsInterface", 1)];
    let metadata = build_test_contract("ERC165", methods);
    let diagnostics = validate_contract(&metadata);

    let warns = warnings_containing(&diagnostics, "supportsInterface");
    assert!(!warns.is_empty(), "expected supportsInterface warning");
    let diag = warns[0];
    assert_eq!(diag.code.as_deref(), Some("W106"));
    assert!(diag.suggestion.is_some());
}

#[test]
fn all_diagnostics_with_codes_have_suggestions() {
    // Verify the invariant: every diagnostic that has a code also has a suggestion.
    let methods = vec![
        build_public_method("transfer", 2),
        build_public_method("approve", 2),
        build_public_method("allowance", 2),
        build_public_method("transferFrom", 3),
        build_public_method("receive", 0),
        build_public_method("supportsInterface", 1),
    ];
    let metadata = build_test_contract("AllPatterns", methods);
    let diagnostics = validate_contract(&metadata);

    for diag in &diagnostics {
        if diag.code.is_some() {
            assert!(
                diag.suggestion.is_some(),
                "diagnostic with code {:?} should have a suggestion: {}",
                diag.code,
                diag.message
            );
        }
    }
}
