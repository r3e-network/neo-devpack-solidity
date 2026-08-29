use super::*;

pub(crate) fn run_single_file(matches: &clap::ArgMatches) {
    fn sanitize_stem_or_fallback(stem: Option<&str>, fallback: String) -> String {
        stem.and_then(sanitize_contract_name).unwrap_or(fallback)
    }

    fn is_output_directory(path: &str, assume_dir_when_missing: bool) -> bool {
        fn looks_like_output_file(path: &str) -> bool {
            path.ends_with(".nef")
                || path.ends_with(".manifest.json")
                || path.ends_with(".json")
                || path.ends_with(".asm")
        }

        if path.ends_with('/') || path.ends_with('\\') {
            return true;
        }
        let output_path = Path::new(path);
        if output_path.is_dir() {
            return true;
        }
        assume_dir_when_missing && !looks_like_output_file(path)
    }

    fn analyze_output_path(path: &str) -> String {
        let output_path = Path::new(path);
        if path.ends_with('/') || path.ends_with('\\') || output_path.is_dir() {
            return output_path
                .join("analysis.json")
                .to_string_lossy()
                .to_string();
        }
        path.to_string()
    }

    let sources: Vec<String> = matches
        .get_many::<String>("source")
        .map(|vals| vals.cloned().collect())
        .unwrap_or_default();

    if sources.is_empty() {
        crate::fatal_error!("error: no input files provided");
    }

    let output_arg = matches.get_one::<String>("output").cloned();

    let format = matches
        .get_one::<String>("format")
        .map(|s| s.as_str())
        .unwrap_or("complete");
    let optimizer_level = matches
        .get_one::<String>("optimize")
        .and_then(|s| s.parse::<u8>().ok())
        .unwrap_or(2)
        .min(3);
    let verbose = matches.get_flag("verbose");
    let nef_source_override = matches.get_one::<String>("nef-source").map(|s| s.as_str());
    let deployer = matches.get_one::<String>("deployer").map(|s| s.as_str());
    let include_paths: Vec<std::path::PathBuf> = matches
        .get_many::<String>("include-path")
        .map(|vals| vals.map(std::path::PathBuf::from).collect())
        .unwrap_or_default();
    // Collect Foundry-style `--remap PREFIX=PATH` and `--remappings FILE`
    // entries into a single ordered list. File entries are appended after
    // CLI entries so explicit CLI remappings win on prefix collisions —
    // matches solc / foundry semantics where the more-specific source
    // takes precedence.
    let mut remappings: Vec<ImportRemapping> = Vec::new();
    if let Some(vals) = matches.get_many::<String>("remap") {
        for spec in vals {
            match parse_remapping(spec) {
                Ok(remap) => remappings.push(remap),
                Err(err) => {
                    crate::fatal_error!("error: {err}");
                }
            }
        }
    }
    if let Some(path) = matches.get_one::<String>("remappings-file") {
        match load_remappings_file(std::path::Path::new(path)) {
            Ok(loaded) => remappings.extend(loaded),
            Err(err) => {
                crate::fatal_error!("error: {err}");
            }
        }
    }
    let contract_filters: Vec<String> = matches
        .get_many::<String>("contract")
        .map(|vals| vals.cloned().collect())
        .unwrap_or_default();
    let use_callt = matches.get_flag("callt");
    let analyze_only = matches.get_flag("analyze");
    let deny_wildcard_permissions = matches.get_flag("deny-wildcard-permissions");
    let deny_wildcard_contracts = matches.get_flag("deny-wildcard-contracts");
    let deny_wildcard_methods = matches.get_flag("deny-wildcard-methods");
    let manifest_permissions_file = matches
        .get_one::<String>("manifest-permissions")
        .map(|s| s.as_str());
    let manifest_permissions_mode = matches
        .get_one::<String>("manifest-permissions-mode")
        .map(|s| s.as_str())
        .unwrap_or("merge");
    let json_errors = matches.get_flag("json-errors");
    let json_warnings = matches.get_flag("json-warnings");
    let warn_suppress: Vec<String> = matches
        .get_many::<String>("Wno")
        .map(|vals| vals.cloned().collect())
        .unwrap_or_default();
    let warn_promote: Vec<String> = matches
        .get_many::<String>("Werror")
        .map(|vals| vals.cloned().collect())
        .unwrap_or_default();

    let manifest_permissions = match manifest_permissions_file {
        Some(path) => match load_manifest_permissions_override(path, manifest_permissions_mode) {
            Ok(override_permissions) => Some(override_permissions),
            Err(err) => {
                crate::fatal_error!("error: {err}");
            }
        },
        None => None,
    };
    let analyze_compile_options = CompileOptions {
        optimizer_level,
        use_callt,
        deny_wildcard_permissions,
        deny_wildcard_contracts,
        deny_wildcard_methods,
        manifest_permissions: manifest_permissions.clone(),
    };

    let file_ids: Vec<String> = if sources.len() > 1 {
        use std::collections::HashMap;
        use std::path::Component;

        fn file_identity(path: &Path, fallback: String) -> String {
            let mut parts: Vec<String> = Vec::new();
            for component in path.components() {
                match component {
                    Component::Normal(os) => {
                        parts.push(os.to_string_lossy().to_string());
                    }
                    Component::CurDir | Component::ParentDir => {}
                    Component::Prefix(_) | Component::RootDir => {}
                }
            }

            if parts.is_empty() {
                return fallback;
            }

            if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
                if let Some(last) = parts.last_mut() {
                    *last = stem.to_string();
                }
            }

            let joined = parts.join("_");
            sanitize_contract_name(&joined).unwrap_or(fallback)
        }

        let mut stem_counts: HashMap<String, usize> = HashMap::new();
        let stems: Vec<String> = sources
            .iter()
            .enumerate()
            .map(|(file_index, input_file)| {
                let file_stem = Path::new(input_file)
                    .file_stem()
                    .and_then(|stem| stem.to_str());
                let sanitized =
                    sanitize_stem_or_fallback(file_stem, format!("contract{file_index}"));
                *stem_counts.entry(sanitized.clone()).or_insert(0) += 1;
                sanitized
            })
            .collect();

        sources
            .iter()
            .enumerate()
            .map(|(file_index, input_file)| {
                let stem = stems[file_index].clone();
                if stem_counts.get(&stem).copied().unwrap_or(0) <= 1 {
                    stem
                } else {
                    file_identity(Path::new(input_file), format!("{stem}-{file_index}"))
                }
            })
            .collect()
    } else {
        Vec::new()
    };

    if verbose {
        if analyze_only {
            eprintln!("Neo DevPack for Solidity v{}", env!("CARGO_PKG_VERSION"));
            eprintln!("Format: {format}");
        } else {
            println!("Neo DevPack for Solidity v{}", env!("CARGO_PKG_VERSION"));
            println!("Format: {format}");
        }
    }

    let deployer_le = match deployer {
        Some(value) => match neo_devpack_solidity::neo::parse_uint160_hex_be(value) {
            Ok(parsed) => Some(parsed),
            Err(err) => {
                crate::fatal_error!("error: invalid --deployer value: {err}");
            }
        },
        None => None,
    };

    if sources.len() > 1 {
        if let Some(output) = &output_arg {
            if !analyze_only && !is_output_directory(output, true) {
                crate::fatal_error!(
                    "error: when compiling multiple input files, --output must be a directory (got '{output}')"
                );
            }
        }

        if verbose {
            if analyze_only {
                eprintln!("Batch mode: {} input file(s)", sources.len());
            } else {
                println!("Batch mode: {} input file(s)", sources.len());
            }
            if let Some(output) = &output_arg {
                if analyze_only {
                    eprintln!("Output directory: {output}");
                } else {
                    println!("Output directory: {output}");
                }
            }
        }
    }

    let mut emitted_any = false;
    let mut analysis_reports = Vec::new();

    for (file_index, input_file) in sources.iter().enumerate() {
        if sources.len() > 1 {
            if analyze_only {
                eprintln!("(info) compiling {input_file}");
            } else {
                println!("(info) compiling {input_file}");
            }
        }

        let resolved = match resolve_solidity_sources_with_options(
            Path::new(input_file),
            &include_paths,
            &remappings,
        ) {
            Ok(resolved) => resolved,
            Err(err) => {
                crate::fatal_error!("Error resolving imports: {err}");
            }
        };
        let input_content = resolved.combined_source;

        if verbose {
            if analyze_only {
                eprintln!(
                    "Resolved {} Solidity source file(s) ({} bytes combined)",
                    resolved.files.len(),
                    input_content.len()
                );
            } else {
                println!(
                    "Resolved {} Solidity source file(s) ({} bytes combined)",
                    resolved.files.len(),
                    input_content.len()
                );
            }
        }

        if analyze_only {
            if let Some(report) = build_upgrade_report(
                input_file,
                &input_content,
                &input_content,
                false,
                &analyze_compile_options,
                &contract_filters,
            ) {
                analysis_reports.push(report);
                emitted_any = true;
            } else if verbose {
                eprintln!(
                    "(info) no matching contract(s) found in {} for --contract {}",
                    input_file,
                    contract_filters.join(", ")
                );
            }
            continue;
        }

        let mut artifacts = compile_input_or_exit(
            &input_content,
            verbose,
            CompileOptions {
                optimizer_level,
                use_callt,
                deny_wildcard_permissions,
                deny_wildcard_contracts,
                deny_wildcard_methods,
                manifest_permissions: manifest_permissions.clone(),
            },
            json_errors,
            json_warnings,
        );

        let had_any_contracts = !artifacts.is_empty();

        if !contract_filters.is_empty() {
            artifacts.retain(|artifact| {
                contract_filters
                    .iter()
                    .any(|name| name == &artifact.metadata.name)
            });
        }

        if artifacts.is_empty() {
            if !contract_filters.is_empty() && verbose {
                println!(
                    "(info) no matching contract(s) found in {} for --contract {}",
                    input_file,
                    contract_filters.join(", ")
                );
            }
            if verbose && !had_any_contracts {
                eprintln!("warning: No contracts were found in {input_file}");
            }
            continue;
        }

        let output_prefix = match (&output_arg, sources.len() > 1) {
            (Some(output), true) => {
                let output_dir = Path::new(output);
                let stem_sanitized = file_ids
                    .get(file_index)
                    .cloned()
                    .unwrap_or_else(|| format!("contract{file_index}"));

                if artifacts.len() == 1 {
                    let contract_name = artifacts[0].metadata.name.as_str();
                    let contract_sanitized =
                        sanitize_stem_or_fallback(Some(contract_name), stem_sanitized.clone());
                    let base = if contract_sanitized == stem_sanitized {
                        stem_sanitized
                    } else {
                        format!("{stem_sanitized}-{contract_sanitized}")
                    };
                    output_dir.join(base).to_string_lossy().to_string()
                } else {
                    output_dir
                        .join(stem_sanitized)
                        .to_string_lossy()
                        .to_string()
                }
            }
            (Some(output), false) => output.clone(),
            (None, true) => file_ids
                .get(file_index)
                .cloned()
                .unwrap_or_else(|| format!("contract{file_index}")),
            (None, false) => Path::new(input_file)
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("contract")
                .to_string(),
        };

        if verbose {
            println!("Input: {input_file}");
            println!("Output prefix: {output_prefix}");
        }

        if artifacts.len() > 1 {
            println!(
                "(info) detected {} contracts – outputs are suffixed with their contract names",
                artifacts.len()
            );
        }

        let promoted_to_error = emit_contract_warnings(
            &artifacts,
            json_warnings,
            json_errors,
            &warn_suppress,
            &warn_promote,
        );
        // `--Werror=<prefix>` must FAIL the build: withhold the NEF/manifest and
        // exit non-zero so a CI gate actually blocks the contract, rather than
        // just relabeling the warning while emitting the artifacts and exit 0.
        if promoted_to_error {
            std::process::exit(1);
        }
        let output_config = OutputConfig {
            format,
            output_prefix: &output_prefix,
            input_file,
            nef_source_override,
            deployer: deployer_le,
            json_errors,
            json_warnings,
        };
        write_contract_outputs(&artifacts, &output_config);
        emitted_any = true;
    }

    if analyze_only {
        if !contract_filters.is_empty() && !emitted_any {
            crate::fatal_error!(
                "error: no matching contract(s) found for --contract {}",
                contract_filters.join(", ")
            );
        }
        if let Some(output) = &output_arg {
            let report_path = analyze_output_path(output);
            if let Err(err) = write_upgrade_reports(&report_path, &analysis_reports) {
                emit_error(&err, "OUTPUT_WRITE_ERROR", json_errors);
                std::process::exit(1);
            }
        } else {
            print_upgrade_reports(&analysis_reports);
        }
        return;
    }

    if !contract_filters.is_empty() && !emitted_any {
        crate::fatal_error!(
            "error: no matching contract(s) found for --contract {}",
            contract_filters.join(", ")
        );
    }

    println!("🎉 Neo DevPack for Solidity compilation completed");
}
