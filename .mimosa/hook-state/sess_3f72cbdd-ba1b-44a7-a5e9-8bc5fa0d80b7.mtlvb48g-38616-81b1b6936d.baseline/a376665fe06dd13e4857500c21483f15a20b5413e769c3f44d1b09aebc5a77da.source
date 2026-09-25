//! Storage Key Tests
//!
//! Tests for storage key computation. The state-variable base slot is
//! `SHA256(name)`; mapping-entry slots are derived on-chain per nesting
//! level as `keccak256(StdLib.serialize(key) || slot)` and are therefore
//! exercised end-to-end by the runtime/integration suites rather than here
//! (the old `derive_mapping_slot`/`KeyFragment` off-chain API implemented a
//! different scheme than the emitted bytecode and was removed).

use neo_devpack_solidity::storage_key::compute_state_slot;

#[test]
fn test_compute_state_slot_basic() {
    let slot = compute_state_slot("balance");
    assert_eq!(slot.len(), 32, "Slot should be 32 bytes");
}

#[test]
fn test_compute_state_slot_deterministic() {
    let slot1 = compute_state_slot("myVar");
    let slot2 = compute_state_slot("myVar");
    assert_eq!(slot1, slot2, "Same name should produce same slot");
}

#[test]
fn test_compute_state_slot_different_names() {
    let slot1 = compute_state_slot("balance");
    let slot2 = compute_state_slot("allowance");
    assert_ne!(
        slot1, slot2,
        "Different names should produce different slots"
    );
}

#[test]
fn test_compute_state_slot_known_digest() {
    // SHA-256("balances") — pins the production base-slot derivation so an
    // accidental hash/scheme change is caught at unit level.
    let slot = compute_state_slot("balances");
    assert_eq!(
        hex::encode(slot),
        "ca75286f2c7875b8b5d80d59c8bce4a640ce976b4e5e0b35116b682a029d5c33"
    );
}
