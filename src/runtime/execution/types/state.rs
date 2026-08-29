//! State management types for execution context.
//!
//! Provides structures for managing iterator state, contract state,
//! storage overlays, and isolated storage keys.

use super::stack::StackItem;

/// Iterator state for SYSCALL streaming operations
///
/// Uses a hybrid approach: a small pre-fetched buffer (`entries`) for
/// responsiveness, backed by an optional `StreamingCursor` that lazy-fetches
/// additional pages from the storage host when the buffer is exhausted.
#[derive(Debug, Clone)]
pub struct IteratorState {
    /// Pre-fetched entries (small initial batch, grows with lazy fetches)
    pub(crate) entries: Vec<StackItem>,
    /// Current position within `entries`
    pub(crate) index: usize,
    /// Cursor for fetching more entries on demand (None = fully materialized)
    pub(crate) cursor: Option<StreamingCursor>,
}

/// Cursor state for lazy batch fetching of storage iterator entries.
///
/// Carries the query parameters and pagination state so that `Iterator.Next`
/// can pull the next page from the storage host without materialising the
/// entire result set upfront.
#[derive(Debug, Clone)]
pub struct StreamingCursor {
    /// Storage key prefix being iterated
    pub(crate) prefix: Vec<u8>,
    /// Finder options flags (BACKWARDS, VALUES_ONLY, etc.)
    pub(crate) options: i64,
    /// Last key *returned* to the consumer (used as pagination cursor for
    /// the next host query; None = fetch from the beginning of the prefix).
    pub(crate) last_key: Option<Vec<u8>>,
    /// Whether the storage host has returned zero results for the last page
    /// (no more entries to fetch).
    pub(crate) exhausted: bool,
    /// Number of entries to fetch per page.
    pub(crate) page_size: usize,
}

/// Contract state for deployed contracts
#[derive(Debug, Clone)]
pub struct ContractState {
    pub(crate) id: u32,
    pub(crate) hash: [u8; 20],
    pub(crate) nef: Vec<u8>,
    pub(crate) manifest: Vec<u8>,
    pub(crate) update_counter: u32,
}

/// Storage overlay entry for transactional storage operations
#[derive(Debug, Clone)]
pub struct OverlayEntry {
    pub(crate) value: Option<Vec<u8>>,
    pub(crate) dirty: bool,
}

/// Storage key that includes account for isolation
/// Used for cross-contract storage isolation to prevent storage collisions
#[allow(dead_code)]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct IsolatedStorageKey {
    /// Account/contract that owns this storage
    account: String,
    /// The actual storage key
    key: Vec<u8>,
}

#[allow(dead_code)]
impl IsolatedStorageKey {
    fn new(account: &str, key: Vec<u8>) -> Self {
        Self {
            account: account.to_string(),
            key,
        }
    }
}

/// Whitelisted fee contract for Policy native contract
#[derive(Debug, Clone)]
pub struct WhitelistedFeeContract {
    /// Script hash of the whitelisted contract
    pub(crate) contract_hash: Vec<u8>,
    /// Maximum fee allowed (in GAS fractions)
    pub(crate) max_fee: u64,
}

/// Oracle request record for Oracle native contract
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OracleRequest {
    /// Unique request identifier
    pub(crate) id: u64,
    /// Original transaction hash that created the request
    pub(crate) original_tx_hash: Vec<u8>,
    /// URL to fetch data from
    pub(crate) url: String,
    /// JSONPath or other filter expression
    pub(crate) filter: String,
    /// Callback contract script hash
    pub(crate) callback_contract: Vec<u8>,
    /// Callback method name
    pub(crate) callback_method: String,
    /// User-provided data forwarded to callback
    pub(crate) user_data: Vec<u8>,
    /// GAS deposited for the request
    pub(crate) gas_for_response: u64,
}

/// Ledger block interop struct matching Neo N3 Block
#[derive(Debug, Clone)]
pub struct LedgerBlock {
    /// Block hash (32 bytes)
    pub(crate) hash: [u8; 32],
    /// Block version
    pub(crate) version: u32,
    /// Hash of the previous block
    pub(crate) prev_hash: [u8; 32],
    /// Merkle root of transactions
    pub(crate) merkle_root: [u8; 32],
    /// Block timestamp (milliseconds since epoch)
    pub(crate) timestamp: u64,
    /// Nonce for consensus
    pub(crate) nonce: u64,
    /// Block index (height)
    pub(crate) index: u64,
    /// Primary consensus node index
    pub(crate) primary_index: u8,
    /// Script hash of the next consensus group
    pub(crate) next_consensus: Vec<u8>,
    /// Number of transactions in the block
    pub(crate) transaction_count: u32,
}

/// Ledger transaction interop struct matching Neo N3 Transaction
#[derive(Debug, Clone)]
pub struct LedgerTransaction {
    /// Transaction hash (32 bytes)
    pub(crate) hash: [u8; 32],
    /// Transaction version
    pub(crate) version: u8,
    /// Random nonce
    pub(crate) nonce: u32,
    /// Sender script hash (20 bytes)
    pub(crate) sender: Vec<u8>,
    /// System fee (in GAS fractions)
    pub(crate) system_fee: u64,
    /// Network fee (in GAS fractions)
    pub(crate) network_fee: u64,
    /// Block index after which the tx is invalid
    pub(crate) valid_until_block: u32,
    /// Transaction script bytecode
    pub(crate) script: Vec<u8>,
}

/// Transaction signer matching Neo N3 Signer interop struct
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TransactionSigner {
    /// Signer account script hash (20 bytes)
    pub(crate) account: Vec<u8>,
    /// Witness scope flags
    pub(crate) scopes: u8,
    /// Allowed contracts (for CustomContracts scope)
    pub(crate) allowed_contracts: Vec<Vec<u8>>,
    /// Allowed groups (for CustomGroups scope)
    pub(crate) allowed_groups: Vec<Vec<u8>>,
}

/// Notary deposit tracking for Notary native contract
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NotaryDeposit {
    /// Depositor account script hash
    pub(crate) account: Vec<u8>,
    /// Deposited GAS amount (in fractions)
    pub(crate) amount: u64,
    /// Block index until which the deposit is locked
    pub(crate) till: u32,
}

/// Type alias for storage overlay entries
pub type StorageOverlayEntries = Vec<(Vec<u8>, Option<Vec<u8>>)>;
