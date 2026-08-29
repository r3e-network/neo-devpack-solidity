use super::*;
use crate::runtime::RuntimeConfig;

#[test]
fn test_storage_operations() {
    let config = RuntimeConfig::default();
    let mut storage = StorageManager::new(&config).unwrap();

    let account = "0x1234567890123456789012345678901234567890";
    let key = b"test_key";
    let value = b"test_value";

    // Initially empty
    assert!(storage.get(account, key).unwrap().is_none());
    assert!(!storage.exists(account, key).unwrap());

    // Set value
    storage.set(account, key, value).unwrap();
    assert_eq!(storage.get(account, key).unwrap(), Some(value.to_vec()));
    assert!(storage.exists(account, key).unwrap());

    // Update value
    let new_value = b"updated_value";
    storage.set(account, key, new_value).unwrap();
    assert_eq!(storage.get(account, key).unwrap(), Some(new_value.to_vec()));

    // Delete value
    storage.delete(account, key).unwrap();
    assert!(storage.get(account, key).unwrap().is_none());
}

#[test]
fn test_pending_changes() {
    let config = RuntimeConfig::default();
    let mut storage = StorageManager::new(&config).unwrap();

    let account = "0x1234567890123456789012345678901234567890";
    let key = b"test_key";
    let value = b"test_value";

    // Set value (creates pending change)
    storage.set(account, key, value).unwrap();

    // Should have pending changes
    let pending = storage.get_pending_changes(account);
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].key, key);
    assert_eq!(pending[0].new_value, Some(value.to_vec()));

    // Commit changes
    let changes = storage.commit(account).unwrap();
    assert_eq!(changes.len(), 1);

    // No more pending changes
    let pending = storage.get_pending_changes(account);
    assert!(pending.is_empty());

    // Value still accessible
    assert_eq!(storage.get(account, key).unwrap(), Some(value.to_vec()));
}

#[test]
fn test_rollback() {
    let config = RuntimeConfig::default();
    let mut storage = StorageManager::new(&config).unwrap();

    let account = "0x1234567890123456789012345678901234567890";
    let key = b"test_key";
    let value = b"test_value";

    // Set initial value and commit
    storage.set(account, key, value).unwrap();
    storage.commit(account).unwrap();

    // Update value (pending)
    let new_value = b"updated_value";
    storage.set(account, key, new_value).unwrap();
    assert_eq!(storage.get(account, key).unwrap(), Some(new_value.to_vec()));

    // Rollback
    storage.rollback(account).unwrap();

    // Should return to committed value
    assert_eq!(storage.get(account, key).unwrap(), Some(value.to_vec()));
    assert!(storage.get_pending_changes(account).is_empty());
}

#[test]
fn test_storage_query() {
    let config = RuntimeConfig::default();
    let mut storage = StorageManager::new(&config).unwrap();

    let account = "0x1234567890123456789012345678901234567890";

    // Set up test data
    storage.set(account, b"prefix_key1", b"value1").unwrap();
    storage.set(account, b"prefix_key2", b"value2").unwrap();
    storage.set(account, b"other_key", b"value3").unwrap();
    storage.commit(account).unwrap();

    // Query with prefix
    let query = StorageQuery::new(account.to_string()).with_prefix(b"prefix_".to_vec());
    let results = storage.query(query).unwrap();

    assert_eq!(results.len(), 2);
    assert!(results
        .iter()
        .any(|(k, v)| k == b"prefix_key1" && v == b"value1"));
    assert!(results
        .iter()
        .any(|(k, v)| k == b"prefix_key2" && v == b"value2"));

    // Query with limit
    let query = StorageQuery::new(account.to_string()).with_limit(2);
    let results = storage.query(query).unwrap();

    assert_eq!(results.len(), 2);
}

#[test]
fn test_gas_cost_calculation() {
    let config = RuntimeConfig::default();
    let mut storage = StorageManager::new(&config).unwrap();

    let account = "0x1234567890123456789012345678901234567890";
    let key = b"test_key";
    let value = b"test_value";

    // Set value
    storage.set(account, key, value).unwrap();

    let gas_cost = storage.calculate_pending_gas_cost(account);
    assert!(gas_cost > 0);

    // Cost should include base cost plus per-byte cost
    let expected_cost =
        storage.gas_costs.sstore_set + (value.len() as u64 * storage.gas_costs.storage_byte_cost);
    assert_eq!(gas_cost, expected_cost);
}

#[test]
fn test_storage_statistics() {
    let config = RuntimeConfig::default();
    let mut storage = StorageManager::new(&config).unwrap();

    let account1 = "0x1111111111111111111111111111111111111111";
    let account2 = "0x2222222222222222222222222222222222222222";

    // Add some data
    storage.set(account1, b"key1", b"value1").unwrap();
    storage.set(account1, b"key2", b"value2").unwrap();
    storage.set(account2, b"key3", b"value3").unwrap();

    // Some committed, some pending
    storage.commit(account1).unwrap();

    let stats = storage.get_statistics();
    assert_eq!(stats.total_accounts, 2);
    assert!(stats.total_keys >= 2); // At least the committed ones
    assert!(stats.total_storage_bytes > 0);
    assert!(stats.read_operations > 0);
    assert!(stats.write_operations > 0);
    assert!(stats.pending_changes > 0);
}

#[test]
fn test_storage_root_hash() {
    let config = RuntimeConfig::default();
    let mut storage = StorageManager::new(&config).unwrap();

    let account = "0x1234567890123456789012345678901234567890";

    // Empty storage should have empty root
    let empty_root = storage.get_storage_root(account).unwrap();
    assert!(!empty_root.is_empty());

    // Add some data and commit
    storage.set(account, b"key1", b"value1").unwrap();
    storage.set(account, b"key2", b"value2").unwrap();
    storage.commit(account).unwrap();

    let root_with_data = storage.get_storage_root(account).unwrap();
    assert_ne!(empty_root, root_with_data);

    // Same data should produce same root
    let root_again = storage.get_storage_root(account).unwrap();
    assert_eq!(root_with_data, root_again);
}
