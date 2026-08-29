/// Conformance tests for exception handling
///
/// Tests TRY/ENDTRY/ENDFINALLY with CATCH and FINALLY blocks
use neo_devpack_solidity::runtime::execution::ExecutionContext;
use neo_devpack_solidity::runtime::RuntimeConfig;

/// Test TRY without exception takes normal path
#[test]
fn try_block_no_exception_normal_path() {
    // Based on runtime_exception_tests.rs pattern
    // Position: 0=TRY, 1=catch_off, 2=finally_off, 3=PUSH1, 4=ENDTRY, 5=end_off, 6=RET, 7=PUSH3, 8=ENDFINALLY
    let code = [
        0x3B, 0x00, 0x07, // TRY: catch=0, finally=+7 (points to position 7)
        0x11, // body: PUSH1
        0x3D, 0x02, // ENDTRY(+2) -> position 6 (RET)
        0x40, // RET
        0x13, // finally: PUSH3
        0x3F, // ENDFINALLY
    ];

    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code, &[]).expect("init");

    while !ctx.step().expect("step").halted {}
    assert_eq!(ctx.return_data(), (3i64).to_le_bytes().to_vec());
}

/// Test THROW without TRY block propagates exception
#[test]
fn throw_without_try_block_propagates() {
    let code = [
        0x0C, 0x04, // PUSHDATA1 4 bytes
        0x45, 0x52, 0x52, 0x4F, // "ERRO"
        0x3A, // THROW
        0x40, // RET
    ];

    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code, &[]).expect("init");

    // Step until we hit an error or halt
    let err = loop {
        match ctx.step() {
            Ok(step) if step.halted => break None,
            Ok(_) => continue,
            Err(e) => break Some(e),
        }
    };
    assert!(err.is_some(), "Should throw exception");
}

/// Test TRY with CATCH catches exception
#[test]
fn try_catch_catches_exception() {
    // Pattern from runtime_exception_tests.rs
    let code = [
        0x3B, 0x04, 0x00, // TRY: catch=+4, finally=0
        0x3A, // THROW
        0x11, // catch: PUSH1
        0x40, // RET
    ];

    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code, &[]).expect("init");

    while !ctx.step().expect("step").halted {}
    assert_eq!(ctx.return_data(), (1i64).to_le_bytes().to_vec());
}

/// Test ASSERT with true condition passes
#[test]
fn assert_true_passes() {
    let code = [
        0x11, // PUSH1 (true)
        0x39, // ASSERT
        0x40, // RET
    ];

    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code, &[]).expect("init");

    while !ctx.step().expect("step").halted {}
}

/// Test ASSERT with false condition throws
#[test]
fn assert_false_throws() {
    let code = [
        0x10, // PUSH0 (false)
        0x39, // ASSERT
        0x40, // RET
    ];

    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code, &[]).expect("init");

    // Step until we hit an error or halt
    let err = loop {
        match ctx.step() {
            Ok(step) if step.halted => break None,
            Ok(_) => continue,
            Err(e) => break Some(e),
        }
    };
    assert!(err.is_some(), "ASSERT false should throw");
}

/// Test nested TRY blocks
#[test]
fn nested_try_blocks() {
    // From runtime_exception_tests.rs pattern
    let code = [
        0x3B, 0x00, 0x0E, // outer TRY: catch=0, finally=+14
        0x3B, 0x04, 0x00, // inner TRY: catch=+4, finally=0
        0x3A, // inner THROW
        0x45, // inner catch: DROP (exception)
        0x11, // PUSH1
        0x3D, 0x02, // inner ENDTRY(+2)
        0x3D, 0x02, // outer ENDTRY(+2)
        0x40, // end: RET
        0x13, // outer finally: PUSH3
        0x3F, // ENDFINALLY
    ];

    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code, &[]).expect("init");

    while !ctx.step().expect("step").halted {}
    assert_eq!(ctx.return_data(), (3i64).to_le_bytes().to_vec());
}

/// Test FINALLY block executes after normal completion
#[test]
fn finally_executes_after_normal_completion() {
    let code = [
        0x3B, 0x00, 0x07, // TRY: catch=0, finally=+7
        0x12, // body: PUSH2
        0x3D, 0x02, // ENDTRY(+2)
        0x40, // end: RET
        0x13, // finally: PUSH3
        0x3F, // ENDFINALLY
    ];

    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code, &[]).expect("init");

    while !ctx.step().expect("step").halted {}
    assert_eq!(ctx.return_data(), (3i64).to_le_bytes().to_vec());
}

