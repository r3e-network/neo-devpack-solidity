use super::*;

impl CompileError {
    /// Convert this compile error into a vector of standard-JSON error objects.
    pub(crate) fn into_errors(self, file: &str) -> Vec<Value> {
        match self {
            CompileError::Diagnostics(diags)
            | CompileError::Semantic(diags)
            | CompileError::Ir(diags)
            | CompileError::ParseErrors(diags) => diags
                .into_iter()
                .map(|diag| diagnostic_to_standard_error(&diag, file))
                .collect(),
            CompileError::Manifest(diag) => {
                vec![diagnostic_to_standard_error(&diag, file)]
            }
            CompileError::Io { path, source } => vec![json!({
                "component": "neo-devpack-solidity",
                "severity": "error",
                "type": "IOError",
                "code": ErrorCode::Nsh0002.to_string(),
                "sourceLocation": { "file": file },
                "formattedMessage": format!("failed to access '{}': {source}", path.display()),
                "message": source.to_string(),
            })],
            CompileError::Message(message) => vec![json!({
                "component": "neo-devpack-solidity",
                "severity": "error",
                "type": "Generic",
                "code": ErrorCode::Nsh0000.to_string(),
                "sourceLocation": { "file": file },
                "formattedMessage": message,
                "message": message,
            })],
        }
    }
}

fn diagnostic_to_standard_error(diag: &Diagnostic, file: &str) -> Value {
    let severity = match diag.severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
    };

    let mut source_location = serde_json::Map::new();
    source_location.insert("file".to_string(), json!(file));
    if let Some(span) = diag.span {
        source_location.insert("start".to_string(), json!(span.offset));
        source_location.insert("end".to_string(), json!(span.offset + span.length));
    }

    let mut obj = json!({
        "component": "neo-devpack-solidity",
        "severity": severity,
        "type": error_type_for_code(&diag.code),
        "code": diag.code.to_string(),
        "sourceLocation": serde_json::Value::Object(source_location),
        "formattedMessage": diag.message,
        "message": diag.message,
    });

    if !diag.suggestions.is_empty() {
        obj["suggestions"] = json!(diag
            .suggestions
            .iter()
            .map(|s| json!({
                "message": s.message,
                "replacement": s.replacement,
            }))
            .collect::<Vec<_>>());
    }

    obj
}

fn error_type_for_code(code: &ErrorCode) -> &'static str {
    match code {
        ErrorCode::Nsh1000
        | ErrorCode::Nsh1001
        | ErrorCode::Nsh1002
        | ErrorCode::Nsh1003
        | ErrorCode::Nsh1004
        | ErrorCode::Nsh1005 => "ParseError",
        ErrorCode::Nsh2000 | ErrorCode::Nsh2001 | ErrorCode::Nsh2002 => "SemanticError",
        ErrorCode::Nsh3000
        | ErrorCode::Nsh3001
        | ErrorCode::Nsh3002
        | ErrorCode::Nsh3003
        | ErrorCode::Nsh3004
        | ErrorCode::Nsh3005 => "IrGeneration",
        ErrorCode::Nsh4000 | ErrorCode::Nsh4001 => "OptimizerError",
        ErrorCode::Nsh5000 | ErrorCode::Nsh5001 | ErrorCode::Nsh5002 => "CodegenError",
        ErrorCode::Nsh6000 | ErrorCode::Nsh6001 | ErrorCode::Nsh6002 => "ManifestGeneration",
        ErrorCode::Nsh0002 => "IOError",
        _ => "Generic",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::{ErrorCode, SourceSpan};

    #[test]
    fn manifest_error_renders_code() {
        let diag = Diagnostic::error(ErrorCode::Nsh6000, "manifest failed")
            .with_span(SourceSpan::at(1, 2));
        let err = CompileError::Manifest(Box::new(diag)).into_errors("test.sol");
        assert_eq!(err[0]["code"], "NSH-6000");
        assert_eq!(err[0]["severity"], "error");
    }

    #[test]
    fn message_error_fallback() {
        let err = CompileError::Message("fallback".into()).into_errors("test.sol");
        assert_eq!(err[0]["code"], "NSH-0000");
    }

    #[test]
    fn diagnostics_include_suggestions() {
        let diag = Diagnostic::error(ErrorCode::Nsh1000, "bad")
            .with_suggestion(crate::diagnostics::Suggestion::new("fix it").with_replacement("ok"));
        let err = CompileError::Diagnostics(vec![diag]).into_errors("test.sol");
        assert!(err[0]["suggestions"].is_array());
    }
}
