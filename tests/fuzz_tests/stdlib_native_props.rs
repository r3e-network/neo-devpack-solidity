//! Property-based tests for the Neo `StdLib` native contract handlers.
//!
//! Target: `src/runtime/execution/execution_impl_part2_native/stdlib.rs` —
//! the `invoke_native_stdlib(method, params)` dispatcher and its sibling
//! base64 codec. These tests drive every assertion through the live CALLT
//! pipeline (script → opcode parser → CALLT dispatch → native handler →
//! result serialisation) using the same `MethodToken` plumbing that
//! `tests/fuzz_tests/baseline_tests.rs::callt_stdlib_itoa_roundtrip_via_token`
//! established. Nothing is hand-called; every invariant is observed at the
//! runtime boundary, so any divergence from the reference oracle (the
//! `base64` crate, Rust's `format!`/`parse`, the `bs58` crate, etc.)
//! surfaces a real cross-contract bug.
//!
//! StdLib handler inventory (from stdlib.rs::invoke_native_stdlib):
//!
//!   IMPLEMENTED (return real values):
//!     * `serialize(item)`        → ByteArray (serde_json of the StackItem)
//!     * `deserialize(bytes)`     → StackItem  (inverse of `serialize`)
//!     * `jsonserialize(item)`    → ByteArray (serde_json::to_string bytes)
//!     * `jsondeserialize(bytes)` → StackItem  (inverse of `jsonserialize`)
//!     * `itoa(value[, base])`    → ByteArray (decimal or hex ASCII)
//!     * `atoi(string[, base])`   → Integer  (i64 saturating)
//!     * `base64encode(bytes)`    → ByteArray (RFC 4648 base64, hand-rolled)
//!     * `base64decode(string)`   → ByteArray (inverse of base64encode)
//!     * `abiencode/abiencodepacked/abidecode` (out of scope here — covered
//!       by `tests/fuzz_tests/native_contract_props.rs` and friends).
//!
//!   UNIMPLEMENTED (fall through to `_ => StackItem::Null`, which serialises
//!   as an empty `return_data` byte array; the IR resolver still WHITELISTS
//!   them so the call dispatches but the native silently no-ops):
//!     * `base58encode` / `base58decode`
//!     * `base58CheckEncode` / `base58CheckDecode`
//!     * `memorycompare` / `memorysearch`
//!     * `hexencode` / `hexdecode`
//!     * `stringsplit` / `strlen`
//!
//! For the unimplemented set we pin the *current* contract — `return_data`
//! is empty (Null serialises to an empty byte slice via
//! `helpers/interop.rs::stack_item_to_bytes`). When/if the runtime fills
//! these in, the harness will start failing and the assertion can be flipped
//! to a real-value check (the reference-crate oracle calls are already
//! computed and asserted as TODO comments inside the harness).

#![allow(clippy::uninlined_format_args)]
#![allow(clippy::needless_borrows_for_generic_args)]

use neo_devpack_solidity::neo::MethodToken;
use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};
use proptest::prelude::*;

/// StdLib native-contract hash (UInt160 LE) — copied from
/// `src/runtime/spec/native_contracts.rs:46`. The same bytes used by every
/// other StdLib CALLT harness in this crate.
const STDLIB_HASH: [u8; 20] = [
    0xc0, 0xef, 0x39, 0xce, 0xe0, 0xe4, 0xe9, 0x25, 0xc6, 0xc2, 0xa0, 0x6a, 0x79, 0xe1, 0x44, 0x0d,
    0xd8, 0x6f, 0xce, 0xac,
];

/// Build a CALLT-only script that pushes `args` (each as a `Vec<u8>` to be
/// emitted via PUSHDATA1) onto the stack in order, then invokes token 0.
///
/// `args` is left-to-right as the native handler receives them — the
/// CALLT impl reverses the popped order so the topmost stack item ends up
/// at `args[arg_count-1]` in the params Array (see calls.rs:82-86).
///
/// PUSHDATA1 is used uniformly because:
///   * It accepts any byte length 0..=255 — the only size we exercise here.
///   * The native handler treats every input as `stack_item_to_bytes(item)`
///     for parsing, so the on-stack representation does not need to match
///     the original Solidity type. ByteArrays serialise to themselves; an
///     empty PUSHDATA1 (length 0) round-trips as an empty buffer.
fn build_callt_script(args: &[Vec<u8>]) -> Vec<u8> {
    let mut script = Vec::with_capacity(8 + args.iter().map(|a| 2 + a.len()).sum::<usize>());
    for arg in args {
        debug_assert!(arg.len() <= 255, "PUSHDATA1 max length is 255");
        script.push(0x0C); // PUSHDATA1
        script.push(arg.len() as u8);
        script.extend_from_slice(arg);
    }
    // CALLT token 0, RET.
    script.extend_from_slice(&[0x37, 0x00, 0x00, 0x40]);
    script
}

