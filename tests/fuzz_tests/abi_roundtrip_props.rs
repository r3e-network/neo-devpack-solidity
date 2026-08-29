//! ABI codec round-trip property tests.
//!
//! For any Solidity value `x` of a supported type `T`, the codec invariant
//! `abi.decode(abi.encode(x), (T)) == x` MUST hold. A divergence here
//! would mean real on-chain calldata / return-data could be mis-decoded —
//! the strongest correctness property we can place on the ABI codec.
//!
//! Strategy: each test compiles a tiny Solidity contract that:
//!   1. binds a value `x` of type `T` (as a baked-in Solidity literal —
//!      this avoids entanglement with the *return*-side encoding shape,
//!      which is itself non-canonical for scalar single returns; see
//!      `differential.rs::differential_abi_encode_uint256_single`),
//!   2. computes `bytes memory enc = abi.encode(x);`,
//!   3. decodes `T memory y = abi.decode(enc, (T));`,
//!   4. compares `y == x` (or, for dynamic types, `keccak256(y) ==
//!      keccak256(x)`) and returns a single `bool`.
//!
//! The single-`bool` return is decoded out of `r.return_data`: an empty
//! / all-zero return means `false`; any non-zero byte means `true`. (See
//! the Boolean encoding rules described in `native_contract_props.rs ::
//! native_policy_is_blocked_returns_false_for_unknown`.)
//!
//! Coverage: uintN (N=8/16/32/64/128/256), intN (signed, INT_MIN/MAX
//! covered), bool, address, bytes1..bytes32, string (0..=128 ASCII),
//! bytes (0..=128 random), tuples `(uint256, address)` and
//! `(uint256, string, bool)`, dynamic arrays `uint256[]` /
//! `address[]` (length 0..=8 each).
//!
//! Failure-mode mapping (per the task brief):
//!   * Sign-extension on negative ints → `roundtrip_intN` for narrow
//!     signed ints (`int8` … `int128`). **Surfaced.** See `#[ignore]`
//!     blocks below.
//!   * Length-prefix vs offset-prefix confusion for dynamic types →
//!     `roundtrip_string` / `roundtrip_bytes` / array tests. **Surfaced.**
//!     See `#[ignore]` blocks below.
//!   * Pad-to-32 vs left-align for bytesN → `roundtrip_bytesN`. PASSES.
//!   * Endianness for address/uintN → `roundtrip_uintN` /
//!     `roundtrip_address`. PASSES.
//!
//! See also:
//!   * `differential.rs::differential_abi_encode_uint256_shape` (existing
//!     shape pin for the EVM-canonical 32-byte BE slot),
//!   * `src/cli/bytecode/bytecode_builtins/builtin_call/abi.rs` (the
//!     compiler-side `abi.encode` / `abi.decode` lowering).

#![allow(unused_imports)]

use super::common::*;
use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::runtime::types::StackItem;
use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};
use proptest::prelude::*;

/// Compile a tiny `check()` contract and return the parsed boolean result.
///
/// The contract MUST expose a single zero-arg method `check()` returning a
/// `bool`. The boolean is decoded from `r.return_data` per the runtime's
/// minimum-width LE convention: empty / all-zero bytes → `false`; any
/// non-zero byte → `true`. (See `decode_uint_le` in `common.rs` and the
/// Boolean encoding rules described in `native_contract_props.rs ::
/// native_policy_is_blocked_returns_false_for_unknown`.)
fn run_check_returns_bool(source: &str) -> Result<bool, String> {
    let arts =
        compile_contracts(source, false, 2).map_err(|e| format!("compile failed: {:?}", e))?;
    let art = arts.first().ok_or_else(|| "no artifact".to_string())?;
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).map_err(|e| format!("rt: {:?}", e))?;
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "check", &[])
        .map_err(|e| format!("host err: {:?}", e))?;
    if !r.success {
        return Err(format!(
            "execution faulted: {:?}; rd_hex={}",
            r.exception.as_ref().map(|e| &e.message),
            hex::encode(&r.return_data)
        ));
    }
    // Boolean ABI encoding: any non-zero byte in return_data ⇒ true.
    let any_truthy = r.return_data.iter().any(|b| *b != 0);
    Ok(any_truthy)
}

// ---------- helpers for emitting Solidity literals ----------

/// Render a Solidity hex literal of the form `hex"AABB.."` for raw bytes.
fn solidity_hex_literal(bytes: &[u8]) -> String {
    format!("hex\"{}\"", hex::encode(bytes))
}

/// Render a Solidity 0x-prefixed literal of length `2*N` for `bytesN`.
fn solidity_bytes_n_literal(bytes: &[u8]) -> String {
    format!("bytes{}(0x{})", bytes.len(), hex::encode(bytes))
}

