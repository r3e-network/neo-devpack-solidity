use super::*;

/// Natspec documentation extracted from Solidity source comments
#[derive(Debug, Clone, Default)]
pub struct NatspecDoc {
    /// @title - Contract title
    pub title: Option<String>,
    /// @author - Contract/function author
    pub author: Option<String>,
    /// @notice - Human-readable description for end users
    pub notice: Option<String>,
    /// @dev - Technical details for developers
    pub dev: Option<String>,
    /// @param descriptions - key is parameter name, value is description
    pub params: Vec<(String, String)>,
    /// @return descriptions
    pub returns: Vec<String>,
    /// @custom tags
    pub custom: Vec<(String, String)>,
}

impl From<NatspecDocIR> for NatspecDoc {
    fn from(ir: NatspecDocIR) -> Self {
        NatspecDoc {
            title: ir.title,
            author: ir.author,
            notice: ir.notice,
            dev: ir.dev,
            params: ir.params,
            returns: ir.returns,
            custom: ir.custom,
        }
    }
}
