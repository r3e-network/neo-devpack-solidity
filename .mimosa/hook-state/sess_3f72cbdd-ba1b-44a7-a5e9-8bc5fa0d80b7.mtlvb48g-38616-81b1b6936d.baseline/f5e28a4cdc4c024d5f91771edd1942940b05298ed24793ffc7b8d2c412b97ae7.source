use super::*;

/// Severity level for standards-detection diagnostics.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum StandardsDiagnosticLevel {
    /// Contract almost matches a standard, or a detected standard has issues.
    Warning,
    /// Informational hint about parameter signatures or events.
    Info,
}

/// A diagnostic emitted during standards detection.
#[derive(Debug, Clone)]
pub(crate) struct StandardsDiagnostic {
    pub(crate) level: StandardsDiagnosticLevel,
    pub(crate) standard: &'static str,
    pub(crate) message: String,
}

/// Result of standards detection: detected standards + any diagnostics.
pub(crate) struct StandardsDetectionResult {
    pub(crate) standards: Vec<String>,
    pub(crate) diagnostics: Vec<StandardsDiagnostic>,
}

pub(crate) fn detect_supported_standards(
    methods: &[FunctionMetadata],
    events: &[EventMetadata],
) -> StandardsDetectionResult {
    let public_methods: Vec<&FunctionMetadata> = methods
        .iter()
        .filter(|m| {
            !matches!(m.kind, FunctionKind::Constructor)
                && matches!(
                    m.visibility,
                    VisibilityKind::Public | VisibilityKind::External
                )
        })
        .collect();
    let names: HashSet<String> = public_methods
        .iter()
        .map(|m| m.name.to_ascii_lowercase())
        .collect();
    let mut standards = Vec::new();
    let mut diagnostics: Vec<StandardsDiagnostic> = Vec::new();

    let has_ownerof = names.contains("ownerof");

    // ── NEP-17: Fungible Token Standard (ERC-20 equivalent) ──────────
    //
    // Claiming a standard the contract cannot fulfil ships a manifest that
    // wallets/exchanges act on (`transfer(from, to, amount, data)` invokes),
    // so detection requires actual signature conformance — not just the five
    // method names: a 4-parameter `transfer` AND a 3-parameter `Transfer`
    // event. A vanilla ERC-20 port (`transfer(to, amount)`) gets a warning
    // explaining how to conform instead of a false NEP-17 claim.
    let nep17_required = ["symbol", "decimals", "totalsupply", "balanceof", "transfer"];
    let nep17_present: Vec<&&str> = nep17_required
        .iter()
        .filter(|m| names.contains(**m))
        .collect();
    let nep17_names_match = nep17_present.len() == nep17_required.len() && !has_ownerof;
    let nep17_transfer_ok = methods_named(&public_methods, "transfer")
        .iter()
        .any(|method| method.parameters.len() == 4);
    let nep17_event_ok = events
        .iter()
        .any(|event| event.name == "Transfer" && event.parameters.len() == 3);

    if nep17_names_match && nep17_transfer_ok && nep17_event_ok {
        standards.push("NEP-17".to_string());
        // Count-conformant, but warn when the parameter TYPES keep the
        // compiler from emitting the native NEP-17 Transfer notification.
        validate_transfer_event(events, "NEP-17", 3, &mut diagnostics);
    } else if nep17_names_match {
        let mut problems: Vec<&str> = Vec::new();
        if !nep17_transfer_ok {
            problems.push("`transfer` must take 4 parameters (from, to, amount, data)");
        }
        if !nep17_event_ok {
            problems.push("a 3-parameter `Transfer(from, to, amount)` event must be declared");
        }
        diagnostics.push(StandardsDiagnostic {
            level: StandardsDiagnosticLevel::Warning,
            standard: "NEP-17",
            message: format!(
                "contract has all NEP-17 method names but does not conform to the standard: {}. \
                 NEP-17 was not added to supportedstandards.",
                problems.join("; "),
            ),
        });
    } else if nep17_present.len() >= 3 && !has_ownerof {
        // Near-miss: contract has most NEP-17 methods but not all
        let missing: Vec<&str> = nep17_required
            .iter()
            .filter(|m| !names.contains(**m))
            .copied()
            .collect();
        diagnostics.push(StandardsDiagnostic {
            level: StandardsDiagnosticLevel::Warning,
            standard: "NEP-17",
            message: format!(
                "contract has {} of {} required NEP-17 methods (missing: {}). \
                 Add the missing method(s) to enable NEP-17 standard detection.",
                nep17_present.len(),
                nep17_required.len(),
                missing.join(", "),
            ),
        });
    }

    // ── NEP-11: Non-Fungible Token Standard (ERC-721 equivalent) ─────
    //
    // NEP-11 mandates the full method set below — `transferFrom` is not even
    // a NEP-11 method, so an ERC-721-shaped contract (balanceOf + ownerOf +
    // transferFrom) must NOT be advertised as NEP-11: wallets would call
    // symbol()/decimals()/totalSupply()/tokensOf()/transfer() and fail. As
    // with NEP-17 above, detection also requires signature conformance: a
    // 3-parameter `transfer(to, tokenId, data)` and a 4-parameter `Transfer`
    // event.
    let nep11_required = [
        "symbol",
        "decimals",
        "totalsupply",
        "balanceof",
        "tokensof",
        "ownerof",
        "transfer",
    ];
    let nep11_present: Vec<&&str> = nep11_required
        .iter()
        .filter(|m| names.contains(**m))
        .collect();
    let nep11_names_match = nep11_present.len() == nep11_required.len();
    let nep11_transfer_ok = methods_named(&public_methods, "transfer")
        .iter()
        .any(|method| method.parameters.len() == 3);
    let nep11_event_ok = events
        .iter()
        .any(|event| event.name == "Transfer" && event.parameters.len() == 4);

    if nep11_names_match && nep11_transfer_ok && nep11_event_ok {
        standards.push("NEP-11".to_string());
        // Count-conformant, but warn when the parameter TYPES keep the
        // compiler from emitting the native NEP-11 Transfer notification.
        validate_transfer_event(events, "NEP-11", 4, &mut diagnostics);
    } else if nep11_names_match {
        let mut problems: Vec<&str> = Vec::new();
        if !nep11_transfer_ok {
            problems.push("`transfer` must take 3 parameters (to, tokenId, data)");
        }
        if !nep11_event_ok {
            problems
                .push("a 4-parameter `Transfer(from, to, amount, tokenId)` event must be declared");
        }
        diagnostics.push(StandardsDiagnostic {
            level: StandardsDiagnosticLevel::Warning,
            standard: "NEP-11",
            message: format!(
                "contract has all NEP-11 method names but does not conform to the standard: {}. \
                 NEP-11 was not added to supportedstandards.",
                problems.join("; "),
            ),
        });
    } else if has_ownerof {
        // Near-miss: ownerOf is a strong NFT signal — list what is missing
        // for genuine NEP-11 support instead of advertising a standard the
        // contract cannot fulfil.
        let missing: Vec<&str> = nep11_required
            .iter()
            .filter(|m| !names.contains(**m))
            .copied()
            .collect();
        diagnostics.push(StandardsDiagnostic {
            level: StandardsDiagnosticLevel::Warning,
            standard: "NEP-11",
            message: format!(
                "contract has `ownerOf` (NFT signal) but is missing required NEP-11 method(s): {}. \
                 Add the missing method(s) to enable NEP-11 standard detection.",
                missing.join(", "),
            ),
        });
    }

    // ── NEP-24: Royalty Standard ─────────────────────────────────────
    //
    // NEP-24's single mandatory method is
    // `royaltyInfo(tokenId, royaltyToken, salePrice)`. `tokenURI` is an
    // ERC-721 metadata method with no NEP-24 significance and must not
    // trigger the royalty standard — marketplaces honouring the manifest
    // would call a nonexistent `royaltyInfo` and fault.
    let royalty_methods = methods_named(&public_methods, "royaltyInfo");
    if royalty_methods
        .iter()
        .any(|method| method.parameters.len() == 3)
    {
        standards.push("NEP-24".to_string());
    } else if !royalty_methods.is_empty() {
        diagnostics.push(StandardsDiagnostic {
            level: StandardsDiagnosticLevel::Info,
            standard: "NEP-24",
            message: "contract has `royaltyInfo` but signature does not match \
                      NEP-24 (`royaltyInfo(tokenId, royaltyToken, salePrice)`)."
                .to_string(),
        });
    }

    // ── NEP-22: Contract Update Function ─────────────────────────────
    let update_methods = methods_named(&public_methods, "update");
    if update_methods
        .iter()
        .any(|method| method.parameters.len() == 3)
    {
        standards.push("NEP-22".to_string());
    } else if !update_methods.is_empty() {
        diagnostics.push(StandardsDiagnostic {
            level: StandardsDiagnosticLevel::Info,
            standard: "NEP-22",
            message: "contract has `update` but no 3-parameter NEP-22 signature \
                      `update(nefFile, manifest, data)`."
                .to_string(),
        });
    }

    // ── NEP-26: NEP-11 Receiver Callback ─────────────────────────────
    let nep11_payment_methods = methods_named(&public_methods, "onNEP11Payment");
    if nep11_payment_methods
        .iter()
        .any(|method| method.parameters.len() == 4)
    {
        standards.push("NEP-26".to_string());
    } else if !nep11_payment_methods.is_empty() {
        diagnostics.push(StandardsDiagnostic {
            level: StandardsDiagnosticLevel::Info,
            standard: "NEP-26",
            message: "contract has `onNEP11Payment` but signature does not match \
                      NEP-26 (`onNEP11Payment(from, amount, tokenId, data)`)."
                .to_string(),
        });
    }

    // ── NEP-27: NEP-17 Receiver Callback ─────────────────────────────
    let nep17_payment_methods = methods_named(&public_methods, "onNEP17Payment");
    if nep17_payment_methods
        .iter()
        .any(|method| method.parameters.len() == 3)
    {
        standards.push("NEP-27".to_string());
    } else if !nep17_payment_methods.is_empty() {
        diagnostics.push(StandardsDiagnostic {
            level: StandardsDiagnosticLevel::Info,
            standard: "NEP-27",
            message: "contract has `onNEP17Payment` but signature does not match \
                      NEP-27 (`onNEP17Payment(from, amount, data)`)."
                .to_string(),
        });
    }

    // ── NEP-29: Deployment/Update Callback ───────────────────────────
    let deploy_methods = methods_named(&public_methods, "_deploy");
    if deploy_methods
        .iter()
        .any(|method| method.parameters.len() == 2)
    {
        standards.push("NEP-29".to_string());
    } else if !deploy_methods.is_empty() {
        diagnostics.push(StandardsDiagnostic {
            level: StandardsDiagnosticLevel::Info,
            standard: "NEP-29",
            message: "contract has `_deploy` but signature does not match \
                      NEP-29 (`_deploy(data, update)`)."
                .to_string(),
        });
    }

    // ── NEP-30: Contract Witness Verification Callback ───────────────
    let verify_methods = methods_named(&public_methods, "verify");
    if verify_methods.len() == 1 && method_returns_bool(verify_methods[0]) {
        standards.push("NEP-30".to_string());
    } else if verify_methods.len() > 1 {
        diagnostics.push(StandardsDiagnostic {
            level: StandardsDiagnosticLevel::Warning,
            standard: "NEP-30",
            message: "contract has multiple `verify` methods. NEP-30 requires a \
                      single `verify` entrypoint."
                .to_string(),
        });
    } else if verify_methods.len() == 1 {
        diagnostics.push(StandardsDiagnostic {
            level: StandardsDiagnosticLevel::Info,
            standard: "NEP-30",
            message: "contract has `verify` but it does not return `bool` as \
                      required by NEP-30."
                .to_string(),
        });
    }

    // ── NEP-31: Contract Destroy Guideline ───────────────────────────
    let destroy_methods = methods_named(&public_methods, "destroy");
    if destroy_methods
        .iter()
        .any(|method| method.parameters.is_empty() && method.return_parameters.is_empty())
    {
        standards.push("NEP-31".to_string());
    } else if !destroy_methods.is_empty() {
        diagnostics.push(StandardsDiagnostic {
            level: StandardsDiagnosticLevel::Info,
            standard: "NEP-31",
            message: "contract has `destroy` but no parameterless void signature \
                      required by NEP-31."
                .to_string(),
        });
    }

    StandardsDetectionResult {
        standards,
        diagnostics,
    }
}

