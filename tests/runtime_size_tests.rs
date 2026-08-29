use neo_devpack_solidity::runtime::execution::ExecutionContext;
use neo_devpack_solidity::runtime::RuntimeConfig;

#[test]
fn size_of_array_and_map() {
    // Build array via PACK(3) then SIZE; then NEWMAP, set one key, SIZE; RET will return last size
    let code = vec![
        0x11, 0x12, 0x13, 0x13, 0xC0, // push 1,2,3,count=3 => PACK (array of 3)
        0xCA, // SIZE -> 3 (codegen mapping)
        0xC8, // NEWMAP
        0x4A, // DUP map (SETITEM consumes the collection)
        0x0C, 0x02, b'a', b'a', 0x11, 0xD0, // set k1->1
        0xCA, // SIZE -> 1
        0x40, // RET
    ];

    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code, &[]).expect("init");
    while !ctx.step().expect("step").halted {}

    // Return data should be map size (1)
    let expected = (1i64).to_le_bytes().to_vec();
    assert_eq!(ctx.return_data(), expected);
}

#[test]
fn size_errors_on_scalar() {
    // PUSH1 then SIZE should error
    let code = [0x11, 0xCA];
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code, &[]).expect("init");
    let err = ctx.step().and_then(|_| ctx.step()).err();
    assert!(err.is_some(), "SIZE on scalar should error");
}
