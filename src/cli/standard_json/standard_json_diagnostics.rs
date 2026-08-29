use super::*;

pub(crate) fn diagnostic_to_standard_error(
    diagnostic: &neo_devpack_solidity::solidity::Diagnostic,
    file: &str,
) -> Value {
    let severity = match diagnostic.severity {
        DiagnosticSeverity::Warning => "warning",
        DiagnosticSeverity::Error => "error",
    };
    let code = diagnostic
        .code
        .as_deref()
        .unwrap_or(match diagnostic.severity {
            DiagnosticSeverity::Warning => "VALIDATION_WARNING",
            DiagnosticSeverity::Error => "VALIDATION_ERROR",
        });

    let mut value = json!({
        "component": "neo-devpack-solidity",
        "severity": severity,
        "type": "Validation",
        "code": code,
        "sourceLocation": { "file": file },
        "formattedMessage": diagnostic.message,
        "message": diagnostic.message,
    });

    if let Some(suggestion) = diagnostic.suggestion.as_deref() {
        value["suggestion"] = json!(suggestion);
    }

    value
}
