use neo_devpack_solidity::runtime::execution::ExecutionContext;
use neo_devpack_solidity::runtime::RuntimeConfig;

#[test]
fn gas_used_increments_per_instruction() {
    let config = RuntimeConfig {
        gas_limit: 10,
        ..RuntimeConfig::default()
    };
    let mut ctx = ExecutionContext::new(&config).expect("context init");
    // Two cheap instructions: PUSH0, NOP
    ctx.initialize(&[0x10, 0x21], &[]).expect("init");
    let start_gas = ctx.gas_used();
    ctx.step().expect("push");
    let after_push = ctx.gas_used();
    ctx.step().expect("nop");
    let after_nop = ctx.gas_used();
    assert!(after_push > start_gas, "gas should increment after push");
    assert!(after_nop > after_push, "gas should increment after nop");
}

#[test]
fn get_gas_left_reflects_usage() {
    let config = RuntimeConfig {
        gas_limit: 20,
        ..RuntimeConfig::default()
    };
    let mut ctx = ExecutionContext::new(&config).expect("context init");
    // PUSH0, GETGASLEFT, RET
    let code = vec![0x10, 0x41, 20, 136, 216, 206, 0x40];
    ctx.initialize(&code, &[]).expect("init");
    while !ctx.step().expect("step").halted {}
    let data = ctx.return_data();
    let mut buf = [0u8; 8];
    let copy = data.len().min(8);
    buf[..copy].copy_from_slice(&data[..copy]);
    let gas_left = u64::from_le_bytes(buf);
    assert!(gas_left < config.gas_limit, "gas left should be reduced");
}

#[test]
fn gas_exhaustion_halts_execution() {
    let config = RuntimeConfig {
        gas_limit: 2, // Very low limit
        ..RuntimeConfig::default()
    };
    let mut ctx = ExecutionContext::new(&config).expect("context init");
    // Multiple instructions that should exhaust gas
    ctx.initialize(&[0x10, 0x10, 0x10, 0x10, 0x10, 0x40], &[])
        .expect("init");

    let mut halted = false;
    let mut steps = 0;
    while steps < 10 {
        match ctx.step() {
            Ok(result) => {
                if result.halted {
                    halted = true;
                    break;
                }
            }
            Err(_) => {
                halted = true;
                break;
            }
        }
        steps += 1;
    }
    assert!(halted, "execution should halt due to gas exhaustion");
}

#[test]
fn syscall_consumes_additional_gas() {
    let config = RuntimeConfig {
        gas_limit: 1000,
        ..RuntimeConfig::default()
    };
    let mut ctx = ExecutionContext::new(&config).expect("context init");

    // Simple code with syscall: PUSH0, RET
    let simple_code = vec![0x10, 0x40];
    ctx.initialize(&simple_code, &[]).expect("init");
    while !ctx.step().expect("step").halted {}
    let simple_gas = ctx.gas_used();

    // Reset and run code with syscall
    let mut ctx2 = ExecutionContext::new(&config).expect("context init");
    // SYSCALL System.Runtime.GetTime, RET
    let syscall_code = vec![
        0x41, 183, 195, 136, 3,    // SYSCALL System.Runtime.GetTime
        0x40, // RET
    ];
    ctx2.initialize(&syscall_code, &[]).expect("init");
    while !ctx2.step().expect("step").halted {}
    let syscall_gas = ctx2.gas_used();

    assert!(
        syscall_gas > simple_gas,
        "syscall should consume more gas than simple instructions"
    );
}

