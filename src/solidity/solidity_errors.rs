use super::*;

/// Errors from Solidity compilation
///
/// `#[allow(dead_code)]` covers the variants — every public entry point
/// in the cli (`compile.rs`) matches them, but they are not all
/// *constructed* in this crate (some are reserved for the public API).
#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum SolidityError {
    #[error("{0}")]
    Frontend(#[from] crate::frontend::FrontendError),

    #[error("{0}")]
    Analysis(String),

    #[error("no contract definitions found in source")]
    NoContracts,

    #[error("contract '{0}' not found")]
    ContractNotFound(String),

    #[error("unsupported feature: {0}")]
    UnsupportedFeature(String),

    #[error("inheritance error: {0}")]
    InheritanceError(String),
}

impl SolidityError {
    /// Create an analysis error
    #[allow(dead_code)]
    pub fn analysis(msg: impl Into<String>) -> Self {
        Self::Analysis(msg.into())
    }

    /// Create an unsupported feature error
    #[allow(dead_code)]
    pub fn unsupported(feature: impl Into<String>) -> Self {
        Self::UnsupportedFeature(feature.into())
    }
}
