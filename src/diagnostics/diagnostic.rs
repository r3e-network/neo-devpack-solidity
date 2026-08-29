//! Core diagnostic data structures.
//!
//! A [`Diagnostic`] is a self-contained, user-facing message with a stable
//! [`ErrorCode`], optional source location ([`SourceSpan`]), optional source
//! file path, and zero or more [`Suggestion`]s. It is the cross-phase
//! currency of the compiler's error reporting.

use crate::diagnostics::{ErrorCode, Severity};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Source location expressed in lines, columns, and byte offsets.
///
/// All fields are zero-indexed for internal use. Rendering code is
/// responsible for converting to one-indexed output when presenting to
/// users.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSpan {
    /// Zero-indexed start line.
    pub start_line: usize,
    /// Zero-indexed start column (in bytes or characters, depending on
    /// producer).
    pub start_col: usize,
    /// Zero-indexed end line, inclusive.
    pub end_line: usize,
    /// Zero-indexed end column, inclusive.
    pub end_col: usize,
    /// Start byte offset in the source file.
    pub offset: usize,
    /// Length of the span in bytes.
    pub length: usize,
}

impl SourceSpan {
    /// Create a single-point span at the given line and column.
    pub fn at(line: usize, col: usize) -> Self {
        Self {
            start_line: line,
            start_col: col,
            end_line: line,
            end_col: col,
            offset: 0,
            length: 0,
        }
    }

    /// Create a span from byte offsets.
    pub fn from_offsets(start: usize, end: usize) -> Self {
        Self {
            start_line: 0,
            start_col: 0,
            end_line: 0,
            end_col: 0,
            offset: start,
            length: end.saturating_sub(start),
        }
    }
}

/// An actionable suggestion attached to a diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Suggestion {
    /// Human-readable message describing the suggestion.
    pub message: String,
    /// Replacement text, if any.
    pub replacement: Option<String>,
    /// Location where the replacement should be applied.
    pub span: SourceSpan,
}

impl Suggestion {
    /// Create a suggestion with a message only.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            replacement: None,
            span: SourceSpan::at(0, 0),
        }
    }

    /// Attach a replacement string to this suggestion.
    pub fn with_replacement(mut self, replacement: impl Into<String>) -> Self {
        self.replacement = Some(replacement.into());
        self
    }
}

/// A unified compiler diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Stable error code (`NSH-XXXX`).
    pub code: ErrorCode,
    /// Severity of the diagnostic.
    pub severity: Severity,
    /// Human-readable message.
    pub message: String,
    /// Source file path, if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<PathBuf>,
    /// Source span, if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<SourceSpan>,
    /// Actionable suggestions.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub suggestions: Vec<Suggestion>,
}

impl Diagnostic {
    /// Create an error diagnostic with the given code and message.
    pub fn error(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            severity: Severity::Error,
            message: message.into(),
            file: None,
            span: None,
            suggestions: Vec::new(),
        }
    }

    /// Create a warning diagnostic with the given code and message.
    pub fn warning(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            severity: Severity::Warning,
            message: message.into(),
            file: None,
            span: None,
            suggestions: Vec::new(),
        }
    }

    /// Attach a source file path.
    pub fn with_file(mut self, file: impl Into<PathBuf>) -> Self {
        self.file = Some(file.into());
        self
    }

    /// Attach a source span.
    pub fn with_span(mut self, span: SourceSpan) -> Self {
        self.span = Some(span);
        self
    }

    /// Attach a suggestion.
    pub fn with_suggestion(mut self, suggestion: Suggestion) -> Self {
        self.suggestions.push(suggestion);
        self
    }
}

impl From<crate::solidity::Diagnostic> for Diagnostic {
    fn from(diag: crate::solidity::Diagnostic) -> Self {
        let severity = Severity::from(diag.severity);
        let code = diag
            .code
            .and_then(|c| error_code_from_legacy(&c))
            .unwrap_or(ErrorCode::Nsh2000);
        let mut suggestion = None;
        if let Some(s) = diag.suggestion {
            suggestion = Some(Suggestion::new(s));
        }
        let mut d = Self {
            code,
            severity,
            message: diag.message,
            file: None,
            span: None,
            suggestions: Vec::new(),
        };
        if let Some(s) = suggestion {
            d = d.with_suggestion(s);
        }
        d
    }
}

impl From<crate::ir::IrDiagnostic> for Diagnostic {
    fn from(diag: crate::ir::IrDiagnostic) -> Self {
        let code = diag
            .code
            .and_then(|c| error_code_from_legacy(&c))
            .unwrap_or(ErrorCode::Nsh3000);
        let suggestion = diag.suggestion.clone();
        let message = format!("function '{}': {}", diag.function_name, diag.message);
        let mut d = Self::error(code, message);
        if let Some(s) = suggestion {
            d = d.with_suggestion(Suggestion::new(s));
        }
        d
    }
}

impl From<crate::frontend::ParseDiagnostic> for Diagnostic {
    fn from(diag: crate::frontend::ParseDiagnostic) -> Self {
        Self::error(ErrorCode::Nsh1000, diag.message)
            .with_span(SourceSpan::from_offsets(diag.start, diag.end))
    }
}

fn error_code_from_legacy(code: &str) -> Option<ErrorCode> {
    Some(match code.chars().next()? {
        // Solidity semantic warnings / errors map to the generic semantic bucket
        // in the new error-code scheme. More specific codes (type mismatch, etc.)
        // can be added as the diagnostics system evolves.
        'E' | 'W' => ErrorCode::Nsh2000,
        // Parser diagnostics use a "P" prefix in the legacy system.
        'P' => ErrorCode::Nsh1000,
        // IR diagnostics use an "I" prefix in the legacy system.
        'I' => ErrorCode::Nsh3000,
        // Unknown legacy codes fall back to the per-phase default (semantic,
        // parser, or IR) rather than being forced to NSH-9000.
        _ => return None,
    })
}

/// Result type returned by compiler pipeline stages.
pub type CompilerResult<T> = Result<T, crate::cli::CompileError>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::{ErrorCode, Severity};

    #[test]
    fn diagnostic_builder() {
        let diag = Diagnostic::error(ErrorCode::Nsh1000, "parse failed")
            .with_file("test.sol")
            .with_span(SourceSpan::at(2, 5))
            .with_suggestion(Suggestion::new("check syntax").with_replacement("fixed"));

        assert_eq!(diag.code, ErrorCode::Nsh1000);
        assert_eq!(diag.severity, Severity::Error);
        assert_eq!(diag.message, "parse failed");
        assert_eq!(diag.file, Some(PathBuf::from("test.sol")));
        assert!(diag.span.is_some());
        assert_eq!(diag.suggestions.len(), 1);
    }

    #[test]
    fn source_span_from_offsets() {
        let span = SourceSpan::from_offsets(10, 25);
        assert_eq!(span.offset, 10);
        assert_eq!(span.length, 15);
    }

    #[test]
    fn suggestion_builder() {
        let s = Suggestion::new("use lowercase").with_replacement("lower");
        assert_eq!(s.message, "use lowercase");
        assert_eq!(s.replacement, Some("lower".to_string()));
    }
}
