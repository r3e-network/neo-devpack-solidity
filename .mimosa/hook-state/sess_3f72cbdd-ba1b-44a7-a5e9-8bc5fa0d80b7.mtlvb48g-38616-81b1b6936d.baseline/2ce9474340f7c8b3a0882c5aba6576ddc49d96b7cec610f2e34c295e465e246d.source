use super::*;

impl StorageManager {
    /// Commit pending changes
    pub fn commit(&mut self, account: &str) -> Result<Vec<StorageChange>, RuntimeError> {
        if let Some(account_storage) = self.storage.get_mut(account) {
            let changes: Vec<StorageChange> =
                account_storage.pending_changes.values().cloned().collect();

            // Apply changes to committed storage
            for (key, change) in account_storage.pending_changes.drain() {
                match change.change_type {
                    StorageChangeType::Create | StorageChangeType::Update => {
                        if let Some(new_value) = change.new_value {
                            account_storage.storage.insert(key, new_value);
                        }
                    }
                    StorageChangeType::Delete => {
                        account_storage.storage.remove(&key);
                    }
                    StorageChangeType::NoChange => {
                        // No change needed
                    }
                }
            }

            Ok(changes)
        } else {
            Ok(Vec::new())
        }
    }

    /// Rollback pending changes
    pub fn rollback(&mut self, account: &str) -> Result<(), RuntimeError> {
        if let Some(account_storage) = self.storage.get_mut(account) {
            account_storage.pending_changes.clear();
        }
        Ok(())
    }

    /// Get all pending changes for account
    pub fn get_pending_changes(&self, account: &str) -> Vec<&StorageChange> {
        self.storage
            .get(account)
            .map(|storage| storage.pending_changes.values().collect())
            .unwrap_or_default()
    }

    /// Calculate total gas cost for pending changes
    pub fn calculate_pending_gas_cost(&self, account: &str) -> u64 {
        self.storage
            .get(account)
            .map(|storage| storage.pending_changes.values().map(|c| c.gas_cost).sum())
            .unwrap_or(0)
    }
}