/// Run a single StdLib method via CALLT and return its raw `return_data`.
/// Any host-level execution error panics — fault-shapes (exception flagged
/// on the result) are surfaced through `success=false` and the caller
/// asserts on it. This matches the convention used by every other
/// StdLib/CryptoLib CALLT harness in baseline_tests.rs.
fn call_stdlib(method: &str, args: &[Vec<u8>]) -> (bool, Vec<u8>, Option<String>) {
    let tokens = vec![MethodToken::new(
        STDLIB_HASH,
        method,
        args.len() as u16,
        true,
        0x0F,
    )];
    let script = build_callt_script(args);
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("runtime");
    let result = rt
        .execute_with_tokens(&script, &[], &tokens)
        .expect("execute_with_tokens must not fail at host level");
    (
        result.success,
        result.return_data,
        result.exception.map(|e| e.message),
    )
}

/// Push an i64 as a PUSHINT64-shaped 8-byte LE arg. The native `itoa` handler
/// reads it via `stack_item_to_int`, which is satisfied by an 8-byte LE buffer
/// just as well as by a real PUSHINT64 — the dispatcher converts ByteArrays
/// to integers via `decode_uint_le`-style fallback.
fn push_i64_le(v: i64) -> Vec<u8> {
    v.to_le_bytes().to_vec()
}

