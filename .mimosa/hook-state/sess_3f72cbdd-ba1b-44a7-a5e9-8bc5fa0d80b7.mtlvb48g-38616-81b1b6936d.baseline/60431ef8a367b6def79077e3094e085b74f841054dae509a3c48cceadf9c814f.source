use super::*;

/// ERC → NEP pattern adaptation diagnostics.
///
/// Detects Ethereum-style patterns in Solidity contracts and emits warnings
/// with actionable guidance for migrating to Neo N3 equivalents.
///
/// Checked patterns:
/// - ERC-20 `transfer(to, amount)` → NEP-17 `transfer(from, to, amount, data)`
/// - ERC-20 `approve`/`allowance`/`transferFrom` → not in NEP-17 spec
/// - ERC-721 `transferFrom(from, to, tokenId)` → NEP-11 `transfer(to, tokenId, data)`
/// - `receive()` / `fallback()` → `onNEP17Payment()` callback
/// - `supportsInterface(bytes4)` → manifest `supportedstandards`
pub(crate) fn validate_erc_nep_patterns(
    metadata: &ContractMetadata,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let public_methods: Vec<&FunctionMetadata> = metadata
        .methods
        .iter()
        .filter(|m| {
            !matches!(m.kind, FunctionKind::Constructor)
                && matches!(
                    m.visibility,
                    VisibilityKind::Public | VisibilityKind::External
                )
        })
        .collect();

    let names_lower: std::collections::HashSet<String> = public_methods
        .iter()
        .map(|m| m.name.to_ascii_lowercase())
        .collect();

    check_erc20_transfer_pattern(&public_methods, &names_lower, diagnostics);
    check_erc20_approve_pattern(&public_methods, &names_lower, diagnostics);
    check_erc721_transfer_from_pattern(&public_methods, &names_lower, diagnostics);
    check_receive_fallback_pattern(&metadata.methods, diagnostics);
    check_supports_interface_pattern(&public_methods, diagnostics);
    check_bn254_precompile_usage(&public_methods, diagnostics);
    check_erc1155_pattern(&public_methods, diagnostics);
    check_erc2612_permit_pattern(&public_methods, diagnostics);
    check_erc4626_vault_pattern(&public_methods, &names_lower, diagnostics);
    check_nep14_multitoken_pattern(&public_methods, &names_lower, diagnostics);
    check_payment_callback(
        &public_methods,
        &names_lower,
        &metadata.methods,
        diagnostics,
    );
    check_nft_payment_callback(
        &public_methods,
        &names_lower,
        &metadata.methods,
        diagnostics,
    );
    check_payable_modifier(&public_methods, diagnostics);
    check_block_timestamp_dependency(&public_methods, diagnostics);
}

/// Detect ERC-20 style `transfer(address, uint256)` and suggest NEP-17 4-param form.
fn check_erc20_transfer_pattern(
    public_methods: &[&FunctionMetadata],
    names: &std::collections::HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let has_ownerof = names.contains("ownerof");
    // Only flag for fungible-token-like contracts (no ownerOf → not NFT)
    if has_ownerof {
        return;
    }

    if let Some(transfer) = public_methods
        .iter()
        .find(|m| m.name.eq_ignore_ascii_case("transfer"))
    {
        let param_count = transfer.parameters.len();
        if param_count == 2 {
            // Classic ERC-20: transfer(address to, uint256 amount)
            diagnostics.push(Diagnostic::warning(
                "function 'transfer' has 2 parameters (ERC-20 pattern). \
                 NEP-17 requires 4 parameters: transfer(from, to, amount, data). \
                 The `from` address is verified via Runtime.checkWitness() and \
                 `data` (type Any) is forwarded to the recipient's onNEP17Payment callback."
            ).with_code("W101")
             .with_suggestion("Add `from` and `data` parameters: `transfer(address from, address to, uint256 amount, bytes data)`"));
        } else if param_count == 3 {
            // Partial migration: transfer(from, to, amount) — missing `data`
            diagnostics.push(Diagnostic::warning(
                "function 'transfer' has 3 parameters, but NEP-17 requires 4: \
                 transfer(from, to, amount, data). The `data` parameter (type Any) \
                 is forwarded to the recipient's onNEP17Payment callback."
            ).with_code("W102")
             .with_suggestion("Add `data` parameter for NEP-17 compliance: `transfer(address from, address to, uint256 amount, bytes data)`"));
        }
    }
}

