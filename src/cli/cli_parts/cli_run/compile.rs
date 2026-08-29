use super::*;

pub(crate) fn compile_input_or_exit(
    input_content: &str,
    verbose: bool,
    options: CompileOptions,
    json_errors: bool,
    json_warnings: bool,
) -> Vec<CompilationArtifacts> {
    match compile_contracts_with_options(input_content, verbose, options) {
        Ok(list) => list,
        Err(CompileError::Diagnostics(diags))
        | Err(CompileError::Semantic(diags))
        | Err(CompileError::Ir(diags))
        | Err(CompileError::ParseErrors(diags)) => {
            for diag in diags {
                let code = diag.code.to_string();
                let suggestion = diag.suggestions.first().map(|s| s.message.as_str());
                match diag.severity {
                    Severity::Warning => emit_warning_with_suggestion(
                        &diag.message,
                        None,
                        json_warnings,
                        Some(&code),
                        suggestion,
                    ),
                    Severity::Error | Severity::Info => {
                        emit_error_with_suggestion(&diag.message, &code, json_errors, suggestion)
                    }
                }
            }
            std::process::exit(1);
        }
        Err(CompileError::Manifest(diag)) => {
            emit_error_with_suggestion(
                &diag.message,
                &diag.code.to_string(),
                json_errors,
                diag.suggestions.first().map(|s| s.message.as_str()),
            );
            std::process::exit(1);
        }
        Err(CompileError::Message(message)) => {
            emit_error(&message, "GENERIC_ERROR", json_errors);
            std::process::exit(1);
        }
        Err(CompileError::Io { path, source }) => {
            emit_error(
                &format!("failed to access '{}': {source}", path.display()),
                &ErrorCode::Nsh0002.to_string(),
                json_errors,
            );
            std::process::exit(1);
        }
    }
}