/// Render a Solidity `address(0x...)` 20-byte literal.
fn solidity_address_literal(addr_bytes: &[u8; 20]) -> String {
    format!("address(0x{})", hex::encode(addr_bytes))
}

/// Quote an ASCII string for embedding inside a Solidity `"..."` literal.
/// Backslashes and double-quotes are escaped; non-printable bytes are
/// rejected (the proptest strategy never generates them).
fn solidity_string_literal(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            c if c.is_ascii() && !c.is_ascii_control() => out.push(c),
            // The ASCII-only strategy guarantees this branch is dead.
            _ => out.push('?'),
        }
    }
    out.push('"');
    out
}

// ==================== Scalar uintN round-trips ====================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// `abi.decode(abi.encode(uint8 x), (uint8)) == x` for every u8.
    /// Catches pad-to-32 / sign-extension confusion on narrow uints.
    /// PASSES — uintN is right-padded to 32 bytes correctly across the
    /// full range, and the decode reads the low byte back.
    #[test]
    fn roundtrip_uint8(x in any::<u8>()) {
        let src = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Codec {{
    function check() external pure returns (bool) {{
        uint8 x = uint8({x});
        bytes memory enc = abi.encode(x);
        uint8 y = abi.decode(enc, (uint8));
        return y == x;
    }}
}}"#);
        let ok = run_check_returns_bool(&src)
            .unwrap_or_else(|e| panic!("uint8 round-trip x={}: {}", x, e));
        prop_assert!(ok, "uint8 round-trip failed for x={}", x);
    }

    /// `uint16` round-trip — full 0..=65535 range.
    #[test]
    fn roundtrip_uint16(x in any::<u16>()) {
        let src = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Codec {{
    function check() external pure returns (bool) {{
        uint16 x = uint16({x});
        bytes memory enc = abi.encode(x);
        uint16 y = abi.decode(enc, (uint16));
        return y == x;
    }}
}}"#);
        let ok = run_check_returns_bool(&src)
            .unwrap_or_else(|e| panic!("uint16 round-trip x={}: {}", x, e));
        prop_assert!(ok, "uint16 round-trip failed for x={}", x);
    }

    /// `uint32` round-trip — full 0..=u32::MAX range.
    #[test]
    fn roundtrip_uint32(x in any::<u32>()) {
        let src = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Codec {{
    function check() external pure returns (bool) {{
        uint32 x = uint32({x});
        bytes memory enc = abi.encode(x);
        uint32 y = abi.decode(enc, (uint32));
        return y == x;
    }}
}}"#);
        let ok = run_check_returns_bool(&src)
            .unwrap_or_else(|e| panic!("uint32 round-trip x={}: {}", x, e));
        prop_assert!(ok, "uint32 round-trip failed for x={}", x);
    }

    /// `uint64` round-trip — full 0..=u64::MAX range.
    #[test]
    fn roundtrip_uint64(x in any::<u64>()) {
        let src = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Codec {{
    function check() external pure returns (bool) {{
        uint64 x = uint64({x});
        bytes memory enc = abi.encode(x);
        uint64 y = abi.decode(enc, (uint64));
        return y == x;
    }}
}}"#);
        let ok = run_check_returns_bool(&src)
            .unwrap_or_else(|e| panic!("uint64 round-trip x={}: {}", x, e));
        prop_assert!(ok, "uint64 round-trip failed for x={}", x);
    }

    /// `uint128` round-trip. Built from a u128 literal so values above
    /// u64::MAX get exercised too.
    #[test]
    fn roundtrip_uint128(x in any::<u128>()) {
        let src = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Codec {{
    function check() external pure returns (bool) {{
        uint128 x = uint128({x});
        bytes memory enc = abi.encode(x);
        uint128 y = abi.decode(enc, (uint128));
        return y == x;
    }}
}}"#);
        let ok = run_check_returns_bool(&src)
            .unwrap_or_else(|e| panic!("uint128 round-trip x={}: {}", x, e));
        prop_assert!(ok, "uint128 round-trip failed for x={}", x);
    }

    /// `uint256` round-trip — synthesised as `(hi << 128) | lo` from two
    /// u128 halves so the upper 128 bits are exercised. Catches BE/LE
    /// confusion on the upper word.
    #[test]
    fn roundtrip_uint256(hi in any::<u128>(), lo in any::<u128>()) {
        let src = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Codec {{
    function check() external pure returns (bool) {{
        uint256 x = (uint256({hi}) << 128) | uint256({lo});
        bytes memory enc = abi.encode(x);
        uint256 y = abi.decode(enc, (uint256));
        return y == x;
    }}
}}"#);
        let ok = run_check_returns_bool(&src)
            .unwrap_or_else(|e| panic!("uint256 round-trip hi={} lo={}: {}", hi, lo, e));
        prop_assert!(ok, "uint256 round-trip failed for hi={} lo={}", hi, lo);
    }
}

