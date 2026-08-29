use super::*;

/// Canonical `Panic(uint256)` ABI selector: `keccak256("Panic(uint256)")[0..4]`.
///
/// Fixed constant (0x4e487b71). Hoisted out of `emit_panic` because that
/// function runs once per panic guard (~41 lowering sites) and re-hashing a
/// constant string every time is pure waste. `tests::panic_selector_matches_keccak`
/// (bottom of this file) recomputes the digest and asserts this value, so the
/// hoisted constant can never silently drift from the canonical selector.
const PANIC_SELECTOR: [u8; 4] = [0x4e, 0x48, 0x7b, 0x71];

/// Task #107 — Emit the canonical EVM `Panic(uint256)` revert envelope for a
/// given panic code and THROW it.
///
/// The envelope shape is:
///   `keccak256("Panic(uint256)")[0..4] || abi.encode(uint256 code)`
///     = `0x4e487b71` (4 bytes) || `<32-byte big-endian code>`
///
/// This matches the shape consumed by `catch Panic(uint code)` in
/// `src/ir/statements/dispatch/try_catch.rs` and by the observe() helper
/// in `tests/fuzz_tests.rs`, so migrated call sites all route through the
/// same canonical path.
///
/// Before Task #103 / #107, sites emitted a raw `PushLiteral(ByteString("Panic: 0xNN")) ; Throw`
/// which the runtime surfaced only as an exception-message string and made
/// `catch Panic(uint code)` fail to bind. This helper replaces that shape
/// everywhere it was used (assert false, div/mod zero, arith over/under-flow,
/// unary negation, enum range cast, abi.decode short buffer, array pop from
/// empty) with the canonical envelope.
///
/// Stack contract: pushes the envelope ByteString then emits `Throw`. No
/// items are popped.
///
/// Emitted instructions:
/// ```text
///   PushLiteral ByteArray([0x4e, 0x48, 0x7b, 0x71])     ; selector
///   PushLiteral ByteArray([0x00 * 31, code])           ; 32-byte BE payload
///   CallBuiltin BytesConcat 2                           ; selector || payload (CAT -> Buffer)
///   Convert ByteArray                                   ; Buffer -> ByteString
///   Throw
/// ```
///
/// The `Convert` is load-bearing: `BytesConcat` lowers to NeoVM `CAT`, whose
/// result is a **Buffer** (mutable). A faulting external call delivers this
/// payload verbatim to the caller's `catch`, where `emit_selector_guard` does
/// `ISTYPE ByteString` — which is FALSE for a Buffer on a real Neo node, so
/// `catch Panic(uint code)` would silently fall through to `catch (bytes)` /
/// the fallback. Normalizing to a ByteString (matching the `Error(string)`
/// envelope, which is already a ByteString) makes `catch Panic` match on-chain.
/// The in-repo simulator's lenient CAT/ISTYPE masked this divergence; verified
/// with a neo-express differential.
pub(crate) fn emit_panic(code: u8, instructions: &mut Vec<Instruction>) {
    instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(
        PANIC_SELECTOR.to_vec(),
    )));
    let mut payload = vec![0u8; 32];
    payload[31] = code;
    instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(payload)));
    instructions.push(Instruction::CallBuiltin {
        builtin: BuiltinCall::BytesConcat,
        arg_count: 2,
    });
    instructions.push(Instruction::Convert {
        target: ConvertTarget::ByteArray,
    });
    instructions.push(Instruction::Throw);
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha3::Digest;

    /// Guard the hoisted `PANIC_SELECTOR` constant against silent drift: the
    /// value must equal the first four bytes of the live Keccak-256 digest of
    /// the canonical signature string.
    #[test]
    fn panic_selector_matches_keccak() {
        let mut hasher = Keccak256::new();
        hasher.update(b"Panic(uint256)");
        let digest = hasher.finalize();
        assert_eq!(PANIC_SELECTOR, [digest[0], digest[1], digest[2], digest[3]]);
    }
}