// ============================================================================
// itoa / atoi
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// itoa(N, 10) round-trips through atoi(., 10) for any signed i32. The
    /// runtime saturates the parsed result into i64 (see stdlib.rs:610), so
    /// every i32 value is round-trippable losslessly.
    ///
    /// Note: base 16 round-trip is covered by
    /// `stdlib_itoa_atoi_hex_roundtrip` below; base 2 is NOT covered because
    /// the native handler explicitly clamps `radix` to {10, 16} in its `atoi`
    /// arm (`if base == 16 { 16 } else { 10 }`, stdlib.rs:594) — passing
    /// `base=2` does NOT route to binary, so the round-trip would assert on
    /// stale-decimal output. That is documented runtime behaviour, not a
    /// test gap.
    #[test]
    fn stdlib_itoa_atoi_decimal_roundtrip(
        n in i32::MIN..=i32::MAX,
    ) {
        // itoa(n, 10): push value, then base.
        let (ok_i, rd_i, exc_i) = call_stdlib(
            "itoa",
            &[push_i64_le(n as i64), push_i64_le(10)],
        );
        prop_assert!(ok_i, "itoa({}, 10) must succeed; exc={:?}", n, exc_i);
        let s = std::str::from_utf8(&rd_i)
            .map(|s| s.to_string())
            .unwrap_or_default();
        prop_assert_eq!(&s, &n.to_string(),
            "itoa({}, 10) must equal format!(\"{{}}\"); got {:?}", n, rd_i);

        // atoi(s, 10): push the ASCII bytes, then base.
        let (ok_a, rd_a, exc_a) = call_stdlib(
            "atoi",
            &[s.as_bytes().to_vec(), push_i64_le(10)],
        );
        prop_assert!(ok_a, "atoi({:?}, 10) must succeed; exc={:?}", s, exc_a);

        // The runtime returns an Integer; the RET path serialises i64 LE.
        // Reading 8-byte LE matches the canonical post-itoa shape used by
        // every prior StdLib roundtrip harness (see baseline_tests.rs:5791).
        let mut buf = [0u8; 8];
        let copy_len = rd_a.len().min(8);
        buf[..copy_len].copy_from_slice(&rd_a[..copy_len]);
        let got = i64::from_le_bytes(buf);
        prop_assert_eq!(got, n as i64,
            "atoi(itoa({})) must round-trip; got {} (rd={:?})", n, got, rd_a);
    }

    /// itoa(N, 16) round-trips through atoi(., 16). The native renders hex
    /// uppercase without `0x` and uses `-ABS` for negatives (stdlib.rs:563-571),
    /// so the round-trip is lossless across the full i32 domain.
    #[test]
    fn stdlib_itoa_atoi_hex_roundtrip(
        n in i32::MIN..=i32::MAX,
    ) {
        let (ok_i, rd_i, exc_i) = call_stdlib(
            "itoa",
            &[push_i64_le(n as i64), push_i64_le(16)],
        );
        prop_assert!(ok_i, "itoa({}, 16) must succeed; exc={:?}", n, exc_i);
        let s = std::str::from_utf8(&rd_i).map(|s| s.to_string()).unwrap_or_default();

        let expected = if n < 0 {
            format!("-{:X}", (n as i64).unsigned_abs())
        } else {
            format!("{:X}", n as u64)
        };
        prop_assert_eq!(&s, &expected,
            "itoa({}, 16) must match StdLib hex shape; got {:?}", n, rd_i);

        let (ok_a, rd_a, exc_a) = call_stdlib(
            "atoi",
            &[s.as_bytes().to_vec(), push_i64_le(16)],
        );
        prop_assert!(ok_a, "atoi({:?}, 16) must succeed; exc={:?}", s, exc_a);
        let mut buf = [0u8; 8];
        let copy_len = rd_a.len().min(8);
        buf[..copy_len].copy_from_slice(&rd_a[..copy_len]);
        let got = i64::from_le_bytes(buf);
        prop_assert_eq!(got, n as i64,
            "atoi(itoa({}, 16), 16) must round-trip; got {} (rd={:?})", n, got, rd_a);
    }

    /// itoa(N, 10) output equals `format!("{N}")`. Pure differential check
    /// against Rust's stdlib formatter — distinct from the round-trip above
    /// because it asserts the exact ASCII shape regardless of any subsequent
    /// atoi call.
    #[test]
    fn stdlib_itoa_decimal_matches_format(
        n in any::<i32>(),
    ) {
        let (ok, rd, exc) = call_stdlib("itoa", &[push_i64_le(n as i64), push_i64_le(10)]);
        prop_assert!(ok, "itoa({}, 10) must succeed; exc={:?}", n, exc);
        let got = std::str::from_utf8(&rd).map(|s| s.to_string()).unwrap_or_default();
        prop_assert_eq!(&got, &format!("{}", n),
            "itoa({}, 10) must equal format!(\"{{}}\"); got {:?}", n, rd);
    }

    /// atoi(s, 10) must equal `s.parse::<i64>()` whenever the input is
    /// well-formed canonical decimal. Malformed inputs are pinned by
    /// `stdlib_atoi_malformed_returns_zero` below.
    #[test]
    fn stdlib_atoi_decimal_matches_parse(
        n in i32::MIN..=i32::MAX,
    ) {
        let s = n.to_string();
        let (ok, rd, exc) = call_stdlib(
            "atoi",
            &[s.as_bytes().to_vec(), push_i64_le(10)],
        );
        prop_assert!(ok, "atoi({:?}, 10) must succeed; exc={:?}", s, exc);
        let mut buf = [0u8; 8];
        let copy_len = rd.len().min(8);
        buf[..copy_len].copy_from_slice(&rd[..copy_len]);
        let got = i64::from_le_bytes(buf);
        let expected: i64 = s.parse().expect("decimal n.to_string() round-trips");
        prop_assert_eq!(got, expected,
            "atoi({:?}) must equal s.parse::<i64>(); got {} (rd={:?})",
            s, got, rd);
    }

    /// atoi of malformed input must error gracefully (no panic). The runtime
    /// substitutes 0 on parse failure (stdlib.rs:607 — `unwrap_or(0)`); we
    /// pin that contract here. Inputs include: empty string, non-numeric
    /// alpha, mixed alpha/digit, and base-10-invalid hex digits.
    ///
    /// NOTE: `--42` (double-negative) is INTENTIONALLY excluded — the runtime
    /// strips a single leading `-` (stdlib.rs:596) and then
    /// `i128::from_str_radix("-42", 10)` succeeds, returning -42, which the
    /// `neg` flag then re-flips to +42. End result: `atoi("--42") == 42`.
    /// That is a **real bug** in the runtime (a double-negative should reject)
    /// — see `stdlib_atoi_double_negative_is_buggy` below for the pinned
    /// observation. When the runtime is fixed to reject this shape, that
    /// pin will start failing and `--42` can be moved into THIS list.
    #[test]
    fn stdlib_atoi_malformed_returns_zero(
        garbage in prop_oneof![
            Just("".to_string()),
            Just("xyz".to_string()),
            Just("12abc".to_string()),
            Just("0xff".to_string()),  // hex prefix in base-10 → invalid
            Just(" \t ".to_string()),  // whitespace-only after trim → empty
            // NOTE: `+42` is NOT in this list — `i128::from_str_radix`
            // (which the runtime delegates to) accepts a leading `+`, so
            // `atoi("+42") == 42` is the documented Rust-stdlib behaviour.
            // Whether StdLib should be stricter is a separate question;
            // pinning that as buggy would be opinion, not divergence.
        ],
    ) {
        let (ok, rd, exc) = call_stdlib(
            "atoi",
            &[garbage.as_bytes().to_vec(), push_i64_le(10)],
        );
        // Critical: must NOT panic / fault. Success path with rd==[0;...].
        prop_assert!(ok,
            "atoi({:?}, 10) on malformed input must NOT fault; exc={:?}",
            garbage, exc);
        let mut buf = [0u8; 8];
        let copy_len = rd.len().min(8);
        buf[..copy_len].copy_from_slice(&rd[..copy_len]);
        let got = i64::from_le_bytes(buf);
        prop_assert_eq!(got, 0,
            "atoi({:?}) malformed input must return 0; got {} (rd={:?})",
            garbage, got, rd);
    }

    /// **BUG PIN — atoi double-negative parses as positive.** The runtime
    /// peels a single `-` then defers to `i128::from_str_radix`, which itself
    /// also accepts a `-`, so the magnitude becomes negative; the outer
    /// `neg` flag then re-negates it. Net result: `atoi("--42") == 42`.
    ///
    ///   Expected (StdLib spec / Rust `i64::from_str`): reject as malformed,
    ///   return 0.
    ///   Observed: returns the inner positive magnitude.
    ///
    /// Repro:
    ///   * input bytes:    `--42` (PUSHDATA1)
    ///   * base:           10
    ///   * `return_data`:  `[42, 0, 0, 0, 0, 0, 0, 0]` (LE i64 = 42)
    ///   * expected:       `[0, 0, ...]` (LE i64 = 0)
    ///
    /// This pin keeps the suite green while documenting the bug; the fix
    /// is a one-liner in stdlib.rs ("after stripping `-`, reject any further
    /// `-` or `+` in `body` before calling `from_str_radix`"). When that
    /// lands, this test will fail and should be deleted in favour of moving
    /// `--42` into `stdlib_atoi_malformed_returns_zero`'s list.
    /// Bug #21 (FIXED): `atoi("--42", 10)` previously returned 42. The fix
    /// at `src/runtime/execution/execution_impl_part2_native/stdlib.rs::atoi`
    /// switched magnitude parsing from `i128::from_str_radix` (which accepts
    /// a leading `-`) to `u128::from_str_radix` (rejects any sign char in
    /// `body`). Now `atoi("--42") == 0`, plus all `++…` / `+-…` shapes.
    #[test]
    fn stdlib_atoi_double_negative_returns_zero(
        _seed in any::<u8>(),
    ) {
        let (ok, rd, exc) = call_stdlib(
            "atoi",
            &[b"--42".to_vec(), push_i64_le(10)],
        );
        prop_assert!(ok, "atoi must not fault on `--42`; exc={:?}", exc);
        let mut buf = [0u8; 8];
        let copy_len = rd.len().min(8);
        buf[..copy_len].copy_from_slice(&rd[..copy_len]);
        let got = i64::from_le_bytes(buf);
        prop_assert_eq!(got, 0,
            "atoi(\"--42\") regression: bug #21 fix slipped — body now \
             accepts a sign char again. Expected 0, got {} (rd={:?})",
            got, rd);
    }

    /// itoa with base outside {10, 16}: the native silently collapses to
    /// decimal (stdlib.rs:563 — `if base == 16 { hex } else { decimal }`).
    /// This pins the documented "no exception channel" semantics — a real
    /// EVM might choose to fault here, but the StdLib spec lets unsupported
    /// bases degrade to base 10. If/when the runtime adds a fault for
    /// invalid bases, this test will start failing and can be flipped.
    #[test]
    fn stdlib_itoa_unsupported_base_falls_back_to_decimal(
        n in i32::MIN..=i32::MAX,
        base in prop_oneof![Just(2i64), Just(8i64), Just(36i64), Just(0i64), Just(-1i64)],
    ) {
        let (ok, rd, exc) = call_stdlib(
            "itoa",
            &[push_i64_le(n as i64), push_i64_le(base)],
        );
        prop_assert!(ok, "itoa({}, {}) must succeed; exc={:?}", n, base, exc);
        let got = std::str::from_utf8(&rd).map(|s| s.to_string()).unwrap_or_default();
        prop_assert_eq!(&got, &n.to_string(),
            "itoa({}, base={}) must fall back to decimal; got {:?}",
            n, base, rd);
    }
}

