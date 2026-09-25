//! Regression tests for the dead-code removal refactor (agent: deadcode).
//!
//! `interop_id_bytes` used to live in the (now deleted) Yul `codegen` module
//! and was the only live item in that cluster. It was moved verbatim to the
//! standalone `interop` module. These tests pin its new public path and the
//! exact interop IDs it must produce, guarding against silent hash mismatches
//! in emitted bytecode.

use neo_devpack_solidity::interop::interop_id_bytes;

/// Known-good Neo N3 interop IDs (first 4 bytes of SHA-256 of the name).
#[test]
fn interop_id_bytes_matches_known_neo_n3_ids() {
    assert_eq!(
        interop_id_bytes("System.Contract.Call"),
        [0x62, 0x7D, 0x5B, 0x52]
    );
    assert_eq!(
        interop_id_bytes("System.Storage.Put"),
        [0xE6, 0x3F, 0x18, 0x84]
    );
    assert_eq!(
        interop_id_bytes("System.Storage.Get"),
        [0x92, 0x5D, 0xE8, 0x31]
    );
    assert_eq!(
        interop_id_bytes("System.Runtime.Notify"),
        [0x95, 0x01, 0x6F, 0x61]
    );
    assert_eq!(
        interop_id_bytes("Neo.Crypto.Keccak256"),
        [0xDC, 0xB1, 0x21, 0xE0]
    );
}

/// The ID must be deterministic and equal to the SHA-256 prefix of the name.
#[test]
fn interop_id_bytes_is_sha256_prefix() {
    use sha2::{Digest, Sha256};

    for name in [
        "System.Contract.Call",
        "System.Storage.Get",
        "System.Crypto.CheckSig",
        "System.Contract.CreateStandardAccount",
    ] {
        let digest = Sha256::digest(name.as_bytes());
        assert_eq!(interop_id_bytes(name), digest[..4], "mismatch for {name}");
        // Deterministic across calls.
        assert_eq!(interop_id_bytes(name), interop_id_bytes(name));
    }
}