/// Detect ERC-20 approve/allowance/transferFrom and note they are not in NEP-17.
fn check_erc20_approve_pattern(
    public_methods: &[&FunctionMetadata],
    names: &std::collections::HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let erc20_extras: Vec<&str> = ["approve", "allowance", "transferfrom"]
        .iter()
        .filter(|n| names.contains(**n))
        .copied()
        .collect();

    if !erc20_extras.is_empty() {
        // Only warn if this still looks like an ERC-20-style token surface.
        // If a contract already exposes canonical NEP-17 transfer(from,to,amount,data)
        // or is NFT-shaped (`ownerOf`), treat approve/allowance/transferFrom as
        // compatibility extensions instead of migration warnings.
        let has_token_signal = names.contains("balanceof") || names.contains("transfer");
        let has_ownerof = names.contains("ownerof");
        let has_nep17_transfer = public_methods
            .iter()
            .any(|m| m.name.eq_ignore_ascii_case("transfer") && m.parameters.len() == 4);

        if has_token_signal && !has_ownerof && !has_nep17_transfer {
            diagnostics.push(Diagnostic::warning(format!(
                "ERC-20 method(s) [{}] detected. These are not part of the NEP-17 spec; \
                 Neo uses Runtime.checkWitness() for authorization instead of the \
                 approve/allowance pattern. You may keep them as extensions, but they \
                 will not contribute to NEP-17 standard detection.",
                erc20_extras.join(", ")
            )).with_code("W103")
             .with_suggestion("Remove approve/allowance or keep as optional extension alongside NEP-17 transfer"));
        }
    }
}

/// Detect ERC-721 `transferFrom(from, to, tokenId)` and suggest NEP-11 `transfer(to, tokenId, data)`.
fn check_erc721_transfer_from_pattern(
    public_methods: &[&FunctionMetadata],
    names: &std::collections::HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let has_ownerof = names.contains("ownerof");
    if !has_ownerof {
        return; // Not an NFT contract
    }

    let has_nep11_transfer = public_methods
        .iter()
        .any(|m| m.name.eq_ignore_ascii_case("transfer") && m.parameters.len() == 3);

    if has_nep11_transfer {
        return;
    }

    if let Some(xfer_from) = public_methods
        .iter()
        .find(|m| m.name.eq_ignore_ascii_case("transferfrom"))
    {
        let param_count = xfer_from.parameters.len();
        if param_count == 3 {
            diagnostics.push(
                Diagnostic::warning(
                    "function 'transferFrom' with 3 parameters (ERC-721 pattern) detected. \
                 NEP-11 uses transfer(to, tokenId, data) with 3 parameters instead. \
                 Authorization is via Runtime.checkWitness(owner), not msg.sender.",
                )
                .with_code("W104")
                .with_suggestion(
                    "Replace `transferFrom(from, to, id)` with `transfer(to, id, data)`",
                ),
            );
        }
    }
}