// ============================================================================
// base64 — round-trip + differential vs the `base64` reference crate
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// base64Decode(base64Encode(b)) == b for any byte buffer up to PUSHDATA1
    /// max length. The native is hand-rolled (stdlib.rs:652-731) and not
    /// shared with any external crate, so this round-trip is the strongest
    /// invariant — it catches alphabet typos, padding off-by-ones, and
    /// chunk-boundary bugs in a single check.
    #[test]
    fn stdlib_base64_roundtrip(
        bytes in prop::collection::vec(any::<u8>(), 0..=180),
    ) {
        // Encode.
        let (ok_e, rd_e, exc_e) = call_stdlib("base64encode", std::slice::from_ref(&bytes));
        prop_assert!(ok_e, "base64Encode(len={}) must succeed; exc={:?}",
            bytes.len(), exc_e);

        // The encoded form is ASCII; PUSHDATA1 caps at 255 — encoded length
        // is ceil(N/3)*4 ≤ 240 for N ≤ 180. Within bounds.
        prop_assert!(rd_e.len() <= 255,
            "base64 encoded length {} exceeds PUSHDATA1 max", rd_e.len());

        // Decode.
        let (ok_d, rd_d, exc_d) = call_stdlib("base64decode", std::slice::from_ref(&rd_e));
        prop_assert!(ok_d, "base64Decode(...) must succeed; exc={:?}", exc_d);
        prop_assert_eq!(&rd_d, &bytes,
            "base64Decode(base64Encode(b)) != b; encoded={:?}, got={:?}, want={:?}",
            std::str::from_utf8(&rd_e).ok(), rd_d, bytes);
    }

    /// Differential: StdLib.base64Encode output equals the `base64` crate's
    /// STANDARD encoding (RFC 4648 with `=` padding). Asserts byte-for-byte
    /// equality — any divergence (alphabet swap, missing pad, URL-safe
    /// chars) surfaces immediately. The runtime explicitly documents
    /// "RFC 4648 standard alphabet" (stdlib.rs:652-654), so this is the
    /// canonical oracle.
    #[test]
    fn stdlib_base64_encode_matches_base64_crate(
        bytes in prop::collection::vec(any::<u8>(), 0..=200),
    ) {
        use base64::Engine;

        let (ok, rd, exc) = call_stdlib("base64encode", std::slice::from_ref(&bytes));
        prop_assert!(ok, "base64Encode must succeed; exc={:?}", exc);
        let got = std::str::from_utf8(&rd).map(|s| s.to_string()).unwrap_or_default();
        let expected = base64::engine::general_purpose::STANDARD.encode(&bytes);
        prop_assert_eq!(&got, &expected,
            "base64Encode disagrees with base64 crate STANDARD; \
             input.len={}, got={:?}, expected={:?}",
            bytes.len(), got, expected);
    }

    /// Differential: StdLib.base64Decode of a `base64`-crate-produced string
    /// equals the original bytes. Mirrors the encode-side oracle from the
    /// opposite direction; together with `stdlib_base64_roundtrip` this
    /// gives 3-way coverage of the codec.
    #[test]
    fn stdlib_base64_decode_matches_base64_crate(
        bytes in prop::collection::vec(any::<u8>(), 0..=200),
    ) {
        use base64::Engine;
        let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
        prop_assume!(encoded.len() <= 255);
        let (ok, rd, exc) = call_stdlib("base64decode", &[encoded.as_bytes().to_vec()]);
        prop_assert!(ok, "base64Decode must succeed; exc={:?}", exc);
        prop_assert_eq!(&rd, &bytes,
            "base64Decode of base64-crate-encoded bytes != original; \
             encoded={:?}, got={:?}, want={:?}", encoded, rd, bytes);
    }

    /// Malformed base64 input — invalid alphabet, bad padding, odd length —
    /// must return an empty buffer (stdlib.rs:642 — the runtime substitutes
    /// `unwrap_or_default()` on parse failure). Critically, must NOT panic.
    ///
    /// NOTE: `"AB=C"` (mid-string padding) is INTENTIONALLY excluded — the
    /// runtime's pad-detection counts trailing `=` only via
    /// `chunk.iter().rev().take_while(...)` (stdlib.rs:710), which gives 0
    /// for `AB=C`, then decodes the literal `=` as 0 and produces 3 bytes
    /// `[0x00, 0x10, 0x02]`. That is a **real bug** — RFC 4648 forbids
    /// non-trailing padding. See `stdlib_base64_mid_padding_is_buggy` below
    /// for the pinned observation.
    #[test]
    fn stdlib_base64_malformed_returns_empty(
        garbage in prop_oneof![
            Just("???".to_string()),                  // invalid chars
            Just("AAA".to_string()),                  // bad length (3 mod 4)
            Just("====".to_string()),                 // all-padding (>2 pads)
            Just("AAAA===AAAA".to_string()),          // misplaced padding (>2 pads in chunk)
        ],
    ) {
        let (ok, rd, exc) = call_stdlib("base64decode", &[garbage.as_bytes().to_vec()]);
        prop_assert!(ok,
            "base64Decode malformed input must NOT fault; input={:?}, exc={:?}",
            garbage, exc);
        prop_assert!(rd.is_empty(),
            "base64Decode of malformed input must return empty; \
             input={:?}, got={:?}", garbage, rd);
    }

    /// **BUG PIN — base64Decode accepts padding-in-middle of a chunk.**
    /// The runtime counts pads only at the trailing position
    /// (stdlib.rs:710 — `chunk.iter().rev().take_while(...)`), so an input
    /// like `"AB=C"` has pad-count = 0 (the trailing char is `C`, not `=`),
    /// and the decode-char path treats `=` as a literal 0 sextet. Net: the
    /// chunk decodes to 3 garbage bytes instead of being rejected.
    ///
    ///   Expected (RFC 4648 strict): reject — `=` may only appear as the
    ///   FINAL one or two characters of the entire input, and the data
    ///   characters before any padding must form a valid sextet sequence.
    ///   Reference: the `base64` crate's STANDARD engine rejects `"AB=C"`
    ///   with `InvalidByte`.
    ///   Observed: returns `[0x00, 0x10, 0x02]`.
    ///
    /// Repro:
    ///   * input bytes:   `AB=C`
    ///   * `return_data`: `[0, 16, 2]`
    ///   * expected:      `[]`
    ///
    /// Fix sketch: in `base64_decode` (stdlib.rs:689), reject any chunk
    /// where a `=` byte appears at index 0 or 1, or where a `=` at index 2
    /// is not followed by a `=` at index 3.
    /// Bug #22 (FIXED): `base64Decode("AB=C")` previously returned the
    /// 3-byte garbage [0x00, 0x10, 0x02]. The fix at
    /// `src/runtime/execution/execution_impl_part2_native/stdlib.rs::base64_decode`
    /// rejects `=` outside the trailing-pad position of the FINAL chunk —
    /// matches RFC 4648 + the `base64` crate.
    #[test]
    fn stdlib_base64_mid_padding_returns_empty(
        _seed in any::<u8>(),
    ) {
        let (ok, rd, exc) = call_stdlib("base64decode", &[b"AB=C".to_vec()]);
        prop_assert!(ok, "base64Decode must not fault on `AB=C`; exc={:?}", exc);
        // Bug #22 fix: should now reject and return empty.
        prop_assert!(rd.is_empty(),
            "base64Decode(\"AB=C\") regression: bug #22 fix slipped — runtime \
             now decodes the mid-padding chunk to non-empty bytes. Expected \
             [], got {:?}", rd);
        // Cross-check the reference oracle still rejects this.
        use base64::Engine;
        let oracle = base64::engine::general_purpose::STANDARD.decode("AB=C");
        prop_assert!(oracle.is_err(),
            "base64 crate must reject `AB=C`; got Ok({:?})", oracle.ok());
    }
}