// ==================== Scalar intN (signed) round-trips ====================
//
// **GAP — narrow signed-int sign-extension on `abi.encode`**.
//
// Observation: for `int8 x = int8(-1);`, calling `abi.encode(x)` produces
// the 32-byte payload
//   `0000000000000000000000000000000000000000000000000000000000000000` ||
//   `ffffffffffffffff` (32B total)
// — i.e. 24 bytes of zero followed by 8 bytes of `0xff`. The EVM-canonical
// encoding for `int8(-1)` is 32 bytes of `0xff`: the sign bit must be
// replicated to the entire 256-bit slot. The current encoder only
// sign-extends to 64 bits (the i64 width of the in-runtime stack item),
// then right-pads with zeros — wrong direction AND wrong width.
//
// Symptom on the wire: a contract that emits an `int8` (or any narrow
// signed int) negative value via `abi.encode` will produce a payload
// that decodes correctly on Solidity 0.8 / Foundry / ethers.js for
// values in [-128, -1] only if the consumer truncates to 8 bits — the
// standard ABI decoders DO NOT, they sign-extend the FULL 32-byte slot
// and would read `0x00...00ffffffffffffffff` as the unsigned integer
// `0xffffffffffffffff` = 18_446_744_073_709_551_615, NOT as `-1`.
//
// Affected types: `int8`, `int16`, `int32`, `int64`, `int128`. The
// `int256` form is NOT affected because the sign-extension width
// (256) coincides with the slot width, so even the current truncated
// path produces the right bytes.
//
// Tests below are `#[ignore]`d with a `// REAL BUG` comment until the
// encoder is fixed in `src/cli/bytecode/bytecode_builtins/builtin_call/
// abi.rs`. Re-enable by removing the `#[ignore]` once the fix lands.

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// `int8` round-trip — full -128..=127 range, including INT_MIN and -1.
    ///
    /// REAL BUG (sign-extension on `abi.encode` for narrow signed ints).
    /// Minimal failing input observed during initial exec: `x = -1`.
    /// Encoded bytes: `00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
    /// 00 00 00 00 00 00 00 00  ff ff ff ff ff ff ff ff` — the high
    /// 24 bytes should be `ff` for canonical sign-extension. See module
    /// docstring for diagnosis.
    #[test]
    fn roundtrip_int8(x in any::<i8>()) {
        let src = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Codec {{
    function check() external pure returns (bool) {{
        int8 x = int8({x});
        bytes memory enc = abi.encode(x);
        int8 y = abi.decode(enc, (int8));
        return y == x;
    }}
}}"#);
        let ok = run_check_returns_bool(&src)
            .unwrap_or_else(|e| panic!("int8 round-trip x={}: {}", x, e));
        prop_assert!(ok, "int8 round-trip failed for x={}", x);
    }

    /// `int16` round-trip — INT_MIN/INT_MAX corner-cases included.
    /// REAL BUG: same sign-extension gap as `int8`.
    #[test]
    fn roundtrip_int16(x in any::<i16>()) {
        let src = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Codec {{
    function check() external pure returns (bool) {{
        int16 x = int16({x});
        bytes memory enc = abi.encode(x);
        int16 y = abi.decode(enc, (int16));
        return y == x;
    }}
}}"#);
        let ok = run_check_returns_bool(&src)
            .unwrap_or_else(|e| panic!("int16 round-trip x={}: {}", x, e));
        prop_assert!(ok, "int16 round-trip failed for x={}", x);
    }

    /// `int32` round-trip. REAL BUG — same sign-extension gap.
    #[test]
    fn roundtrip_int32(x in any::<i32>()) {
        let src = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Codec {{
    function check() external pure returns (bool) {{
        int32 x = int32({x});
        bytes memory enc = abi.encode(x);
        int32 y = abi.decode(enc, (int32));
        return y == x;
    }}
}}"#);
        let ok = run_check_returns_bool(&src)
            .unwrap_or_else(|e| panic!("int32 round-trip x={}: {}", x, e));
        prop_assert!(ok, "int32 round-trip failed for x={}", x);
    }

    /// `int64` round-trip. REAL BUG — same sign-extension gap.
    #[test]
    fn roundtrip_int64(x in any::<i64>()) {
        let src = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Codec {{
    function check() external pure returns (bool) {{
        int64 x = int64({x});
        bytes memory enc = abi.encode(x);
        int64 y = abi.decode(enc, (int64));
        return y == x;
    }}
}}"#);
        let ok = run_check_returns_bool(&src)
            .unwrap_or_else(|e| panic!("int64 round-trip x={}: {}", x, e));
        prop_assert!(ok, "int64 round-trip failed for x={}", x);
    }

    /// `int128` round-trip — including `type(int128).min` and `.max`
    /// where the sign bit is the most-significant of a 16-byte half-slot.
    /// REAL BUG — same sign-extension gap.
    #[test]
    fn roundtrip_int128(x in any::<i128>()) {
        let src = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Codec {{
    function check() external pure returns (bool) {{
        int128 x = int128({x});
        bytes memory enc = abi.encode(x);
        int128 y = abi.decode(enc, (int128));
        return y == x;
    }}
}}"#);
        let ok = run_check_returns_bool(&src)
            .unwrap_or_else(|e| panic!("int128 round-trip x={}: {}", x, e));
        prop_assert!(ok, "int128 round-trip failed for x={}", x);
    }

    /// `int256` round-trip — synthesised from two u128 halves so the full
    /// signed-256 range gets exercised, including `type(int256).min`
    /// (= 1 << 255) when both halves are at their extremes. Special-cased
    /// constants `0`, `-1`, `type(int256).min`, `type(int256).max` plus
    /// random pairings.
    ///
    /// PASSES — the full-width int256 path encodes/decodes correctly
    /// (the slot width matches the type width, so the sign-extension gap
    /// in narrow ints is invisible here).
    #[test]
    fn roundtrip_int256(
        kind in prop_oneof![
            Just("zero"),
            Just("neg_one"),
            Just("min"),
            Just("max"),
            Just("rand"),
        ],
        rand_lo in any::<u128>(),
        rand_hi in any::<u128>(),
    ) {
        let literal = match kind {
            "zero"    => "int256(0)".to_string(),
            "neg_one" => "int256(-1)".to_string(),
            "min"     => "type(int256).min".to_string(),
            "max"     => "type(int256).max".to_string(),
            // Random: build a 256-bit unsigned, then reinterpret as int256.
            _ => format!(
                "int256(uint256((uint256({hi}) << 128) | uint256({lo})))",
                hi = rand_hi, lo = rand_lo
            ),
        };
        let src = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Codec {{
    function check() external pure returns (bool) {{
        int256 x = {literal};
        bytes memory enc = abi.encode(x);
        int256 y = abi.decode(enc, (int256));
        return y == x;
    }}
}}"#);
        let ok = run_check_returns_bool(&src)
            .unwrap_or_else(|e| panic!("int256 round-trip kind={}: {}", kind, e));
        prop_assert!(ok, "int256 round-trip failed for kind={}", kind);
    }
}

