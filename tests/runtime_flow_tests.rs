use neo_devpack_solidity::runtime::execution::ExecutionContext;
use neo_devpack_solidity::runtime::RuntimeConfig;

#[test]
fn jmp_out_of_bounds_produces_error() {
    // JMP with offset that would jump past bytecode end
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    // JMP uses a 1-byte signed relative offset from the beginning of the instruction.
    ctx.initialize(&[0x22, 0x0A], &[]).expect("init");

    // First step executes JMP and should error
    let result = ctx.step();
    assert!(result.is_err(), "expected JMP out of bounds to error");
    let err = format!("{}", result.unwrap_err());
    assert!(
        err.contains("jump target out of bounds"),
        "unexpected error: {err}"
    );
}

#[test]
fn jmpif_skips_when_condition_false() {
    // PUSH0 (false), JMPIF -> RET, PUSH1, RET
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    // JMPIF offset is relative to the JMPIF opcode position.
    // Layout: PUSH0, JMPIF +3 -> RET, PUSH1, RET
    let code = [0x10, 0x24, 0x03, 0x11, 0x40];
    ctx.initialize(&code, &[]).expect("init");

    ctx.step().expect("push0");
    let step = ctx.step().expect("jmpif");
    assert_eq!(
        step.instruction_pointer as usize, 3,
        "JMPIF should advance past instruction when condition is false"
    );

    // Next instruction should be PUSH1
    let next = ctx.step().expect("push1");
    assert_eq!(next.instruction_pointer as usize, 4);
}

#[test]
fn jmpif_jumps_forward_when_true() {
    // PUSH1 (true), JMPIF -> RET, NOP, RET
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    // Layout: PUSH1, JMPIF +3 -> RET, NOP, RET
    let code = [0x11, 0x24, 0x03, 0x21, 0x40];
    ctx.initialize(&code, &[]).expect("init");

    ctx.step().expect("push1");
    let step = ctx.step().expect("jmpif");
    assert_eq!(
        step.instruction_pointer as usize, 4,
        "JMPIF should jump to RET target"
    );

    let ret = ctx.step().expect("ret");
    assert!(ret.halted, "RET should halt execution");
}

#[test]
fn jmpifnot_jumps_when_false() {
    // PUSH0 (false), JMPIFNOT -> RET
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    // Layout: PUSH0, JMPIFNOT +3 -> RET, NOP, RET
    let code = [0x10, 0x26, 0x03, 0x21, 0x40];
    ctx.initialize(&code, &[]).expect("init");

    ctx.step().expect("push0");
    let step = ctx.step().expect("jmpifnot");
    assert_eq!(
        step.instruction_pointer as usize, 4,
        "JMPIFNOT should skip to RET when condition is false"
    );
    let ret = ctx.step().expect("ret");
    assert!(ret.halted, "RET should halt execution");
}

#[test]
fn jmp_accepts_wide_offsets() {
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    let target: u32 = 250;
    // JMP_L uses a 4-byte signed relative offset from the beginning of the instruction.
    let mut code = vec![0x23];
    code.extend_from_slice(&(target as i32).to_le_bytes());
    let padding = target as usize - 5;
    code.extend(std::iter::repeat_n(0x21, padding)); // fill with NOPs
    code.push(0x40); // RET at jump target

    ctx.initialize(&code, &[]).expect("init");

    let step = ctx.step().expect("jmp");
    assert_eq!(
        step.instruction_pointer, target,
        "JMP_L should support 4-byte relative offsets"
    );
}