// ============================================================================
// base58 — currently UNIMPLEMENTED in the runtime
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// `base58Encode` is NOT implemented in `invoke_native_stdlib` — it falls
    /// through to `_ => StackItem::Null`, which serialises to an empty
    /// `return_data` byte vector at RET. This test pins that documented
    /// no-op contract. When the runtime gains a real base58 implementation,
    /// this assertion will start failing and the harness can be flipped to
    /// the differential check (the `bs58` crate is already in dev-deps and
    /// the expected output is computed below as a staging value for the
    /// flip).
    ///
    /// Captures: any panic, fault, or wrong-shape return on this code path.
    #[test]
    fn stdlib_base58_encode_currently_returns_empty_no_panic(
        bytes in prop::collection::vec(any::<u8>(), 0..=64),
    ) {
        let (ok, rd, exc) = call_stdlib("base58encode", std::slice::from_ref(&bytes));
        prop_assert!(ok,
            "base58Encode(len={}) must NOT fault even when unimplemented; exc={:?}",
            bytes.len(), exc);
        // STAGED — when implemented, swap to:
        //   let expected = bs58::encode(&bytes).into_string();
        //   let got = std::str::from_utf8(&rd).unwrap_or("");
        //   prop_assert_eq!(got, expected, ...);
        let _staged_expected = bs58::encode(&bytes).into_string();
        prop_assert!(rd.is_empty(),
            "base58Encode is currently unimplemented (stdlib.rs::invoke_native_stdlib \
             default arm). Contract: returns empty Null. If this fires with a \
             non-empty rd, the native landed — flip the assertion to the staged \
             differential. rd={:?}, expected-when-implemented={:?}",
            rd, _staged_expected);
    }

    /// Same shape for `base58Decode` — pins the unimplemented contract.
    #[test]
    fn stdlib_base58_decode_currently_returns_empty_no_panic(
        bytes in prop::collection::vec(any::<u8>(), 0..=64),
    ) {
        // We feed it the bs58-crate encoding so when the runtime IS wired
        // up, the round-trip will hold (and this pin will need flipping).
        let encoded = bs58::encode(&bytes).into_string();
        prop_assume!(encoded.len() <= 255);
        let (ok, rd, exc) = call_stdlib("base58decode", &[encoded.as_bytes().to_vec()]);
        prop_assert!(ok,
            "base58Decode must NOT fault even when unimplemented; exc={:?}", exc);
        prop_assert!(rd.is_empty(),
            "base58Decode is currently unimplemented; got non-empty rd={:?} \
             — flip to differential: assert_eq!(rd, original_bytes={:?})",
            rd, bytes);
    }
}

