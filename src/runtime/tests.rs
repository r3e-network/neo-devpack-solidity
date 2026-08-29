use super::*;
use crate::opcode::OpCode;
use crate::runtime::types::StackItem;
use hex;
use proptest::prelude::*;
use sha2::{Digest as Sha2Digest, Sha256};
use sha3::Keccak256;

fn metadata_script() -> Vec<u8> {
    let syscalls = [
        "System.Runtime.GetTime",
        "System.Runtime.GetCallingScriptHash",
    ];
    let mut script = Vec::with_capacity(syscalls.len() * 5 + 1);
    for name in syscalls {
        script.push(OpCode::SYSCALL.byte());
        let mut hasher = Sha256::new();
        hasher.update(name.as_bytes());
        let digest = hasher.finalize();
        script.extend_from_slice(&[digest[0], digest[1], digest[2], digest[3]]);
    }
    script.push(OpCode::RET.byte());
    script
}

#[test]
fn test_runtime_creation() {
    let config = RuntimeConfig::default();
    let runtime = NeoRuntime::new(config);
    assert!(runtime.is_ok());
}

#[test]
fn test_contract_deployment() {
    let config = RuntimeConfig::default();
    let mut runtime = NeoRuntime::new(config).unwrap();

    let bytecode = vec![0x60, 0x01, 0x60, 0x02, 0x01]; // Simple ADD bytecode
    let result = runtime.deploy_contract(&bytecode, &[]);

    assert!(result.is_ok());
    let address = result.unwrap();
    assert!(address.starts_with("0x"));
    assert_eq!(address.len(), 42); // 0x + 40 hex chars
}

#[test]
fn test_contract_deployment_addresses_are_deterministic_and_unique_per_nonce() {
    let config = RuntimeConfig::default();
    let mut runtime = NeoRuntime::new(config).unwrap();

    let bytecode = vec![0x60, 0x01, 0x60, 0x02, 0x01];

    let first = runtime
        .deploy_contract(&bytecode, &[])
        .expect("first deploy");
    let second = runtime
        .deploy_contract(&bytecode, &[])
        .expect("second deploy");

    assert_ne!(
        first, second,
        "nonce should make subsequent addresses unique"
    );

    let mut input = Vec::with_capacity(20 + std::mem::size_of::<u64>());
    input.extend_from_slice(runtime.execution_context.default_account_bytes());
    input.extend_from_slice(&0u64.to_le_bytes());
    let hash = Keccak256::digest(&input);
    let expected_first = format!("0x{}", hex::encode(&hash[12..32]));
    assert_eq!(first, expected_first);
}

#[test]
fn test_function_selector_calculation() {
    let config = RuntimeConfig::default();
    let runtime = NeoRuntime::new(config).unwrap();

    let selector = runtime
        .calculate_function_selector("transfer(address,uint256)")
        .unwrap();
    // Known selector for transfer function
    assert_eq!(selector, [0xa9, 0x05, 0x9c, 0xbb]);
}

fn expected_selector(signature: &str) -> [u8; 4] {
    let mut hasher = Keccak256::new();
    hasher.update(signature.as_bytes());
    let digest = hasher.finalize();
    let mut selector = [0u8; 4];
    selector.copy_from_slice(&digest[..4]);
    selector
}

fn function_signature_strategy() -> impl Strategy<Value = String> {
    let name = proptest::string::string_regex("[a-zA-Z_][a-zA-Z0-9_]{0,15}").expect("regex");
    let ty = prop_oneof![
        Just("uint256".to_string()),
        Just("int256".to_string()),
        Just("address".to_string()),
        Just("bool".to_string()),
        Just("bytes32".to_string()),
    ];
    (name, prop::collection::vec(ty, 0..=4)).prop_map(|(name, params)| {
        if params.is_empty() {
            format!("{name}()")
        } else {
            format!("{name}({})", params.join(","))
        }
    })
}

fn stack_item_strategy() -> impl Strategy<Value = StackItem> {
    prop_oneof![
        any::<i64>().prop_map(StackItem::Integer),
        any::<u64>().prop_map(StackItem::UnsignedInteger),
        any::<bool>().prop_map(StackItem::Boolean),
        prop::collection::vec(any::<u8>(), 0..16).prop_map(StackItem::byte_array),
        Just(StackItem::Null),
    ]
}

fn stack_item_arguments_strategy() -> impl Strategy<Value = Vec<StackItem>> {
    prop::collection::vec(stack_item_strategy(), 0..8)
}

