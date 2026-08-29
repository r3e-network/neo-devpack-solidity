use super::*;

#[test]
fn opcode_table_includes_core_surface() {
    assert!(OPCODES.contains_key(&0x34), "CALL should be registered");
    assert!(OPCODES.contains_key(&0xC3), "NEWARRAY should be registered");
    assert!(
        OPCODES.contains_key(&0x0C),
        "PUSHDATA1 should be registered"
    );
    assert!(OPCODES.contains_key(&0xE0), "ABORTMSG should be registered");
    assert!(OPCODES.len() >= 120, "expected a wide opcode surface");
}

#[test]
fn syscall_table_hashes_are_stable() {
    let id = interop_id_bytes("System.Runtime.Notify");
    assert!(SYSCALLS.contains_key(&id));
    assert_eq!(SYSCALLS.get(&id).unwrap().name, "System.Runtime.Notify");
}

#[test]
fn native_contracts_present() {
    assert!(native_contract_name(
        b"\xcf\x76\xe2\x8b\xd0\x06\x2c\x4a\x47\x8e\xe3\x55\x61\x01\x13\x19\xf3\xcf\xa4\xd2"
    )
    .is_some());
}
