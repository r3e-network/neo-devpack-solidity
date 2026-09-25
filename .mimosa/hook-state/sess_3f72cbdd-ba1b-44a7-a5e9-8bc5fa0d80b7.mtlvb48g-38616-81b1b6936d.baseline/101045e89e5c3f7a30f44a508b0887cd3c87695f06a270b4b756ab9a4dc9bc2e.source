use neo_devpack_solidity::runtime::execution::ExecutionContext;
use neo_devpack_solidity::runtime::RuntimeConfig;

#[test]
fn normalize_account_accepts_prefixed() {
    let cfg = RuntimeConfig::default();
    let mut ctx = ExecutionContext::new(&cfg).expect("ctx");
    // override caller with prefixed hash should succeed
    ctx.override_caller_account("0x0123456789abcdef0123456789abcdef01234567")
        .expect("valid address");
}

#[test]
fn normalize_account_accepts_unprefixed_address() {
    let cfg = RuntimeConfig::default();
    let mut ctx = ExecutionContext::new(&cfg).expect("ctx");
    ctx.override_caller_account("0123456789abcdef0123456789abcdef01234567")
        .expect("valid address without prefix");
}

#[test]
fn normalize_account_accepts_uppercase_prefixed() {
    let cfg = RuntimeConfig::default();
    let mut ctx = ExecutionContext::new(&cfg).expect("ctx");
    ctx.override_caller_account("0X0123456789abcdef0123456789abcdef01234567")
        .expect("valid address with uppercase prefix");
}

#[test]
fn normalize_account_rejects_odd_length_address() {
    let cfg = RuntimeConfig::default();
    let mut ctx = ExecutionContext::new(&cfg).expect("ctx");
    let result = ctx.override_caller_account("0x123456789abcdef0123456789abcdef01234567");
    assert!(result.is_err(), "expected invalid address to error");
}

#[test]
fn normalize_account_rejects_uppercase_prefixed_odd_length_address() {
    let cfg = RuntimeConfig::default();
    let mut ctx = ExecutionContext::new(&cfg).expect("ctx");
    let result = ctx.override_caller_account("0X123456789abcdef0123456789abcdef01234567");
    assert!(result.is_err(), "expected invalid address to error");
}
