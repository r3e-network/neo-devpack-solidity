// ==================== compute_state_slot Tests ====================

#[test]
fn test_compute_state_slot_deterministic() {
    let slot1 = compute_state_slot("balance");
    let slot2 = compute_state_slot("balance");
    assert_eq!(slot1, slot2);
}

#[test]
fn test_compute_state_slot_different_names() {
    let slot1 = compute_state_slot("balance");
    let slot2 = compute_state_slot("owner");
    assert_ne!(slot1, slot2);
}

#[test]
fn test_compute_state_slot_length() {
    let slot = compute_state_slot("test");
    assert_eq!(slot.len(), 32);
}

#[test]
fn test_compute_state_slot_empty_name() {
    let slot = compute_state_slot("");
    assert_eq!(slot.len(), 32);
    // SHA-256 of empty string is a known value
    let expected = Sha256::digest(b"");
    assert_eq!(slot[..], expected[..]);
}

#[test]
fn test_compute_state_slot_unicode() {
    let slot = compute_state_slot("余额");
    assert_eq!(slot.len(), 32);
    // Should be deterministic
    let slot2 = compute_state_slot("余额");
    assert_eq!(slot, slot2);
}

