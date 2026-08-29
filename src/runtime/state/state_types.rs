use super::*;

/// State manager for account and contract states
#[derive(Debug, Clone, Default)]
pub struct StateManager {
    pub(crate) accounts: HashMap<String, AccountState>,
    pub(crate) snapshots: Vec<StateSnapshot>,
    pub(crate) change_log: Vec<StateChange>,
    pub(crate) change_count: u64,
}

/// Account state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountState {
    pub address: String,
    pub balance: u64,
    pub nonce: u64,
    pub code: Option<Vec<u8>>,
    pub code_hash: Option<String>,
    pub storage_root: Option<String>,
    pub created_at: u64,
}

impl AccountState {
    /// Check if this is a contract account
    pub fn is_contract(&self) -> bool {
        self.code.is_some()
    }

    /// Get code size in bytes
    pub fn code_size(&self) -> usize {
        self.code.as_ref().map(|c| c.len()).unwrap_or(0)
    }
}

/// State snapshot for rollback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub id: u64,
    pub timestamp: u64,
    pub accounts: HashMap<String, AccountState>,
    pub description: String,
}

/// State query interface
#[derive(Debug, Default)]
pub struct StateQuery {
    pub account: Option<String>,
    pub include_code: bool,
    pub include_storage: bool,
    pub block_number: Option<u64>,
}

impl StateQuery {
    /// Create a query for a specific account
    pub fn for_account(address: impl Into<String>) -> Self {
        Self {
            account: Some(address.into()),
            ..Default::default()
        }
    }

    /// Include code in the query result
    pub fn with_code(mut self) -> Self {
        self.include_code = true;
        self
    }

    /// Include storage in the query result
    pub fn with_storage(mut self) -> Self {
        self.include_storage = true;
        self
    }
}

/// State update batch
#[derive(Debug)]
pub struct StateBatch {
    pub changes: Vec<StateChange>,
    pub atomic: bool,
}

/// State statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateStatistics {
    pub total_accounts: usize,
    pub total_balance: u64,
    pub contract_accounts: usize,
    pub external_accounts: usize,
    pub total_changes: u64,
    pub snapshots_count: usize,
}
