use super::*;

/// Emit per-contract warnings, applying `--Wno-<prefix>` suppression and
/// `--Werror=<prefix>` promotion. Returns `true` if ANY warning was promoted to
/// an error, so the caller can fail the build (exit non-zero) and withhold the
/// artifacts — otherwise `--Werror` would be cosmetic and a CI gate relying on
/// it (e.g. to block full-wildcard-permission or security-warning contracts)
/// would silently pass.
#[must_use]
pub(crate) fn emit_contract_warnings(
    artifacts: &[CompilationArtifacts],
    json_warnings: bool,
    json_errors: bool,
    suppress_prefixes: &[String],
    promote_prefixes: &[String],
) -> bool {
    let mut promoted_any = false;
    for artifact in artifacts {
        for warning in &artifact.warnings {
            let code = warning.code.as_deref().unwrap_or(match warning.severity {
                DiagnosticSeverity::Warning => "VALIDATION_WARNING",
                DiagnosticSeverity::Error => "VALIDATION_ERROR",
            });

            // --Wno-<prefix>: suppress warnings whose code starts with prefix
            if suppress_prefixes
                .iter()
                .any(|p| code.starts_with(p.as_str()))
            {
                continue;
            }

            // --Werror=<prefix>: promote matching warnings to errors
            if promote_prefixes
                .iter()
                .any(|p| code.starts_with(p.as_str()))
            {
                emit_error(&warning.message, code, json_errors);
                promoted_any = true;
                continue;
            }

            emit_warning_with_suggestion(
                &warning.message,
                Some(&artifact.metadata.name),
                json_warnings,
                Some(code),
                warning.suggestion.as_deref(),
            );
        }
    }
    promoted_any
}

pub(crate) struct OutputConfig<'a> {
    pub(crate) format: &'a str,
    pub(crate) output_prefix: &'a str,
    pub(crate) input_file: &'a str,
    pub(crate) nef_source_override: Option<&'a str>,
    pub(crate) deployer: Option<[u8; 20]>,
    pub(crate) json_errors: bool,
    pub(crate) json_warnings: bool,
}

impl<'a> OutputConfig<'a> {
    /// Resolve the source path string written into the emitted NEF header.
    /// Falls back to the compiled input file when the caller did not
    /// override it. Centralised so the three format branches below cannot
    /// drift apart.
    fn nef_source(&self) -> &str {
        self.nef_source_override.unwrap_or(self.input_file)
    }
}

/// The name the chain will use when deriving the deployed contract hash is the
/// one written into the manifest, which a `@custom:neo.manifest.name` override
/// can change after `metadata.name` was set. Read it back from the final
/// manifest so the printed "predicted contract hash" matches what
/// `ContractManagement.Deploy` computes; fall back to `metadata.name` when the
/// manifest has no string `name` (it always should).
fn manifest_name(artifact: &CompilationArtifacts) -> &str {
    artifact
        .manifest
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(artifact.metadata.name.as_str())
}

pub(crate) fn write_contract_outputs(
    artifacts: &[CompilationArtifacts],
    config: &OutputConfig<'_>,
) {
    for (index, artifact) in artifacts.iter().enumerate() {
        let prefix = contract_output_prefix(
            config.output_prefix,
            &artifact.metadata.name,
            index,
            artifacts.len(),
        );

        match config.format {
            "nef" => {
                let nef_path = if prefix.ends_with(".nef") {
                    prefix.clone()
                } else {
                    format!("{prefix}.nef")
                };
                let checksum = match write_nef_file(
                    &nef_path,
                    &artifact.bytecode,
                    &artifact.tokens,
                    config.nef_source(),
                    config.json_warnings,
                ) {
                    Ok(checksum) => checksum,
                    Err(err) => {
                        emit_error(&err, "OUTPUT_WRITE_ERROR", config.json_errors);
                        std::process::exit(1);
                    }
                };
                println!(
                    "✅ [{}] NEF file generated: {nef_path}",
                    artifact.metadata.name
                );
                if let Some(sender) = config.deployer {
                    let predicted = neo_devpack_solidity::neo::compute_contract_hash(
                        sender,
                        checksum,
                        manifest_name(artifact),
                    );
                    println!(
                        "   • Predicted contract hash (sender {}, checksum 0x{checksum:08x}): {}",
                        neo_devpack_solidity::neo::format_uint160_hex_be(&sender),
                        neo_devpack_solidity::neo::format_uint160_hex_be(&predicted)
                    );
                }
            }
            "manifest" => {
                let manifest_path = if prefix.ends_with(".manifest.json") {
                    prefix.clone()
                } else {
                    format!("{prefix}.manifest.json")
                };
                if let Err(err) = write_manifest_file(&manifest_path, &artifact.manifest) {
                    emit_error(&err, "OUTPUT_WRITE_ERROR", config.json_errors);
                    std::process::exit(1);
                }
                println!(
                    "✅ [{}] Manifest file generated: {manifest_path}",
                    artifact.metadata.name
                );
            }
            "complete" => {
                let nef_path = format!("{prefix}.nef");
                let manifest_path = format!("{prefix}.manifest.json");
                let checksum = match write_nef_file(
                    &nef_path,
                    &artifact.bytecode,
                    &artifact.tokens,
                    config.nef_source(),
                    config.json_warnings,
                ) {
                    Ok(checksum) => checksum,
                    Err(err) => {
                        emit_error(&err, "OUTPUT_WRITE_ERROR", config.json_errors);
                        std::process::exit(1);
                    }
                };
                if let Err(err) = write_manifest_file(&manifest_path, &artifact.manifest) {
                    emit_error(&err, "OUTPUT_WRITE_ERROR", config.json_errors);
                    std::process::exit(1);
                }
                println!(
                    "✅ [{}] Contract files generated:\n   📄 {nef_path}\n   📄 {manifest_path}",
                    artifact.metadata.name
                );
                if let Some(sender) = config.deployer {
                    let predicted = neo_devpack_solidity::neo::compute_contract_hash(
                        sender,
                        checksum,
                        manifest_name(artifact),
                    );
                    println!(
                        "   • Predicted contract hash (sender {}, checksum 0x{checksum:08x}): {}",
                        neo_devpack_solidity::neo::format_uint160_hex_be(&sender),
                        neo_devpack_solidity::neo::format_uint160_hex_be(&predicted)
                    );
                }
            }
            "json" => {
                let json_path = if prefix.ends_with(".json") {
                    prefix.clone()
                } else {
                    format!("{prefix}.json")
                };
                if let Err(err) = write_json_file(
                    &json_path,
                    &artifact.bytecode,
                    &artifact.tokens,
                    &artifact.manifest,
                    &artifact.metadata,
                    config.nef_source(),
                    config.json_warnings,
                ) {
                    emit_error(&err, "OUTPUT_WRITE_ERROR", config.json_errors);
                    std::process::exit(1);
                }
                println!(
                    "✅ [{}] JSON file generated: {json_path}",
                    artifact.metadata.name
                );
            }
            "assembly" => {
                let asm_path = if prefix.ends_with(".asm") {
                    prefix.clone()
                } else {
                    format!("{prefix}.asm")
                };
                if let Err(err) = write_assembly_file(&asm_path, &artifact.bytecode) {
                    emit_error(&err, "OUTPUT_WRITE_ERROR", config.json_errors);
                    std::process::exit(1);
                }
                println!(
                    "✅ [{}] Assembly file generated: {asm_path}",
                    artifact.metadata.name
                );
            }
            other => {
                println!("Unsupported format: {other}");
            }
        }
    }
}
