use super::*;

type StorageKeyValues = Vec<(Vec<u8>, Vec<u8>)>;

impl NeoRuntime {
    /// Get current state of the runtime
    pub fn get_state_snapshot(&self) -> state::StateSnapshot {
        self.state_manager.get_snapshot()
    }

    /// Restore state from snapshot
    pub fn restore_state(&mut self, snapshot: state::StateSnapshot) -> Result<(), RuntimeError> {
        self.state_manager
            .restore_snapshot(snapshot)
            .map_err(|e| RuntimeError::StateError {
                message: e.to_string(),
            })
    }

    /// Get storage value for account and key
    pub fn get_storage(
        &mut self,
        account: &str,
        key: &[u8],
    ) -> Result<Option<Vec<u8>>, RuntimeError> {
        self.storage_manager
            .get(account, key)
            .map_err(|e| RuntimeError::StorageError {
                message: e.to_string(),
            })
    }

    /// Set storage value for account and key
    pub fn set_storage(
        &mut self,
        account: &str,
        key: &[u8],
        value: &[u8],
    ) -> Result<(), RuntimeError> {
        self.storage_manager
            .set(account, key, value)
            .map_err(|e| RuntimeError::StorageError {
                message: e.to_string(),
            })
    }

    /// Find all storage `(key, value)` pairs for `account` whose keys start with
    /// `prefix`, returned in byte-lexicographic key order.
    ///
    /// Delegates to `StorageManager::query` with an explicit prefix filter; an
    /// empty `prefix` slice matches every key for the account (common "scan all"
    /// pattern used by Neo N3 storage iterators).
    pub fn storage_find(
        &mut self,
        account: &str,
        prefix: &[u8],
    ) -> Result<StorageKeyValues, RuntimeError> {
        let query = storage::StorageQuery {
            account: account.to_string(),
            key_prefix: Some(prefix.to_vec()),
            limit: None,
            include_pending: true,
            start_after_key: None,
        };
        self.storage_manager
            .query(query)
            .map_err(|e| RuntimeError::StorageError {
                message: e.to_string(),
            })
    }

    /// Get account balance
    pub fn get_balance(&self, account: &str) -> Result<u64, RuntimeError> {
        self.state_manager
            .get_balance(account)
            .map_err(|e| RuntimeError::StateError {
                message: e.to_string(),
            })
    }

    /// Set account balance
    pub fn set_balance(&mut self, account: &str, balance: u64) -> Result<(), RuntimeError> {
        self.state_manager
            .set_balance(account, balance)
            .map_err(|e| RuntimeError::StateError {
                message: e.to_string(),
            })
    }

    /// Get runtime statistics
    pub fn get_statistics(&self) -> RuntimeStatistics {
        RuntimeStatistics {
            total_gas_used: self.gas_tracker.used(),
            total_instructions_executed: self.execution_context.instruction_count(),
            max_stack_depth: self.execution_context.max_stack_depth(),
            storage_reads: self.storage_manager.read_count(),
            storage_writes: self.storage_manager.write_count(),
            state_changes: self.state_manager.change_count(),
        }
    }
}
