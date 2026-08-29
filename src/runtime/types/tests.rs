use super::*;

#[test]
fn test_runtime_value_conversion() {
    let value = RuntimeValue::UnsignedInteger(42);
    let bytes = value.to_bytes();
    let restored = RuntimeValue::from_bytes(&bytes);

    assert_eq!(value, restored);
    assert!(value.is_truthy());
    assert_eq!(value.type_name(), "unsigned_integer");
}

#[test]
fn test_address_creation() {
    let addr1 = Address::new("0x1234567890123456789012345678901234567890".to_string()).unwrap();
    let addr2 = Address::new("1234567890123456789012345678901234567890".to_string()).unwrap();

    assert_eq!(addr1.as_str(), addr2.as_str());
    assert!(!addr1.is_zero());

    let zero_addr = Address::new("0x0000000000000000000000000000000000000000".to_string()).unwrap();
    assert!(zero_addr.is_zero());
}

#[test]
fn test_address_rejects_invalid_hex() {
    assert!(Address::new("0xzzzz567890123456789012345678901234567890".to_string()).is_err());
}

#[test]
fn test_address_rejects_odd_length_hex() {
    assert!(Address::new("0x123456789012345678901234567890123456789".to_string()).is_err());
}

#[test]
fn test_gas_operations() {
    let gas1 = Gas::new(1000);
    let gas2 = Gas::new(500);

    let sum = gas1.saturating_add(gas2);
    assert_eq!(sum.amount(), 1500);

    let diff = gas1.saturating_sub(gas2);
    assert_eq!(diff.amount(), 500);

    assert!(gas1.sufficient_for(gas2));
    assert!(!gas2.sufficient_for(gas1));
}

#[test]
fn test_balance_operations() {
    let balance1 = Balance::new(1000);
    let balance2 = Balance::new(300);

    let sum = balance1.saturating_add(balance2);
    assert_eq!(sum.amount(), 1300);

    let diff = balance1.checked_sub(balance2).unwrap();
    assert_eq!(diff.amount(), 700);

    assert!(balance2.checked_sub(balance1).is_err());

    assert!(balance1.sufficient_for(balance2));
    assert!(Balance::new(0).is_zero());
}

#[test]
fn test_block_number_operations() {
    let block = BlockNumber::new(100);
    let next = block.next();
    let prev = next.prev().unwrap();

    assert_eq!(next.number(), 101);
    assert_eq!(prev.number(), 100);

    let genesis = BlockNumber::new(0);
    assert!(genesis.prev().is_none());
}

#[test]
fn test_timestamp_operations() {
    let now = Timestamp::now();
    let future = now.add_seconds(3600);
    let past = now.sub_seconds(1800);

    assert!(future.timestamp() > now.timestamp());
    assert!(past.timestamp() < now.timestamp());
}

#[test]
fn test_runtime_value_truthy() {
    assert!(!RuntimeValue::Null.is_truthy());
    assert!(!RuntimeValue::Boolean(false).is_truthy());
    assert!(RuntimeValue::Boolean(true).is_truthy());
    assert!(!RuntimeValue::Integer(0).is_truthy());
    assert!(RuntimeValue::Integer(42).is_truthy());
    assert!(!RuntimeValue::UnsignedInteger(0).is_truthy());
    assert!(RuntimeValue::UnsignedInteger(42).is_truthy());
    assert!(!RuntimeValue::ByteString(vec![]).is_truthy());
    assert!(RuntimeValue::ByteString(vec![1, 2, 3]).is_truthy());
}

#[test]
fn test_stack_item_conversion() {
    let value = RuntimeValue::Integer(42);
    let stack_item = value.to_stack_item();
    let restored = RuntimeValue::from_stack_item(&stack_item);

    assert_eq!(value, restored);
}

#[test]
fn test_hash_types() {
    let tx_hash = TransactionHash::new("0xabcdef1234567890".to_string()).unwrap();
    let block_hash = BlockHash::new("0xfedcba0987654321".to_string()).unwrap();

    assert_eq!(tx_hash.as_str(), "0xabcdef1234567890");
    assert_eq!(block_hash.as_str(), "0xfedcba0987654321");

    let bytes = tx_hash.to_bytes();
    let restored = TransactionHash::from_bytes(&bytes);
    assert_eq!(tx_hash.as_str(), restored.as_str());
}

#[test]
fn test_hash_types_reject_invalid_hex() {
    assert!(TransactionHash::new("0xhello".to_string()).is_err());
    assert!(BlockHash::new("0xxy".to_string()).is_err());
}

#[test]
fn test_hash_types_reject_odd_length_hex() {
    assert!(TransactionHash::new("0xabc".to_string()).is_err());
    assert!(BlockHash::new("0Xabc".to_string()).is_err());
    assert!(BlockHash::new("abc".to_string()).is_err());
}