// ==================== bool / address ====================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// `bool` round-trip — both `true` and `false` exercised.
    #[test]
    fn roundtrip_bool(x in any::<bool>()) {
        let lit = if x { "true" } else { "false" };
        let src = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Codec {{
    function check() external pure returns (bool) {{
        bool x = {lit};
        bytes memory enc = abi.encode(x);
        bool y = abi.decode(enc, (bool));
        return y == x;
    }}
}}"#);
        let ok = run_check_returns_bool(&src)
            .unwrap_or_else(|e| panic!("bool round-trip x={}: {}", x, e));
        prop_assert!(ok, "bool round-trip failed for x={}", x);
    }

    /// `address` round-trip — exercises the 20-byte UInt160 encoding.
    /// Catches Neo-LE → EVM-BE mismatches that would corrupt a `msg.sender`
    /// or any address-typed cross-contract argument.
    #[test]
    fn roundtrip_address(addr in prop::array::uniform20(any::<u8>())) {
        let lit = solidity_address_literal(&addr);
        let src = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Codec {{
    function check() external pure returns (bool) {{
        address x = {lit};
        bytes memory enc = abi.encode(x);
        address y = abi.decode(enc, (address));
        return y == x;
    }}
}}"#);
        let ok = run_check_returns_bool(&src)
            .unwrap_or_else(|e| panic!("address round-trip {:?}: {}", addr, e));
        prop_assert!(ok, "address round-trip failed for addr={:?}", addr);
    }
}

// ==================== bytesN (fixed-width) ====================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// `bytesN` round-trip for every N in 1..=32. Catches pad-direction
    /// bugs (`bytesN` is left-aligned in its 32-byte slot, vs uintN which
    /// is right-aligned). A regression that left-aligns uints or
    /// right-aligns bytesN would fail this OR `roundtrip_uintN`.
    #[test]
    fn roundtrip_bytes_n(
        n in 1usize..=32,
        seed in any::<[u8; 32]>(),
    ) {
        // Take the first n bytes of the 32-byte seed.
        let bytes = &seed[..n];
        let lit = solidity_bytes_n_literal(bytes);
        let src = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Codec {{
    function check() external pure returns (bool) {{
        bytes{n} x = {lit};
        bytes memory enc = abi.encode(x);
        bytes{n} y = abi.decode(enc, (bytes{n}));
        return y == x;
    }}
}}"#);
        let ok = run_check_returns_bool(&src)
            .unwrap_or_else(|e| panic!("bytes{} round-trip {}: {}", n, hex::encode(bytes), e));
        prop_assert!(ok, "bytes{} round-trip failed for value=0x{}",
                     n, hex::encode(bytes));
    }
}

