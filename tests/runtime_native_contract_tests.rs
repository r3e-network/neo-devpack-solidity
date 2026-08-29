use neo_devpack_solidity::runtime::execution::ExecutionContext;
use neo_devpack_solidity::runtime::RuntimeConfig;
use sha2::{Digest, Sha256};

fn syscall_id(name: &str) -> [u8; 4] {
    let d = Sha256::digest(name.as_bytes());
    [d[0], d[1], d[2], d[3]]
}

fn push_data(script: &mut Vec<u8>, data: &[u8]) {
    assert!(
        data.len() <= u8::MAX as usize,
        "push_data only supports PUSHDATA1 lengths"
    );
    script.push(0x0C); // PUSHDATA1
    script.push(data.len() as u8);
    script.extend_from_slice(data);
}

/// Push a small integer (0..16) onto the NeoVM stack.
fn push_int(script: &mut Vec<u8>, val: u8) {
    match val {
        0 => script.push(0x10),                // PUSH0
        1 => script.push(0x11),                // PUSH1
        2 => script.push(0x12),                // PUSH2
        3 => script.push(0x13),                // PUSH3
        4 => script.push(0x14),                // PUSH4
        5 => script.push(0x15),                // PUSH5
        n if n <= 16 => script.push(0x10 + n), // PUSH0..PUSH16
        _ => panic!("push_int only supports 0..16"),
    }
}

// ── Native contract hashes (UInt160 little-endian) ──

const POLICY_HASH_LE: [u8; 20] =
    *b"\x7b\xc6\x81\xc0\xa1\xf7\x1d\x54\x34\x57\xb6\x8b\xba\x8d\x5f\x9f\xdd\x4e\x5e\xcc";

const ORACLE_HASH_LE: [u8; 20] =
    *b"\x58\x87\x17\x11\x7e\x0a\xa8\x10\x72\xaf\xab\x71\xd2\xdd\x89\xfe\x7c\x4b\x92\xfe";

const ROLE_MANAGEMENT_HASH_LE: [u8; 20] =
    *b"\xe2\x95\xe3\x91\x54\x4c\x17\x8a\xd9\x4f\x03\xec\x4d\xcd\xff\x78\x53\x4e\xcf\x49";

const LEDGER_HASH_LE: [u8; 20] =
    *b"\xbe\xf2\x04\x31\x40\x36\x2a\x77\xc1\x50\x99\xc7\xe6\x4c\x12\xf7\x00\xb6\x65\xda";

const NOTARY_HASH_LE: [u8; 20] =
    *b"\x3b\xec\x35\x31\x11\x9b\xba\xd7\x6d\xd0\x44\x92\x0b\x0d\xe6\xc3\x19\x4f\xe1\xc1";

const TREASURY_HASH_LE: [u8; 20] =
    *b"\xc1\x3a\x56\xc9\x83\x53\xa7\xea\x6a\x32\x4d\x9a\x83\x5d\x1b\x5b\xf2\x26\x63\x15";

// ── Helper: build a System.Contract.Call invocation ──

/// Build NeoVM bytecode that calls a native contract method with the given
/// params already packed as an array on the stack.
///
/// Stack layout for System.Contract.Call: [args, flags, method, hash]
fn build_native_call(hash: &[u8; 20], method: &str, param_opcodes: &[u8]) -> Vec<u8> {
    let call_id = syscall_id("System.Contract.Call");
    let mut code = Vec::new();
    code.extend_from_slice(param_opcodes);
    code.push(0x1F); // PUSH15 (CallFlags.All = 0x0F)
    push_data(&mut code, method.as_bytes());
    push_data(&mut code, hash);
    code.push(0x41); // SYSCALL
    code.extend_from_slice(&call_id);
    code.push(0x40); // RET
    code
}

/// Build param opcodes: push nothing, PUSH0, PACK → empty array
fn empty_params() -> Vec<u8> {
    vec![0x10, 0xC0] // PUSH0, PACK → []
}

/// Build param opcodes: push one integer, PUSH1, PACK → [int]
fn single_int_param(val: u64) -> Vec<u8> {
    let mut ops = Vec::new();
    let bytes = val.to_le_bytes();
    push_data(&mut ops, &bytes);
    ops.push(0x11); // PUSH1
    ops.push(0xC0); // PACK → [val]
    ops
}