/// Determine whether an event declaration will be emitted (and declared in
/// the manifest) in NATIVE NEP `Transfer` shape, and for which standard.
///
/// Mirrors the emit lowering / manifest builder by delegating to the shared
/// `ir::native_transfer_standard` predicate. The enum set passed to the
/// canonicalizer is empty: enum names can never spell `address` / `uint256`
/// / `bytes32` / `bytes` (reserved words), so positive matches are identical
/// with or without enum knowledge.
pub(crate) fn event_native_transfer_standard(
    event: &EventMetadata,
) -> Option<ir::NativeTransferStandard> {
    let enum_names: HashSet<String> = HashSet::new();
    let canonical_types: Vec<String> = event
        .parameters
        .iter()
        .map(|param| ir::event_canonical_param_type(&param.ty, &enum_names))
        .collect();
    ir::native_transfer_standard(&event.name, event.anonymous, &canonical_types)
}

/// Check that a `Transfer` event exists with the expected parameter count
/// AND the parameter types that make the compiler emit it in native NEP
/// shape (`[from, to, amount(, tokenId)]`, readable by NEP trackers).
pub(crate) fn validate_transfer_event(
    events: &[EventMetadata],
    standard: &'static str,
    expected_params: usize,
    diagnostics: &mut Vec<StandardsDiagnostic>,
) {
    let expected_kind = if expected_params == 4 {
        ir::NativeTransferStandard::Nep11
    } else {
        ir::NativeTransferStandard::Nep17
    };
    let transfer_event = events.iter().find(|e| e.name == "Transfer");
    match transfer_event {
        None => {
            diagnostics.push(StandardsDiagnostic {
                level: StandardsDiagnosticLevel::Warning,
                standard,
                message: format!(
                    "{standard} detected but contract is missing the required `Transfer` event \
                     ({expected_params} parameters expected).",
                ),
            });
        }
        Some(evt) if evt.parameters.len() != expected_params => {
            diagnostics.push(StandardsDiagnostic {
                level: StandardsDiagnosticLevel::Info,
                standard,
                message: format!(
                    "{standard} `Transfer` event has {} parameter(s), expected {expected_params}.",
                    evt.parameters.len(),
                ),
            });
        }
        Some(evt) if event_native_transfer_standard(evt) != Some(expected_kind) => {
            diagnostics.push(StandardsDiagnostic {
                level: StandardsDiagnosticLevel::Warning,
                standard,
                message: format!(
                    "{standard} `Transfer` event parameter types do not match the standard \
                     signature, so it will be emitted in EVM log shape (unreadable by NEP \
                     trackers). Declare it as `Transfer(address from, address to, uint256 \
                     amount{})` for native emission.",
                    if expected_params == 4 {
                        ", bytes tokenId"
                    } else {
                        ""
                    },
                ),
            });
        }
        _ => {} // Event present in native NEP shape — all good.
    }
}

