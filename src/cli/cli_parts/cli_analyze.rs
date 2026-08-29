use super::*;

#[derive(Debug, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct UpgradeSummary {
    total_findings: usize,
    warning_count: usize,
    error_count: usize,
    auto_compatible_count: usize,
    manual_migration_count: usize,
    manifest_review_count: usize,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FileUpgradeReport {
    input: String,
    compile_success: bool,
    contracts: Vec<String>,
    summary: UpgradeSummary,
    findings: Vec<neo_devpack_solidity::solidity::UpgradeFinding>,
}

fn serialize_upgrade_reports(reports: &[FileUpgradeReport]) -> Result<String, String> {
    let payload = json!({ "files": reports });
    serde_json::to_string_pretty(&payload)
        .map_err(|err| format!("failed to serialize analyze report: {err}"))
}

fn summarize_upgrade_findings(
    findings: &[neo_devpack_solidity::solidity::UpgradeFinding],
) -> UpgradeSummary {
    use neo_devpack_solidity::solidity::{UpgradeCategory, UpgradeSeverity};

    let mut summary = UpgradeSummary {
        total_findings: findings.len(),
        ..UpgradeSummary::default()
    };

    for finding in findings {
        match finding.severity {
            UpgradeSeverity::Warning => summary.warning_count += 1,
            UpgradeSeverity::Error => summary.error_count += 1,
            UpgradeSeverity::Info => {}
        }

        match finding.category {
            UpgradeCategory::AutoCompatible => summary.auto_compatible_count += 1,
            UpgradeCategory::ManualMigration => summary.manual_migration_count += 1,
            UpgradeCategory::ManifestReview => summary.manifest_review_count += 1,
        }
    }

    summary
}

fn classify_upgrade_category(message: &str) -> neo_devpack_solidity::solidity::UpgradeCategory {
    use neo_devpack_solidity::solidity::UpgradeCategory;

    let lower = message.to_ascii_lowercase();

    if lower.contains("wildcard manifest permissions")
        || lower.contains("wildcard contract manifest permissions")
        || lower.contains("wildcard method manifest permissions")
        || (lower.contains("manifest") && lower.contains("permissions"))
    {
        UpgradeCategory::ManifestReview
    } else if lower.contains("auto-mapped") || lower.contains("compiler-compatible") {
        UpgradeCategory::AutoCompatible
    } else {
        UpgradeCategory::ManualMigration
    }
}

fn collect_overload_findings(
    metadata: &ContractMetadata,
) -> Vec<neo_devpack_solidity::solidity::UpgradeFinding> {
    use neo_devpack_solidity::solidity::{UpgradeCategory, UpgradeFinding, UpgradeSeverity};

    let mut groups: HashMap<&str, usize> = HashMap::new();
    for method in &metadata.methods {
        if !matches!(method.kind, FunctionKind::Regular) {
            continue;
        }
        if !matches!(
            method.visibility,
            VisibilityKind::Public | VisibilityKind::External
        ) {
            continue;
        }
        *groups.entry(method.name.as_str()).or_insert(0) += 1;
    }

    groups
        .into_iter()
        .filter(|&(_, count)| count > 1)
        .map(|(name, _)| {
            UpgradeFinding::new(
                "analysis",
                UpgradeSeverity::Warning,
                UpgradeCategory::ManifestReview,
                Some("SCAN_EXPORTED_OVERLOADS"),
                Some(metadata.name.clone()),
                format!(
                    "contract '{}' exports overloaded function '{}'; Neo manifest ABI keeps one canonical name and mangles the other overloads.",
                    metadata.name, name
                ),
                Some(
                    "Avoid overloaded public/external APIs, or ensure downstream callers use the Neo manifest method names.",
                ),
            )
        })
        .collect()
}

