use neo_devpack_solidity::runtime::execution::ExecutionContext;
use neo_devpack_solidity::runtime::RuntimeConfig;
use sha2::{Digest, Sha256};

fn syscall_id(name: &str) -> [u8; 4] {
    let mut h = Sha256::new();
    h.update(name.as_bytes());
    let d = h.finalize();
    [d[0], d[1], d[2], d[3]]
}

#[test]
fn iterator_token_reports_iterator_type() {
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("ctx");
    let find = syscall_id("System.Storage.Find");
    let get_context = syscall_id("System.Storage.GetContext");

    // System.Iterator.Dispose is not a Neo N3 syscall. Instead, validate that the iterator token
    // returned from Storage.Find is recognized as an iterator (ISTYPE 0x80).
    let script = vec![
        0x41,
        get_context[0],
        get_context[1],
        get_context[2],
        get_context[3],
        0x0C,
        0x00,
        0x10, // options = 0
        0x41,
        find[0],
        find[1],
        find[2],
        find[3],
        0xD9,
        0x80, // ISTYPE 0x80 (iterator)
        0x40, // RET
    ];

    ctx.initialize(&script, &[]).expect("init");
    while !ctx.step().expect("step").halted {}
    assert_eq!(ctx.return_data(), vec![1u8]);
}