/// Detect `receive()` / `fallback()` and suggest `onNEP17Payment()`.
fn check_receive_fallback_pattern(
    all_methods: &[FunctionMetadata],
    diagnostics: &mut Vec<Diagnostic>,
) {
    // Distinguishing a user-written onNEP17Payment from the compiler-generated
    // one: the convert phase remaps a source `receive()` into an
    // `onNEP17Payment` ONLY when no explicit onNEP17Payment exists; when one
    // DOES exist, the `receive()` is kept verbatim (see
    // `src/solidity/convert/functions.rs`). So after convert:
    //   - `receive` survives in `all_methods` ⟺ there's a user-written
    //     onNEP17Payment (otherwise the receive would have been remapped into
    //     it).
    // The fallback case is trickier — fallback is never auto-remapped, so a
    // surviving `fallback` + an `onNEP17Payment` could be the remapped receive.
    // To stay sound we only treat the (receive + onNEP17Payment) coexist case
    // as the hard error, and leave fallback+onNEP17Payment at a warning.
    let has_onnep17 = all_methods
        .iter()
        .any(|m| m.name.eq_ignore_ascii_case("onnep17payment"));
    // A surviving `receive` method means convert kept it because an explicit
    // onNEP17Payment already existed — the dead-code trap is real.
    let has_surviving_receive = all_methods
        .iter()
        .any(|m| m.name.eq_ignore_ascii_case("receive"));

    for method in all_methods {
        let name_lower = method.name.to_ascii_lowercase();
        if name_lower == "receive" || name_lower == "fallback" {
            // M-FE1 hard error: only when `receive()` specifically survived
            // convert alongside an explicit onNEP17Payment (the unambiguous
            // dead-code case). fallback() stays a warning — its coexist
            // ambiguity is documented in the audit but less dangerous.
            let is_hard_error = name_lower == "receive" && has_onnep17 && has_surviving_receive;
            if is_hard_error {
                diagnostics.push(
                    Diagnostic::error(format!(
                        "function '{}' is dead code on Neo N3: the contract already defines \
                         onNEP17Payment, which is the ONLY callback Neo N3 invokes for incoming \
                         NEP-17 transfers. The '{}' body would never execute — remove it and \
                         consolidate the deposit-handling logic into onNEP17Payment.",
                        method.name, method.name
                    ))
                    .with_code("E105")
                    .with_suggestion(
                        "Remove the receive()/fallback() and move its logic into \
                         onNEP17Payment(address from, uint256 amount, bytes data)",
                    ),
                );
            } else if has_onnep17 {
                diagnostics.push(
                    Diagnostic::warning(format!(
                        "function '{}' has no effect on Neo N3. The contract already defines \
                         onNEP17Payment which is the correct Neo callback for receiving tokens.",
                        method.name
                    ))
                    .with_code("W105")
                    .with_suggestion("Remove — the existing onNEP17Payment handler is sufficient"),
                );
            } else {
                diagnostics.push(Diagnostic::warning(format!(
                    "function '{}' has no effect on Neo N3. Use onNEP17Payment(address from, \
                     uint256 amount, bytes data) to handle incoming token payments.",
                    method.name
                )).with_code("W105")
                 .with_suggestion("Replace with `function onNEP17Payment(address from, uint256 amount, bytes data)`"));
            }
        }
    }
}

/// Detect `supportsInterface(bytes4)` and note that Neo uses manifest instead.
fn check_supports_interface_pattern(
    public_methods: &[&FunctionMetadata],
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Some(si) = public_methods
        .iter()
        .find(|m| m.name == "supportsInterface")
    {
        if si.parameters.len() == 1 {
            diagnostics.push(
                Diagnostic::warning(
                    "function 'supportsInterface' (EIP-165) is unnecessary on Neo N3. \
                 Neo uses the manifest 'supportedstandards' array for interface \
                 detection, which the compiler populates automatically.",
                )
                .with_code("W106")
                .with_suggestion("Remove — Neo N3 uses manifest-based interface discovery"),
            );
        }
    }
}

/// Detect ERC-1155 multi-token pattern and note Neo N3 has no direct equivalent.
fn check_erc1155_pattern(public_methods: &[&FunctionMetadata], diagnostics: &mut Vec<Diagnostic>) {
    let has_safe_transfer = public_methods
        .iter()
        .any(|m| m.name.eq_ignore_ascii_case("safeTransferFrom") && m.parameters.len() == 5);
    let has_batch_transfer = public_methods
        .iter()
        .any(|m| m.name.eq_ignore_ascii_case("safeBatchTransferFrom") && m.parameters.len() == 5);

    if has_safe_transfer || has_batch_transfer {
        diagnostics.push(
            Diagnostic::warning(
                "ERC-1155 multi-token pattern detected. Neo N3 does not have a direct \
             NEP equivalent for multi-token contracts.",
            )
            .with_code("W107")
            .with_suggestion(
                "Split into separate NEP-17 (fungible) and NEP-11 (non-fungible) contracts",
            ),
        );
    }
}