pub(crate) fn build_upgrade_report(
    input_file: &str,
    analysis_source: &str,
    compile_source: &str,
    verbose: bool,
    options: &CompileOptions,
    contract_filters: &[String],
) -> Option<FileUpgradeReport> {
    use neo_devpack_solidity::solidity::{
        analyse_all_sources, analyze_upgrade_patterns, UpgradeCategory, UpgradeFinding,
        UpgradeSeverity,
    };

    let mut findings = analyze_upgrade_patterns(analysis_source);
    let analyzed_contracts = analyse_all_sources(compile_source).unwrap_or_default();
    let mut contracts: Vec<String> = analyzed_contracts
        .iter()
        .map(|metadata| metadata.name.clone())
        .collect();

    if !contract_filters.is_empty() {
        contracts.retain(|name| contract_filters.iter().any(|filter| filter == name));
        if contracts.is_empty() {
            return None;
        }
    }

    for metadata in &analyzed_contracts {
        if !contract_filters.is_empty()
            && !contract_filters
                .iter()
                .any(|filter| filter == &metadata.name)
        {
            continue;
        }
        findings.extend(collect_overload_findings(metadata));
    }

    let compile_success =
        match compile_contracts_with_options(compile_source, verbose, options.clone()) {
            Ok(mut artifacts) => {
                if !contract_filters.is_empty() {
                    artifacts.retain(|artifact| {
                        contract_filters
                            .iter()
                            .any(|filter| filter == &artifact.metadata.name)
                    });
                }

                for artifact in artifacts {
                    let contract_name = artifact.metadata.name.clone();
                    for warning in artifact.warnings {
                        findings.push(UpgradeFinding::new(
                            "compile",
                            match warning.severity {
                                DiagnosticSeverity::Warning => UpgradeSeverity::Warning,
                                DiagnosticSeverity::Error => UpgradeSeverity::Error,
                            },
                            classify_upgrade_category(&warning.message),
                            warning.code,
                            Some(contract_name.clone()),
                            warning.message,
                            warning.suggestion,
                        ));
                    }
                }
                true
            }
            Err(CompileError::Diagnostics(diags)) => {
                for diag in diags {
                    let severity = match diag.severity {
                        Severity::Info => UpgradeSeverity::Info,
                        Severity::Warning => UpgradeSeverity::Warning,
                        Severity::Error => UpgradeSeverity::Error,
                    };
                    findings.push(UpgradeFinding::new(
                        "validation",
                        severity,
                        classify_upgrade_category(&diag.code.to_string()),
                        Some(diag.code.to_string()),
                        None::<String>,
                        diag.message,
                        diag.suggestions.first().map(|s| s.message.clone()),
                    ));
                }
                false
            }
            Err(CompileError::Semantic(diags)) => {
                for diag in diags {
                    let severity = match diag.severity {
                        Severity::Info => UpgradeSeverity::Info,
                        Severity::Warning => UpgradeSeverity::Warning,
                        Severity::Error => UpgradeSeverity::Error,
                    };
                    findings.push(UpgradeFinding::new(
                        "semantic",
                        severity,
                        classify_upgrade_category(&diag.message),
                        Some(diag.code.to_string()),
                        None::<String>,
                        diag.message,
                        diag.suggestions.first().map(|s| s.message.clone()),
                    ));
                }
                false
            }
            Err(CompileError::Ir(diags)) => {
                for diag in diags {
                    findings.push(UpgradeFinding::new(
                        "ir",
                        UpgradeSeverity::Error,
                        classify_upgrade_category(&diag.message),
                        Some(diag.code.to_string()),
                        None::<String>,
                        diag.message,
                        diag.suggestions.first().map(|s| s.message.clone()),
                    ));
                }
                false
            }
            Err(CompileError::Manifest(diag)) => {
                findings.push(UpgradeFinding::new(
                "manifest",
                UpgradeSeverity::Error,
                UpgradeCategory::ManifestReview,
                Some(diag.code.to_string()),
                None::<String>,
                diag.message,
                Some(
                    "Review the inferred Neo permissions and contract metadata before deployment.",
                ),
            ));
                false
            }
            Err(CompileError::ParseErrors(diags)) => {
                for diag in diags {
                    let category = classify_upgrade_category(&diag.message);
                    findings.push(UpgradeFinding::new(
                        "compile",
                        UpgradeSeverity::Error,
                        category,
                        Some(diag.code.to_string()),
                        None::<String>,
                        diag.message,
                        diag.suggestions.first().map(|s| s.message.clone()),
                    ));
                }
                false
            }
            Err(CompileError::Message(message)) => {
                findings.push(UpgradeFinding::new(
                    "compile",
                    UpgradeSeverity::Error,
                    classify_upgrade_category(&message),
                    Some(ErrorCode::Nsh0000.to_string()),
                    None::<String>,
                    message,
                    None::<String>,
                ));
                false
            }
            Err(CompileError::Io { path, source }) => {
                findings.push(UpgradeFinding::new(
                    "compile",
                    UpgradeSeverity::Error,
                    UpgradeCategory::ManualMigration,
                    Some(ErrorCode::Nsh0002.to_string()),
                    None::<String>,
                    format!("failed to access '{}': {source}", path.display()),
                    None::<String>,
                ));
                false
            }
        };

    Some(FileUpgradeReport {
        input: input_file.to_string(),
        compile_success,
        contracts,
        summary: summarize_upgrade_findings(&findings),
        findings,
    })
}

pub(crate) fn print_upgrade_reports(reports: &[FileUpgradeReport]) {
    match serialize_upgrade_reports(reports) {
        Ok(serialized) => println!("{serialized}"),
        Err(err) => {
            crate::fatal_error!("error: {err}");
        }
    }
}

pub(crate) fn write_upgrade_reports(
    path: &str,
    reports: &[FileUpgradeReport],
) -> Result<(), String> {
    let serialized = serialize_upgrade_reports(reports)?;
    ensure_output_dir(path)?;
    fs::write(path, serialized)
        .map_err(|err| format!("Failed to write analyze report '{path}': {err}"))
}