// ==================== string / bytes (dynamic) ====================
//
// **GAP — `abi.decode` for dynamic types is a no-op pass-through**.
//
// Observation: for `string memory x = "abc"; bytes memory enc =
// abi.encode(x); string memory y = abi.decode(enc, (string));`, the
// returned `y` is byte-for-byte identical to `enc` — that is, the
// 96-byte payload `[offset=0x20][len=3]["abc" + 29B pad]`. The decoder
// is NOT walking the offset → length → data layout; it is returning the
// encoded blob unchanged.
//
// Symptom: any subsequent operation on the "decoded" string will treat
// 96 bytes of ABI scaffolding as the string content. `bytes(y).length`
// surfaces as `Execution failed: SIZE: unsupported type` because the
// underlying value isn't a stack item that supports `SIZE` — it's a
// nested struct-shaped value the decoder produced for the `bytes memory`
// it didn't actually decode. The keccak comparison fails because the
// original `bytes("abc")` has length 3 while the "decoded" value has
// length 96 (or fails outright).
//
// Affected types: `string`, `bytes`, dynamic arrays (`uint256[]`,
// `address[]`), and any tuple containing one of those. See
// `src/cli/bytecode/bytecode_builtins/builtin_call/abi.rs::lower_decode`
// (or the corresponding decoder entry for dynamic-tail layouts).
//
// Tests below are `#[ignore]`d with a `// REAL BUG` comment.

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    /// `string` round-trip for ASCII strings of length 0..=128.
    ///
    /// REAL BUG: `abi.decode` for `string` returns the encoded bytes
    /// verbatim (96-byte head+pad), not the original string content.
    /// Catches: length-prefix vs offset-prefix walk in the dynamic-tail
    /// decoder. Empty strings might pass by coincidence (no payload).
    #[test]
    fn roundtrip_string(s in "[a-zA-Z0-9 _.,:;!?+\\-*/=()@#$%&]{0,128}") {
        let lit = solidity_string_literal(&s);
        let src = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Codec {{
    function check() external pure returns (bool) {{
        string memory x = {lit};
        bytes memory enc = abi.encode(x);
        string memory y = abi.decode(enc, (string));
        // String comparison via keccak256 of the underlying bytes —
        // direct `==` on `string memory` is not supported in Solidity 0.8.
        return keccak256(bytes(y)) == keccak256(bytes(x));
    }}
}}"#);
        let ok = run_check_returns_bool(&src)
            .unwrap_or_else(|e| panic!("string round-trip s={:?}: {}", s, e));
        prop_assert!(ok, "string round-trip failed for s={:?}", s);
    }

    /// `bytes` round-trip for arbitrary 0..=128-byte payloads.
    ///
    /// REAL BUG: same dynamic-decode no-op as `string` — the "decoded"
    /// `bytes memory` is the 96-byte encoded blob, not the original
    /// payload.
    #[test]
    fn roundtrip_bytes(b in prop::collection::vec(any::<u8>(), 0..=128)) {
        let lit = solidity_hex_literal(&b);
        let src = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Codec {{
    function check() external pure returns (bool) {{
        bytes memory x = {lit};
        bytes memory enc = abi.encode(x);
        bytes memory y = abi.decode(enc, (bytes));
        return keccak256(y) == keccak256(x);
    }}
}}"#);
        let ok = run_check_returns_bool(&src)
            .unwrap_or_else(|e| panic!("bytes round-trip {}B: {}", b.len(), e));
        prop_assert!(ok, "bytes round-trip failed for {}B (hex={})",
                     b.len(), hex::encode(&b));
    }
}

