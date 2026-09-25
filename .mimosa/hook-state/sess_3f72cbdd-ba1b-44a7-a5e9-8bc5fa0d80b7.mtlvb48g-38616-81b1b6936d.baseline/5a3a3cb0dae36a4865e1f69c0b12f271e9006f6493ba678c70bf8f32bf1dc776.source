use super::*;
use crate::runtime::RuntimeConfig;

#[test]
fn test_account_creation() {
    let config = RuntimeConfig::default();
    let mut state_manager = StateManager::new(&config).unwrap();

    let address = "0x1234567890123456789012345678901234567890";
    let balance = 1000u64;

    assert!(!state_manager.account_exists(address));

    let result = state_manager.create_account(address, balance);
    assert!(result.is_ok());
    assert!(state_manager.account_exists(address));
    assert_eq!(state_manager.get_balance(address).unwrap(), balance);
}

#[test]
fn test_balance_operations() {
    let config = RuntimeConfig::default();
    let mut state_manager = StateManager::new(&config).unwrap();

    let address = "0x1234567890123456789012345678901234567890";

    // Set balance for non-existent account (should create it)
    let result = state_manager.set_balance(address, 500);
    assert!(result.is_ok());
    assert_eq!(state_manager.get_balance(address).unwrap(), 500);

    // Update balance
    let result = state_manager.set_balance(address, 1000);
    assert!(result.is_ok());
    assert_eq!(state_manager.get_balance(address).unwrap(), 1000);
}

#[test]
fn test_transfer() {
    let config = RuntimeConfig::default();
    let mut state_manager = StateManager::new(&config).unwrap();

    let from = "0x1111111111111111111111111111111111111111";
    let to = "0x2222222222222222222222222222222222222222";

    // Set up accounts
    state_manager.create_account(from, 1000).unwrap();
    state_manager.create_account(to, 0).unwrap();

    // Transfer
    let result = state_manager.transfer(from, to, 300);
    assert!(result.is_ok());

    assert_eq!(state_manager.get_balance(from).unwrap(), 700);
    assert_eq!(state_manager.get_balance(to).unwrap(), 300);
}

#[test]
fn test_insufficient_balance_transfer() {
    let config = RuntimeConfig::default();
    let mut state_manager = StateManager::new(&config).unwrap();

    let from = "0x1111111111111111111111111111111111111111";
    let to = "0x2222222222222222222222222222222222222222";

    state_manager.create_account(from, 100).unwrap();
    state_manager.create_account(to, 0).unwrap();

    // Try to transfer more than balance
    let result = state_manager.transfer(from, to, 200);
    assert!(result.is_err());

    // Balances should remain unchanged
    assert_eq!(state_manager.get_balance(from).unwrap(), 100);
    assert_eq!(state_manager.get_balance(to).unwrap(), 0);
}

#[test]
fn test_snapshots() {
    let config = RuntimeConfig::default();
    let mut state_manager = StateManager::new(&config).unwrap();

    let address = "0x1234567890123456789012345678901234567890";

    // Initial state
    state_manager.create_account(address, 1000).unwrap();

    // Create snapshot
    let snapshot_id = state_manager.create_snapshot("Initial state".to_string());

    // Modify state
    state_manager.set_balance(address, 2000).unwrap();
    assert_eq!(state_manager.get_balance(address).unwrap(), 2000);

    // Restore snapshot
    if let Some(snapshot) = state_manager.snapshots.get(snapshot_id as usize) {
        let result = state_manager.restore_snapshot(snapshot.clone());
        assert!(result.is_ok());
    }

    // State should be restored
    assert_eq!(state_manager.get_balance(address).unwrap(), 1000);
}

#[test]
fn test_code_operations() {
    let config = RuntimeConfig::default();
    let mut state_manager = StateManager::new(&config).unwrap();

    let address = "0x1234567890123456789012345678901234567890";
    let code = vec![0x60, 0x01, 0x60, 0x02, 0x01]; // Simple bytecode

    // Set code
    let result = state_manager.set_code(address, &code);
    assert!(result.is_ok());

    // Get code
    let retrieved_code = state_manager.get_code(address);
    assert_eq!(retrieved_code, Some(code.as_slice()));

    // Check account was created
    assert!(state_manager.account_exists(address));
    let account = state_manager.get_account(address).unwrap();
    assert!(account.code_hash.is_some());
}

#[test]
fn test_state_statistics() {
    let config = RuntimeConfig::default();
    let mut state_manager = StateManager::new(&config).unwrap();

    // Create some accounts
    state_manager
        .create_account("0x1111111111111111111111111111111111111111", 1000)
        .unwrap();
    state_manager
        .create_account("0x2222222222222222222222222222222222222222", 2000)
        .unwrap();

    // Set code for one account (make it a contract)
    state_manager
        .set_code("0x1111111111111111111111111111111111111111", &[0x60, 0x01])
        .unwrap();

    let stats = state_manager.get_statistics();
    assert_eq!(stats.total_accounts, 2);
    assert_eq!(stats.total_balance, 3000);
    assert_eq!(stats.contract_accounts, 1);
    assert_eq!(stats.external_accounts, 1);
    assert!(stats.total_changes > 0);
}
