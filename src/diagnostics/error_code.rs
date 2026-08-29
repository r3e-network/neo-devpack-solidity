//! Structured `NSH-XXXX` error codes used throughout the compiler.
//!
//! Error codes are grouped by compiler phase. Each code has a short stable
//! identifier and a human-readable description. The numeric ranges are
//! documented in the architecture specification and must not be reassigned
//! without updating the regression test suite.

use std::fmt;

/// Stable error code identifier for compiler diagnostics.
///
/// Every user-visible error must carry one of these codes. The `Display`
/// implementation renders the code in the canonical `NSH-XXXX` form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum ErrorCode {
    // General / CLI / IO (NSH-0xxx)
    /// Unspecified general error, used only as a last resort.
    Nsh0000,
    /// CLI argument parsing failed or missing required input.
    Nsh0001,
    /// Failed to read or write a file.
    Nsh0002,
    /// Output directory is missing or not writable.
    Nsh0003,
    /// Unsupported compiler option or combination.
    Nsh0004,

    // Frontend / parser (NSH-1xxx)
    /// Solidity parsing failed.
    Nsh1000,
    /// Unsupported Solidity pragma version.
    Nsh1001,
    /// Import resolution failed.
    Nsh1002,
    /// Requested contract not found in source.
    Nsh1003,
    /// Unsupported top-level Solidity construct.
    Nsh1004,
    /// Empty source or no contract definitions found.
    Nsh1005,

    // Semantic analysis (NSH-2xxx)
    /// Semantic analysis error.
    Nsh2000,
    /// Type mismatch.
    Nsh2001,
    /// Visibility or state mutability violation.
    Nsh2002,

    // IR lowering (NSH-3xxx)
    /// IR lowering failed.
    Nsh3000,
    /// Unsupported expression or statement in IR lowering.
    Nsh3001,
    /// Unsupported function call or member access.
    Nsh3002,
    /// Array operation not supported.
    Nsh3003,
    /// Literal value out of range or invalid.
    Nsh3004,
    /// Return statement lowering failed.
    Nsh3005,

    // Optimizer (NSH-4xxx)
    /// Optimizer pass failed.
    Nsh4000,
    /// Constant folding overflow or invalid result.
    Nsh4001,

    // Codegen (NSH-5xxx)
    /// Bytecode generation failed.
    Nsh5000,
    /// Unsupported opcode lowering.
    Nsh5001,
    /// Stack or local slot overflow.
    Nsh5002,

    // Manifest (NSH-6xxx)
    /// Manifest generation failed.
    Nsh6000,
    /// Invalid permission or wildcard specification.
    Nsh6001,
    /// Standard detection or metadata error.
    Nsh6002,

    // Toolchain (NSH-7xxx)
    /// neo-forge or toolchain error.
    Nsh7000,

    // Internal (NSH-9xxx)
    /// Internal compiler error; should be reported as a bug.
    Nsh9000,
}

impl ErrorCode {
    /// Human-readable description of this error code.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Nsh0000 => "general compiler error",
            Self::Nsh0001 => "CLI argument error",
            Self::Nsh0002 => "file I/O error",
            Self::Nsh0003 => "output directory error",
            Self::Nsh0004 => "unsupported compiler option",
            Self::Nsh1000 => "Solidity parsing failed",
            Self::Nsh1001 => "unsupported Solidity version",
            Self::Nsh1002 => "import resolution failed",
            Self::Nsh1003 => "contract not found",
            Self::Nsh1004 => "unsupported Solidity construct",
            Self::Nsh1005 => "empty source or no contract definitions",
            Self::Nsh2000 => "semantic analysis error",
            Self::Nsh2001 => "type mismatch",
            Self::Nsh2002 => "visibility or state mutability violation",
            Self::Nsh3000 => "IR lowering failed",
            Self::Nsh3001 => "unsupported expression or statement",
            Self::Nsh3002 => "unsupported function call or member access",
            Self::Nsh3003 => "unsupported array operation",
            Self::Nsh3004 => "literal value out of range",
            Self::Nsh3005 => "return statement lowering failed",
            Self::Nsh4000 => "optimizer pass failed",
            Self::Nsh4001 => "constant folding overflow",
            Self::Nsh5000 => "bytecode generation failed",
            Self::Nsh5001 => "unsupported opcode lowering",
            Self::Nsh5002 => "stack or local slot overflow",
            Self::Nsh6000 => "manifest generation failed",
            Self::Nsh6001 => "invalid permission specification",
            Self::Nsh6002 => "standard detection or metadata error",
            Self::Nsh7000 => "toolchain error",
            Self::Nsh9000 => "internal compiler error",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The variant name is `NshXXXX`; render it as `NSH-XXXX`.
        let raw = format!("{self:?}");
        let digits = raw.trim_start_matches("Nsh");
        write!(f, "NSH-{digits}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_code_display() {
        assert_eq!(ErrorCode::Nsh1000.to_string(), "NSH-1000");
        assert_eq!(ErrorCode::Nsh3004.to_string(), "NSH-3004");
        assert_eq!(ErrorCode::Nsh9000.to_string(), "NSH-9000");
    }

    #[test]
    fn error_code_descriptions_non_empty() {
        for code in [
            ErrorCode::Nsh0000,
            ErrorCode::Nsh1001,
            ErrorCode::Nsh2000,
            ErrorCode::Nsh3000,
            ErrorCode::Nsh4000,
            ErrorCode::Nsh5000,
            ErrorCode::Nsh6000,
            ErrorCode::Nsh9000,
        ] {
            assert!(!code.description().is_empty());
        }
    }
}