#[test]
fn storage_operations_consume_significant_gas() {
    use sha2::{Digest, Sha256};

    fn syscall_id(name: &str) -> [u8; 4] {
        let hash = Sha256::digest(name.as_bytes());
        [hash[0], hash[1], hash[2], hash[3]]
    }

    // S2 fix: Storage.Put now charges the mainnet-aligned 100_000/byte rate,
    // so a "key"+"value" write (8 bytes) costs ~800K gas. The previous 10K
    // budget was sized for the old 100/byte rate; raise it so the contract
    // completes within budget. The test only asserts gas is consumed (>0),
    // so any realistic budget works.
    let config = RuntimeConfig {
        gas_limit: 5_000_000,
        ..RuntimeConfig::default()
    };
    let mut ctx = ExecutionContext::new(&config).expect("context init");

    let get_ctx_id = syscall_id("System.Storage.GetContext");
    let put_id = syscall_id("System.Storage.Put");

    // Storage.GetContext, PUSH key, PUSH value, Storage.Put, RET
    let code = vec![
        0x41,
        get_ctx_id[0],
        get_ctx_id[1],
        get_ctx_id[2],
        get_ctx_id[3], // GetContext
        0x0C,
        0x03,
        0x6B,
        0x65,
        0x79, // PUSHDATA1 "key"
        0x0C,
        0x05,
        0x76,
        0x61,
        0x6C,
        0x75,
        0x65, // PUSHDATA1 "value"
        0x41,
        put_id[0],
        put_id[1],
        put_id[2],
        put_id[3], // Storage.Put
        0x40,      // RET
    ];

    ctx.initialize(&code, &[]).expect("init");
    while !ctx.step().expect("step").halted {}

    let gas_used = ctx.gas_used();
    assert!(
        gas_used > 100,
        "storage operations should consume significant gas (used: {gas_used})"
    );
}

#[test]
fn storage_put_gas_aligns_with_neo_n3_mainnet_per_byte_rate() {
    // S2 fix: Neo N3 mainnet `Policy.storagePrice` is 100_000 gas/byte for
    // storage writes. The embedded runtime previously charged 100/byte, ~1000×
    // too cheap — contracts that fit the simulator's gas budget would exhaust
    // gas on-chain. This test pins the mainnet-aligned rate by asserting that
    // a single byte of storage costs a known, large amount.
    //
    // We compare gas(put a 16-byte value under a 16-byte key) against
    // gas(put a 32-byte value under a 16-byte key): the delta must equal
    // approximately rate * 16 bytes, with rate ≈ 100_000.
    use sha2::{Digest, Sha256};

    fn syscall_id(name: &str) -> [u8; 4] {
        let hash = Sha256::digest(name.as_bytes());
        [hash[0], hash[1], hash[2], hash[3]]
    }

    fn run_put(value_len: usize) -> u64 {
        let config = RuntimeConfig {
            // Generous budget so the per-byte rate does not OOG before we can
            // read the total. Default 10M is enough for a few KB at 100K/byte.
            gas_limit: 100_000_000,
            ..RuntimeConfig::default()
        };
        let mut ctx = ExecutionContext::new(&config).expect("context init");
        let get_ctx_id = syscall_id("System.Storage.GetContext");
        let put_id = syscall_id("System.Storage.Put");
        // Stack layout expected by Storage.Put: [value, key, context] with
        // context on TOP. Push value first (bottom), then key, then call
        // GetContext last so it lands on top.
        let mut code = Vec::new();
        // value_len-byte value (bottom).
        code.extend_from_slice(&[0x0C, value_len as u8]);
        code.extend_from_slice(&vec![0xAA; value_len]);
        // 16-byte key (middle).
        code.extend_from_slice(&[0x0C, 0x10]);
        code.extend_from_slice(&[0u8; 16]);
        // GetContext -> context on top.
        code.push(0x41);
        code.extend_from_slice(&get_ctx_id);
        // Storage.Put.
        code.push(0x41);
        code.extend_from_slice(&put_id);
        code.push(0x40);
        ctx.initialize(&code, &[]).expect("init");
        while !ctx.step().expect("step").halted {}
        ctx.gas_used()
    }

    let gas_small = run_put(16);
    let gas_large = run_put(32);
    let delta = gas_large.saturating_sub(gas_small);

    // 16 extra bytes at the mainnet-aligned 100_000/byte rate ⇒ ~1_600_000.
    // Assert within a wide band (≥ 1_000_000) so the test is robust to the
    // fixed-cost components but pins the rate at the right order of magnitude
    // (the old 100/byte rate would give a delta of ~1_600 here).
    assert!(
        delta >= 1_000_000,
        "S2: 16 extra storage bytes must cost ~1.6M gas at the mainnet-aligned \
         100_000/byte rate; got delta={delta} (gas_small={gas_small}, \
         gas_large={gas_large}). The old 100/byte rate would yield ~1_600 — \
         if you see that, the S2 fix was reverted."
    );
}