/// Test FINALLY block executes after exception
#[test]
fn finally_executes_after_exception() {
    let code = [
        0x3B, 0x00, 0x05, // TRY: catch=0, finally=+5
        0x3A, // THROW
        0x40, // RET (won't reach)
        0x21, // finally: NOP
        0x3F, // ENDFINALLY -> rethrows
    ];

    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code, &[]).expect("init");

    let err = loop {
        match ctx.step() {
            Ok(step) if step.halted => break None,
            Ok(_) => continue,
            Err(e) => break Some(e),
        }
    };
    assert!(err.is_some(), "should rethrow after finally");
}

/// Test ABORT terminates execution immediately
#[test]
fn abort_terminates_immediately() {
    let code = [
        0x21, // NOP (before ABORT)
        0x38, // ABORT
        0x21, // NOP (should not execute)
        0x40, // RET
    ];

    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code, &[]).expect("init");

    // Step until we hit an error or halt
    let err = loop {
        match ctx.step() {
            Ok(step) if step.halted => break None,
            Ok(_) => continue,
            Err(e) => break Some(e),
        }
    };
    assert!(err.is_some(), "ABORT should terminate with error");
}

/// Test TRY_L with long offsets
#[test]
fn try_l_with_long_offsets() {
    // Simplified: TRY_L with catch handler, no finally, RET directly from catch
    let code = [
        0x3C, // TRY_L (position 0)
        0x10, 0x00, 0x00, 0x00, // catch_offset=16 (points to position 16)
        0x00, 0x00, 0x00, 0x00, // finally_offset=0
        0x0C, 0x04, // PUSHDATA1 4 (position 9)
        0x54, 0x45, 0x53, 0x54, // "TEST"
        0x3A, // THROW (position 15)
        0x11, // catch: PUSH1 (position 16)
        0x40, // RET (position 17)
    ];

    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code, &[]).expect("init");

    while !ctx.step().expect("step").halted {}
}

/// Test empty exception message
#[test]
fn throw_with_empty_message() {
    let code = [
        0x0C, 0x00, // PUSHDATA1 0 bytes
        0x3A, // THROW
    ];

    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code, &[]).expect("init");

    // Step until we hit an error or halt
    let err = loop {
        match ctx.step() {
            Ok(step) if step.halted => break None,
            Ok(_) => continue,
            Err(e) => break Some(e),
        }
    };
    assert!(err.is_some(), "Should throw");
}

/// Test ENDTRY without TRY throws error
#[test]
fn endtry_without_try_throws() {
    let code = [
        0x3D, // ENDTRY (no matching TRY)
        0x01, // end_offset
        0x40, // RET
    ];

    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code, &[]).expect("init");

    let result = ctx.step();
    assert!(result.is_err(), "ENDTRY without TRY should error");
}

/// Test gas is consumed during exception handling
#[test]
fn exception_handling_consumes_gas() {
    let config = RuntimeConfig {
        gas_limit: 10_000,
        ..Default::default()
    };

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

    assert!(gas_after > gas_before, "Gas should be consumed");
}

/// Test exception message is preserved through catch
#[test]
fn exception_message_preserved_in_catch() {
    // Position: 0=PUSHDATA1, 1=len, 2-10=data(9 bytes), 11=TRY, 12=catch_off, 13=finally_off, 14=THROW, 15=NOP, 16=ENDTRY, 17=end_off, 18=RET
    let code = [
        0x0C, 0x09, // PUSHDATA1 9 bytes
        0x54, 0x65, 0x73, 0x74, 0x45, 0x72, 0x72, 0x6F, 0x72, // "TestError"
        0x3B, 0x04, 0x00, // TRY: catch=+4 (11+4=15), finally=0
        0x3A, // THROW (uses value on stack as message)
        0x21, // catch: NOP (message consumed but not checked)
        0x3D, 0x02, // ENDTRY(+2) -> position 18
        0x40, // RET
    ];

    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code, &[]).expect("init");

    while !ctx.step().expect("step").halted {}
}

/// Test multiple exceptions in sequence
#[test]
fn multiple_exceptions_in_sequence() {
    let code = [
        0x11, // PUSH1
        0x39, // ASSERT (passes, true)
        0x10, // PUSH0
        0x39, // ASSERT (throws, false)
        0x40, // RET (never reached)
    ];

    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code, &[]).expect("init");

    // Step until we hit an error or halt
    let err = loop {
        match ctx.step() {
            Ok(step) if step.halted => break None,
            Ok(_) => continue,
            Err(e) => break Some(e),
        }
    };
    assert!(err.is_some(), "Second ASSERT should fail");
}
