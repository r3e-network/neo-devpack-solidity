//! Property-based tests for Solidity **custom-error** revert envelopes
//! observed via `try / catch (bytes memory data)`.
//!
//! Distinct from `task107_catch_panic_tests.rs` (Panic envelopes,
//! selector 0x4e487b71) and `differential.rs` (Error(string) shapes,
//! selector 0x08c379a0). Custom errors with multi-arg payloads are the
//! modern Solidity revert mechanism (gas-cheap vs. `revert("string")`):
//!
//!   `revert MyError(uint256 a, address b, bytes32 c)`
//!     → revert payload =
//!         keccak256("MyError(uint256,address,bytes32)")[..4]
//!         || abi.encode(a, b, c)
//!
//! These proptests round-trip the payload through a `try {…} catch
//! (bytes memory data) { return data; }` arm and assert byte-for-byte
//! equality with an in-process reference envelope built from `sha3` +
//! manual ABI encoding. A regression where the compiler drops args,
//! mis-canonicalises types in the signature, or pads dynamic tails
//! incorrectly surfaces here as a payload mismatch.
//!
//! The fourth probe (`panic_vs_error_vs_custom_dispatch`) addresses
//! catch-arm routing: a try block followed by `catch Panic(uint)` /
//! `catch Error(string)` / `catch (bytes)` arms must dispatch each
//! generated revert source to **exactly** the right arm. A regression
//! where a Panic gets routed to the bytes arm (or an Error(string) gets
//! routed to a custom-error bytes arm) silently swallows information
//! production contracts depend on.
//!
//! Reference reading:
//!   - `tests/fuzz_tests/task107_catch_panic_tests.rs` — Panic envelope shape
//!   - `src/ir/build/panic.rs::emit_panic` — Panic emission
//!   - `src/ir/statements/dispatch/return_revert.rs` — revert lowering
//!   - `src/ir/statements/dispatch/try_catch.rs` — catch classification

#![allow(unused_imports)]
#![allow(clippy::uninlined_format_args)]

use super::common::*;
use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::runtime::types::StackItem;
use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};
use proptest::prelude::*;
use sha3::{Digest, Keccak256};

/// Compute the 4-byte selector for an error signature, e.g.
/// `error_selector("MyError(uint256,address,bytes32)")`.
fn error_selector(signature: &str) -> [u8; 4] {
    let digest = Keccak256::digest(signature.as_bytes());
    [digest[0], digest[1], digest[2], digest[3]]
}

/// 32-byte big-endian slot for a u64.
fn be32_u64(v: u64) -> [u8; 32] {
    let mut slot = [0u8; 32];
    slot[24..].copy_from_slice(&v.to_be_bytes());
    slot
}

/// 32-byte big-endian slot for a 20-byte address (right-aligned, EVM-style).
/// `addr_evm_be` must already be in EVM byte order (the test caller is
/// responsible for any Neo-LE↔EVM-BE flip).
fn be32_address(addr_evm_be: &[u8; 20]) -> [u8; 32] {
    let mut slot = [0u8; 32];
    slot[12..].copy_from_slice(addr_evm_be);
    slot
}

/// 32-byte slot for a bytes32 (already 32 bytes, left-aligned per ABI spec).
fn slot_bytes32(b: &[u8; 32]) -> [u8; 32] {
    *b
}