// ==================== Tuples (head/tail split) ====================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    /// Tuple `(uint256, address)` — both static-head, no tail. Pins the
    /// simplest static-tuple encoding. PASSES.
    #[test]
    fn roundtrip_tuple_uint_addr(
        v_lo in any::<u128>(),
        v_hi in any::<u128>(),
        addr in prop::array::uniform20(any::<u8>()),
    ) {
        let addr_lit = solidity_address_literal(&addr);
        let src = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Codec {{
    function check() external pure returns (bool) {{
        uint256 a = (uint256({hi}) << 128) | uint256({lo});
        address b = {addr};
        bytes memory enc = abi.encode(a, b);
        (uint256 a2, address b2) = abi.decode(enc, (uint256, address));
        return a2 == a && b2 == b;
    }}
}}"#, hi = v_hi, lo = v_lo, addr = addr_lit);
        let ok = run_check_returns_bool(&src)
            .unwrap_or_else(|e| panic!("(uint256,address) tuple round-trip: {}", e));
        prop_assert!(ok, "(uint256,address) tuple round-trip failed");
    }

    /// Tuple `(uint256, string, bool)` — exercises the head/tail split:
    /// the static `uint256` and `bool` heads sit in the first 96 bytes
    /// while the dynamic `string` lives in the tail with its offset
    /// stored in the head. The decoder must walk the offset correctly.
    ///
    /// REAL BUG (downstream of the dynamic-decode no-op): the decoded
    /// `string` is the encoded blob, then `bytes(b2)` triggers the
    /// `SIZE: unsupported type` fault. Same root cause as
    /// `roundtrip_string`.
    #[test]
    fn roundtrip_tuple_uint_string_bool(
        v in any::<u64>(),
        s in "[a-zA-Z0-9 _.]{0,64}",
        flag in any::<bool>(),
    ) {
        let s_lit = solidity_string_literal(&s);
        let flag_lit = if flag { "true" } else { "false" };
        let src = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Codec {{
    function check() external pure returns (bool) {{
        uint256 a = uint256({v});
        string memory b = {s_lit};
        bool c = {flag_lit};
        bytes memory enc = abi.encode(a, b, c);
        (uint256 a2, string memory b2, bool c2) =
            abi.decode(enc, (uint256, string, bool));
        return a2 == a
            && keccak256(bytes(b2)) == keccak256(bytes(b))
            && c2 == c;
    }}
}}"#);
        let ok = run_check_returns_bool(&src)
            .unwrap_or_else(|e| panic!(
                "(uint256,string,bool) tuple round-trip v={} s={:?} flag={}: {}",
                v, s, flag, e
            ));
        prop_assert!(ok,
            "(uint256,string,bool) tuple round-trip failed for v={} s={:?} flag={}",
            v, s, flag);
    }
}

// ==================== Dynamic arrays ====================
//
// **GAP — `abi.decode` for dynamic arrays not implemented**.
//
// Observation: for `uint256[] memory a = new uint256[](2); a[0]=1; a[1]=2;
// bytes memory enc = abi.encode(a); uint256[] memory b = abi.decode(enc,
// (uint256[]));`, the decode produces a value whose `.length` operation
// faults with `Execution failed: SIZE: unsupported type` — the decoder
// did not produce a valid array stack item.
//
// Affected types: `uint256[]`, `address[]`, and any other dynamic array
// type. Tests `#[ignore]`d with a `// REAL BUG` comment until the
// decoder lands array support in
// `src/cli/bytecode/bytecode_builtins/builtin_call/abi.rs`.

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    /// `uint256[]` round-trip for arrays of length 0..=8.
    ///
    /// REAL BUG: `abi.decode` for `uint256[]` does not produce a usable
    /// array — `.length` faults with "SIZE: unsupported type".
    #[test]
    fn roundtrip_uint256_array(
        elems in prop::collection::vec(any::<u64>(), 0..=8),
    ) {
        // Build the array via `new uint256[](n)` + per-element assignment.
        // Solidity's `[a,b,c]` array literal infers a fixed-width type; the
        // dynamic form here is what the wire format actually exercises.
        let n = elems.len();
        let mut assigns = String::new();
        for (i, v) in elems.iter().enumerate() {
            assigns.push_str(&format!("        a[{i}] = uint256({v});\n"));
        }
        let mut compare = String::from("        bool eq = a.length == b.length;\n");
        for i in 0..n {
            compare.push_str(&format!(
                "        eq = eq && a[{i}] == b[{i}];\n"
            ));
        }
        let src = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Codec {{
    function check() external pure returns (bool) {{
        uint256[] memory a = new uint256[]({n});
{assigns}        bytes memory enc = abi.encode(a);
        uint256[] memory b = abi.decode(enc, (uint256[]));
{compare}        return eq;
    }}
}}"#);
        let ok = run_check_returns_bool(&src)
            .unwrap_or_else(|e| panic!("uint256[] round-trip n={}: {}", n, e));
        prop_assert!(ok, "uint256[] round-trip failed for n={} elems={:?}", n, elems);
    }

    /// `address[]` round-trip — same dynamic-array layout, but each
    /// element is a 20-byte UInt160 (right-padded to 32 bytes in the
    /// ABI).
    ///
    /// REAL BUG: same as `uint256[]` — array decode unsupported.
    #[test]
    fn roundtrip_address_array(
        addrs in prop::collection::vec(prop::array::uniform20(any::<u8>()), 0..=8),
    ) {
        let n = addrs.len();
        let mut assigns = String::new();
        for (i, a) in addrs.iter().enumerate() {
            assigns.push_str(&format!(
                "        a[{i}] = {lit};\n",
                lit = solidity_address_literal(a)
            ));
        }
        let mut compare = String::from("        bool eq = a.length == b.length;\n");
        for i in 0..n {
            compare.push_str(&format!(
                "        eq = eq && a[{i}] == b[{i}];\n"
            ));
        }
        let src = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Codec {{
    function check() external pure returns (bool) {{
        address[] memory a = new address[]({n});
{assigns}        bytes memory enc = abi.encode(a);
        address[] memory b = abi.decode(enc, (address[]));
{compare}        return eq;
    }}
}}"#);
        let ok = run_check_returns_bool(&src)
            .unwrap_or_else(|e| panic!("address[] round-trip n={}: {}", n, e));
        prop_assert!(ok, "address[] round-trip failed for n={}", n);
    }
}