proptest! {
    #[test]
    fn function_selector_matches_keccak_first_four_bytes(signature in function_signature_strategy()) {
        let runtime = NeoRuntime::new(RuntimeConfig::default()).expect("runtime");
        let selector = runtime
            .calculate_function_selector(&signature)
            .expect("selector");
        prop_assert_eq!(selector, expected_selector(&signature));
    }

    #[test]
    fn prepare_function_call_prefixes_selector_and_appends_encoded_arguments(
        signature in function_signature_strategy(),
        args in stack_item_arguments_strategy(),
    ) {
        let runtime = NeoRuntime::new(RuntimeConfig::default()).expect("runtime");
        let encoded = runtime
            .prepare_function_call(&signature, &args)
            .expect("prepared call data");

        let mut expected = Vec::new();
        expected.extend_from_slice(&expected_selector(&signature));
        for arg in &args {
            expected.extend_from_slice(&arg.to_bytes());
        }

        prop_assert_eq!(encoded, expected);
    }
}

#[test]
fn test_execution_result() {
    let result = ExecutionResult {
        success: true,
        return_data: vec![0x01, 0x02, 0x03],
        gas_used: 1000,
        gas_limit: 10000,
        exception: None,
        state_changes: vec![],
        logs: vec![],
        stack_trace: None,
        metadata: ExecutionMetadata::default(),
    };

    assert!(result.is_success());
    assert_eq!(result.gas_efficiency(), 0.1);
    assert!(!result.out_of_gas());
    assert_eq!(result.return_hex(), "010203");
}

#[test]
fn test_runtime_statistics() {
    let stats = RuntimeStatistics {
        total_gas_used: 5000,
        total_instructions_executed: 100,
        max_stack_depth: 10,
        storage_reads: 5,
        storage_writes: 3,
        state_changes: 2,
    };

    assert_eq!(stats.total_gas_used, 5000);
    assert_eq!(stats.total_instructions_executed, 100);
}

#[test]
fn test_state_operations() {
    let config = RuntimeConfig::default();
    let mut runtime = NeoRuntime::new(config).unwrap();

    let account = "0x1234567890123456789012345678901234567890";
    let key = b"test_key";
    let value = b"test_value";

    // Set storage
    let result = runtime.set_storage(account, key, value);
    assert!(result.is_ok());

    // Get storage
    let retrieved = runtime.get_storage(account, key).unwrap();
    assert_eq!(retrieved, Some(value.to_vec()));
}

#[test]
fn test_balance_operations() {
    let config = RuntimeConfig::default();
    let mut runtime = NeoRuntime::new(config).unwrap();

    let account = "0x1234567890123456789012345678901234567890";
    let balance = 1000u64;

    // Set balance
    let result = runtime.set_balance(account, balance);
    assert!(result.is_ok());

    // Get balance
    let retrieved = runtime.get_balance(account).unwrap();
    assert_eq!(retrieved, balance);
}

#[test]
fn test_runtime_metadata_overrides_apply_once() {
    let mut runtime = NeoRuntime::new(RuntimeConfig::default()).unwrap();
    let overrides = ExecutionOverrides {
        block_height: Some(42),
        timestamp: Some(1_337),
        caller_account: Some("0x0102030405060708090a0102030405060708090a".to_string()),
        value: None,
    };

    let script = metadata_script();
    let result = runtime
        .execute_with_overrides(&script, &[], &overrides)
        .expect("execution");
    assert!(result.success);

    assert_eq!(result.metadata.block_height, Some(42));
    assert_eq!(result.metadata.timestamp, Some(1_337));
    assert_eq!(
        result.metadata.caller_account.as_deref(),
        Some("0x0102030405060708090a0102030405060708090a")
    );

    assert!(runtime.execution_context.pending_block_height().is_none());
    assert!(runtime.execution_context.pending_timestamp().is_none());
    assert!(runtime.execution_context.pending_caller_account().is_none());

    assert_eq!(runtime.execution_context.block_height(), Some(42));
    assert_eq!(runtime.execution_context.timestamp(), Some(1_337));

    let mut expected = hex::decode("0102030405060708090a0102030405060708090a").expect("valid hex");
    expected.reverse();
    assert_eq!(
        runtime.execution_context.caller_account(),
        Some(expected.as_slice())
    );

    // Next execution without overrides should fall back to defaults.
    let second = runtime
        .execute_with_overrides(&metadata_script(), &[], &ExecutionOverrides::default())
        .expect("second execution");

    // Task #105 — `default_timestamp` is now pinned to 1_704_067_200
    // (2024-01-01T00:00:00Z) in `RuntimeConfig::default()`, so the
    // fallback-when-no-override path yields that value rather than 0.
    assert_eq!(runtime.execution_context.block_height(), Some(0));
    assert_eq!(runtime.execution_context.timestamp(), Some(1_704_067_200));
    // With no caller override, the entry frame's caller is the transaction's
    // entry script — a DISTINCT identity from the executing contract hash (see
    // `entry_script_hash`). This is what keeps `tx.origin` separate from
    // `address(this)` so a nested call's `msg.sender` (the caller's executing
    // hash) can diverge from `tx.origin`. The no-override `caller_account`
    // therefore falls back to `entry_script_hash`, not `default_account_bytes`.
    let expected_caller = runtime.execution_context.entry_script_hash();
    assert_eq!(
        runtime.execution_context.caller_account(),
        Some(expected_caller.as_slice())
    );

    assert_eq!(second.metadata.block_height, Some(0));
    assert_eq!(second.metadata.timestamp, Some(1_704_067_200));
    // `capture_metadata` formats the caller script hash through
    // `format_uint160_hex_be` (reversing the stored LE bytes to the Neo
    // display order), so the expected value must do the same. The no-override
    // caller is the distinct `entry_script_hash` (see above).
    let mut entry_be = runtime.execution_context.entry_script_hash();
    entry_be.reverse();
    let expected_default = format!("0x{}", hex::encode(entry_be));
    assert_eq!(
        second.metadata.caller_account.as_deref(),
        Some(expected_default.as_str())
    );
}

