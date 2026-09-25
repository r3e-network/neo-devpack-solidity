use super::*;

impl StorageManager {
    pub(crate) fn calculate_storage_gas_cost(
        &self,
        change_type: &StorageChangeType,
        value_size: usize,
    ) -> u64 {
        match change_type {
            StorageChangeType::Create => {
                self.gas_costs.sstore_set + (value_size as u64 * self.gas_costs.storage_byte_cost)
            }
            StorageChangeType::Update => {
                self.gas_costs.sstore_reset + (value_size as u64 * self.gas_costs.storage_byte_cost)
            }
            StorageChangeType::Delete => self.gas_costs.sstore_clear,
            StorageChangeType::NoChange => 0,
        }
    }

    pub(crate) fn calculate_merkle_root(&self, items: &[(&Vec<u8>, &Vec<u8>)]) -> Vec<u8> {
        use sha3::{Digest, Keccak256};

        if items.is_empty() {
            return self.empty_root_hash();
        }

        // Simple implementation - hash all items together
        // In a real implementation, this would build a proper Merkle tree
        let mut hasher = Keccak256::new();
        for (key, value) in items {
            hasher.update(key);
            hasher.update(value);
        }
        hasher.finalize().to_vec()
    }

    pub(crate) fn empty_root_hash(&self) -> Vec<u8> {
        use sha3::{Digest, Keccak256};
        Keccak256::digest(b"").to_vec()
    }
}
