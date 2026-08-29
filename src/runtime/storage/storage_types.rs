use super::*;

pub(crate) type StorageEntries = Vec<(Vec<u8>, Vec<u8>)>;

/// Storage manager for contract storage
#[derive(Debug)]
pub struct StorageManager {
    pub(crate) storage: HashMap<String, AccountStorage>,
    pub(crate) read_count: u64,
    pub(crate) write_count: u64,
    pub(crate) gas_costs: StorageGasCosts,
}

/// Account-specific storage
#[derive(Debug, Clone)]
pub struct AccountStorage {
    pub account: String,
    pub storage: BTreeMap<Vec<u8>, Vec<u8>>,
    pub pending_changes: HashMap<Vec<u8>, StorageChange>,
}

/// Storage change record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageChange {
    pub key: Vec<u8>,
    pub old_value: Option<Vec<u8>>,
    pub new_value: Option<Vec<u8>>,
    pub change_type: StorageChangeType,
    pub gas_cost: u64,
}

impl StorageChange {
    /// Check if this is a write operation
    pub fn is_write(&self) -> bool {
        matches!(
            self.change_type,
            StorageChangeType::Create | StorageChangeType::Update
        )
    }

    /// Get the key as hex string
    pub fn key_hex(&self) -> String {
        hex::encode(&self.key)
    }

    /// Get the size difference in bytes
    pub fn size_delta(&self) -> i64 {
        let old_size = self.old_value.as_ref().map(|v| v.len()).unwrap_or(0) as i64;
        let new_size = self.new_value.as_ref().map(|v| v.len()).unwrap_or(0) as i64;
        new_size - old_size
    }
}

/// Types of storage changes
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StorageChangeType {
    Create,   // New key-value pair
    Update,   // Update existing value
    Delete,   // Delete key-value pair
    NoChange, // Value unchanged
}

impl StorageChangeType {
    /// Check if this is a write operation
    pub fn is_write(&self) -> bool {
        matches!(self, Self::Create | Self::Update | Self::Delete)
    }

    /// Get type as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Update => "update",
            Self::Delete => "delete",
            Self::NoChange => "no_change",
        }
    }
}

/// Storage gas costs (Neo N3 compatible)
#[derive(Debug, Clone)]
pub struct StorageGasCosts {
    pub sstore_set: u64,        // Store new value
    pub sstore_reset: u64,      // Update existing value
    pub sstore_clear: u64,      // Clear existing value
    pub sload: u64,             // Load value
    pub storage_byte_cost: u64, // Cost per byte stored
}

/// Storage query options
#[derive(Debug, Default)]
pub struct StorageQuery {
    pub account: String,
    pub key_prefix: Option<Vec<u8>>,
    pub limit: Option<usize>,
    pub include_pending: bool,
    /// Cursor for paginated queries: only return entries whose key is
    /// strictly greater than this value (forward) or strictly less than
    /// this value (backward). Used by the streaming iterator to fetch
    /// pages on demand without materialising the entire result set.
    pub start_after_key: Option<Vec<u8>>,
}

impl StorageQuery {
    /// Create a query for an account
    pub fn for_account(account: impl Into<String>) -> Self {
        Self {
            account: account.into(),
            ..Default::default()
        }
    }
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StorageStatistics {
    pub total_accounts: usize,
    pub total_keys: usize,
    pub total_storage_bytes: usize,
    pub read_operations: u64,
    pub write_operations: u64,
    pub pending_changes: usize,
}

impl StorageStatistics {
    /// Get total operations
    pub fn total_operations(&self) -> u64 {
        self.read_operations + self.write_operations
    }

    /// Get read/write ratio
    pub fn read_write_ratio(&self) -> f64 {
        if self.write_operations == 0 {
            return f64::INFINITY;
        }
        self.read_operations as f64 / self.write_operations as f64
    }
}
