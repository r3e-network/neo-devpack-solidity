use super::*;

pub fn compile_contracts(
    source: &str,
    verbose: bool,
    optimizer_level: u8,
) -> Result<Vec<CompilationArtifacts>, CompileError> {
    compile_contracts_with_options(
        source,
        verbose,
        CompileOptions {
            optimizer_level,
            use_callt: false,
            deny_wildcard_permissions: false,
            deny_wildcard_contracts: false,
            deny_wildcard_methods: false,
            manifest_permissions: None,
        },
    )
}

fn apply_manifest_permissions_override(
    manifest: &mut serde_json::Value,
    metadata: &ContractMetadata,
    override_permissions: &ManifestPermissionsOverride,
) -> Result<(), CompileError> {
    let mut inferred = parse_manifest_permissions_from_manifest(manifest).map_err(|err| {
        CompileError::Message(format!(
            "Failed to parse inferred manifest permissions: {err}"
        ))
    })?;

    match override_permissions.mode {
        ManifestPermissionsMode::Merge => {
            merge_manifest_permissions(&mut inferred, &override_permissions.permissions);
        }
        ManifestPermissionsMode::ReplaceWildcards => {
            let had_wildcards = inferred
                .iter()
                .any(|(contract, methods)| contract == "*" || methods.is_wildcard());

            let is_empty = override_permissions.permissions.is_empty();

            if !had_wildcards && !is_empty {
                return Err(CompileError::Manifest(Box::new(Diagnostic::error(
                    ErrorCode::Nsh6001,
                    format!(
                        "contract '{}' specifies a --manifest-permissions override with `replace-wildcards` mode, but the inferred manifest has no wildcards to replace. Use `merge` mode to append explicit entries, or `replace-wildcards` only for dynamically-calling contracts.",
                        metadata.name
                    ),
                ))));
            }

            if had_wildcards && is_empty {
                return Err(CompileError::Manifest(Box::new(Diagnostic::error(
                    ErrorCode::Nsh6001,
                    format!(
                        "contract '{}' specifies a --manifest-permissions override with `replace-wildcards` mode, but the override is empty. An explicit permission allowlist must be provided to replace the dynamic wildcards.",
                        metadata.name
                    ),
                ))));
            }

            inferred.retain(|contract, methods| contract != "*" && !methods.is_wildcard());
            merge_manifest_permissions(&mut inferred, &override_permissions.permissions);
        }
    }

    manifest["permissions"] = manifest_permissions_to_json(inferred);
    Ok(())
}

pub fn compile_contracts_with_options(
    source: &str,
    verbose: bool,
    options: CompileOptions,
) -> Result<Vec<CompilationArtifacts>, CompileError> {
    use neo_devpack_solidity::frontend::FrontendError;
    use neo_devpack_solidity::solidity::SolidityError;

    // Map every SolidityError variant explicitly at the boundary instead of
    // flattening via a catch-all `to_string()`. Each message string mirrors the
    // variant's `#[error]` Display impl verbatim, so this is a structural
    // change (no information lost to a catch-all), not an output change.
    // ParseDiagnostics stays a structured ParseErrors so standard-JSON emits one
    // error per diagnostic with a precise sourceLocation. Adding a new variant
    // elsewhere will make this match non-exhaustive and force an update here.
    let metadatas = analyse_all_sources(source).map_err(|err| match err {
        SolidityError::Frontend(FrontendError::ParseDiagnostics(diags)) => {
            CompileError::from_parse_diagnostics(diags)
        }
        SolidityError::Frontend(FrontendError::Parse(msg)) => {
            CompileError::Message(format!("Solidity parsing failed:\n{msg}"))
        }
        SolidityError::Frontend(FrontendError::UnsupportedVersion(version)) => {
            CompileError::Message(format!("Unsupported Solidity version: {version}"))
        }
        SolidityError::Frontend(FrontendError::ImportError { path, reason }) => {
            CompileError::Message(format!("Failed to resolve import '{path}': {reason}"))
        }
        SolidityError::Frontend(FrontendError::ContractNotFound(name)) => {
            CompileError::Message(format!("Contract '{name}' not found in source"))
        }
        SolidityError::Frontend(FrontendError::UnsupportedConstruct(kind)) => {
            CompileError::Message(format!(
                "internal error: unsupported top-level Solidity construct '{kind}' (please file \
                 a bug — the compiler may need updating for a newer Solidity grammar)"
            ))
        }
        SolidityError::Analysis(msg) => CompileError::Message(msg),
        SolidityError::NoContracts => {
            CompileError::Message("no contract definitions found in source".into())
        }
        SolidityError::ContractNotFound(name) => {
            CompileError::Message(format!("contract '{name}' not found"))
        }
        SolidityError::UnsupportedFeature(msg) => {
            CompileError::Message(format!("unsupported feature: {msg}"))
        }
        SolidityError::InheritanceError(msg) => {
            CompileError::Message(format!("inheritance error: {msg}"))
        }
    })?;

    // Compile each contract in parallel. `compile_metadata` consumes owned
    // metadata and a freshly-cloned `CompileOptions`, so there is no shared
    // mutable state between iterations; rayon preserves output order via
    // `collect`, keeping artifact ordering identical to the sequential path.
    // When `verbose` is set the diagnostic `println!` output from concurrent
    // compilations may interleave — `verbose` is a debug knob rather than a
    // structured log channel, so this is acceptable and no mutex is added (it
    // would serialise the hot path and defeat the parallelism).
    use rayon::prelude::*;
    metadatas
        .into_par_iter()
        .map(|metadata| compile_metadata(metadata, verbose, options.clone()))
        .collect()
}

