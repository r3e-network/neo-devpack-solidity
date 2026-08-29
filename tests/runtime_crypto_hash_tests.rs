use neo_devpack_solidity::runtime::execution::ExecutionContext;
use neo_devpack_solidity::runtime::RuntimeConfig;
use ripemd::Ripemd160;
use sha2::{Digest, Sha256};

fn syscall_id(name: &str) -> [u8; 4] {
    let mut h = Sha256::new();
    h.update(name.as_bytes());
    let d = h.finalize();
    [d[0], d[1], d[2], d[3]]
}

fn push_data(script: &mut Vec<u8>, data: &[u8]) {
    assert!(
        data.len() <= u8::MAX as usize,
        "push_data only supports PUSHDATA1 lengths"
    );
    script.push(0x0C); // PUSHDATA1
    script.push(data.len() as u8);
    script.extend_from_slice(data);
}

// CryptoLib native contract hash in NeoVM stack byte order (UInt160 little-endian).
const CRYPTOLIB_HASH_LE: [u8; 20] = [
    0x1B, 0xF5, 0x75, 0xAB, 0x11, 0x89, 0x68, 0x84, 0x13, 0x61, 0x0A, 0x35, 0xA1, 0x28, 0x86, 0xCD,
    0xE0, 0xB6, 0x6C, 0x72,
];

#[test]
fn hash160_matches_ripemd_sha256() {
    // Hash160 is not a syscall in Neo N3; use CryptoLib.sha256 + CryptoLib.ripemd160.
    let call_id = syscall_id("System.Contract.Call");
    let mut code = vec![
        0x57, 0x01, 0x00, // INITSLOT 1 local, 0 args
    ];

    // Stack order for System.Contract.Call: [args, flags, method, hash]
    // CryptoLib.sha256("data")
    push_data(&mut code, b"data");
    code.push(0x11); // PUSH1
    code.push(0xC0); // PACK
    code.push(0x1F); // PUSH15 (CallFlags.All = 0x0F)
    push_data(&mut code, b"sha256");
    push_data(&mut code, &CRYPTOLIB_HASH_LE);
    code.push(0x41); // SYSCALL
    code.extend_from_slice(&call_id);
    code.push(0x70); // STLOC0

    // CryptoLib.ripemd160(sha256)
    code.push(0x68); // LDLOC0
    code.push(0x11); // PUSH1
    code.push(0xC0); // PACK
    code.push(0x1F); // PUSH15 (CallFlags.All)
    push_data(&mut code, b"ripemd160");
    push_data(&mut code, &CRYPTOLIB_HASH_LE);
    code.push(0x41); // SYSCALL
    code.extend_from_slice(&call_id);
    code.push(0x40); // RET
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code, &[]).expect("init");
    while !ctx.step().expect("step").halted {}
    let expected = Ripemd160::digest(Sha256::digest(b"data"));
    assert_eq!(ctx.return_data(), expected.to_vec());
}

#[test]
fn hash256_double_sha256() {
    // Hash256 is not a syscall in Neo N3; use CryptoLib.sha256 twice.
    let call_id = syscall_id("System.Contract.Call");
    let mut code = vec![
        0x57, 0x01, 0x00, // INITSLOT 1 local, 0 args
    ];

    // CryptoLib.sha256("abc")
    push_data(&mut code, b"abc");
    code.push(0x11); // PUSH1
    code.push(0xC0); // PACK
    code.push(0x1F); // PUSH15 (CallFlags.All = 0x0F)
    push_data(&mut code, b"sha256");
    push_data(&mut code, &CRYPTOLIB_HASH_LE);
    code.push(0x41); // SYSCALL
    code.extend_from_slice(&call_id);
    code.push(0x70); // STLOC0

    // CryptoLib.sha256(digest1)
    code.push(0x68); // LDLOC0
    code.push(0x11); // PUSH1
    code.push(0xC0); // PACK
    code.push(0x1F); // PUSH15 (CallFlags.All)
    push_data(&mut code, b"sha256");
    push_data(&mut code, &CRYPTOLIB_HASH_LE);
    code.push(0x41); // SYSCALL
    code.extend_from_slice(&call_id);
    code.push(0x40); // RET
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code, &[]).expect("init");
    while !ctx.step().expect("step").halted {}
    let expected = {
        let first = Sha256::digest(b"abc");
        Sha256::digest(first)
    };
    assert_eq!(ctx.return_data(), expected.to_vec());
}

#[test]
fn checksig_returns_boolean_result() {
    // Use 33-byte compressed secp256k1 pubkey and test DER signature to verify syscall execution
    let mut code = vec![0x0C, 0x21];
    code.extend_from_slice(&[2u8; 33]);
    // DER sig with correct length prefix (0x44 = 68 bytes)
    code.extend_from_slice(&[0x0C, 0x46]);
    code.extend_from_slice(&[0x30, 0x44, 0x02, 0x20]);
    code.extend_from_slice(&[1u8; 32]); // r
    code.extend_from_slice(&[0x02, 0x20]);
    code.extend_from_slice(&[2u8; 32]); // s
    code.extend_from_slice(&[0x41, 86, 231, 179, 39, 0x40]); // CheckSig, RET
    let mut ctx = ExecutionContext::new(&RuntimeConfig::default()).expect("context init");
    ctx.initialize(&code, &[]).expect("init");
    while !ctx.step().expect("step").halted {}
    assert!(ctx.return_data().len() == 1);
}
