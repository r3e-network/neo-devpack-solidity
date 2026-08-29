use super::*;

impl StorageManager {
    /// Get read count
    pub fn read_count(&self) -> u64 {
        self.read_count
    }

    /// Get write count
    pub fn write_count(&self) -> u64 {
        self.write_count
    }

    /// Get storage statistics
    pub fn get_statistics(&self) -> StorageStatistics {
        let total_accounts = self.storage.len();
        let total_keys: usize = self.storage.values().map(|s| s.storage.len()).sum();
        let total_storage_bytes: usize = self
            .storage
            .values()
            .flat_map(|s| s.storage.iter())
            .map(|(k, v)| k.len() + v.len())
            .sum();
        let pending_changes: usize = self.storage.values().map(|s| s.pending_changes.len()).sum();

        StorageStatistics {
            total_accounts,
            total_keys,
            total_storage_bytes,
            read_operations: self.read_count,
            write_operations: self.write_count,
            pending_changes,
        }
    }

    /// Clear all storage (for testing)
    pub fn clear(&mut self) {
        self.storage.clear();
        self.read_count = 0;
        self.write_count = 0;
    }
}
