//! Neo N3 interop service identifiers.
//!
//! Computes the 4-byte interop IDs used to encode syscalls in NeoVM bytecode.

use sha2::{Digest, Sha256};

// Compute Neo N3 interop ID (first 4 bytes of SHA-256 of method name, little-endian order)
/// Compute the 4-byte interop ID for a Neo syscall name
/// This is used to identify syscalls in NeoVM bytecode
pub fn interop_id_bytes(name: &str) -> [u8; 4] {
    let mut hasher = Sha256::new();
    hasher.update(name.as_bytes());
    let digest = hasher.finalize();
    [digest[0], digest[1], digest[2], digest[3]]
}

#[cfg(test)]
mod interop_tests {
    use super::interop_id_bytes;

    #[test]
    fn test_interop_ids() {
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
            interop_id_bytes("Neo.Crypto.Keccak256"),
            [0xDC, 0xB1, 0x21, 0xE0]
        );
    }
}
