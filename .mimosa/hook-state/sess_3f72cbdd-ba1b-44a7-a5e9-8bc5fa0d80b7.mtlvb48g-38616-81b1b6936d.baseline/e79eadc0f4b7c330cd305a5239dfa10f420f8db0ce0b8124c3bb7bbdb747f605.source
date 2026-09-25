use super::*;

impl Default for StorageGasCosts {
    fn default() -> Self {
        // Neo N3 compatible gas costs
        Self {
            sstore_set: 8000,       // Create new storage entry
            sstore_reset: 8000,     // Update existing storage entry
            sstore_clear: 8000,     // Delete storage entry
            sload: 4000,            // Load from storage
            storage_byte_cost: 400, // Cost per byte stored
        }
    }
}
