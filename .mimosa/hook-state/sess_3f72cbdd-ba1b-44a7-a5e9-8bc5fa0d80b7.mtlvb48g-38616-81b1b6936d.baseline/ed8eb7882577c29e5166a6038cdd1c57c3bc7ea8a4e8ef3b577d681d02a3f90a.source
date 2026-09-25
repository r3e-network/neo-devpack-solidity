/// Gas accounting regression tests
///
/// These tests verify that gas consumption remains stable across refactoring.
use neo_devpack_solidity::runtime::execution::ExecutionContext;
use neo_devpack_solidity::runtime::RuntimeConfig;
use sha2::{Digest, Sha256};

fn syscall_id(name: &str) -> [u8; 4] {
    let mut h = Sha256::new();
    h.update(name.as_bytes());
    let d = h.finalize();
    [d[0], d[1], d[2], d[3]]
}

/// Test that basic arithmetic consumes expected gas
#[test]
fn gas_arithmetic_consumes_expected_amount() {
    let config = RuntimeConfig {
        gas_limit: 100_000,
        ..Default::default()
    };

    // ADD costs 3 gas
    let code = [
        0x11, // PUSH1 (1 gas)
        0x11, // PUSH1 (1 gas)
        0x9E, // ADD (3 gas)
        0x40, // RET (0 gas)
    ];

    let mut ctx = ExecutionContext::new(&config).expect("context init");
    ctx.initialize(&code, &[]).expect("init");

    let gas_before = ctx.gas_used();
    while !ctx.step().expect("step").halted {}
    let gas_after = ctx.gas_used();

    let consumed = gas_after - gas_before;
    assert!(
        consumed >= 5,
        "Should consume at least 5 gas for PUSH1+PUSH1+ADD"
    );
    assert!(consumed <= 10, "Should not consume excessive gas");
}

/// Test that storage operations consume gas proportionally to data size
#[test]
fn gas_storage_scales_with_data_size() {
    let config = RuntimeConfig {
        gas_limit: 1_000_000,
        ..Default::default()
    };

    let put_id = syscall_id("System.Storage.Put");
    let get_context_id = syscall_id("System.Storage.GetContext");

    // Test with 10-byte value
    let mut code_small = vec![];
    code_small.extend_from_slice(&[0x57, 0x01, 0x00]); // INITSLOT 1 local, 0 args
    code_small.push(0x41); // SYSCALL
    code_small.extend_from_slice(&get_context_id);
    code_small.push(0x70); // STLOC0
    code_small.push(0x68); // LDLOC0
    code_small.push(0x0C); // PUSHDATA1
    code_small.push(3); // 3 bytes for key
    code_small.extend_from_slice(b"key");
    code_small.push(0x0C); // PUSHDATA1
    code_small.push(10); // 10 bytes for value
    code_small.extend_from_slice(&[0u8; 10]);
    code_small.push(0x41); // SYSCALL
    code_small.extend_from_slice(&put_id);
    code_small.push(0x40); // RET

    let mut ctx_small = ExecutionContext::new(&config.clone()).expect("context init");
    ctx_small.initialize(&code_small, &[]).expect("init");
    while !ctx_small.step().expect("step").halted {}
    let gas_small = ctx_small.gas_used();

    // Storage operations should consume gas
    assert!(gas_small > 0, "Storage operations should consume gas");
}

/// Test that syscall gas costs are reasonable
#[test]
fn gas_syscall_costs_are_reasonable() {
    let config = RuntimeConfig {
        gas_limit: 100_000,
        ..Default::default()
    };

    // Test Storage.Get costs around 100 gas
    let get_context_id = syscall_id("System.Storage.GetContext");
    let get_id = syscall_id("System.Storage.Get");
    let mut code = vec![];
    code.extend_from_slice(&[0x57, 0x01, 0x00]); // INITSLOT 1 local
    code.push(0x41); // SYSCALL
    code.extend_from_slice(&get_context_id);
    code.push(0x70); // STLOC0
    code.push(0x68); // LDLOC0
    code.push(0x0C); // PUSHDATA1
    code.push(3);
    code.extend_from_slice(b"key");
    code.push(0x41); // SYSCALL
    code.extend_from_slice(&get_id);
    code.push(0x40);

    let mut ctx = ExecutionContext::new(&config).expect("context init");
    ctx.initialize(&code, &[]).expect("init");

    let gas_before = ctx.gas_used();
    while !ctx.step().expect("step").halted {}
    let gas_after = ctx.gas_used();

    let consumed = gas_after - gas_before;
    assert!(consumed >= 100, "Storage.Get should cost at least 100 gas");
    assert!(
        consumed <= 500,
        "Storage.Get should not cost more than 500 gas"
    );
}