/// Returns all public/external methods with a given name (case-insensitive).
pub(crate) fn methods_named<'a>(
    public_methods: &[&'a FunctionMetadata],
    name: &str,
) -> Vec<&'a FunctionMetadata> {
    public_methods
        .iter()
        .copied()
        .filter(|method| method.name.eq_ignore_ascii_case(name))
        .collect()
}

/// Best-effort bool return detection for standards validation.
pub(crate) fn method_returns_bool(method: &FunctionMetadata) -> bool {
    if method.return_parameters.len() != 1 {
        return false;
    }
    let ret = &method.return_parameters[0];
    if let Some(neo_type) = &ret.neo_type {
        if matches!(neo_type, NeoType::Boolean) {
            return true;
        }
    }
    ret.ty.eq_ignore_ascii_case("bool")
}

/// Task #28: hard-validate standards the user EXPLICITLY asserted via
/// `@custom:neo.manifest.supportedstandards`. For NEP-17 / NEP-11 the
/// required methods and `Transfer` event are mandatory — shipping a
/// contract that claims the standard without them is a manifest error,
/// not a warning. Implicit (auto-detected) standards are gated on full
/// conformance in `detect_supported_standards` (method names plus the
/// `transfer` signature and `Transfer` event shape), so near-misses
/// surface as warnings instead of false manifest claims.
pub(crate) fn validate_declared_standards(
    declared: &[String],
    methods: &[FunctionMetadata],
    events: &[EventMetadata],
) -> Result<(), String> {
    let public_methods: Vec<&FunctionMetadata> = methods
        .iter()
        .filter(|m| {
            !matches!(m.kind, FunctionKind::Constructor)
                && matches!(
                    m.visibility,
                    VisibilityKind::Public | VisibilityKind::External
                )
        })
        .collect();
    let names: HashSet<String> = public_methods
        .iter()
        .map(|m| m.name.to_ascii_lowercase())
        .collect();
    let transfer_event = events.iter().find(|e| e.name == "Transfer");

    for std in declared {
        // `(required method names, Transfer-event param count,
        // native Transfer-event kind, transfer-method param count)`
        let (required, expected_transfer_params, expected_kind, expected_transfer_method_params): (
            &[&str],
            usize,
            ir::NativeTransferStandard,
            usize,
        ) = match std.as_str() {
            "NEP-17" => (
                &["symbol", "decimals", "totalsupply", "balanceof", "transfer"],
                3,
                ir::NativeTransferStandard::Nep17,
                4,
            ),
            "NEP-11" => (
                // Mandatory NEP-11 method set — must match the auto-detection
                // list (`nep11_required`) so the explicit-declaration gate is no
                // weaker than auto-detection. `totalSupply` and `tokensOf` are
                // mandatory per the NEP-11 spec; omitting them previously let a
                // contract ship a manifest falsely advertising `supportedstandards:
                // ["NEP-11"]` while lacking methods wallets/indexers rely on.
                &[
                    "symbol",
                    "decimals",
                    "totalsupply",
                    "balanceof",
                    "tokensof",
                    "ownerof",
                    "transfer",
                ],
                4,
                ir::NativeTransferStandard::Nep11,
                3,
            ),
            _ => continue,
        };
        let missing: Vec<&str> = required
            .iter()
            .copied()
            .filter(|m| !names.contains(*m))
            .collect();
        if !missing.is_empty() {
            return Err(format!(
                "contract declares `{std}` in supportedstandards but is missing required method(s): {}. \
                 Either implement the missing method(s) or remove `{std}` from @custom:neo.manifest.supportedstandards.",
                missing.join(", "),
            ));
        }
        let transfer_method_ok = public_methods.iter().any(|m| {
            m.name.eq_ignore_ascii_case("transfer")
                && m.parameters.len() == expected_transfer_method_params
        });
        if !transfer_method_ok {
            return Err(format!(
                "contract declares `{std}` in supportedstandards but no `transfer` overload takes \
                 {expected_transfer_method_params} parameter(s) as the {std} spec requires. \
                 Fix the `transfer` signature or remove `{std}` from @custom:neo.manifest.supportedstandards."
            ));
        }
        match transfer_event {
            None => {
                return Err(format!(
                    "contract declares `{std}` in supportedstandards but is missing the required `Transfer` event \
                     ({expected_transfer_params} parameters expected). Either emit a `Transfer` event or remove `{std}` \
                     from @custom:neo.manifest.supportedstandards."
                ));
            }
            Some(evt) if evt.parameters.len() != expected_transfer_params => {
                return Err(format!(
                    "contract declares `{std}` in supportedstandards but its `Transfer` event has {} parameter(s); \
                     {std} requires {expected_transfer_params}. Fix the event signature or remove `{std}` \
                     from @custom:neo.manifest.supportedstandards.",
                    evt.parameters.len(),
                ));
            }
            Some(evt) if event_native_transfer_standard(evt) != Some(expected_kind) => {
                // Right arity but wrong parameter types (or `anonymous`):
                // the compiler will emit this event in EVM log shape, which
                // NEP trackers cannot read — the declared standard would be
                // a lie. Hard error, same policy as a missing event.
                return Err(format!(
                    "contract declares `{std}` in supportedstandards but its `Transfer` event does not match the \
                     {std} signature `Transfer(address from, address to, uint256 amount{})`, so it will not be \
                     emitted as a native NEP transfer notification. Fix the event parameter types or remove `{std}` \
                     from @custom:neo.manifest.supportedstandards.",
                    if expected_transfer_params == 4 { ", bytes-or-bytes32 tokenId" } else { "" },
                ));
            }
            _ => {}
        }
    }
    Ok(())
}
