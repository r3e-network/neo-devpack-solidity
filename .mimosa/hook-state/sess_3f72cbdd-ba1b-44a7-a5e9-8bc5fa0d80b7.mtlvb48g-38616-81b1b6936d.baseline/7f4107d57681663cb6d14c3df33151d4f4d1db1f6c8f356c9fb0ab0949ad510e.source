use super::*;

impl ExecutionContext {
    pub(crate) fn murmur3_32(data: &[u8], seed: u32) -> u32 {
        const C1: u32 = 0xcc9e2d51;
        const C2: u32 = 0x1b873593;
        let mut h1: u32 = seed;
        let len = data.len();

        // Process 4-byte chunks
        let chunks = data.chunks_exact(4);
        let remainder = chunks.remainder();

        for chunk in chunks {
            let k1 = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            let k1 = k1.wrapping_mul(C1);
            let k1 = k1.rotate_left(15);
            let k1 = k1.wrapping_mul(C2);

            h1 ^= k1;
            h1 = h1.rotate_left(13);
            h1 = h1.wrapping_mul(5).wrapping_add(0xe6546b64);
        }

        // Process remaining bytes
        if !remainder.is_empty() {
            let mut k1: u32 = 0;
            for (i, &byte) in remainder.iter().enumerate() {
                k1 |= (byte as u32) << (i * 8);
            }
            let k1 = k1.wrapping_mul(C1);
            let k1 = k1.rotate_left(15);
            let k1 = k1.wrapping_mul(C2);
            h1 ^= k1;
        }

        // Finalization
        h1 ^= len as u32;
        h1 ^= h1 >> 16;
        h1 = h1.wrapping_mul(0x85ebca6b);
        h1 ^= h1 >> 13;
        h1 = h1.wrapping_mul(0xc2b2ae35);
        h1 ^= h1 >> 16;

        h1
    }

    /// Verify a NIST P-256 (secp256r1) ECDSA signature over a 32-byte prehash.
    ///
    /// Neo N3's `System.Crypto.CheckSig` / `CheckMultisig` (and the default
    /// account witness) verify ECDSA on **secp256r1** with a **64-byte compact**
    /// signature over the transaction signing hash. The pre-fix simulator
    /// verified secp256k1 and accepted DER (variable length), so it could mark a
    /// signature valid that a real node rejects — the worst kind of auth
    /// false-positive. This mirrors the p256 path already used by the
    /// `CryptoLib.verifyWithECDsa` handler.
    ///
    /// # Arguments
    /// * `message`   - the 32-byte signing hash used directly as the prehash
    /// * `pubkey`    - SEC1-encoded point (33-byte compressed or 65-byte uncompressed)
    /// * `signature` - 64-byte compact (r ‖ s) ECDSA signature
    ///
    /// # Returns
    /// `true` only if the signature is a valid secp256r1 signature; malformed,
    /// wrong-length, or wrong-curve inputs reject.
    pub(crate) fn verify_secp256r1_with_message(
        message: &[u8],
        pubkey: &[u8],
        signature: &[u8],
    ) -> bool {
        use p256::ecdsa::signature::hazmat::PrehashVerifier;
        use p256::ecdsa::{Signature, VerifyingKey};
        if message.len() != 32 {
            return false;
        }
        // Neo witness signatures are exactly 64-byte compact; DER is not a
        // valid Neo verification-script signature form.
        if signature.len() != 64 {
            return false;
        }
        let Ok(vk) = VerifyingKey::from_sec1_bytes(pubkey) else {
            return false;
        };
        let Ok(sig) = Signature::from_slice(signature) else {
            return false;
        };
        vk.verify_prehash(message, &sig).is_ok()
    }

    /// Get the current message hash for signature verification.
    ///
    /// Returns the host-injected transaction signing hash when one was armed
    /// via [`Self::override_signing_hash`] for this execution (S3 fix),
    /// otherwise `None`.
    ///
    /// `None` (the default when no host has injected a real signing hash) is
    /// the honest answer: the embedded runtime has no script container from
    /// which to derive a verifiable Neo N3 transaction digest, so any
    /// signature check against a fabricated hash is meaningless. Callers
    /// (`System.Crypto.CheckSig` / `CheckMultisig`) treat `None` as
    /// "verification fails" (push `false`), making rejection the default.
    /// Hosts that need meaningful results inject the real digest via
    /// [`Self::override_signing_hash`].
    pub(crate) fn get_current_message_hash(&self) -> Option<[u8; 32]> {
        self.active_signing_hash
    }
}
