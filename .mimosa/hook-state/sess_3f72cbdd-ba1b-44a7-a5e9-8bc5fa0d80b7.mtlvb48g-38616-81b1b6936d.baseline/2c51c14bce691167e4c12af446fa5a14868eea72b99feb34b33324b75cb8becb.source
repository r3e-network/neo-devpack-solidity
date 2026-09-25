//! Storage Key Computation Module
//!
//! Computes deterministic storage keys for Neo N3 smart contract state
//! variables, matching the layout actually emitted by the compiler's
//! bytecode (see `src/cli/bytecode/bytecode_helpers/storage/mapping.rs`).
//!
//! # Key Derivation (production scheme)
//!
//! - Base slot of a state variable: `SHA256(variable_name)` —
//!   [`compute_state_slot`]. Struct fields use the same hash over the
//!   qualified `"Struct::field"` name (see `src/ir/build/inference.rs`).
//! - Mapping entry / dynamic-array element: derived ITERATIVELY per nesting
//!   level on-chain as `slot = keccak256(StdLib.serialize(key) || slot)`,
//!   where `serialize` is Neo's StdLib binary serializer applied to the
//!   canonicalized key stack item and the current slot is appended LAST.
//!   Dynamic arrays store their length at the base slot and element `i` at
//!   `keccak256(serialize(i) || slot)`.
//! - Struct field inside a mapping value: `keccak256(field_key || slot)`
//!   where `field_key = SHA256("Struct::field")`.
//!
//! The mapping-entry derivation depends on Neo's StdLib `serialize` native
//! call (the serialized form embeds the stack-item type byte), so it cannot
//! be reproduced faithfully off-chain without replicating Neo's
//! BinarySerializer byte-for-byte. A previous `derive_mapping_slot` /
//! `KeyFragment` API in this module implemented a DIFFERENT, untyped
//! SHA-256-based scheme that matched nothing the compiler emits; it has been
//! removed so off-chain consumers cannot silently compute wrong keys.

use sha2::{Digest, Sha256};

/// Compute the canonical storage slot hash for a state variable name.
///
/// The returned value is the 32-byte SHA-256 digest of the UTF-8 name.
pub fn compute_state_slot(name: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(name.as_bytes());
    let digest = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

#[cfg(test)]
mod tests;