/// Detect ERC-2612 permit pattern and note Neo uses checkWitness instead.
fn check_erc2612_permit_pattern(
    public_methods: &[&FunctionMetadata],
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Some(permit) = public_methods
        .iter()
        .find(|m| m.name.eq_ignore_ascii_case("permit"))
    {
        if permit.parameters.len() == 7 {
            diagnostics.push(
                Diagnostic::warning(
                    "ERC-2612 permit pattern detected (7-parameter permit function). \
                 Neo N3 uses Runtime.checkWitness() for authorization; off-chain \
                 signature permits are not needed.",
                )
                .with_code("W108")
                .with_suggestion("Use `Runtime.checkWitness()` instead of off-chain signatures"),
            );
        }
    }
}

/// Detect ERC-4626 tokenized vault pattern and suggest NEP-17 replacement.
fn check_erc4626_vault_pattern(
    public_methods: &[&FunctionMetadata],
    names: &std::collections::HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let has_deposit = public_methods
        .iter()
        .any(|m| m.name.eq_ignore_ascii_case("deposit") && m.parameters.len() == 2);
    let has_withdraw = public_methods
        .iter()
        .any(|m| m.name.eq_ignore_ascii_case("withdraw") && m.parameters.len() == 3);
    let has_convert_shares = names.contains("converttoshares");
    let has_convert_assets = names.contains("converttoassets");

    if has_deposit && has_withdraw && (has_convert_shares || has_convert_assets) {
        diagnostics.push(Diagnostic::warning(
            "ERC-4626 tokenized vault pattern detected. The vault logic compiles \
             correctly, but replace ERC-20 token interactions with NEP-17 equivalents."
        ).with_code("W109")
         .with_suggestion("Replace ERC-20 interactions with NEP-17 equivalents; use Runtime.checkWitness() for authorization"));
    }
}

/// Detect BN254 elliptic curve precompile usage and suggest Neo alternatives.
fn check_bn254_precompile_usage(
    public_methods: &[&FunctionMetadata],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let bn254_indicators = [
        "ecadd",
        "ecmul",
        "ecpairing",
        "bn256add",
        "bn256scalarmul",
        "bn256pairing",
    ];
    for method in public_methods {
        let name_lower = method.name.to_ascii_lowercase();
        if bn254_indicators.iter().any(|ind| name_lower.contains(ind)) {
            diagnostics.push(Diagnostic::warning(format!(
                "function '{}' appears to use BN254 elliptic curve operations (ecAdd/ecMul/ecPairing). \
                 These precompiles (addresses 0x06, 0x07, 0x08) are not available on Neo N3.",
                method.name
            )).with_code("W110")
             .with_suggestion("Use CryptoLib BLS12-381 operations instead"));
        }
    }
}

/// Detect NEP-14 (Multi-Token) pattern and suggest splitting to NEP-17/NEP-11.
fn check_nep14_multitoken_pattern(
    public_methods: &[&FunctionMetadata],
    names: &std::collections::HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let has_balanceof_batch = public_methods
        .iter()
        .any(|m| m.name.eq_ignore_ascii_case("balanceOfBatch"));
    let has_safe_batch_transfer = public_methods
        .iter()
        .any(|m| m.name.eq_ignore_ascii_case("safeBatchTransferFrom"));
    let has_uri = names.contains("uri");

    if has_balanceof_batch || (has_safe_batch_transfer && has_uri) {
        diagnostics.push(
            Diagnostic::warning(
                "NEP-14 multi-token pattern detected. Neo N3 does not have a direct \
             NEP-14 equivalent. Consider splitting into separate NEP-17 (fungible) \
             and NEP-11 (non-fungible) contracts.",
            )
            .with_code("W111")
            .with_suggestion("Deploy separate NEP-17 and NEP-11 contracts"),
        );
    }
}