/// abi.encode(string) tail = `[BE32(len) || data || zero-pad to 32]`.
/// Caller prepends the 32-byte offset (always 0x20 for a top-level single
/// dynamic arg).
fn abi_encode_string_tail(s: &[u8]) -> Vec<u8> {
    let padded = s.len().div_ceil(32) * 32;
    let mut out = Vec::with_capacity(32 + padded);
    out.extend_from_slice(&be32_u64(s.len() as u64));
    out.extend_from_slice(s);
    out.resize(32 + padded, 0);
    out
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    /// **a. `custom_error_static_args_roundtrip`** —
    /// `error MyError(uint256, address, bytes32)`. Static-only args means
    /// the entire payload is `selector || BE32(a) || BE32(addr) || b32`,
    /// 100 bytes total. Compares byte-for-byte against the in-process
    /// reference.
    #[test]
    fn custom_error_static_args_roundtrip(
        a in 0u64..=u64::MAX,
        addr_le in prop::array::uniform20(any::<u8>()),
        b32 in prop::array::uniform32(any::<u8>()),
    ) {
        // Build reference envelope: selector || abi.encode(a, addr, b32).
        // Address slot uses EVM-BE order (Neo stores addresses LE; the
        // compiler's abi.encode flips them, see baseline_tests.rs::pack
        // precedent).
        let mut addr_be = addr_le;
        addr_be.reverse();
        let selector = error_selector("MyError(uint256,address,bytes32)");
        let mut expected = Vec::with_capacity(4 + 96);
        expected.extend_from_slice(&selector);
        expected.extend_from_slice(&be32_u64(a));
        expected.extend_from_slice(&be32_address(&addr_be));
        expected.extend_from_slice(&slot_bytes32(&b32));

        // Source: the catch arm returns `data` so the runtime delivers
        // the raw payload bytes through `ExecutionResult.return_data`.
        let src = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {{
    error MyError(uint256, address, bytes32);
    function fail() external pure {{
        revert MyError({a}, address(uint160(uint256(uint160(0x{addr_hex})))), bytes32(0x{b32_hex}));
    }}
    function probe() external returns (bytes memory) {{
        try this.fail() {{ return ""; }}
        catch (bytes memory data) {{ return data; }}
    }}
}}"#,
            a = a,
            addr_hex = hex::encode(&addr_be), // hex literal in solidity = EVM-BE
            b32_hex = hex::encode(&b32),
        );

        let arts = compile_contracts(&src, false, 2)
            .unwrap_or_else(|e| panic!("custom_error static-args compile: {:?}", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt.call_method(&art.bytecode, &art.tokens, &art.manifest,
            "probe", &[] as &[StackItem]).expect("probe call");
        prop_assert!(r.success,
            "custom_error static-args: probe() must succeed via catch; exc={:?}",
            r.exception.as_ref().map(|e| &e.message));
        prop_assert_eq!(r.return_data.clone(), expected.clone(),
            "custom_error static-args envelope mismatch:\n got: {}\nwant: {}",
            hex::encode(&r.return_data), hex::encode(&expected));
    }

    /// **b1. `custom_error_dynamic_string_roundtrip`** —
    /// `error WithString(string)`. Dynamic-tail: payload =
    /// `selector || BE32(0x20) || BE32(len) || str_bytes || pad32`.
    #[test]
    fn custom_error_dynamic_string_roundtrip(
        s in "[a-zA-Z0-9 _]{0,64}",
    ) {
        let selector = error_selector("WithString(string)");
        let mut expected = Vec::with_capacity(4 + 32 + 32 + 64);
        expected.extend_from_slice(&selector);
        // Top-level dynamic offset = 0x20.
        expected.extend_from_slice(&be32_u64(0x20));
        expected.extend_from_slice(&abi_encode_string_tail(s.as_bytes()));

        let src = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {{
    error WithString(string);
    function fail() external pure {{
        revert WithString("{lit}");
    }}
    function probe() external returns (bytes memory) {{
        try this.fail() {{ return ""; }}
        catch (bytes memory data) {{ return data; }}
    }}
}}"#, lit = s);

        let arts = compile_contracts(&src, false, 2)
            .unwrap_or_else(|e| panic!("WithString compile: {:?}", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt.call_method(&art.bytecode, &art.tokens, &art.manifest,
            "probe", &[] as &[StackItem]).expect("probe call");
        prop_assert!(r.success,
            "WithString({:?}): probe() must succeed via catch; exc={:?}",
            s, r.exception.as_ref().map(|e| &e.message));
        prop_assert_eq!(r.return_data.clone(), expected.clone(),
            "WithString({:?}) envelope mismatch:\n got: {}\nwant: {}",
            s, hex::encode(&r.return_data), hex::encode(&expected));
    }

    /// **b2. `custom_error_dynamic_bytes_roundtrip`** —
    /// `error WithBytes(bytes)`. Same dynamic-tail layout as string
    /// (Solidity ABI treats `bytes` and `string` identically here).
    #[test]
    fn custom_error_dynamic_bytes_roundtrip(
        data in prop::collection::vec(any::<u8>(), 0..=64),
    ) {
        let selector = error_selector("WithBytes(bytes)");
        let mut expected = Vec::with_capacity(4 + 32 + 32 + 64);
        expected.extend_from_slice(&selector);
        expected.extend_from_slice(&be32_u64(0x20));
        expected.extend_from_slice(&abi_encode_string_tail(&data));

        // Render the bytes literal as `hex"..."` to keep the source
        // ASCII-safe even for non-printable inputs.
        let src = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {{
    error WithBytes(bytes);
    function fail() external pure {{
        revert WithBytes(hex"{hex}");
    }}
    function probe() external returns (bytes memory) {{
        try this.fail() {{ return ""; }}
        catch (bytes memory raw) {{ return raw; }}
    }}
}}"#, hex = hex::encode(&data));

        let arts = compile_contracts(&src, false, 2)
            .unwrap_or_else(|e| panic!("WithBytes compile: {:?}", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt.call_method(&art.bytecode, &art.tokens, &art.manifest,
            "probe", &[] as &[StackItem]).expect("probe call");
        prop_assert!(r.success,
            "WithBytes({}B): probe() must succeed via catch; exc={:?}",
            data.len(), r.exception.as_ref().map(|e| &e.message));
        prop_assert_eq!(r.return_data.clone(), expected.clone(),
            "WithBytes({}B) envelope mismatch:\n got: {}\nwant: {}",
            data.len(), hex::encode(&r.return_data), hex::encode(&expected));
    }

    /// **c. `custom_error_mixed_static_dynamic`** —
    /// `error Mixed(uint256 a, string memory msg_, address b)`. Verifies
    /// both the head (`BE32(a) || BE32(offset_to_str) || BE32(b)`) and
    /// tail (`BE32(len) || str_bytes || pad32`) encodings.
    ///
    /// Head occupies 3 slots (96 bytes) so the offset to the dynamic
    /// payload is 0x60.
    #[test]
    fn custom_error_mixed_static_dynamic(
        a in 0u64..=u64::MAX,
        msg in "[a-zA-Z0-9 _]{0,48}",
        addr_le in prop::array::uniform20(any::<u8>()),
    ) {
        let mut addr_be = addr_le;
        addr_be.reverse();
        let selector = error_selector("Mixed(uint256,string,address)");
        let mut expected = Vec::new();
        expected.extend_from_slice(&selector);
        // Head: 3 slots — uint256(a), offset-to-string-tail, address(b).
        expected.extend_from_slice(&be32_u64(a));
        expected.extend_from_slice(&be32_u64(0x60));
        expected.extend_from_slice(&be32_address(&addr_be));
        // Tail: length + UTF-8 bytes + zero-pad to 32.
        expected.extend_from_slice(&abi_encode_string_tail(msg.as_bytes()));

        let src = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {{
    error Mixed(uint256 a, string msg_, address b);
    function fail() external pure {{
        revert Mixed({a}, "{lit}", address(uint160(uint256(uint160(0x{addr_hex})))));
    }}
    function probe() external returns (bytes memory) {{
        try this.fail() {{ return ""; }}
        catch (bytes memory data) {{ return data; }}
    }}
}}"#,
            a = a, lit = msg, addr_hex = hex::encode(&addr_be),
        );

        let arts = compile_contracts(&src, false, 2)
            .unwrap_or_else(|e| panic!("Mixed compile: {:?}", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt.call_method(&art.bytecode, &art.tokens, &art.manifest,
            "probe", &[] as &[StackItem]).expect("probe call");
        prop_assert!(r.success,
            "Mixed: probe() must succeed via catch; exc={:?}",
            r.exception.as_ref().map(|e| &e.message));
        prop_assert_eq!(r.return_data.clone(), expected.clone(),
            "Mixed envelope mismatch (a={}, msg={:?}):\n got: {}\nwant: {}",
            a, msg, hex::encode(&r.return_data), hex::encode(&expected));
    }
}

// ==================== d. panic_vs_error_vs_custom_dispatch ====================
//
// Catch-arm routing is the safety-critical bit: if a Panic gets routed
// to the `catch (bytes)` arm (because the selector guard misclassified
// 0x4e487b71), the contract silently swallows what was a logic bug.
// Conversely, if a custom error gets routed to `catch Error(string)`
// (because the guard's selector check is too loose), the caller's
// decoder reads garbage. This proptest generates one of three revert
// sources per case and asserts the catch arm fires correctly:
//
//   src_kind=0  → divide-by-zero (Panic 0x12)
//                 expected arm: `catch Panic(uint code)`     → 0xa1
//   src_kind=1  → require(false, "msg")  (Error(string))
//                 expected arm: `catch Error(string memory)` → 0xa2
//   src_kind=2  → revert MyErr(x)        (custom error)
//                 expected arm: `catch (bytes memory)`        → 0xa3
//
// A regression where any source routes to the wrong arm surfaces as a
// disagreement between `r.return_data` and the expected sentinel.

proptest! {
    #![proptest_config(ProptestConfig::with_cases(18))]

    #[test]
    fn panic_vs_error_vs_custom_dispatch(
        src_kind in 0u8..=2u8,
        // Random uint for the custom-error payload (case 2 only). Kept
        // small so it fits in the LE-decoded scalar return; the assertion
        // below only checks the sentinel byte, so the value is irrelevant.
        x in 1u64..=1_000_000u64,
    ) {
        // Source-kind specific willPanic body.
        let body = match src_kind {
            0 => "uint y = 0; uint z = 1 / y; return z;".to_string(),
            1 => "require(false, \"err\"); return 0;".to_string(),
            _ => format!("revert MyErr({x}); return 0;"),
        };

        let src = format!(r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {{
    error MyErr(uint256);
    function willRevert() external pure returns (uint) {{
        {body}
    }}
    function handle() external returns (uint) {{
        try this.willRevert() returns (uint) {{ return 0xa0; }}
        catch Panic(uint /*code*/) {{ return 0xa1; }}
        catch Error(string memory) {{ return 0xa2; }}
        catch (bytes memory) {{ return 0xa3; }}
    }}
}}"#, body = body);

        let arts = compile_contracts(&src, false, 2)
            .unwrap_or_else(|e| panic!("dispatch kind={} compile: {:?}", src_kind, e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt.call_method(&art.bytecode, &art.tokens, &art.manifest,
            "handle", &[] as &[StackItem]).expect("handle call");
        prop_assert!(r.success,
            "dispatch kind={}: handle() must succeed via catch; exc={:?}",
            src_kind, r.exception.as_ref().map(|e| &e.message));

        let got = decode_uint_le(&r.return_data);
        let expected = match src_kind {
            0 => num_bigint::BigUint::from(0xa1u64), // Panic
            1 => num_bigint::BigUint::from(0xa2u64), // Error(string)
            _ => num_bigint::BigUint::from(0xa3u64), // bytes (custom error)
        };
        prop_assert_eq!(got.clone(), expected.clone(),
            "dispatch kind={} (0=panic 1=error-str 2=custom-err): expected sentinel \
             0x{:x} but got 0x{} (rd_hex={}). A wrong arm here means a misrouted catch:\n  \
             0xa0 = try-success branch fired (revert was NOT thrown)\n  \
             0xa1 = Panic(uint) arm caught\n  \
             0xa2 = Error(string) arm caught\n  \
             0xa3 = catch (bytes) arm caught",
            src_kind, expected, got.to_str_radix(16), hex::encode(&r.return_data));
    }
}