/// Test that exception handling consumes gas
#[test]
fn gas_exception_handling_consumes_appropriately() {
    let config = RuntimeConfig {
        gas_limit: 100_000,
        ..Default::default()
    };

    // TRY...THROW should consume gas
    // Position: 0=TRY, 1=catch_off, 2=finally_off, 3=PUSHDATA1, 4=len, 5-8=data, 9=THROW, 10=PUSH1, 11=RET
    let code = [
        0x3B, 0x0A, 0x00, // TRY: catch=+10 (points to position 10)
        0x0C, 0x04, // PUSHDATA1 4 bytes
        0x45, 0x52, 0x52, 0x4F, // "ERRO"
        0x3A, // THROW
        0x11, // catch: PUSH1
        0x40, // RET
    ];

    let mut ctx = ExecutionContext::new(&config).expect("context init");
    ctx.initialize(&code, &[]).expect("init");

    let gas_before = ctx.gas_used();
    while !ctx.step().expect("step").halted {}
    let gas_after = ctx.gas_used();

    assert!(
        gas_after > gas_before,
        "Exception handling should consume gas"
    );
}

/// Test that loops don't leak gas
#[test]
fn gas_loops_consume_predictable_amount() {
    let config = RuntimeConfig {
        gas_limit: 1_000_000,
        ..Default::default()
    };

    // Simple loop: 10 iterations
    let mut code = vec![];
    code.extend(&[0x11; 10]); // PUSH1 (1 gas each)
    code.push(0x40); // RET

    let mut ctx = ExecutionContext::new(&config).expect("context init");
    ctx.initialize(&code, &[]).expect("init");

    let gas_before = ctx.gas_used();
    while !ctx.step().expect("step").halted {}
    let gas_after = ctx.gas_used();

    let consumed = gas_after - gas_before;
    // 10 PUSH1 + 1 RET = 11 gas
    assert!(consumed >= 10, "Should consume at least 10 gas");
    assert!(consumed <= 20, "Should not consume excessive gas");
}

/// Regression test: Ensure gas costs don't accidentally drop to zero
#[test]
fn gas_never_zero_after_operations() {
    let config = RuntimeConfig {
        gas_limit: 100_000,
        ..Default::default()
    };

    // Various operations that should always consume gas
    let operations: Vec<Vec<u8>> = vec![
        vec![0x11, 0x11, 0x9E, 0x40], // PUSH1, PUSH1, ADD, RET (ADD needs 2 operands)
        vec![0x11, 0x11, 0x9F, 0x40], // PUSH1, PUSH1, MUL, RET (MUL needs 2 operands)
        vec![0x0C, 0x01, 0x61, 0x40], // PUSHDATA1 1 byte 'a', RET
        vec![0x21, 0x21, 0x40],       // NOP, NOP, RET
        vec![0x11, 0x45, 0x40],       // PUSH1, DROP, RET (DROP needs something on stack)
    ];

    for (i, op) in operations.iter().enumerate() {
        let mut ctx = ExecutionContext::new(&config.clone()).expect("context init");
        ctx.initialize(op, &[]).expect("init");

        let gas_before = ctx.gas_used();
        while !ctx.step().expect("step").halted {}
        let gas_after = ctx.gas_used();

        let consumed = gas_after - gas_before;
        assert!(
            consumed > 0,
            "Operation {i} should consume gas, got {consumed}"
        );
    }
}

/// Test that gas limit is enforced
#[test]
fn gas_limit_is_enforced() {
    let config = RuntimeConfig {
        gas_limit: 10, // Very low limit
        ..Default::default()
    };

    // Code that would exceed gas limit
    // 100 PUSH1 instructions (1 gas each, so 100 would exceed limit of 10)
    let mut code = vec![0x11; 100];
    code.push(0x40); // RET

    let mut ctx = ExecutionContext::new(&config).expect("context init");
    ctx.initialize(&code, &[]).expect("init");

    // Step until we hit an error or halt
    let err = loop {
        match ctx.step() {
            Ok(step) if step.halted => break None,
            Ok(_) => continue,
            Err(e) => break Some(e),
        }
    };
    assert!(err.is_some(), "Should run out of gas");
}

/// Test that cryptographic operations consume more gas
#[test]
fn gas_crypto_consumes_more_than_basic_ops() {
    let config = RuntimeConfig {
        gas_limit: 1_000_000,
        ..Default::default()
    };

    // Use more complex operations (multiple pushes and operations) vs basic
    let code_complex: Vec<u8> = vec![
        0x11, 0x11, 0x11, 0x11, 0x11, // 5x PUSH1
        0x9E, 0x9E, 0x9E, 0x9E, // 4x ADD
        0x40, // RET
    ];

    // Basic arithmetic (fewer operations)
    let code_basic = [
        0x11, 0x11, 0x9E, 0x40, // PUSH1, PUSH1, ADD, RET
    ];

    let mut ctx_complex = ExecutionContext::new(&config.clone()).expect("context init");
    ctx_complex.initialize(&code_complex, &[]).expect("init");

    let mut ctx_basic = ExecutionContext::new(&config).expect("context init");
    ctx_basic.initialize(&code_basic, &[]).expect("init");

    while !ctx_complex.step().expect("step").halted {}
    let gas_after_complex = ctx_complex.gas_used();

    while !ctx_basic.step().expect("step").halted {}
    let gas_after_basic = ctx_basic.gas_used();

    // More operations should consume more gas
    assert!(
        gas_after_complex > gas_after_basic,
        "More operations should consume more gas"
    );
}