// ============================================================================
// memoryCompare / memoryIndexOf — also unimplemented
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// `memoryCompare(left, right)` is whitelisted by the IR resolver
    /// (resolve.rs::resolve_stdlib_member) but NOT implemented in the native
    /// dispatcher — falls through to Null → empty return_data. Pin the
    /// no-panic contract; when implemented, assertion flips to compare with
    /// `left.cmp(&right)` clamped to {-1, 0, 1}.
    #[test]
    fn stdlib_memorycompare_currently_returns_empty_no_panic(
        left in prop::collection::vec(any::<u8>(), 0..=64),
        right in prop::collection::vec(any::<u8>(), 0..=64),
    ) {
        let (ok, rd, exc) = call_stdlib("memorycompare", &[left.clone(), right.clone()]);
        prop_assert!(ok,
            "memoryCompare must NOT fault even when unimplemented; exc={:?}", exc);
        // Staged reference for the eventual flip:
        let _staged: i32 = match left.cmp(&right) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        };
        prop_assert!(rd.is_empty(),
            "memoryCompare is unimplemented; got rd={:?} (staged expected={})",
            rd, _staged);
    }

    /// `memorySearch(haystack, needle)` — IR resolver name is `memorySearch`
    /// (NOT `memoryIndexOf`; the latter does not appear in the IR
    /// whitelist). The handler is unimplemented; pin no-panic + empty.
    /// When implemented, expected: index of first occurrence as i64-LE, or
    /// -1 (encoded as 8-byte LE 0xff..ff) on miss.
    #[test]
    fn stdlib_memorysearch_currently_returns_empty_no_panic(
        haystack in prop::collection::vec(any::<u8>(), 0..=64),
        needle in prop::collection::vec(any::<u8>(), 0..=8),
    ) {
        let (ok, rd, exc) = call_stdlib("memorysearch", &[haystack.clone(), needle.clone()]);
        prop_assert!(ok,
            "memorySearch must NOT fault even when unimplemented; exc={:?}", exc);
        // Staged reference for the eventual flip:
        let _staged: i64 = if needle.is_empty() {
            0
        } else {
            haystack.windows(needle.len())
                .position(|w| w == needle.as_slice())
                .map(|i| i as i64)
                .unwrap_or(-1)
        };
        prop_assert!(rd.is_empty(),
            "memorySearch is unimplemented; got rd={:?} (staged expected={})",
            rd, _staged);
    }
}

