use super::*;

impl StorageQuery {
    /// Create new storage query
    pub fn new(account: String) -> Self {
        Self {
            account,
            key_prefix: None,
            limit: None,
            include_pending: false,
            start_after_key: None,
        }
    }

    /// Set key prefix filter
    pub fn with_prefix(mut self, prefix: Vec<u8>) -> Self {
        self.key_prefix = Some(prefix);
        self
    }

    /// Set result limit
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Include pending changes in results
    pub fn include_pending(mut self) -> Self {
        self.include_pending = true;
        self
    }
}
