//! Validates the runtime `CryptoLib.verifyWithECDsa` handler (NamedCurveHash 23
//! = secp256r1 + SHA256, Neo's default) against a GENUINE P-256 ECDSA signature.
//!
//! Previously the handler was a stub that returned `false` unconditionally, so
//! every signature check failed in the test oracle. These tests sign a real
//! P-256 message and confirm the handler accepts the valid signature and rejects
//! a tampered one — proving the curve/hash/byte-order wiring is correct.

use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::runtime::types::StackItem;
use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};
use p256::ecdsa::signature::hazmat::PrehashSigner;
use p256::ecdsa::{Signature, SigningKey};
use sha2::{Digest, Sha256};

const SRC: &str = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
contract C {
    function verify(bytes memory m, bytes memory pk, bytes memory s) external pure returns (bool) {
        return CryptoLib.verifyWithECDsa(m, pk, s, 23);
    }
}"#;

fn run_verify(message: &[u8], pubkey: &[u8], sig: &[u8]) -> bool {
    let arts = compile_contracts(SRC, false, 2).expect("compile");
    let art = arts.iter().find(|a| a.metadata.name == "C").expect("C");
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("runtime");
    let res = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "verify",
            &[
                StackItem::byte_array(message.to_vec()),
                StackItem::byte_array(pubkey.to_vec()),
                StackItem::byte_array(sig.to_vec()),
            ],
        )
        .expect("verify call host-level");
    assert!(
        res.success,
        "verify() must execute; exc={:?}",
        res.exception
    );
    // bool return is a single 0x00/0x01 byte.
    res.return_data.first().copied().unwrap_or(0) != 0
}

fn key_and_sig(message: &[u8]) -> (Vec<u8>, Vec<u8>) {
    // Deterministic, fixed signing key (a valid non-zero P-256 scalar).
    let sk = SigningKey::from_slice(&[0x42u8; 32]).expect("valid P-256 scalar");
    let vk = sk.verifying_key();
    // Compressed SEC1 public key (33 bytes); the handler also accepts uncompressed.
    let pubkey = vk.to_encoded_point(true).as_bytes().to_vec();
    // Neo hashes the message with SHA256 then verifies over the digest, so sign
    // the SHA256 prehash.
    let digest = Sha256::digest(message);
    let sig: Signature = sk.sign_prehash(&digest).expect("sign");
    (pubkey, sig.to_bytes().to_vec())
}

#[test]
fn verify_with_ecdsa_secp256r1_accepts_valid_signature() {
    let message = b"neo-solidity verifyWithECDsa conformance";
    let (pubkey, sig) = key_and_sig(message);
    assert_eq!(sig.len(), 64, "P-256 signature must be 64 bytes (r||s)");
    assert!(
        run_verify(message, &pubkey, &sig),
        "a genuine secp256r1 signature over SHA256(message) must verify true"
    );
}

#[test]
fn verify_with_ecdsa_secp256r1_rejects_tampered_message() {
    let message = b"neo-solidity verifyWithECDsa conformance";
    let (pubkey, sig) = key_and_sig(message);
    let mut tampered = message.to_vec();
    tampered[0] ^= 0x01;
    assert!(
        !run_verify(&tampered, &pubkey, &sig),
        "a signature must NOT verify against a tampered message"
    );
}

#[test]
fn verify_with_ecdsa_rejects_wrong_length_signature() {
    let message = b"x";
    let (pubkey, _sig) = key_and_sig(message);
    assert!(
        !run_verify(message, &pubkey, &[0u8; 10]),
        "a malformed (wrong-length) signature must verify false, not fault"
    );
}
