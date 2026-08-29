use neo_devpack_solidity::runtime::execution::ExecutionContext;
use neo_devpack_solidity::runtime::RuntimeConfig;

#[test]
fn cat_concatenates_bytes() {
    // push "ab", push "cd", CAT, RET
    let code = [0x0C, 0x02, b'a', b'b', 0x0C, 0x02, b'c', b'd', 0x8B, 0x40];
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code, &[]).expect("init");

    while !ctx.step().expect("step").halted {}

    assert_eq!(ctx.return_data(), b"abcd");
}

#[test]
fn left_and_right_slice() {
    // data "hello" -> LEFT 2 -> RIGHT 3 -> RET (returns right slice)
    let code = [
        0x0C, 0x05, b'h', b'e', b'l', b'l', b'o', // push bytes
        0x12, // push 2
        0x8D, // LEFT 2 leaves "he"
        0x13, // push 3
        0x8E, // RIGHT 3 of "he" -> out of bounds, expect error
    ];
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code, &[]).expect("init");
    let mut err = None;
    loop {
        match ctx.step() {
            Ok(step) => {
                if step.halted {
                    break;
                }
            }
            Err(e) => {
                err = Some(e);
                break;
            }
        }
    }
    assert!(
        err.is_some(),
        "RIGHT beyond length should error due to bounds"
    );
}

#[test]
fn substr_extracts_segment() {
    // "hello" index 1 count 3 -> "ell"
    let code = [
        0x0C, 0x05, b'h', b'e', b'l', b'l', b'o', // bytes
        0x11, // index 1
        0x13, // count 3
        0x8C, // SUBSTR
        0x40,
    ];
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code, &[]).expect("init");

    while !ctx.step().expect("step").halted {}

    assert_eq!(ctx.return_data(), b"ell");
}

#[test]
fn newbuffer_allocates_zeroed_bytes() {
    // len=3 -> NEWBUFFER -> RET should return three zero bytes
    let code = [0x13, 0x88, 0x40];
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code, &[]).expect("init");
    while !ctx.step().expect("step").halted {}
    assert_eq!(ctx.return_data(), vec![0u8; 3]);
}

#[test]
fn memcpy_copies_slice_into_destination() {
    // dst = NEWBUFFER(4), dst_off=1, src="abcd", src_off=0, count=2 => MEMCPY -> RET
    // Real NeoVM MEMCPY: Pop 5, Push 0. Must DUP dst before MEMCPY so we
    // can RET it afterwards.
    let code = [
        0x14, // push 4
        0x88, // NEWBUFFER -> [dst]
        0x4A, // DUP -> [dst, dst]
        0x11, // dst_offset = 1
        0x0C, 0x04, b'a', b'b', b'c', b'd', // src bytes
        0x10, // src_offset = 0
        0x12, // count = 2
        0x89, // MEMCPY -> [dst]  (Pop 5, Push 0; DUP'd copy remains)
        0x40, // RET
    ];
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code, &[]).expect("init");
    while !ctx.step().expect("step").halted {}

    assert_eq!(ctx.return_data(), vec![0, b'a', b'b', 0]);
}