/// Detect missing onNEP17Payment in contracts that handle payments.
fn check_payment_callback(
    _public_methods: &[&FunctionMetadata],
    names: &std::collections::HashSet<String>,
    all_methods: &[FunctionMetadata],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let has_transfer = names.contains("transfer");
    let has_onnep17payment = all_methods
        .iter()
        .any(|m| m.name.eq_ignore_ascii_case("onnep17payment"));

    if has_transfer && !has_onnep17payment {
        diagnostics.push(
            Diagnostic::warning(
                "Contract has transfer function but no onNEP17Payment callback. \
             Other contracts cannot send tokens to this contract.",
            )
            .with_code("W113")
            .with_suggestion(
                "Add onNEP17Payment(address from, uint256 amount, bytes data) callback",
            ),
        );
    }
}

/// Detect missing onNEP11Payment in NFT contracts.
fn check_nft_payment_callback(
    _public_methods: &[&FunctionMetadata],
    names: &std::collections::HashSet<String>,
    all_methods: &[FunctionMetadata],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let has_ownerof = names.contains("ownerof");
    let has_onnep11payment = all_methods
        .iter()
        .any(|m| m.name.eq_ignore_ascii_case("onnep11payment"));

    if has_ownerof && !has_onnep11payment {
        diagnostics.push(
            Diagnostic::warning(
                "NFT contract (has ownerOf) but missing onNEP11Payment callback. \
             Other contracts cannot send NFTs to this contract.",
            )
            .with_code("W114")
            .with_suggestion(
                // NEP-11 / NEP-26 token IDs are ByteStrings (`bytes`), not fixed
                // 32-byte words — the devpack's own INEP11Receiver uses `bytes`,
                // and `bytes32` would mis-encode any non-32-byte id.
                "Add onNEP11Payment(address from, uint256 amount, bytes tokenId, bytes data)",
            ),
        );
    }
}

/// Detect payable modifier usage and warn about Neo N3 differences.
fn check_payable_modifier(public_methods: &[&FunctionMetadata], diagnostics: &mut Vec<Diagnostic>) {
    use crate::solidity::StateMutability;

    for method in public_methods {
        // Check if function is marked payable but is not onNEP17Payment
        if method.state_mutability == StateMutability::Payable
            && !method.name.eq_ignore_ascii_case("onnep17payment")
            && !method.name.eq_ignore_ascii_case("onnep11payment")
        {
            diagnostics.push(
                Diagnostic::warning(format!(
                    "function '{}' has payable modifier which has no effect on Neo N3. \
                 Use onNEP17Payment callback to receive token payments.",
                    method.name
                ))
                .with_code("W116")
                .with_suggestion(
                    "Remove payable or implement onNEP17Payment(address, uint256, bytes)",
                ),
            );
        }
    }
}

/// Detect heavy dependence on block.timestamp which can be manipulated in Neo N3.
fn check_block_timestamp_dependency(
    public_methods: &[&FunctionMetadata],
    diagnostics: &mut Vec<Diagnostic>,
) {
    for method in public_methods {
        let name_lower = method.name.to_ascii_lowercase();
        let time_sensitive = name_lower.contains("auition")
            || name_lower.contains("timelock")
            || name_lower.contains("deadline")
            || name_lower.contains("expire");

        if time_sensitive {
            diagnostics.push(Diagnostic::warning(format!(
                "function '{}' appears to be time-sensitive. block.timestamp on Neo N3 is \
                 deterministic but can be affected by block production timing.",
                method.name
            )).with_code("W117")
             .with_suggestion("Consider adding additional verification mechanisms for time-critical operations"));
        }
    }
}
