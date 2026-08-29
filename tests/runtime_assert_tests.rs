use neo_devpack_solidity::runtime::execution::ExecutionContext;
use neo_devpack_solidity::runtime::RuntimeConfig;

#[test]
fn assert_passes_on_truthy() {
    let code = [0x11, 0x39, 0x40]; // PUSH1, ASSERT, RET
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code, &[]).expect("init");
    while !ctx.step().expect("step").halted {}
    assert!(ctx.return_data().is_empty());
}

#[test]
fn assert_fails_on_false() {
    let code = [0x10, 0x39]; // PUSH0, ASSERT
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code, &[]).expect("init");
    let err = loop {
        match ctx.step() {
            Ok(step) if step.halted => break None,
            Ok(_) => continue,
            Err(e) => break Some(e),
        }
    };
    assert!(err.is_some(), "ASSERT on false should error");
}

#[test]
fn abortmsg_returns_error_with_message() {
    // push message "boom" then ABORTMSG
    let code = [0x0C, 0x04, b'b', b'o', b'o', b'm', 0xE0];
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code, &[]).expect("init");
    let mut err = None;
    loop {
        match ctx.step() {
            Ok(step) if step.halted => break,
            Ok(_) => continue,
            Err(e) => {
                err = Some(e);
                break;
            }
        }
    }
    assert!(err.is_some(), "ABORTMSG should fail");
    let msg = format!("{}", err.unwrap());
    assert!(msg.contains("boom"), "message should propagate");
}

#[test]
fn assertmsg_passes_when_true_and_fails_when_false() {
    // true, "ok" -> ASSERTMSG -> RET (should succeed)
    let code_ok = [0x11, 0x0C, 0x02, b'o', b'k', 0xE1, 0x40];
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code_ok, &[]).expect("init");
    while !ctx.step().expect("step").halted {}

    // false, "fail" -> ASSERTMSG should error
    let code_fail = [0x10, 0x0C, 0x04, b'f', b'a', b'i', b'l', 0xE1];
    let mut ctx2 = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx2.initialize(&code_fail, &[]).expect("init");
    let mut err = None;
    loop {
        match ctx2.step() {
            Ok(step) if step.halted => break,
            Ok(_) => continue,
            Err(e) => {
                err = Some(e);
                break;
            }
        }
    }
    assert!(err.is_some(), "ASSERTMSG on false should fail");
    let msg = format!("{}", err.unwrap());
    assert!(msg.contains("fail"), "ASSERTMSG should include message");
}
