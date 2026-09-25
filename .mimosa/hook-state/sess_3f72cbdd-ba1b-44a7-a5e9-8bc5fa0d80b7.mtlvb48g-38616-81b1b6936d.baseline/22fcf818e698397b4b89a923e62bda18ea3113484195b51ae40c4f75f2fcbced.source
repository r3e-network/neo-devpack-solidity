//! Unified compiler diagnostics module.
//!
//! Provides the cross-phase error representation used by the Neo Solidity
//! compiler: [`Diagnostic`](crate::diagnostics::Diagnostic), [`ErrorCode`](crate::diagnostics::ErrorCode), [`SourceSpan`](crate::diagnostics::SourceSpan), [`Severity`](crate::diagnostics::Severity), and
//! [`Suggestion`](crate::diagnostics::Suggestion). All compiler stages (parser, semantic analysis, IR lowering,
//! optimizer, codegen, manifest) are expected to surface user-visible errors
//! through [`Diagnostic`](crate::diagnostics::Diagnostic) so the CLI can render them consistently with an
//! `NSH-XXXX` code and source location.
//!
//! The module intentionally does not depend on external diagnostic rendering
//! libraries such as `miette` or `ariadne`; the formatting lives in
//! `report::Report` and can be replaced later without changing the core data model.

mod diagnostic;
mod error_code;
mod report;

pub use diagnostic::{CompilerResult, Diagnostic, SourceSpan, Suggestion};
pub use error_code::ErrorCode;

use std::fmt;

/// Severity of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Severity {
    /// Error that prevents successful compilation.
    Error,
    /// Warning that does not stop compilation but should be addressed.
    Warning,
    /// Informational note, usually attached to a primary error or warning.
    Info,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Error => write!(f, "error"),
            Self::Warning => write!(f, "warning"),
            Self::Info => write!(f, "info"),
        }
    }
}

impl From<crate::solidity::DiagnosticSeverity> for Severity {
    fn from(severity: crate::solidity::DiagnosticSeverity) -> Self {
        match severity {
            crate::solidity::DiagnosticSeverity::Error => Self::Error,
            crate::solidity::DiagnosticSeverity::Warning => Self::Warning,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_display() {
        assert_eq!(Severity::Error.to_string(), "error");
        assert_eq!(Severity::Warning.to_string(), "warning");
        assert_eq!(Severity::Info.to_string(), "info");
    }

    #[test]
    fn severity_from_solidity() {
        assert_eq!(
            Severity::from(crate::solidity::DiagnosticSeverity::Error),
            Severity::Error
        );
        assert_eq!(
            Severity::from(crate::solidity::DiagnosticSeverity::Warning),
            Severity::Warning
        );
    }
}
