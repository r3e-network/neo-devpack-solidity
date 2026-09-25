//! Storage / balance property tests.
//!
//! Extracted from the top-level "Storage Fuzz Tests" banner in
//! `tests/fuzz_tests.rs`. Contents unchanged from the pre-split file.

#![allow(unused_imports)]

use super::common::*;
use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};
use proptest::prelude::*;

// ==================== Storage Fuzz Tests ====================

proptest! {
    // Test storage roundtrip: write then read returns original value
    #[test]
    fn storage_roundtrip_preserves_data(
        key in prop::collection::vec(any::<u8>(), 1..64),
        value in prop::collection::vec(any::<u8>(), 1..256)
    ) {
        let mut runtime = NeoRuntime::new(RuntimeConfig::default()).expect("Failed to create runtime");
        let account = "0x1234567890123456789012345678901234567890";

        runtime.set_storage(account, &key, &value).expect("Failed to set storage");
        let retrieved = runtime.get_storage(account, &key).expect("Failed to get storage");

        prop_assert_eq!(retrieved, Some(value));
    }

    // Test storage overwrite: later write takes precedence
    #[test]
    fn storage_overwrite_updates_value(
        key in prop::collection::vec(any::<u8>(), 1..32),
        value1 in prop::collection::vec(any::<u8>(), 1..128),
        value2 in prop::collection::vec(any::<u8>(), 1..128)
    ) {
        let mut runtime = NeoRuntime::new(RuntimeConfig::default()).expect("Failed to create runtime");
        let account = "0x1234567890123456789012345678901234567890";

        runtime.set_storage(account, &key, &value1).expect("Failed to set storage");
        runtime.set_storage(account, &key, &value2).expect("Failed to overwrite storage");

        let retrieved = runtime.get_storage(account, &key).expect("Failed to get storage");
        prop_assert_eq!(retrieved, Some(value2));
    }

    // Test storage isolation: different accounts don't see each other's data
    #[test]
    fn storage_isolation_between_accounts(
        key in prop::collection::vec(any::<u8>(), 1..32),
        value1 in prop::collection::vec(any::<u8>(), 1..64),
        value2 in prop::collection::vec(any::<u8>(), 1..64)
    ) {
        let mut runtime = NeoRuntime::new(RuntimeConfig::default()).expect("Failed to create runtime");
        let account1 = "0x1111111111111111111111111111111111111111";
        let account2 = "0x2222222222222222222222222222222222222222";

        runtime.set_storage(account1, &key, &value1).expect("Failed to set storage 1");
        runtime.set_storage(account2, &key, &value2).expect("Failed to set storage 2");

        let retrieved1 = runtime.get_storage(account1, &key).expect("Failed to get storage 1");
        let retrieved2 = runtime.get_storage(account2, &key).expect("Failed to get storage 2");

        prop_assert_eq!(retrieved1, Some(value1));
        prop_assert_eq!(retrieved2, Some(value2));
    }

    // Test storage with empty value - note: empty values may be treated as deleted
    #[test]
    fn storage_empty_value_handling(
        key in prop::collection::vec(any::<u8>(), 1..32)
    ) {
        let mut runtime = NeoRuntime::new(RuntimeConfig::default()).expect("Failed to create runtime");
        let account = "0x1234567890123456789012345678901234567890";
        let empty_value: Vec<u8> = vec![];

        runtime.set_storage(account, &key, &empty_value).expect("Failed to set empty value");
        let retrieved = runtime.get_storage(account, &key).expect("Failed to get storage");

        // Empty values may be stored as None (deleted) or Some([]) depending on implementation
        prop_assert!(retrieved == Some(empty_value) || retrieved.is_none());
    }

    // Test storage with large values
    #[test]
    fn storage_large_value_roundtrip(
        key in prop::collection::vec(any::<u8>(), 1..32),
        value in prop::collection::vec(any::<u8>(), 1000..5000)
    ) {
        let mut runtime = NeoRuntime::new(RuntimeConfig::default()).expect("Failed to create runtime");
        let account = "0x1234567890123456789012345678901234567890";

        runtime.set_storage(account, &key, &value).expect("Failed to set large value");
        let retrieved = runtime.get_storage(account, &key).expect("Failed to get storage");

        prop_assert_eq!(retrieved, Some(value));
    }

    // Test balance operations: set and get roundtrip
    #[test]
    fn balance_roundtrip(
        balance in 0u64..10_000_000_000u64
    ) {
        let mut runtime = NeoRuntime::new(RuntimeConfig::default()).expect("Failed to create runtime");
        let account = "0x1234567890123456789012345678901234567890";

        runtime.set_balance(account, balance).expect("Failed to set balance");
        let retrieved = runtime.get_balance(account).expect("Failed to get balance");

        prop_assert_eq!(retrieved, balance);
    }

    // Test balance isolation: different accounts have independent balances
    #[test]
    fn balance_isolation(
        balance1 in 0u64..1_000_000_000u64,
        balance2 in 0u64..1_000_000_000u64
    ) {
        let mut runtime = NeoRuntime::new(RuntimeConfig::default()).expect("Failed to create runtime");
        let account1 = "0x1111111111111111111111111111111111111111";
        let account2 = "0x2222222222222222222222222222222222222222";

        runtime.set_balance(account1, balance1).expect("Failed to set balance 1");
        runtime.set_balance(account2, balance2).expect("Failed to set balance 2");

        let retrieved1 = runtime.get_balance(account1).expect("Failed to get balance 1");
        let retrieved2 = runtime.get_balance(account2).expect("Failed to get balance 2");

        prop_assert_eq!(retrieved1, balance1);
        prop_assert_eq!(retrieved2, balance2);
    }

    // Test storage key ordering - using unique keys only
    #[test]
    fn storage_keys_maintain_order(
        unique_keys in prop::collection::hash_set(prop::collection::vec(any::<u8>(), 1..16), 1..20)
    ) {
        let mut runtime = NeoRuntime::new(RuntimeConfig::default()).expect("Failed to create runtime");
        let account = "0x1234567890123456789012345678901234567890";

        let keys: Vec<_> = unique_keys.into_iter().collect();

        for (i, key) in keys.iter().enumerate() {
            let value = (i as u64).to_le_bytes().to_vec();
            runtime.set_storage(account, key, &value).expect("Failed to set storage");
        }

        for (i, key) in keys.iter().enumerate() {
            let retrieved = runtime.get_storage(account, key).expect("Failed to get storage");
            let expected = (i as u64).to_le_bytes().to_vec();
            prop_assert_eq!(retrieved, Some(expected), "Key {:?} should have value {}", key, i);
        }
    }
}
