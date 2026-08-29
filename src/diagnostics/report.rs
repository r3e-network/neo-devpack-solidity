//! Diagnostic rendering for terminal and JSON consumers.
//!
//! The [`Report`] type turns a collection of [`Diagnostic`]s into a string
//! suitable for stderr or standard-JSON output. It is intentionally simple:
//! no external rendering libraries are used, so the output can be evolved
//! incrementally and the unit tests remain fast and deterministic.
//!
//! The public API is currently exercised primarily by unit tests; the CLI
//! uses the lower-level `emit_error`/`emit_warning` helpers. The formatter
//! will be wired into the main reporting path in P1 (diagnostic formatting
//! upgrade).
#![allow(dead_code)]

use crate::diagnostics::{Diagnostic, Severity, SourceSpan};
use serde_json::Value;

/// Formatter for a batch of diagnostics.
#[derive(Debug, Clone, Default)]
pub struct Report {
    diagnostics: Vec<Diagnostic>,
}

impl Report {
    /// Create an empty report.
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    /// Create a report from a single diagnostic.
    pub fn from_diagnostic(diagnostic: Diagnostic) -> Self {
        Self {
            diagnostics: vec![diagnostic],
        }
    }

    /// Add a diagnostic to this report.
    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Add many diagnostics to this report.
    pub fn extend(&mut self, diagnostics: impl IntoIterator<Item = Diagnostic>) {
        self.diagnostics.extend(diagnostics);
    }

    /// Return true if the report contains at least one error.
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
    }

    /// Format the report as a human-readable string.
    pub fn render(&self) -> String {
        if self.diagnostics.is_empty() {
            return String::new();
        }

        let mut out = String::new();
        for (idx, diagnostic) in self.diagnostics.iter().enumerate() {
            if idx > 0 {
                out.push('\n');
            }
            self.render_diagnostic(diagnostic, &mut out);
        }
        out
    }

    fn render_diagnostic(&self, diagnostic: &Diagnostic, out: &mut String) {
        let severity = diagnostic.severity.to_string();
        let location = diagnostic
            .file
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        let span = diagnostic.span.map(render_span).unwrap_or_default();

        let location_part = match (location.is_empty(), span.is_empty()) {
            (true, true) => String::new(),
            (false, true) => format!("{location}: "),
            (true, false) => format!("{span}: "),
            (false, false) => format!("{location}{span}: "),
        };

        out.push_str(&format!(
            "{severity}: {}{}: {}\n",
            location_part, diagnostic.code, diagnostic.message
        ));

        for suggestion in &diagnostic.suggestions {
            out.push_str(&format!("  help: {}\n", suggestion.message));
            if let Some(replacement) = &suggestion.replacement {
                out.push_str(&format!("    -> {replacement}\n"));
            }
        }
    }

    /// Render the report as a JSON value compatible with solc's standard-JSON
    /// error objects.
    pub fn to_json(&self) -> Value {
        let errors: Vec<Value> = self.diagnostics.iter().map(diagnostic_to_json).collect();
        serde_json::Value::Array(errors)
    }
}

fn render_span(span: SourceSpan) -> String {
    if span.start_line == span.end_line {
        if span.start_col == span.end_col {
            format!(":{}:{}", span.start_line + 1, span.start_col + 1)
        } else {
            format!(
                ":{}:{}-{}:{}",
                span.start_line + 1,
                span.start_col + 1,
                span.start_line + 1,
                span.end_col + 1
            )
        }
    } else {
        format!(
            ":{}:{}-{}:{}",
            span.start_line + 1,
            span.start_col + 1,
            span.end_line + 1,
            span.end_col + 1
        )
    }
}

fn diagnostic_to_json(diagnostic: &Diagnostic) -> Value {
    let mut obj = serde_json::json!({
        "severity": diagnostic.severity.to_string(),
        "type": if diagnostic.severity == Severity::Error { "CompilerError" } else { "CompilerWarning" },
        "code": diagnostic.code.to_string(),
        "message": diagnostic.message,
        "formattedMessage": diagnostic.message,
    });

    let mut location = serde_json::Map::new();
    if let Some(file) = &diagnostic.file {
        location.insert("file".to_string(), serde_json::json!(file));
    }
    if let Some(span) = diagnostic.span {
        location.insert(
            "start".to_string(),
            serde_json::json!({
                "line": span.start_line,
                "column": span.start_col,
                "offset": span.offset,
            }),
        );
        location.insert(
            "end".to_string(),
            serde_json::json!({
                "line": span.end_line,
                "column": span.end_col,
                "offset": span.offset + span.length,
            }),
        );
    }
    if !location.is_empty() {
        obj.as_object_mut()
            .expect("diagnostic JSON must be an object")
            .insert(
                "sourceLocation".to_string(),
                serde_json::Value::Object(location),
            );
    }

    if !diagnostic.suggestions.is_empty() {
        obj["suggestions"] = serde_json::json!(diagnostic
            .suggestions
            .iter()
            .map(|s| serde_json::json!({
                "message": s.message,
                "replacement": s.replacement,
            }))
            .collect::<Vec<_>>());
    }

    obj
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::{ErrorCode, Suggestion};

    #[test]
    fn empty_report_renders_empty() {
        let report = Report::new();
        assert_eq!(report.render(), "");
        assert!(!report.has_errors());
    }

    #[test]
    fn report_detects_errors() {
        let mut report = Report::new();
        report.push(Diagnostic::warning(ErrorCode::Nsh0000, "a warning"));
        assert!(!report.has_errors());
        report.push(Diagnostic::error(ErrorCode::Nsh1000, "an error"));
        assert!(report.has_errors());
    }

    #[test]
    fn render_includes_code_and_location() {
        let diag = Diagnostic::error(ErrorCode::Nsh1000, "parse failed")
            .with_file("test.sol")
            .with_span(SourceSpan::at(2, 5));
        let report = Report::from_diagnostic(diag);
        let rendered = report.render();
        assert!(rendered.contains("NSH-1000"));
        assert!(rendered.contains("test.sol:3:6"));
        assert!(rendered.contains("parse failed"));
    }

    #[test]
    fn json_contains_code_and_location() {
        let diag = Diagnostic::error(ErrorCode::Nsh1000, "parse failed")
            .with_file("test.sol")
            .with_span(SourceSpan::at(2, 5));
        let json = Report::from_diagnostic(diag).to_json();
        let first = json.as_array().unwrap().first().unwrap();
        assert_eq!(first["code"], "NSH-1000");
        assert_eq!(first["message"], "parse failed");
        assert!(first["sourceLocation"].is_object());
    }

    #[test]
    fn suggestion_rendered() {
        let diag = Diagnostic::error(ErrorCode::Nsh1000, "bad syntax")
            .with_suggestion(Suggestion::new("fix it").with_replacement("fixed"));
        let rendered = Report::from_diagnostic(diag).render();
        assert!(rendered.contains("help: fix it"));
    }
}