#[test]
fn test_override_caller_account_rejects_invalid_hex() {
    let mut runtime = NeoRuntime::new(RuntimeConfig::default()).unwrap();
    let err = runtime
        .override_caller_account("0x123")
        .expect_err("should reject odd-length hex");
    assert!(matches!(err, RuntimeError::ConfigurationError { .. }));
    assert!(runtime.execution_context.pending_caller_account().is_none());
}

// Task #50: `~uint256(x)` must produce `u256::MAX - x`, not `!(x as u64)`,
// when the operand arrives as a wide ByteArray (e.g. via PUSHINT256). Also
// OR/XOR must accept ByteArray operands instead of rejecting with
// "Invalid operand(s) for bitwise X". These pins cover the fixed cases.
#[test]
fn test_bitwise_not_uint256_zero_returns_all_ones() {
    // Bytecode: PUSHINT256 0 ; INVERT ; RET
    let mut bytecode = vec![OpCode::PUSHINT256.byte()];
    bytecode.extend_from_slice(&[0u8; 32]);
    bytecode.push(OpCode::INVERT.byte());
    bytecode.push(OpCode::RET.byte());

    let mut runtime = NeoRuntime::new(RuntimeConfig::default()).unwrap();
    let result = runtime.execute(&bytecode, &[]).expect("host ok");
    assert!(result.success, "exec failed: {:?}", result.exception);
    // `~0` (unsigned 2^256-1) is the all-ones 256-bit value. The conformant
    // NeoVM representation is the 32-byte TWO'S-COMPLEMENT integer `-1`
    // (`0xFF` * 32) — exactly what a real node computes for INVERT of 0. Read
    // as UNSIGNED, that is 2^256 - 1.
    assert_eq!(
        result.return_data,
        vec![0xFFu8; 32],
        "expected ~0 to be the 32-byte all-ones two's-complement value"
    );
    let unsigned = num_bigint::BigInt::from_bytes_le(num_bigint::Sign::Plus, &result.return_data);
    let expected = (num_bigint::BigInt::from(1) << 256u32) - num_bigint::BigInt::from(1);
    assert_eq!(unsigned, expected, "unsigned value of ~0 must be 2^256 - 1");
}

#[test]
fn test_bitwise_or_wide_bytearray_accepts_operands() {
    // PUSHINT256 2^63 ; PUSHINT256 1 ; OR ; RET — previously rejected.
    let mut bytecode = vec![OpCode::PUSHINT256.byte()];
    let mut lhs = [0u8; 32];
    lhs[7] = 0x80; // 2^63 in little-endian
    bytecode.extend_from_slice(&lhs);
    bytecode.push(OpCode::PUSHINT256.byte());
    let mut rhs = [0u8; 32];
    rhs[0] = 0x01;
    bytecode.extend_from_slice(&rhs);
    bytecode.push(OpCode::OR.byte());
    bytecode.push(OpCode::RET.byte());

    let mut runtime = NeoRuntime::new(RuntimeConfig::default()).unwrap();
    let result = runtime.execute(&bytecode, &[]).expect("host ok");
    assert!(result.success, "exec failed: {:?}", result.exception);
    // Expect 2^63 | 1 = 0x8000000000000001.
    let expected = num_bigint::BigUint::from(1u64 << 63) | num_bigint::BigUint::from(1u64);
    let got = num_bigint::BigUint::from_bytes_le(&result.return_data);
    assert_eq!(got, expected, "got {:?}", result.return_data);
}

#[test]
fn test_execute_with_overrides_rejects_invalid_metadata() {
    let mut runtime = NeoRuntime::new(RuntimeConfig::default()).unwrap();
    let overrides = ExecutionOverrides {
        block_height: Some(77),
        timestamp: Some(555),
        caller_account: Some("0x123".to_string()),
        value: None,
    };

    let err = runtime
        .execute_with_overrides(&[], &[], &overrides)
        .expect_err("invalid caller should error");
    assert!(matches!(err, RuntimeError::ConfigurationError { .. }));

    assert!(runtime.execution_context.pending_block_height().is_none());
    assert!(runtime.execution_context.pending_timestamp().is_none());
    assert!(runtime.execution_context.pending_caller_account().is_none());
}
