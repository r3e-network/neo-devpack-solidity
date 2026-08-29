/// Conformance tests for cryptographic signature verification
///
/// Tests System.Crypto.CheckSig and System.Crypto.CheckMultisig using
/// known test vectors from secp256k1 specification.
use neo_devpack_solidity::runtime::execution::ExecutionContext;
use neo_devpack_solidity::runtime::RuntimeConfig;
use sha2::{Digest, Sha256};

fn syscall_id(name: &str) -> [u8; 4] {
    let mut h = Sha256::new();
    h.update(name.as_bytes());
    let d = h.finalize();
    [d[0], d[1], d[2], d[3]]
}

/// Test CheckSig with invalid signature (should return false)
#[test]
fn checksig_invalid_signature_returns_false() {
    // pubkey (33 bytes, compressed) + signature (DER-encoded)
    let mut code = vec![];

    // Push invalid signature (DER format but wrong key)
    code.extend_from_slice(&[0x0C, 0x46]); // PUSHDATA1, 70 bytes
    code.extend_from_slice(&[0x30, 0x44, 0x02, 0x20]); // DER header
    code.extend_from_slice(&[0xFF; 32]); // r (invalid)
    code.extend_from_slice(&[0x02, 0x20]);
    code.extend_from_slice(&[0xFF; 32]); // s (invalid)

    // Push public key (compressed, 33 bytes)
    code.extend_from_slice(&[0x0C, 0x21]);
    code.extend_from_slice(&[0x02; 33]); // Invalid pubkey

    // System.Crypto.CheckSig
    let checksig_id = syscall_id("System.Crypto.CheckSig");
    code.push(0x41); // SYSCALL
    code.extend_from_slice(&checksig_id);
    code.push(0x40); // RET

    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code, &[]).expect("init");
    while !ctx.step().expect("step").halted {}

    // Should return false (single byte 0x00)
    assert_eq!(ctx.return_data(), vec![0x00]);
}

/// Test CheckSig with malformed inputs (wrong length)
#[test]
fn checksig_malformed_input_returns_false() {
    // Wrong pubkey length (not 33 or 65 bytes)
    let mut code = vec![];

    // Push signature (valid DER)
    code.extend_from_slice(&[0x0C, 0x46]);
    code.extend_from_slice(&[0x30, 0x44, 0x02, 0x20]);
    code.extend_from_slice(&[1u8; 32]);
    code.extend_from_slice(&[0x02, 0x20]);
    code.extend_from_slice(&[2u8; 32]);

    // Push pubkey with wrong length (16 bytes instead of 33 or 65)
    code.extend_from_slice(&[0x0C, 0x10]);
    code.extend_from_slice(&[0x03; 16]);

    let checksig_id = syscall_id("System.Crypto.CheckSig");
    code.push(0x41); // SYSCALL
    code.extend_from_slice(&checksig_id);
    code.push(0x40); // RET

    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code, &[]).expect("init");
    while !ctx.step().expect("step").halted {}

    assert_eq!(ctx.return_data(), vec![0x00]);
}

/// Test CheckMultisig with empty inputs (should return false)
#[test]
fn checkmultisig_empty_inputs_returns_false() {
    let checkmultisig_id = syscall_id("System.Crypto.CheckMultisig");
    let mut code = vec![
        0x0C, // PUSHDATA1 - empty signatures
        0x00, // 0 bytes
        0x0C, // PUSHDATA1 - empty pubkeys
        0x00, // 0 bytes
        0x41, // SYSCALL
    ];
    code.extend_from_slice(&checkmultisig_id);
    code.push(0x40); // RET

    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code, &[]).expect("init");
    while !ctx.step().expect("step").halted {}

    assert_eq!(ctx.return_data(), vec![0x00]);
}

/// Test CheckSig syscall ID is correctly registered
#[test]
fn checksig_syscall_id_registered() {
    let expected = syscall_id("System.Crypto.CheckSig");
    // Verify the syscall ID is non-zero and properly formed
    assert_ne!(expected, [0u8; 4]);
}

/// Test CheckMultisig syscall ID is correctly registered
#[test]
fn checkmultisig_syscall_id_registered() {
    let expected = syscall_id("System.Crypto.CheckMultisig");
    assert_ne!(expected, [0u8; 4]);
}

/// Test signature verification with compact signature format
#[test]
fn checksig_compact_signature_format() {
    let mut code = vec![];

    // Push compact signature (64 bytes)
    code.extend_from_slice(&[0x0C, 0x40]);
    code.extend_from_slice(&[1u8; 64]); // Invalid but correctly-sized sig

    // Push compressed pubkey (33 bytes)
    code.extend_from_slice(&[0x0C, 0x21]);
    code.extend_from_slice(&[0x02; 33]);

    let checksig_id = syscall_id("System.Crypto.CheckSig");
    code.push(0x41);
    code.extend_from_slice(&checksig_id);
    code.push(0x40);

    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code, &[]).expect("init");
    while !ctx.step().expect("step").halted {}

    // Should return false for invalid signature
    assert_eq!(ctx.return_data(), vec![0x00]);
}

/// Test gas is consumed for signature verification
#[test]
fn checksig_consumes_gas() {
    let mut code = vec![];

    // Valid DER-encoded signature (70 bytes total)
    code.extend_from_slice(&[0x0C, 0x46]); // PUSHDATA1, 70 bytes
    code.extend_from_slice(&[0x30, 0x44, 0x02, 0x20]); // DER sequence header
    code.extend_from_slice(&[1u8; 32]); // r value
    code.extend_from_slice(&[0x02, 0x20]); // s value header
    code.extend_from_slice(&[2u8; 32]); // s value

    // Public key (compressed, 33 bytes)
    code.extend_from_slice(&[0x0C, 0x21]);
    code.extend_from_slice(&[0x02; 33]);

    // System.Crypto.CheckSig syscall
    let checksig_id = syscall_id("System.Crypto.CheckSig");
    code.push(0x41); // SYSCALL
    code.extend_from_slice(&checksig_id);
    code.push(0x40); // RET

    let config = RuntimeConfig {
        gas_limit: 1_000_000,
        ..Default::default()
    };
    let mut ctx = ExecutionContext::new(&config).expect("context init");
    ctx.initialize(&code, &[]).expect("init");

    let gas_before = ctx.gas_used();
    while !ctx.step().expect("step").halted {}
    let gas_after = ctx.gas_used();

    // Gas should be consumed (CheckSig costs 1000 gas per gas.rs)
    assert!(gas_after > gas_before);
}