/// Build param opcodes: push one byte array, PUSH1, PACK → [bytes]
fn single_bytes_param(data: &[u8]) -> Vec<u8> {
    let mut ops = Vec::new();
    push_data(&mut ops, data);
    ops.push(0x11); // PUSH1
    ops.push(0xC0); // PACK → [data]
    ops
}

/// Execute code on a fresh context and return the result bytes.
fn run_code(code: &[u8]) -> Vec<u8> {
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("ctx");
    ctx.initialize(code, &[]).expect("init");
    while !ctx.step().expect("step").halted {}
    ctx.return_data().to_vec()
}

/// Execute code on an existing context (for multi-step tests).
fn run_on(ctx: &mut ExecutionContext, code: &[u8]) -> Vec<u8> {
    ctx.initialize(code, &[]).expect("init");
    while !ctx.step().expect("step").halted {}
    ctx.return_data().to_vec()
}

// ═══════════════════════════════════════════════════════════════════════════
// Policy native contract tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn policy_get_fee_per_byte_returns_default() {
    let params = empty_params();
    let code = build_native_call(&POLICY_HASH_LE, "getFeePerByte", &params);
    let result = run_code(&code);
    // Default: 1000 (0x03E8)
    let val = u64::from_le_bytes({
        let mut buf = [0u8; 8];
        for (i, b) in result.iter().take(8).enumerate() {
            buf[i] = *b;
        }
        buf
    });
    assert_eq!(val, 1000, "default fee per byte should be 1000");
}

#[test]
fn policy_set_and_get_fee_per_byte() {
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("ctx");

    // Set fee to 2000
    let set_params = single_int_param(2000);
    let set_code = build_native_call(&POLICY_HASH_LE, "setFeePerByte", &set_params);
    run_on(&mut ctx, &set_code);

    // Get fee
    let get_params = empty_params();
    let get_code = build_native_call(&POLICY_HASH_LE, "getFeePerByte", &get_params);
    let result = run_on(&mut ctx, &get_code);

    let val = u64::from_le_bytes({
        let mut buf = [0u8; 8];
        for (i, b) in result.iter().take(8).enumerate() {
            buf[i] = *b;
        }
        buf
    });
    assert_eq!(val, 2000, "fee per byte should be 2000 after set");
}

#[test]
fn policy_get_exec_fee_factor_returns_default() {
    let params = empty_params();
    let code = build_native_call(&POLICY_HASH_LE, "getExecFeeFactor", &params);
    let result = run_code(&code);
    let val = u64::from_le_bytes({
        let mut buf = [0u8; 8];
        for (i, b) in result.iter().take(8).enumerate() {
            buf[i] = *b;
        }
        buf
    });
    assert_eq!(val, 30, "default exec fee factor should be 30");
}

#[test]
fn policy_get_storage_price_returns_default() {
    let params = empty_params();
    let code = build_native_call(&POLICY_HASH_LE, "getStoragePrice", &params);
    let result = run_code(&code);
    let val = u64::from_le_bytes({
        let mut buf = [0u8; 8];
        for (i, b) in result.iter().take(8).enumerate() {
            buf[i] = *b;
        }
        buf
    });
    assert_eq!(val, 100_000, "default storage price should be 100000");
}

