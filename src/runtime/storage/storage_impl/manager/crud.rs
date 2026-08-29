use super::*;

impl StorageManager {
    /// Get storage value
    pub fn get(&mut self, account: &str, key: &[u8]) -> Result<Option<Vec<u8>>, RuntimeError> {
        self.read_count += 1;

        let account_storage = self.storage.get(account);

        if let Some(storage) = account_storage {
            // Check pending changes first
            if let Some(change) = storage.pending_changes.get(key) {
                match change.change_type {
                    StorageChangeType::Delete => return Ok(None),
                    _ => {
                        if let Some(ref new_value) = change.new_value {
                            return Ok(Some(new_value.clone()));
                        }
                    }
                }
            }

            // Check committed storage
            if let Some(value) = storage.storage.get(key) {
                return Ok(Some(value.clone()));
            }
        }

        Ok(None)
    }

    /// Set storage value
    pub fn set(&mut self, account: &str, key: &[u8], value: &[u8]) -> Result<(), RuntimeError> {
        self.write_count += 1;

        let old_value = self.get(account, key)?;

        // Calculate gas cost
        let change_type = match &old_value {
            None => StorageChangeType::Create,
            Some(old) if old == value => StorageChangeType::NoChange,
            Some(_) if value.is_empty() => StorageChangeType::Delete,
            Some(_) => StorageChangeType::Update,
        };

        let gas_cost = self.calculate_storage_gas_cost(&change_type, value.len());

        // Get or create account storage
        let account_storage =
            self.storage
                .entry(account.to_string())
                .or_insert_with(|| AccountStorage {
                    account: account.to_string(),
                    storage: BTreeMap::new(),
                    pending_changes: HashMap::new(),
                });

        // Record pending change
        let change = StorageChange {
            key: key.to_vec(),
            old_value,
            new_value: if value.is_empty() {
                None
            } else {
                Some(value.to_vec())
            },
            change_type,
            gas_cost,
        };

        account_storage.pending_changes.insert(key.to_vec(), change);

        Ok(())
    }

    /// Delete storage value
    pub fn delete(&mut self, account: &str, key: &[u8]) -> Result<(), RuntimeError> {
        self.set(account, key, &[])
    }

    /// Check if storage key exists
    pub fn exists(&mut self, account: &str, key: &[u8]) -> Result<bool, RuntimeError> {
        Ok(self.get(account, key)?.is_some())
    }
}
