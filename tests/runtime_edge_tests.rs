use neo_devpack_solidity::runtime::execution::ExecutionContext;
use neo_devpack_solidity::runtime::RuntimeConfig;

#[test]
fn stack_push_overflow_errors_after_limit() {
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("ctx");
    // Push up to the NeoVM stack depth limit (2048)
    for _ in 0..2048 {
        ctx.push_stack(neo_devpack_solidity::runtime::execution::StackItem::Integer(1))
            .expect("push within limit");
    }
    let overflow = ctx.push_stack(neo_devpack_solidity::runtime::execution::StackItem::Integer(2));
    assert!(overflow.is_err(), "pushing past stack limit should error");
}

#[test]
fn read_memory_out_of_bounds_fails() {
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("ctx");
    ctx.initialize(&[], &[]).expect("init");
    let err = ctx.read_memory(10, 1).expect_err("oob read should fail");
    assert!(
        format!("{err}").to_ascii_lowercase().contains("memory"),
        "expected memory error, got: {err}"
    );
}

#[test]
fn pop_on_empty_stack_errors() {
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("ctx");
    let result = ctx.pop_stack();
    assert!(result.is_err(), "pop on empty stack should error");
}

#[test]
fn returndatacopy_bounds_checked() {
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("ctx");
    ctx.set_return_data(vec![1, 2, 3]);
    let err = ctx
        .returndatacopy(0, 2, 5)
        .expect_err("copy beyond buffer should fail");
    assert!(
        format!("{err}")
            .to_ascii_lowercase()
            .contains("out of bounds"),
        "unexpected error: {err}"
    );
}
