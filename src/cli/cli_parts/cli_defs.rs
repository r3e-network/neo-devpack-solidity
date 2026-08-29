use super::*;
use crate::diagnostics::Diagnostic;
use std::path::PathBuf;

pub(crate) const COMPILER_ID: &str = concat!("neo-devpack-solidity-", env!("CARGO_PKG_VERSION"));
pub(crate) const COMPILER_EMAIL: &str = "Jimmy <jimmy@r3e.network>";
pub(crate) const VERSION_STR: &str = env!("CARGO_PKG_VERSION");

pub(crate) fn compiler_version_string_4() -> String {
    let mut parts = VERSION_STR
        .split('.')
        .map(|p| p.parse::<u32>().unwrap_or(0));
    let major = parts.next().unwrap_or(0);
    let minor = parts.next().unwrap_or(0);
    let patch = parts.next().unwrap_or(0);
    format!("{major}.{minor}.{patch}.0")
}

/// Print a fatal error message to stderr and exit with code 1.
///
/// This is the standard CLI fatal-error pattern: all user-facing fatal
/// errors go through this function so the pattern is centralized and
/// consistent. Use this instead of raw `eprintln!` + `exit(1)`.
///
/// # Safety / correctness note
///
/// `fatal_error!` must only be invoked from the CLI entry layer (this
/// module or `src/cli/cli_parts/cli_run`). Compiler internals should
/// propagate structured errors via [`CompileError`] instead.
#[macro_export]
macro_rules! fatal_error {
    ($($arg:tt)*) => {{
        eprintln!($($arg)*);
        std::process::exit(1);
    }};
}

#[derive(Clone, Debug)]
pub struct CompilationArtifacts {
    pub metadata: ContractMetadata,
    pub bytecode: Vec<u8>,
    pub tokens: Vec<neo_devpack_solidity::neo::MethodToken>,
    pub manifest: Value,
    pub warnings: Vec<neo_devpack_solidity::solidity::Diagnostic>,
}

/// The top-level error type returned by the compiler pipeline.
///
/// All variants carry one or more [`Diagnostic`]s with a stable `NSH-XXXX`
/// error code. The legacy internal diagnostic types are converted to
/// [`Diagnostic`] when constructing a `CompileError`.
#[derive(Debug)]
pub enum CompileError {
    /// Frontend diagnostics (warnings and errors) from source analysis.
    Diagnostics(Vec<Diagnostic>),
    /// Semantic-analysis diagnostics.
    Semantic(Vec<Diagnostic>),
    /// IR lowering diagnostics.
    Ir(Vec<Diagnostic>),
    /// Manifest-generation diagnostic.
    Manifest(Box<Diagnostic>),
    /// Structured parser diagnostics.
    ParseErrors(Vec<Diagnostic>),
    /// IO failure while reading a source file or writing an output.
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    /// Fallback message-only error, used only during transition.
    Message(String),
}

impl CompileError {
    /// Convert a collection of frontend diagnostics into a compile error.
    pub fn from_solidity_diagnostics(diags: Vec<crate::solidity::Diagnostic>) -> Self {
        Self::Diagnostics(diags.into_iter().map(Diagnostic::from).collect())
    }

    /// Convert a collection of IR diagnostics into a compile error.
    pub fn from_ir_diagnostics(diags: Vec<crate::ir::IrDiagnostic>) -> Self {
        Self::Ir(diags.into_iter().map(Diagnostic::from).collect())
    }

    /// Convert a collection of parser diagnostics into a compile error.
    pub fn from_parse_diagnostics(diags: Vec<crate::frontend::ParseDiagnostic>) -> Self {
        Self::ParseErrors(diags.into_iter().map(Diagnostic::from).collect())
    }
}