// ==================== Nested-dynamic arrays ====================
//
// `string[]`, `bytes[]`, `T[][]`: each element is itself dynamic, so the
// array tail is a RECURSIVE head/tail layout — a length word, then `n`
// 32-byte element offsets (relative to the start of the head section),
// then the `n` element tails. Implemented by
// `emit_abi_dynamic_nested_array_tail` (encode) and
// `emit_abi_decode_nested_array_tail_runtime` (decode) in
// `src/ir/expressions/calls/builtins/helpers.rs`. Previously these shapes
// fell back to `StdLib.serialize` (Neo-native, NOT EVM-ABI), so on-chain
// calldata / return-data for them was non-conformant.

/// Build the canonical EVM ABI encoding of `abi.encode(string[] arr)` in
/// Rust (independent of the compiler) so a decode test can pin the
/// compiler's decoder against a known-correct on-chain layout.
fn evm_encode_string_array(arr: &[&str]) -> Vec<u8> {
    fn slot_u(v: u64) -> [u8; 32] {
        let mut s = [0u8; 32];
        s[24..32].copy_from_slice(&v.to_be_bytes());
        s
    }
    fn elem_tail(s: &[u8]) -> Vec<u8> {
        let mut out = slot_u(s.len() as u64).to_vec();
        let padded = (s.len() + 31) / 32 * 32;
        let mut data = s.to_vec();
        data.resize(padded, 0);
        out.extend_from_slice(&data);
        out
    }
    // Array tail: length, head (n offsets), tail (n element encodings).
    let n = arr.len();
    let mut heads = Vec::new();
    let mut tails = Vec::new();
    let mut off = (n * 32) as u64;
    for s in arr {
        heads.extend_from_slice(&slot_u(off));
        let t = elem_tail(s.as_bytes());
        off += t.len() as u64;
        tails.extend_from_slice(&t);
    }
    let mut array_tail = slot_u(n as u64).to_vec();
    array_tail.extend_from_slice(&heads);
    array_tail.extend_from_slice(&tails);
    // Top-level `abi.encode(single dynamic)`: a head word holding the tail
    // offset (0x20), then the array tail.
    let mut out = slot_u(0x20).to_vec();
    out.extend_from_slice(&array_tail);
    out
}

/// CONFORMANCE: decode a HARDCODED, independently-built EVM `string[]`
/// encoding and verify the decoded elements. This pins the decoder to the
/// real on-chain ABI layout (not merely to the compiler's own encoder).
#[test]
fn decode_evm_string_array_conformance() {
    let enc = evm_encode_string_array(&["a", "bb", "ccc"]);
    // Sanity: 0x20 head + (len + 3 offsets + 3×(len+data)) tail.
    assert_eq!(enc.len(), 32 + 32 + 3 * 32 + 3 * 64);
    let hexbuf = hex::encode(&enc);
    let src = format!(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Codec {{
    function check() external pure returns (bool) {{
        bytes memory enc = hex"{hexbuf}";
        string[] memory b = abi.decode(enc, (string[]));
        bool eq = b.length == 3;
        eq = eq && keccak256(bytes(b[0])) == keccak256(bytes("a"));
        eq = eq && keccak256(bytes(b[1])) == keccak256(bytes("bb"));
        eq = eq && keccak256(bytes(b[2])) == keccak256(bytes("ccc"));
        return eq;
    }}
}}"#
    );
    let ok = run_check_returns_bool(&src)
        .unwrap_or_else(|e| panic!("EVM string[] decode conformance: {}", e));
    assert!(
        ok,
        "decoded string[] did not match the canonical EVM payload"
    );
}