fn compile_metadata(
    mut metadata: ContractMetadata,
    verbose: bool,
    options: CompileOptions,
) -> Result<CompilationArtifacts, CompileError> {
    fn manifest_allows_permission(manifest: &Value, contract: &str, method: &str) -> bool {
        let permissions = manifest
            .get("permissions")
            .and_then(|v| v.as_array())
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        permissions.iter().any(|entry| {
            let Some(entry_contract) = entry.get("contract").and_then(|v| v.as_str()) else {
                return false;
            };
            let methods_value = entry.get("methods").unwrap_or(&Value::Null);

            let contract_matches =
                entry_contract == "*" || entry_contract.eq_ignore_ascii_case(contract);
            if !contract_matches {
                return false;
            }

            match methods_value {
                Value::String(s) => s == "*",
                Value::Array(arr) => arr
                    .iter()
                    .filter_map(|v| v.as_str())
                    .any(|name| name == method),
                _ => false,
            }
        })
    }

    let optimizer_level = options.optimizer_level.min(3);
    let diagnostics = validate_contract(&metadata);
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    for diagnostic in diagnostics {
        match diagnostic.severity {
            DiagnosticSeverity::Warning => warnings.push(diagnostic),
            DiagnosticSeverity::Error => errors.push(diagnostic),
        }
    }

    if !errors.is_empty() {
        return Err(CompileError::Diagnostics(
            errors.into_iter().map(Diagnostic::from).collect(),
        ));
    }

    if let Err(diags) = build_semantic_model(&metadata) {
        return Err(CompileError::Semantic(
            diags.into_iter().map(Diagnostic::from).collect(),
        ));
    }

    ensure_deploy_stub(&mut metadata)?;
    let has_parameterised_constructor = metadata
        .methods
        .iter()
        .any(|m| matches!(m.kind, FunctionKind::Constructor) && !m.parameters.is_empty());

    let (ir_module, ir_warnings) = ir::Module::from_contract_with_warnings(&metadata)
        .map_err(CompileError::from_ir_diagnostics)?;
    warnings.extend(ir_warnings);
    let ir_module = optimize_ir(ir_module, optimizer_level);

    if verbose {
        println!(
            "Semantic model built: {} functions, {} state variables",
            ir_module.functions.len(),
            ir_module.state_variables.len()
        );

        for function in &ir_module.functions {
            let instruction_count: usize = function
                .basic_blocks
                .iter()
                .map(|block| block.instructions.len())
                .sum();
            println!(
                "   • IR function '{}' (kind: {:?}) => {} instruction(s)",
                function.name, function.kind, instruction_count
            );
        }
    }

    // Pass optimizer_level to bytecode generation for optimization passes.
    let bytecode_output = generate_contract_bytecode(
        &mut metadata,
        &ir_module,
        verbose,
        optimizer_level,
        options.use_callt,
    )
    .map_err(CompileError::Message)?;
    let mut manifest_warnings: Vec<String> = Vec::new();
    let mut manifest = build_manifest(
        &metadata,
        &ir_module,
        &bytecode_output.script,
        &bytecode_output.tokens,
        &mut manifest_warnings,
    )
    .map_err(|e| CompileError::Manifest(Box::new(Diagnostic::error(ErrorCode::Nsh6000, e.0))))?;
    for w in manifest_warnings {
        warnings.push(neo_devpack_solidity::solidity::Diagnostic::warning(w));
    }

    if let Some(source_override) =
        load_manifest_permissions_override_from_natspec(&metadata).map_err(CompileError::Message)?
    {
        apply_manifest_permissions_override(&mut manifest, &metadata, &source_override)?;
    }

    if let Some(override_permissions) = &options.manifest_permissions {
        apply_manifest_permissions_override(&mut manifest, &metadata, override_permissions)?;
    }

    if has_parameterised_constructor {
        let stdlib_hash_le = codegen::native_contract_hash(ir::NativeContract::StdLib);
        let stdlib_hash_be = stdlib_hash_le.iter().rev().copied().collect::<Vec<_>>();
        let stdlib_contract = format!("0x{}", hex::encode(stdlib_hash_be));

        let has_json_deserialize =
            manifest_allows_permission(&manifest, &stdlib_contract, "jsonDeserialize");
        let has_deserialize =
            manifest_allows_permission(&manifest, &stdlib_contract, "deserialize");

        if !(has_json_deserialize && has_deserialize) {
            warnings.push(neo_devpack_solidity::solidity::Diagnostic::warning(format!(
                    "contract '{}' has a parameterised constructor; deploy it by passing constructor args through `_deploy(data, update)`. Neo-Express: pass a JSON array string via `-d '[7]'`; SDKs that support StackItems may pass an Array directly. The injected deploy prologue uses `StdLib.jsonDeserialize` with a `StdLib.deserialize` fallback, so the manifest must allow these methods.",
                    metadata.name
                )));
        }
    }

    let (
        has_wildcard_contract,
        has_wildcard_methods,
        has_full_wildcard_permissions,
        wildcard_contract_only_nep_callbacks,
    ) = manifest["permissions"]
        .as_array()
        .map(|permissions| {
            let mut wildcard_contract = false;
            let mut wildcard_methods = false;
            let mut full_wildcard = false;
            let mut wildcard_contract_only_nep_callbacks = true;
            const NEP_CALLBACK_METHODS: [&str; 3] =
                ["onNEP11Payment", "onNEP17Payment", "onOracleResponse"];
            for entry in permissions {
                let contract_is_wildcard = entry["contract"] == "*";
                let methods_is_wildcard = entry["methods"] == "*";
                wildcard_contract |= contract_is_wildcard;
                wildcard_methods |= methods_is_wildcard;
                full_wildcard |= contract_is_wildcard && methods_is_wildcard;

                if contract_is_wildcard {
                    if methods_is_wildcard {
                        wildcard_contract_only_nep_callbacks = false;
                    } else if let Some(methods) = entry["methods"].as_array() {
                        for method in methods {
                            match method.as_str() {
                                Some(name) if NEP_CALLBACK_METHODS.contains(&name) => {}
                                _ => wildcard_contract_only_nep_callbacks = false,
                            }
                        }
                    } else {
                        wildcard_contract_only_nep_callbacks = false;
                    }
                }
            }
            if !wildcard_contract {
                wildcard_contract_only_nep_callbacks = false;
            }
            (
                wildcard_contract,
                wildcard_methods,
                full_wildcard,
                wildcard_contract_only_nep_callbacks,
            )
        })
        .unwrap_or((false, false, false, false));

    if has_full_wildcard_permissions {
        let message = format!(
            "contract '{}' requires full wildcard manifest permissions (contract='*', methods='*') because at least one contract call is fully dynamic (unknown target + method) or could not be statically analysed. This is usually not acceptable for production deployments; prefer static calls or restrict the call surface, or provide an explicit allowlist using @custom:neo.manifest.permissions. Use --deny-wildcard-permissions to make this a hard error.",
            metadata.name
        );
        if options.deny_wildcard_permissions
            || options.deny_wildcard_contracts
            || options.deny_wildcard_methods
        {
            return Err(CompileError::Manifest(Box::new(Diagnostic::error(
                ErrorCode::Nsh6001,
                message,
            ))));
        }
        warnings.push(
            neo_devpack_solidity::solidity::Diagnostic::warning(message)
                .with_code("MANIFEST_FULL_WILDCARD"),
        );
    } else {
        if has_wildcard_contract {
            let message = format!(
                "contract '{}' requires wildcard contract manifest permissions (contract='*') due to dynamic contract calls. This is riskier than fixed contract hashes; use --deny-wildcard-contracts to make this a hard error, or provide an explicit allowlist using @custom:neo.manifest.permissions.",
                metadata.name
            );
            if options.deny_wildcard_contracts && !wildcard_contract_only_nep_callbacks {
                return Err(CompileError::Manifest(Box::new(Diagnostic::error(
                    ErrorCode::Nsh6001,
                    message,
                ))));
            }
            if !wildcard_contract_only_nep_callbacks {
                warnings.push(
                    neo_devpack_solidity::solidity::Diagnostic::warning(message)
                        .with_code("MANIFEST_WILDCARD_CONTRACT"),
                );
            }
        }

        if has_wildcard_methods {
            let message = format!(
                "contract '{}' requires wildcard method manifest permissions (methods='*') due to dynamic method calls. This is riskier than fixed method names; use --deny-wildcard-methods to make this a hard error, or provide an explicit allowlist using @custom:neo.manifest.permissions.",
                metadata.name
            );
            if options.deny_wildcard_methods {
                return Err(CompileError::Manifest(Box::new(Diagnostic::error(
                    ErrorCode::Nsh6001,
                    message,
                ))));
            }
            warnings.push(
                neo_devpack_solidity::solidity::Diagnostic::warning(message)
                    .with_code("MANIFEST_WILDCARD_METHODS"),
            );
        }
    }

    Ok(CompilationArtifacts {
        metadata,
        bytecode: bytecode_output.script,
        tokens: bytecode_output.tokens,
        manifest,
        warnings,
    })
}
