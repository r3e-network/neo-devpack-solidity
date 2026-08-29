use super::*;

impl StorageManager {
    /// Query storage with filters
    pub fn query(&mut self, query: StorageQuery) -> Result<StorageEntries, RuntimeError> {
        let mut results = Vec::new();

        if let Some(account_storage) = self.storage.get(&query.account) {
            // Collect from committed storage
            for (key, value) in &account_storage.storage {
                if let Some(ref prefix) = query.key_prefix {
                    if !key.starts_with(prefix) {
                        continue;
                    }
                }
                results.push((key.clone(), value.clone()));
            }

            // Include pending changes if requested
            if query.include_pending {
                for (key, change) in &account_storage.pending_changes {
                    if let Some(ref prefix) = query.key_prefix {
                        if !key.starts_with(prefix) {
                            continue;
                        }
                    }

                    match &change.change_type {
                        StorageChangeType::Delete => {
                            results.retain(|(k, _)| k != key);
                        }
                        _ => {
                            if let Some(ref new_value) = change.new_value {
                                // Remove old entry if exists and add new one
                                results.retain(|(k, _)| k != key);
                                results.push((key.clone(), new_value.clone()));
                            }
                        }
                    }
                }
            }
        }

        // Sort by key for consistent results (must happen before cursor filtering)
        results.sort_by(|a, b| a.0.cmp(&b.0));

        // Apply cursor-based pagination: filter out keys before the cursor
        if let Some(ref cursor) = query.start_after_key {
            results.retain(|(k, _)| k.as_slice() > cursor.as_slice());
        }

        // Apply limit
        if let Some(limit) = query.limit {
            results.truncate(limit);
        }

        Ok(results)
    }

    /// Get storage root hash for account (Neo compatibility)
    pub fn get_storage_root(&self, account: &str) -> Result<String, RuntimeError> {
        if let Some(account_storage) = self.storage.get(account) {
            // Calculate Merkle root of storage items
            let mut items: Vec<_> = account_storage.storage.iter().collect();
            items.sort_by_key(|(k, _)| *k);

            let root_hash = self.calculate_merkle_root(&items);
            Ok(hex::encode(root_hash))
        } else {
            Ok(hex::encode(self.empty_root_hash()))
        }
    }
}