/// CONFORMANCE: the compiler's ENCODER must reproduce the canonical EVM
/// `string[]` bytes. The contract encodes `["a","bb","ccc"]` and compares
/// `keccak256(enc)` against the hash of the independently-built reference
/// — a byte-exact equality check that fails on any layout divergence.
#[test]
fn encode_evm_string_array_conformance() {
    use sha3::{Digest, Keccak256};
    let expected = evm_encode_string_array(&["a", "bb", "ccc"]);
    let expected_hash = hex::encode(Keccak256::digest(&expected));
    let src = format!(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Codec {{
    function check() external pure returns (bool) {{
        string[] memory a = new string[](3);
        a[0] = "a";
        a[1] = "bb";
        a[2] = "ccc";
        bytes memory enc = abi.encode(a);
        return keccak256(enc) == hex"{expected_hash}";
    }}
}}"#
    );
    let ok = run_check_returns_bool(&src)
        .unwrap_or_else(|e| panic!("EVM string[] encode conformance: {}", e));
    assert!(
        ok,
        "encoded string[] bytes diverge from the canonical EVM payload"
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    /// `string[]` round-trip for arrays of length 0..=6, elements 0..=40
    /// chars. Exercises the recursive encode→decode path end to end.
    #[test]
    fn roundtrip_string_array(
        strs in prop::collection::vec("[a-zA-Z0-9 ]{0,40}", 0..=6),
    ) {
        let n = strs.len();
        let mut assigns = String::new();
        for (i, s) in strs.iter().enumerate() {
            assigns.push_str(&format!("        a[{i}] = \"{s}\";\n"));
        }
        let mut compare = String::from("        bool eq = a.length == b.length;\n");
        for i in 0..n {
            compare.push_str(&format!(
                "        eq = eq && keccak256(bytes(a[{i}])) == keccak256(bytes(b[{i}]));\n"
            ));
        }
        let src = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Codec {{
    function check() external pure returns (bool) {{
        string[] memory a = new string[]({n});
{assigns}        bytes memory enc = abi.encode(a);
        string[] memory b = abi.decode(enc, (string[]));
{compare}        return eq;
    }}
}}"#);
        let ok = run_check_returns_bool(&src)
            .unwrap_or_else(|e| panic!("string[] round-trip n={}: {}", n, e));
        prop_assert!(ok, "string[] round-trip failed for n={} strs={:?}", n, strs);
    }

    /// `bytes[]` round-trip — same recursive layout as `string[]`, element
    /// payloads are arbitrary bytes 0..=40 long.
    #[test]
    fn roundtrip_bytes_array(
        items in prop::collection::vec(prop::collection::vec(any::<u8>(), 0..=40), 0..=6),
    ) {
        let n = items.len();
        let mut assigns = String::new();
        for (i, b) in items.iter().enumerate() {
            assigns.push_str(&format!("        a[{i}] = hex\"{}\";\n", hex::encode(b)));
        }
        let mut compare = String::from("        bool eq = a.length == b.length;\n");
        for i in 0..n {
            compare.push_str(&format!(
                "        eq = eq && keccak256(a[{i}]) == keccak256(b[{i}]);\n"
            ));
        }
        let src = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Codec {{
    function check() external pure returns (bool) {{
        bytes[] memory a = new bytes[]({n});
{assigns}        bytes memory enc = abi.encode(a);
        bytes[] memory b = abi.decode(enc, (bytes[]));
{compare}        return eq;
    }}
}}"#);
        let ok = run_check_returns_bool(&src)
            .unwrap_or_else(|e| panic!("bytes[] round-trip n={}: {}", n, e));
        prop_assert!(ok, "bytes[] round-trip failed for n={}", n);
    }

    /// `uint256[][]` round-trip — a two-level nested dynamic array, the
    /// deepest recursion the encoder/decoder exercise in practice.
    #[test]
    fn roundtrip_uint256_2d_array(
        rows in prop::collection::vec(prop::collection::vec(any::<u64>(), 0..=4), 0..=4),
    ) {
        let n = rows.len();
        let mut assigns = String::new();
        for (i, row) in rows.iter().enumerate() {
            assigns.push_str(&format!("        a[{i}] = new uint256[]({});\n", row.len()));
            for (j, v) in row.iter().enumerate() {
                assigns.push_str(&format!("        a[{i}][{j}] = uint256({v});\n"));
            }
        }
        let mut compare = String::from("        bool eq = a.length == b.length;\n");
        for (i, row) in rows.iter().enumerate() {
            compare.push_str(&format!("        eq = eq && a[{i}].length == b[{i}].length;\n"));
            for j in 0..row.len() {
                compare.push_str(&format!("        eq = eq && a[{i}][{j}] == b[{i}][{j}];\n"));
            }
        }
        let src = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract Codec {{
    function check() external pure returns (bool) {{
        uint256[][] memory a = new uint256[][]({n});
{assigns}        bytes memory enc = abi.encode(a);
        uint256[][] memory b = abi.decode(enc, (uint256[][]));
{compare}        return eq;
    }}
}}"#);
        let ok = run_check_returns_bool(&src)
            .unwrap_or_else(|e| panic!("uint256[][] round-trip n={}: {}", n, e));
        prop_assert!(ok, "uint256[][] round-trip failed for n={} rows={:?}", n, rows);
    }
}