// ============================================================================
// serialize / deserialize / jsonSerialize / jsonDeserialize
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// `deserialize(serialize(x))` round-trip for the SCALAR shape that
    /// PUSHDATA1 can land on the stack — a `ByteArray`.
    ///
    /// S1 fix: `serialize` now emits the Neo N3 BinarySerializer wire format
    /// (type-tagged little-endian), NOT JSON. For a ByteArray the wire form is
    /// `[0x00, varint(len), bytes...]`, so for len ≤ 128 the encoded size is
    /// ≤ 130 bytes — always within the PUSHDATA1 budget, and the deserialise
    /// leg is exercised unconditionally.
    #[test]
    fn stdlib_serialize_deserialize_bytearray_roundtrip(
        bytes in prop::collection::vec(any::<u8>(), 0..=128),
    ) {
        let (ok_s, rd_s, exc_s) = call_stdlib("serialize", std::slice::from_ref(&bytes));
        prop_assert!(ok_s, "serialize must succeed; exc={:?}", exc_s);

        // Wire format: [0x00 (ByteArray tag), varint(len), bytes...].
        // The empty case still emits the tag + zero-length varint.
        prop_assert!(!rd_s.is_empty(),
            "serialize of ByteArray must produce non-empty binary; bytes.len={}",
            bytes.len());
        prop_assert_eq!(
            rd_s.first(), Some(&0x00),
            "serialize(ByteArray) must start with the 0x00 ByteArray tag; got {:?}",
            rd_s
        );
        prop_assert!(
            !rd_s.starts_with(b"{") && !rd_s.starts_with(b"["),
            "serialize must NOT emit JSON (S1 regression); got {:?}",
            std::str::from_utf8(&rd_s).ok()
        );

        // PUSHDATA1 caps the deserialise-leg arg at 255 bytes; the binary
        // encoding is tag(1) + varint(≤2) + bytes(≤128) so we are always safe,
        // but guard explicitly against a future size change.
        prop_assume!(rd_s.len() <= 255);

        // Deserialise back — passes the binary bytes as the input arg.
        let (ok_d, rd_d, exc_d) = call_stdlib("deserialize", std::slice::from_ref(&rd_s));
        prop_assert!(ok_d, "deserialize must succeed; exc={:?}", exc_d);

        // The round-tripped StackItem renders to the same byte payload at
        // RET. ByteArray → ByteArray preserves bytes verbatim.
        prop_assert_eq!(&rd_d, &bytes,
            "deserialize(serialize(b)) must equal b; got={:?}, want={:?}, bin={:?}",
            rd_d, bytes, rd_s);
    }

    /// Same shape for jsonSerialize/jsonDeserialize. The JSON-flavoured pair
    /// uses a more compact representation than the binary `serialize`
    /// (bytes-as-base64 vs full {"type","value"} envelope), so the size
    /// budget is friendlier; we still bail when the encoded form overflows
    /// PUSHDATA1.
    #[test]
    fn stdlib_jsonserialize_jsondeserialize_bytearray_roundtrip(
        bytes in prop::collection::vec(any::<u8>(), 0..=96),
    ) {
        let (ok_s, rd_s, exc_s) = call_stdlib("jsonserialize", std::slice::from_ref(&bytes));
        prop_assert!(ok_s, "jsonSerialize must succeed; exc={:?}", exc_s);
        prop_assert!(!rd_s.is_empty(),
            "jsonSerialize of ByteArray must produce non-empty JSON");

        prop_assume!(rd_s.len() <= 255);

        let (ok_d, rd_d, exc_d) = call_stdlib("jsondeserialize", std::slice::from_ref(&rd_s));
        prop_assert!(ok_d, "jsonDeserialize must succeed; exc={:?}", exc_d);
        prop_assert_eq!(&rd_d, &bytes,
            "jsonDeserialize(jsonSerialize(b)) must equal b; got={:?}, want={:?}, json={:?}",
            rd_d, bytes,
            std::str::from_utf8(&rd_s).ok());
    }

    /// S1 fix: `deserialize` now parses the Neo N3 BinarySerializer wire
    /// format, so many short byte streams ARE valid (any input starting with
    /// 0x00..0x03 / 0x40 / 0x80 followed by a well-formed payload). This test
    /// focuses on inputs that are DEFINITELY malformed for the binary format
    /// — a tag byte whose declared payload is truncated — and asserts the
    /// graceful Null fallback (stdlib.rs `unwrap_or(StackItem::Null)`).
    /// Critically: must NOT panic.
    #[test]
    fn stdlib_deserialize_malformed_returns_null_no_panic(
        extra in prop::collection::vec(any::<u8>(), 0..=32),
    ) {
        // Build a definitely-truncated input: ByteArray tag (0x00) claiming a
        // 255-byte payload, followed by only `extra.len()` bytes. The decoder
        // must hit the length check, return None, and the handler falls back
        // to Null.
        let mut garbage = vec![0x00, 0xFF];
        garbage.extend_from_slice(&extra);

        let (ok, rd, exc) = call_stdlib("deserialize", std::slice::from_ref(&garbage));
        prop_assert!(ok,
            "deserialize of malformed bytes must NOT fault at host level; \
             input={:?}, exc={:?}", garbage, exc);
        prop_assert!(rd.is_empty(),
            "deserialize of malformed (truncated-payload) binary must return \
             empty (Null); input={:?}, got={:?}",
            garbage, rd);
    }

    /// Same for jsonDeserialize — non-UTF8 bytes are coerced to empty string
    /// (stdlib.rs:360 — `String::from_utf8(...).unwrap_or_default()`); empty
    /// string is rejected by `serde_json::from_str`, which falls through to
    /// `unwrap_or(StackItem::Null)`. Non-UTF8 input must NOT panic.
    #[test]
    fn stdlib_jsondeserialize_malformed_returns_null_no_panic(
        garbage in prop_oneof![
            // High-bit-set bytes: invalid UTF-8 unless they form a valid
            // continuation, which is exceedingly unlikely for random bytes.
            prop::collection::vec(128u8..=255u8, 1..=64),
            // Random ASCII that's not a JSON object.
            prop::collection::vec(any::<u8>().prop_filter("not a JSON object byte", |b| *b != b'{'), 1..=64),
        ],
    ) {
        let (ok, rd, exc) = call_stdlib("jsondeserialize", std::slice::from_ref(&garbage));
        prop_assert!(ok,
            "jsonDeserialize of malformed bytes must NOT fault; \
             input.len={}, exc={:?}", garbage.len(), exc);
        prop_assert!(rd.is_empty(),
            "jsonDeserialize of malformed bytes must return empty (Null); \
             input.len={}, got={:?}", garbage.len(), rd);
    }
}

/// A clearly-malformed binary tag (0x05 is not a valid StackItem type in the
/// Neo BinarySerializer format the runtime models) must fall back to Null
/// without panicking. (Outside the proptest! block because it's a fixed-input
/// regression guard, not a property test.)
#[test]
fn stdlib_deserialize_unknown_tag_falls_back_to_null() {
    let (ok, rd, _exc) = call_stdlib("deserialize", &[vec![0x05, 0xAA, 0xBB]]);
    assert!(ok, "unknown-tag deserialize must not fault");
    // Unknown tags are decoded defensively as Null (see neo_binary_deserialize_from).
    // Null renders to empty return_data at RET.
    assert!(
        rd.is_empty(),
        "unknown-tag deserialize must yield Null (empty return_data); got {:?}",
        rd
    );
}
