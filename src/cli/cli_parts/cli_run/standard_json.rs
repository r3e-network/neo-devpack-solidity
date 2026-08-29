use super::*;

pub(crate) fn try_run_standard_json(matches: &clap::ArgMatches) -> bool {
    fn standard_json_cli_error_code(message: &str) -> &'static str {
        let lower = message.to_ascii_lowercase();
        if lower.contains("failed to parse standard json input") {
            "STANDARD_JSON_PARSE_ERROR"
        } else {
            "STANDARD_JSON_INPUT_ERROR"
        }
    }

    fn emit_standard_json_cli_error(output_path: Option<&str>, message: &str) {
        let payload = json!({
            "contracts": {},
            "sources": {},
            "errors": [{
                "component": "neo-devpack-solidity",
                "severity": "error",
                "type": "StandardJsonInput",
                "code": standard_json_cli_error_code(message),
                "formattedMessage": message,
                "message": message,
            }]
        });

        let serialized = serde_json::to_string_pretty(&payload)
            .unwrap_or_else(|_| "{\"errors\":[{\"code\":\"STANDARD_JSON_INPUT_ERROR\",\"message\":\"failed to serialize error output\"}]}".to_string());

        if let Some(path) = output_path {
            if let Err(err) = fs::write(path, &serialized) {
                crate::fatal_error!("error: Failed to write standard JSON output: {err}");
            }
        } else {
            println!("{serialized}");
        }
    }

    let use_standard_json = matches.get_flag("standard-json");
    if !use_standard_json {
        return false;
    }
    if matches.get_flag("analyze") {
        crate::fatal_error!("error: --analyze is currently supported only in single-file mode");
    }

    use std::io::{self, IsTerminal, Read};

    let input_path = matches
        .get_one::<String>("standard-json-input")
        .map(|value| value.as_str());
    let output_path = matches.get_one::<String>("output").map(|s| s.as_str());
    let optimizer_level = matches
        .get_one::<String>("optimize")
        .and_then(|s| s.parse::<u8>().ok())
        .unwrap_or(2)
        .min(3);
    let use_callt = matches.get_flag("callt");
    let deny_wildcard_permissions = matches.get_flag("deny-wildcard-permissions");
    let deny_wildcard_contracts = matches.get_flag("deny-wildcard-contracts");
    let deny_wildcard_methods = matches.get_flag("deny-wildcard-methods");
    let nef_source = matches.get_one::<String>("nef-source").map(|s| s.as_str());
    let manifest_permissions_file = matches
        .get_one::<String>("manifest-permissions")
        .map(|s| s.as_str());
    let manifest_permissions_mode = matches
        .get_one::<String>("manifest-permissions-mode")
        .map(|s| s.as_str())
        .unwrap_or("merge");
    let contract_names: Vec<String> = matches
        .get_many::<String>("contract")
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

    let result = if let Some(path) = input_path {
        process_standard_json(
            path,
            output_path,
            StandardJsonOptions {
                optimizer_level,
                use_callt,
                deny_wildcard_permissions,
                deny_wildcard_contracts,
                deny_wildcard_methods,
                nef_source,
                manifest_permissions,
                contract_names,
            },
        )
    } else {
        if io::stdin().is_terminal() {
            crate::fatal_error!("error: --standard-json requires --input <FILE> or JSON on stdin");
        }

        let mut content = String::new();
        if let Err(err) = io::stdin().read_to_string(&mut content) {
            crate::fatal_error!("error: Failed to read standard JSON input from stdin: {err}");
        }

        process_standard_json_content(
            &content,
            output_path,
            StandardJsonOptions {
                optimizer_level,
                use_callt,
                deny_wildcard_permissions,
                deny_wildcard_contracts,
                deny_wildcard_methods,
                nef_source,
                manifest_permissions,
                contract_names,
            },
        )
    };

    if let Err(err) = result {
        emit_standard_json_cli_error(output_path, &err);
        std::process::exit(1);
    }

    true
}
