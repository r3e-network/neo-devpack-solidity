use neo_devpack_solidity::runtime::execution::ExecutionContext;
use neo_devpack_solidity::runtime::RuntimeConfig;

#[test]
fn pack_and_pickitem_returns_expected_element() {
    // Push 1,2,3 then count=3 -> PACK -> push index 1 -> PICKITEM -> RET
    let code = [0x11, 0x12, 0x13, 0x13, 0xC0, 0x11, 0xCE, 0x40];
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code, &[]).expect("init");

    while !ctx.step().expect("step").halted {}

    let expected = (2i64).to_le_bytes().to_vec();
    assert_eq!(
        ctx.return_data(),
        expected,
        "PICKITEM should fetch array element at index 1"
    );
}

#[test]
fn newarray_and_setitem_store_and_load() {
    // size=2 -> NEWARRAY -> DUP -> push index 0, value 5 -> SETITEM -> push index 0 -> PICKITEM -> RET
    // Note: SETITEM consumes the collection, so keep a reference via DUP.
    let code = [0x12, 0xC3, 0x4A, 0x10, 0x15, 0xD0, 0x10, 0xCE, 0x40];
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code, &[]).expect("init");

    while !ctx.step().expect("step").halted {}

    let expected = (5i64).to_le_bytes().to_vec();
    assert_eq!(
        ctx.return_data(),
        expected,
        "SETITEM then PICKITEM should round-trip stored value"
    );
}

#[test]
fn newmap_set_and_get_roundtrip() {
    // NEWMAP -> key "aa" -> value 7 -> SETITEM -> key "aa" -> PICKITEM -> RET
    let mut code = vec![0xC8, 0x4A]; // NEWMAP + DUP
                                     // key "aa"
    code.extend_from_slice(&[0x0C, 0x02, b'a', b'a']);
    // value 7
    code.extend_from_slice(&[0x17]);
    code.push(0xD0); // SETITEM
                     // key "aa" again
    code.extend_from_slice(&[0x0C, 0x02, b'a', b'a']);
    code.push(0xCE); // PICKITEM
    code.push(0x40); // RET

    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code, &[]).expect("init");

    while !ctx.step().expect("step").halted {}

    let expected = (7i64).to_le_bytes().to_vec();
    assert_eq!(
        ctx.return_data(),
        expected,
        "map SETITEM/PICKITEM should round-trip stored value"
    );
}

#[test]
fn haskey_and_keys_values_work_for_map() {
    // NEWMAP -> set k1->1, k2->2 -> DUP map -> push k1 -> HASKEY (consumes dup) -> SWAP -> KEYS -> NIP -> RET
    let mut code = vec![0xC8];
    code.extend_from_slice(&[0x4A, 0x0C, 0x02, b'a', b'a', 0x11, 0xD0]); // DUP + k1->1
    code.extend_from_slice(&[0x4A, 0x0C, 0x02, b'b', b'b', 0x12, 0xD0]); // DUP + k2->2
    code.push(0x4A); // DUP map for HASKEY
    code.extend_from_slice(&[0x0C, 0x02, b'a', b'a', 0xCB]); // HASKEY on dup
    code.push(0x50); // SWAP to bring map on top
    code.push(0xCC); // KEYS
    code.push(0x46); // NIP (drop HASKEY result)
    code.push(0x40); // RET

    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code, &[]).expect("init");

    while !ctx.step().expect("step").halted {}

    let values = ctx.return_data(); // keys serialized
    assert!(
        !values.is_empty(),
        "keys should be serialized into return data by RET"
    );
}

#[test]
fn append_and_size_alias_operate_on_arrays() {
    // NEWARRAY0 -> DUP -> PUSH5 -> APPEND -> SIZE (0xCA) -> RET should return size=1
    // Note: APPEND consumes the collection, so keep a reference via DUP.
    let code = [0xC2, 0x4A, 0x15, 0xCF, 0xCA, 0x40];
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code, &[]).expect("init");

    while !ctx.step().expect("step").halted {}

    let expected = (1i64).to_le_bytes().to_vec();
    assert_eq!(ctx.return_data(), expected, "size after append should be 1");
}

#[test]
fn remove_then_pickitem0_returns_remaining_element() {
    // [1,2] -> REMOVE index 0 -> PICKITEM0 -> RET should return 2
    // Note: REMOVE consumes the collection, so keep a reference via DUP.
    let code = [0x11, 0x12, 0x12, 0xC0, 0x4A, 0x10, 0xD2, 0x10, 0xCE, 0x40];
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code, &[]).expect("init");

    while !ctx.step().expect("step").halted {}

    // NeoVM PACK builds arrays in stack-pop order, so [1,2] becomes [2,1].
    // Removing index 0 drops the first element (2), leaving 1.
    let expected = (1i64).to_le_bytes().to_vec();
    assert_eq!(
        ctx.return_data(),
        expected,
        "remove should drop first element"
    );
}

#[test]
fn packmap_builds_map_and_supports_pickitem() {
    // NeoVM PACKMAP pops KEY first (top), then value — so each pair is pushed
    // value-then-key: {a:1, b:2}, count=2 -> PACKMAP -> push "a" -> PICKITEM -> RET
    let code = [
        0x11, // value 1
        0x0C, 0x01, b'a', // key a
        0x12, // value 2
        0x0C, 0x01, b'b', // key b
        0x12, // count=2
        0xBE, // PACKMAP
        0x0C, 0x01, b'a', // key a again
        0xCE, // PICKITEM
        0x40, // RET
    ];
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code, &[]).expect("init");
    while !ctx.step().expect("step").halted {}

    let expected = (1i64).to_le_bytes().to_vec();
    assert_eq!(ctx.return_data(), expected);
}