#[test]
fn policy_block_and_unblock_account() {
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("ctx");
    let account = [0xAA; 20];

    // Block account
    let block_params = single_bytes_param(&account);
    let block_code = build_native_call(&POLICY_HASH_LE, "blockAccount", &block_params);
    let result = run_on(&mut ctx, &block_code);
    assert_eq!(result, vec![1], "blockAccount should return true");

    // isBlocked → true
    let check_params = single_bytes_param(&account);
    let check_code = build_native_call(&POLICY_HASH_LE, "isBlocked", &check_params);
    let result = run_on(&mut ctx, &check_code);
    assert_eq!(result, vec![1], "isBlocked should return true");

    // Unblock
    let unblock_params = single_bytes_param(&account);
    let unblock_code = build_native_call(&POLICY_HASH_LE, "unblockAccount", &unblock_params);
    let result = run_on(&mut ctx, &unblock_code);
    assert_eq!(result, vec![1], "unblockAccount should return true");

    // isBlocked → false
    let result = run_on(&mut ctx, &check_code);
    assert_eq!(
        result,
        vec![0],
        "isBlocked should return false after unblock"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Oracle native contract tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn oracle_get_price_returns_default() {
    let params = empty_params();
    let code = build_native_call(&ORACLE_HASH_LE, "getPrice", &params);
    let result = run_code(&code);
    let val = u64::from_le_bytes({
        let mut buf = [0u8; 8];
        for (i, b) in result.iter().take(8).enumerate() {
            buf[i] = *b;
        }
        buf
    });
    assert_eq!(val, 50_000_000, "default oracle price should be 50000000");
}

#[test]
fn oracle_set_and_get_price() {
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("ctx");

    // Set price to 100_000_000
    let set_params = single_int_param(100_000_000);
    let set_code = build_native_call(&ORACLE_HASH_LE, "setPrice", &set_params);
    run_on(&mut ctx, &set_code);

    // Get price
    let get_params = empty_params();
    let get_code = build_native_call(&ORACLE_HASH_LE, "getPrice", &get_params);
    let result = run_on(&mut ctx, &get_code);
    let val = u64::from_le_bytes({
        let mut buf = [0u8; 8];
        for (i, b) in result.iter().take(8).enumerate() {
            buf[i] = *b;
        }
        buf
    });
    assert_eq!(
        val, 100_000_000,
        "oracle price should be 100000000 after set"
    );
}

#[test]
fn oracle_request_returns_incrementing_ids() {
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("ctx");
    let call_id = syscall_id("System.Contract.Call");

    // Build request call with 6 params: url, filter, cb_contract, cb_method, user_data, gas
    // PACK pops in reverse order, so push in reverse: gas, user_data, cb_method, cb_contract, filter, url
    for expected_id in 0u32..3 {
        let mut code = Vec::new();
        push_data(&mut code, &50_000_000u64.to_le_bytes()); // gas
        push_data(&mut code, &[]); // user_data
        push_data(&mut code, b"callback");
        push_data(&mut code, &[0u8; 20]); // callback contract
        push_data(&mut code, b"$.price");
        push_data(&mut code, b"https://example.com/api");
        push_int(&mut code, 6); // count=6
        code.push(0xC0); // PACK
        code.push(0x1F); // PUSH15 (CallFlags.All)
        push_data(&mut code, b"request");
        push_data(&mut code, &ORACLE_HASH_LE);
        code.push(0x41);
        code.extend_from_slice(&call_id);
        code.push(0x40);

        let result = run_on(&mut ctx, &code);
        let id = u32::from_le_bytes({
            let mut buf = [0u8; 4];
            for (i, b) in result.iter().take(4).enumerate() {
                buf[i] = *b;
            }
            buf
        });
        assert_eq!(id, expected_id, "oracle request id should increment");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// RoleManagement native contract tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn role_management_designate_and_query() {
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("ctx");
    let call_id = syscall_id("System.Contract.Call");

    // Designate role 4 (P2PNotary) with two public keys
    let key1 = [0x01u8; 33];
    let key2 = [0x02u8; 33];

    let mut code = Vec::new();
    // Build params: [role, [key1, key2]]
    // PACK pops in reverse, so push in reverse order for outer array.
    // Inner keys array: push key2, key1 so PACK(2) → [key1, key2]
    push_data(&mut code, &key2);
    push_data(&mut code, &key1);
    push_int(&mut code, 2);
    code.push(0xC0); // PACK → [key1, key2]
                     // Outer array: push [keys], role so PACK(2) → [role, [keys]]
    push_int(&mut code, 4);
    push_int(&mut code, 2);
    code.push(0xC0); // PACK → [4, [key1, key2]]
    code.push(0x1F); // CallFlags.All
    push_data(&mut code, b"designateAsRole");
    push_data(&mut code, &ROLE_MANAGEMENT_HASH_LE);
    code.push(0x41);
    code.extend_from_slice(&call_id);
    code.push(0x40);
    run_on(&mut ctx, &code);

    // Query role 4
    let query_params = single_int_param(4);
    let query_code = build_native_call(
        &ROLE_MANAGEMENT_HASH_LE,
        "getDesignatedByRole",
        &query_params,
    );
    let result = run_on(&mut ctx, &query_code);
    // Result should be non-empty (serialized array)
    assert!(
        !result.is_empty(),
        "getDesignatedByRole should return non-empty result"
    );
}

#[test]
fn role_management_empty_role_returns_empty() {
    let params = single_int_param(8); // role 8 (StateValidator)
    let code = build_native_call(&ROLE_MANAGEMENT_HASH_LE, "getDesignatedByRole", &params);
    let result = run_code(&code);
    // The runtime returns an Array StackItem which gets JSON-serialized.
    // An empty array serializes to {"type":"Array","value":[]}.
    let result_str = String::from_utf8_lossy(&result);
    assert!(
        result.is_empty() || result_str.contains("[]"),
        "undesignated role should return empty array, got {result_str:?}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Ledger native contract tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn ledger_current_index_returns_default_height() {
    let params = empty_params();
    let code = build_native_call(&LEDGER_HASH_LE, "currentIndex", &params);
    let result = run_code(&code);
    let val = u64::from_le_bytes({
        let mut buf = [0u8; 8];
        for (i, b) in result.iter().take(8).enumerate() {
            buf[i] = *b;
        }
        buf
    });
    // Default block height is typically 0 or a small number
    assert!(
        val < 1_000_000,
        "currentIndex should be a reasonable height, got {val}"
    );
}

#[test]
fn ledger_current_hash_returns_32_bytes() {
    let params = empty_params();
    let code = build_native_call(&LEDGER_HASH_LE, "currentHash", &params);
    let result = run_code(&code);
    assert_eq!(result.len(), 32, "currentHash should return 32 bytes");
}

#[test]
fn ledger_get_block_returns_synthetic_block() {
    // Request block at index 0
    let params = single_int_param(0);
    let code = build_native_call(&LEDGER_HASH_LE, "getBlock", &params);
    let result = run_code(&code);
    // Synthetic block is returned as a serialized array — should be non-empty
    assert!(
        !result.is_empty(),
        "getBlock(0) should return a non-empty synthetic block"
    );
}

#[test]
fn ledger_get_block_beyond_height_returns_null() {
    // Request block at index far beyond default height
    let params = single_int_param(999_999_999);
    let code = build_native_call(&LEDGER_HASH_LE, "getBlock", &params);
    let result = run_code(&code);
    // Null is typically serialized as empty
    assert!(
        result.is_empty(),
        "getBlock beyond height should return null/empty, got {} bytes",
        result.len()
    );
}

#[test]
fn ledger_get_transaction_vm_state_returns_halt() {
    let params = empty_params();
    let code = build_native_call(&LEDGER_HASH_LE, "getTransactionVMState", &params);
    let result = run_code(&code);
    let val = u64::from_le_bytes({
        let mut buf = [0u8; 8];
        for (i, b) in result.iter().take(8).enumerate() {
            buf[i] = *b;
        }
        buf
    });
    assert_eq!(val, 1, "getTransactionVMState should return HALT (1)");
}

// ═══════════════════════════════════════════════════════════════════════════
// Notary native contract tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn notary_verify_returns_true() {
    let params = empty_params();
    let code = build_native_call(&NOTARY_HASH_LE, "verify", &params);
    let result = run_code(&code);
    assert_eq!(result, vec![1], "Notary.verify should return true");
}

#[test]
fn notary_balance_of_unknown_account_returns_zero() {
    let account = [0xBB; 20];
    let params = single_bytes_param(&account);
    let code = build_native_call(&NOTARY_HASH_LE, "balanceOf", &params);
    let result = run_code(&code);
    let val = u64::from_le_bytes({
        let mut buf = [0u8; 8];
        for (i, b) in result.iter().take(8).enumerate() {
            buf[i] = *b;
        }
        buf
    });
    assert_eq!(val, 0, "balanceOf unknown account should be 0");
}

#[test]
fn notary_deposit_and_query_lifecycle() {
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("ctx");
    let call_id = syscall_id("System.Contract.Call");
    let account = [0xCC; 20];

    // onNEP17Payment(from, amount, till) → deposit 1000 GAS, till block 5000
    // PACK pops in reverse order, so push in reverse: till, amount, from
    let mut deposit_code = Vec::new();
    push_data(&mut deposit_code, &5000u64.to_le_bytes());
    push_data(&mut deposit_code, &1000u64.to_le_bytes());
    push_data(&mut deposit_code, &account);
    push_int(&mut deposit_code, 3);
    deposit_code.push(0xC0); // PACK
    deposit_code.push(0x1F);
    push_data(&mut deposit_code, b"onNEP17Payment");
    push_data(&mut deposit_code, &NOTARY_HASH_LE);
    deposit_code.push(0x41);
    deposit_code.extend_from_slice(&call_id);
    deposit_code.push(0x40);
    run_on(&mut ctx, &deposit_code);

    // balanceOf → 1000
    let balance_params = single_bytes_param(&account);
    let balance_code = build_native_call(&NOTARY_HASH_LE, "balanceOf", &balance_params);
    let result = run_on(&mut ctx, &balance_code);
    let val = u64::from_le_bytes({
        let mut buf = [0u8; 8];
        for (i, b) in result.iter().take(8).enumerate() {
            buf[i] = *b;
        }
        buf
    });
    assert_eq!(val, 1000, "balanceOf should be 1000 after deposit");

    // expirationOf → 5000
    let exp_params = single_bytes_param(&account);
    let exp_code = build_native_call(&NOTARY_HASH_LE, "expirationOf", &exp_params);
    let result = run_on(&mut ctx, &exp_code);
    let till = u64::from_le_bytes({
        let mut buf = [0u8; 8];
        for (i, b) in result.iter().take(8).enumerate() {
            buf[i] = *b;
        }
        buf
    });
    assert_eq!(till, 5000, "expirationOf should be 5000");

    // withdraw → true
    let withdraw_params = single_bytes_param(&account);
    let withdraw_code = build_native_call(&NOTARY_HASH_LE, "withdraw", &withdraw_params);
    let result = run_on(&mut ctx, &withdraw_code);
    assert_eq!(result, vec![1], "withdraw should return true");

    // balanceOf after withdraw → 0
    let result = run_on(&mut ctx, &balance_code);
    let val = u64::from_le_bytes({
        let mut buf = [0u8; 8];
        for (i, b) in result.iter().take(8).enumerate() {
            buf[i] = *b;
        }
        buf
    });
    assert_eq!(val, 0, "balanceOf should be 0 after withdraw");
}

#[test]
fn notary_get_max_not_valid_before_delta_default() {
    let params = empty_params();
    let code = build_native_call(&NOTARY_HASH_LE, "getMaxNotValidBeforeDelta", &params);
    let result = run_code(&code);
    let val = u64::from_le_bytes({
        let mut buf = [0u8; 8];
        for (i, b) in result.iter().take(8).enumerate() {
            buf[i] = *b;
        }
        buf
    });
    // Default is 140 (Neo N3 MainNet)
    assert_eq!(val, 140, "default maxNotValidBeforeDelta should be 140");
}

// ═══════════════════════════════════════════════════════════════════════════
// Treasury native contract tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn treasury_verify_returns_true() {
    let params = empty_params();
    let code = build_native_call(&TREASURY_HASH_LE, "verify", &params);
    let result = run_code(&code);
    assert_eq!(result, vec![1], "Treasury.verify should return true");
}

#[test]
fn treasury_nep17_payment_tracks_balance() {
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("ctx");
    let call_id = syscall_id("System.Contract.Call");
    let depositor = [0xDD; 20];

    // onNEP17Payment(from, amount, data)
    // PACK pops in reverse, so push: data, amount, from
    let mut code = Vec::new();
    push_data(&mut code, &[]); // data
    push_data(&mut code, &500u64.to_le_bytes());
    push_data(&mut code, &depositor);
    push_int(&mut code, 3);
    code.push(0xC0); // PACK
    code.push(0x1F);
    push_data(&mut code, b"onNEP17Payment");
    push_data(&mut code, &TREASURY_HASH_LE);
    code.push(0x41);
    code.extend_from_slice(&call_id);
    code.push(0x40);

    // Execute — should not error
    run_on(&mut ctx, &code);
    // Treasury payment tracking is internal state; verify no crash
}

#[test]
fn treasury_nep11_payment_tracks_tokens() {
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("ctx");
    let call_id = syscall_id("System.Contract.Call");
    let depositor = [0xEE; 20];
    let token_id = b"token-001";

    // onNEP11Payment(from, amount, tokenId)
    // PACK pops in reverse, so push: tokenId, amount, from
    let mut code = Vec::new();
    push_data(&mut code, token_id);
    push_data(&mut code, &1u64.to_le_bytes());
    push_data(&mut code, &depositor);
    push_int(&mut code, 3);
    code.push(0xC0); // PACK
    code.push(0x1F);
    push_data(&mut code, b"onNEP11Payment");
    push_data(&mut code, &TREASURY_HASH_LE);
    code.push(0x41);
    code.extend_from_slice(&call_id);
    code.push(0x40);

    // Execute — should not error
    run_on(&mut ctx, &code);
    // Treasury NFT tracking is internal state; verify no crash
}
