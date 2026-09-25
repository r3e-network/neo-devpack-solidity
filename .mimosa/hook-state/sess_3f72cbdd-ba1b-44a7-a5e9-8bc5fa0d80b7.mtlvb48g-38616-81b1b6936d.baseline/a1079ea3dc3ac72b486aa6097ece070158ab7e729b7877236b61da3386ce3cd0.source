use super::*;

impl StorageManager {
    /// Get storage size for account
    pub fn get_storage_size(&self, account: &str) -> Result<usize, RuntimeError> {
        if let Some(storage) = self.storage.get(account) {
            let committed_size: usize =
                storage.storage.iter().map(|(k, v)| k.len() + v.len()).sum();

            let pending_size: usize = storage
                .pending_changes
                .iter()
                .map(|(k, change)| {
                    k.len() + change.new_value.as_ref().map(|v| v.len()).unwrap_or(0)
                })
                .sum();

            Ok(committed_size + pending_size)
        } else {
            Ok(0)
        }
    }
}
