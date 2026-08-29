use neo_devpack_solidity::runtime::execution::ExecutionContext;
use neo_devpack_solidity::runtime::RuntimeConfig;

#[test]
fn try_catches_throw_and_jumps_to_handler() {
    // TRY (catch at +4, no finally) -> body THROW -> catch: PUSH1 RET
    let code = [
        0x3B, 0x04, 0x00, // catch offset = +4, finally offset = 0
        0x3A, // THROW
        0x11, // catch block starts here: PUSH1
        0x40, // RET
    ];

    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("ctx init");
    ctx.initialize(&code, &[]).expect("init");

    while !ctx.step().expect("step").halted {}

    let expected = (1i64).to_le_bytes().to_vec();
    assert_eq!(ctx.return_data(), expected);
}

#[test]
fn endtry_jumps_to_finally_on_success() {
    // TRY with finally at +7, body PUSH2 ENDTRY(+2), finally PUSH3 ENDFINALLY -> RET sees 3
    let code = [
        0x3B, 0x00, 0x07, // catch offset = 0 (none), finally offset = +7
        0x12, // body: PUSH2
        0x3D, 0x02, // ENDTRY(+2) -> jumps to finally, stores end pointer at RET
        0x40, // end: RET (runs after ENDFINALLY)
        0x13, // finally: PUSH3
        0x3F, // ENDFINALLY
    ];

    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("ctx init");
    ctx.initialize(&code, &[]).expect("init");

    while !ctx.step().expect("step").halted {}

    let expected = (3i64).to_le_bytes().to_vec();
    assert_eq!(ctx.return_data(), expected);
}

#[test]
fn finally_rethrows_when_no_catch() {
    // TRY with no catch (0), finally at +5, body THROW -> finally runs then rethrows
    let code = [
        0x3B, 0x00, 0x05, // catch offset = 0 (none), finally offset = +5 (NOP)
        0x3A, // body: THROW
        0x40, // RET (won't reach)
        0x21, // finally: NOP
        0x3F, // ENDFINALLY -> should rethrow
    ];

    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("ctx init");
    ctx.initialize(&code, &[]).expect("init");

    let err = loop {
        match ctx.step() {
            Ok(step) if step.halted => break None,
            Ok(_) => continue,
            Err(e) => break Some(e),
        }
    };
    assert!(err.is_some(), "should rethrow after finally when no catch");
}

#[test]
fn nested_try_allows_outer_finally_after_inner_catch() {
    // Outer TRY (no catch) with FINALLY; inner TRY has CATCH and handles the throw.
    // Outer ENDTRY executes FINALLY and does not rethrow.
    let code = [
        0x3B, 0x00, 0x0E, // outer TRY: catch=0, finally=+14
        0x3B, 0x04, 0x00, // inner TRY: catch=+4, finally=0
        0x3A, // inner THROW
        0x45, // inner catch: DROP (exception)
        0x11, // PUSH1
        0x3D, 0x02, // ENDTRY(+2) -> to outer ENDTRY
        0x3D, 0x02, // outer ENDTRY(+2) -> end at RET, jumps to finally
        0x40, // end: RET (runs after outer ENDFINALLY)
        0x13, // outer finally: PUSH3
        0x3F, // ENDFINALLY
    ];

    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("ctx init");
    ctx.initialize(&code, &[]).expect("init");

    while !ctx.step().expect("step").halted {}

    let expected = (3i64).to_le_bytes().to_vec();
    assert_eq!(ctx.return_data(), expected);
}
