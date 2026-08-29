use neo_devpack_solidity::runtime::execution::ExecutionContext;
use neo_devpack_solidity::runtime::RuntimeConfig;

#[test]
fn ret_without_value_clears_return_data() {
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&[0x40], &[]).expect("init");
    // The StepResult opcode name is only populated when debugging is enabled
    // (the hot path skips the per-instruction String alloc); this test
    // inspects it, so opt in.
    ctx.enable_debugging();

    let step = ctx.step().expect("step RET");
    assert_eq!(step.opcode, "RET");
    assert!(ctx.return_data().is_empty(), "return data should be empty");
    assert_eq!(
        step.instruction_pointer as usize, 1,
        "instruction pointer should advance to end"
    );
}

#[test]
fn ret_with_value_uses_top_of_stack() {
    // PUSH1 (0x11) followed by RET (0x40)
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&[0x11, 0x40], &[]).expect("init");

    ctx.step().expect("push1");
    let step = ctx.step().expect("ret");

    let expected = (1i64).to_le_bytes().to_vec();
    assert_eq!(
        ctx.return_data(),
        expected,
        "RET should use top-of-stack bytes"
    );
    assert_eq!(
        step.instruction_pointer as usize, 2,
        "instruction pointer should advance beyond bytecode"
    );
}
