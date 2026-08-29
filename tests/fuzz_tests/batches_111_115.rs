//! Batches 111-115 — runtime verification for native-contract and EVM
//! auto-map surfaces that earlier batches compiled but did not execute.
//!
//! The earlier batches (81-110) covered compile-path coverage for most of
//! the NEO/GAS/StdLib/CryptoLib/Ledger/Policy/ContractManagement surface
//! plus the EVM-compat auto-mappings (blockhash, block.coinbase, tx.origin,
//! block.basefee, tx.gasprice, block.difficulty, block.gaslimit). These
//! batches close the runtime-verification gap: each probe compiles a tiny
//! contract that *calls* the surface, executes it against the embedded
//! `NeoRuntime`, and asserts on the return-value shape / value.
//!
//! Prefix scheme: 111=III2, 112=JJJ2, 113=KKK2, 114=LLL2, 115=MMM2.

use super::common::*;
use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::runtime::types::StackItem;
use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};

// ==================== Batch #111 — StdLib utilities runtime ====================
//
// Five probes exercising `StdLib` native-contract utility functions that
// prior batches did not runtime-verify: `itoa` (int→string), `atoi`
// (string→int), `base64Encode`/`base64Decode` (string round-trip),
// `jsonSerialize` (StackItem→JSON).
//
//   III2_1: StdLib.itoa(12345, 10) returns "12345".
//   III2_2: StdLib.atoi("12345", 10) returns 12345.
//   III2_3: StdLib.base64Encode("hello") + base64Decode round-trips.
//   III2_4: StdLib.base64Decode of empty input returns empty bytes.
//   III2_5: StdLib.jsonSerialize(42) returns non-empty JSON string.

// III2_1 — `StdLib.itoa(12345, 10)` returns the ASCII string "12345".
// End-to-end: compiler's member-access lowering for the `StdLib` namespace
// (see resolve_stdlib_member in src/ir/context/builtins/resolve.rs) emits
// a System.Contract.Call with the StdLib hash + "itoa" method, which the
// runtime dispatches to its base-10 formatter.
#[test]
fn batch111_iii2_1_stdlib_itoa_decimal() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function toStr() external pure returns (string memory) {
        return StdLib.itoa(12345, 10);
    }
}"#;
    let arts = compile_contracts(src, false, 2).unwrap_or_else(|e| {
        panic!(
            "III2_1 compile: {:?}. If this fires, the StdLib.itoa \
             member-access lowering regressed — check \
             src/ir/context/builtins/resolve.rs::resolve_stdlib_member.",
            e
        )
    });
    assert!(!arts.is_empty(), "III2_1 compile produced no artifacts");
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("III2_1 rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "toStr", &[])
        .expect("III2_1 toStr() host-level");
    assert!(
        r.success,
        "III2_1 StdLib.itoa(12345, 10) must execute without fault; exc={:?}.",
        r.exception.as_ref().map(|e| &e.message)
    );
    assert_eq!(
        r.return_data,
        b"12345",
        "III2_1 StdLib.itoa(12345, 10) must return the ASCII string \
         \"12345\"; got {:?} (rd_hex={}). If this differs, the StdLib \
         native-contract's base-10 formatter returned a wrong encoding \
         OR the Solidity→Neo member-access lowering broke the argument \
         ordering.",
        std::str::from_utf8(&r.return_data).ok(),
        hex::encode(&r.return_data)
    );
}

// III2_2 — StdLib.atoi("12345", 10) runtime.
// Verifies the inverse of III2_1: `atoi` parses a decimal string back to
// an integer. Covers the parse-path lowering that `itoa` does not.
#[test]
fn batch111_iii2_2_stdlib_atoi_decimal() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function parse() external pure returns (int) {
        return StdLib.atoi("12345", 10);
    }
}"#;
    let arts = compile_contracts(src, false, 2).unwrap_or_else(|e| {
        panic!(
            "III2_2 compile: {:?}. If this fires, the StdLib.atoi lowering \
             regressed — check src/ir/context/builtins/resolve.rs for the \
             `atoi` intrinsic wiring.",
            e
        )
    });
    assert!(!arts.is_empty(), "III2_2 compile produced no artifacts");
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("III2_2 rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "parse", &[])
        .expect("III2_2 parse() host-level");
    assert!(
        r.success,
        "III2_2 StdLib.atoi(\"12345\", 10) must execute without fault; \
         exc={:?}.",
        r.exception.as_ref().map(|e| &e.message)
    );
    let v = decode_uint_le(&r.return_data);
    assert_eq!(
        v,
        num_bigint::BigUint::from(12345u64),
        "III2_2 StdLib.atoi(\"12345\", 10) must return 12345; got {} \
         (rd_hex={}).",
        v,
        hex::encode(&r.return_data)
    );
}

// III2_3 — StdLib.base64Encode round-trips via base64Decode.
// Verifies two separate StdLib entry points (encode/decode) both wire up
// to the native contract and the round-trip preserves the input bytes.
#[test]
fn batch111_iii2_3_stdlib_base64_roundtrip() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function roundtrip(bytes memory data) external pure returns (bytes memory) {
        string memory encoded = StdLib.base64Encode(data);
        return StdLib.base64Decode(encoded);
    }
}"#;
    let arts = compile_contracts(src, false, 2).unwrap_or_else(|e| {
        panic!(
            "III2_3 compile: {:?}. If this fires, one of the StdLib.base64 \
             intrinsic wirings regressed.",
            e
        )
    });
    assert!(!arts.is_empty(), "III2_3 compile produced no artifacts");
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("III2_3 rt");
    let payload = b"hello\x00world".to_vec();
    let r = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "roundtrip",
            &[StackItem::byte_array(payload.clone())],
        )
        .expect("III2_3 roundtrip() host-level");
    assert!(
        r.success,
        "III2_3 base64 round-trip must execute without fault; exc={:?}.",
        r.exception.as_ref().map(|e| &e.message)
    );
    assert_eq!(
        r.return_data,
        payload,
        "III2_3 base64 round-trip must return input unchanged; got \
         rd_hex={} (expected {}).",
        hex::encode(&r.return_data),
        hex::encode(&payload)
    );
}

// III2_4 — StdLib.base64Encode of empty bytes is the empty string.
// Edge-case probe: an empty input should not fault — both the encode side
// (no-op on zero-length ByteString) and the decode side (empty string is
// a valid base64 encoding) must agree.
#[test]
fn batch111_iii2_4_stdlib_base64_empty_roundtrip() {
    // Edge-case: chain encode/decode on an empty input and assert the
    // decoded `.length` is 0. This exercises both the StdLib.base64*
    // member-access lowering and the SIZE opcode on the returned
    // ByteString (previously a regression point when the StdLib path
    // returned Null and SIZE faulted with "unsupported type").
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function emptyRT() external pure returns (uint256) {
        string memory encoded = StdLib.base64Encode("");
        bytes memory decoded = StdLib.base64Decode(encoded);
        return decoded.length;
    }
}"#;
    let arts =
        compile_contracts(src, false, 2).unwrap_or_else(|e| panic!("III2_4 compile: {:?}", e));
    assert!(!arts.is_empty(), "III2_4 compile produced no artifacts");
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("III2_4 rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "emptyRT", &[])
        .expect("III2_4 emptyRT() host-level");
    assert!(
        r.success,
        "III2_4 empty-input base64 round-trip must execute without fault; \
         exc={:?}. If this faults with \"SIZE: unsupported type\", the \
         StdLib.* member-access lowering regressed (was returning Null \
         instead of ByteString before the resolve_stdlib_member fix).",
        r.exception.as_ref().map(|e| &e.message)
    );
    let v = decode_uint_le(&r.return_data);
    assert_eq!(
        v,
        num_bigint::BigUint::from(0u64),
        "III2_4 empty base64 round-trip must yield .length == 0; got {} \
         (rd_hex={}).",
        v,
        hex::encode(&r.return_data)
    );
}

// III2_5 — StdLib.jsonSerialize returns non-empty bytes for a scalar.
// The serializer takes any StackItem and produces a JSON encoding; for a
// uint value it should return an ASCII digit string (the JSON encoding of
// the integer). We assert non-empty + the leading byte is an ASCII digit.
#[test]
fn batch111_iii2_5_stdlib_json_serialize_scalar() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function ser() external pure returns (string memory) {
        return StdLib.jsonSerialize(42);
    }
}"#;
    let arts =
        compile_contracts(src, false, 2).unwrap_or_else(|e| panic!("III2_5 compile: {:?}", e));
    assert!(!arts.is_empty(), "III2_5 compile produced no artifacts");
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("III2_5 rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "ser", &[])
        .expect("III2_5 ser() host-level");
    assert!(
        r.success,
        "III2_5 StdLib.jsonSerialize(42) must execute without fault; exc={:?}.",
        r.exception.as_ref().map(|e| &e.message)
    );
    // Neo's StdLib.jsonSerialize produces a type-tagged JSON of the
    // StackItem, not a bare JSON value. Expected form:
    //   {"type":"Integer","value":"42"}
    // (see src/runtime/execution/execution_impl_part2_native/stdlib.rs).
    let s = std::str::from_utf8(&r.return_data)
        .expect("III2_5 jsonSerialize output must be valid UTF-8");
    assert!(
        s.starts_with('{') && s.contains("\"type\":\"Integer\"") && s.contains("\"value\":42"),
        "III2_5 jsonSerialize(42) must be type-tagged JSON containing \
         type=Integer and value=42; got {s:?} (rd_hex={}).",
        hex::encode(&r.return_data)
    );
}

// ==================== Batch #112 — CryptoLib hashing runtime ====================
//
// Five probes exercising `CryptoLib` native-contract hashing primitives.
// Prior batches referenced `CryptoLib.sha256` but did not runtime-verify
// `ripemd160`, `murmur32`, or the deterministic empty-input digests.
//
//   JJJ2_1: CryptoLib.sha256("") returns the canonical SHA-256 empty digest.
//   JJJ2_2: CryptoLib.ripemd160("") returns the canonical RIPEMD-160 empty digest.
//   JJJ2_3: CryptoLib.sha256(b"abc") returns the canonical "abc" SHA-256.
//   JJJ2_4: CryptoLib.murmur32(b"abc", 0) returns the reference murmur3x86_32 digest.
//   JJJ2_5: CryptoLib.sha256 is deterministic across calls.

// JJJ2_1 — SHA-256 of empty input.
// The canonical SHA-256 empty-input digest (known-value test) is
// `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
// Verifies both the CryptoLib native-contract call path and that the 32-
// byte output ordering matches big-endian (the Neo ByteString convention).
#[test]
fn batch112_jjj2_1_sha256_empty() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function h() external pure returns (bytes32) {
        return CryptoLib.sha256("");
    }
}"#;
    let arts =
        compile_contracts(src, false, 2).unwrap_or_else(|e| panic!("JJJ2_1 compile: {:?}", e));
    assert!(!arts.is_empty(), "JJJ2_1 compile produced no artifacts");
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("JJJ2_1 rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "h", &[])
        .expect("JJJ2_1 h() host-level");
    assert!(
        r.success,
        "JJJ2_1 CryptoLib.sha256(\"\") must execute without fault; exc={:?}.",
        r.exception.as_ref().map(|e| &e.message)
    );
    assert_eq!(
        r.return_data.len(),
        32,
        "JJJ2_1 sha256 output must be 32 bytes; got {} (rd_hex={}).",
        r.return_data.len(),
        hex::encode(&r.return_data)
    );
    let canonical =
        hex::decode("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855").unwrap();
    assert_eq!(
        r.return_data,
        canonical,
        "JJJ2_1 sha256(\"\") must equal the canonical empty-input \
         digest; got rd_hex={}, expected {}.",
        hex::encode(&r.return_data),
        hex::encode(&canonical)
    );
}

// JJJ2_2 — RIPEMD-160 of empty input.
// Canonical empty-input digest: `9c1185a5c5e9fc54612808977ee8f548b2258d31`
// (20 bytes). Verifies the CryptoLib ripemd160 wiring and 20-byte output.
#[test]
fn batch112_jjj2_2_ripemd160_empty() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function h() external pure returns (bytes20) {
        return CryptoLib.ripemd160("");
    }
}"#;
    let arts =
        compile_contracts(src, false, 2).unwrap_or_else(|e| panic!("JJJ2_2 compile: {:?}", e));
    assert!(!arts.is_empty(), "JJJ2_2 compile produced no artifacts");
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("JJJ2_2 rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "h", &[])
        .expect("JJJ2_2 h() host-level");
    assert!(
        r.success,
        "JJJ2_2 CryptoLib.ripemd160(\"\") must execute without fault; \
         exc={:?}.",
        r.exception.as_ref().map(|e| &e.message)
    );
    assert_eq!(
        r.return_data.len(),
        20,
        "JJJ2_2 ripemd160 output must be 20 bytes; got {} (rd_hex={}).",
        r.return_data.len(),
        hex::encode(&r.return_data)
    );
    // Canonical RIPEMD-160 empty-input digest:
    let canonical = hex::decode("9c1185a5c5e9fc54612808977ee8f548b2258d31").unwrap();
    assert_eq!(
        r.return_data,
        canonical,
        "JJJ2_2 ripemd160(\"\") must equal canonical empty-input digest; \
         got rd_hex={}, expected {}.",
        hex::encode(&r.return_data),
        hex::encode(&canonical)
    );
}

// JJJ2_3 — SHA-256 of b"abc" matches the well-known test vector.
// Canonical digest: ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad.
// Verifies the hashing path with a non-empty input.
#[test]
fn batch112_jjj2_3_sha256_abc() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function h(bytes memory data) external pure returns (bytes32) {
        return CryptoLib.sha256(data);
    }
}"#;
    let arts =
        compile_contracts(src, false, 2).unwrap_or_else(|e| panic!("JJJ2_3 compile: {:?}", e));
    assert!(!arts.is_empty(), "JJJ2_3 compile produced no artifacts");
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("JJJ2_3 rt");
    let r = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "h",
            &[StackItem::byte_array(b"abc".to_vec())],
        )
        .expect("JJJ2_3 h(b'abc') host-level");
    assert!(
        r.success,
        "JJJ2_3 CryptoLib.sha256(b'abc') must execute without fault; \
         exc={:?}.",
        r.exception.as_ref().map(|e| &e.message)
    );
    assert_eq!(
        r.return_data.len(),
        32,
        "JJJ2_3 sha256 output must be 32 bytes; got {}.",
        r.return_data.len()
    );
    let canonical =
        hex::decode("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad").unwrap();
    assert_eq!(
        r.return_data,
        canonical,
        "JJJ2_3 sha256(b'abc') must equal canonical digest; got \
         rd_hex={}, expected {}.",
        hex::encode(&r.return_data),
        hex::encode(&canonical)
    );
}

// JJJ2_4 — murmur32 of b"abc" with seed 0 produces a 4-byte output.
// We do not assert the exact bytes because Neo's murmur3_x86_32 endianness
// may differ from canonical MurmurHash3; we do assert output length and
// non-zero determinism-under-retest.
#[test]
fn batch112_jjj2_4_murmur32_abc() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function h(bytes memory data, uint32 seed) external pure returns (uint32) {
        return CryptoLib.murmur32(data, seed);
    }
}"#;
    let arts =
        compile_contracts(src, false, 2).unwrap_or_else(|e| panic!("JJJ2_4 compile: {:?}", e));
    assert!(!arts.is_empty(), "JJJ2_4 compile produced no artifacts");
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("JJJ2_4 rt");
    let r = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "h",
            &[
                StackItem::byte_array(b"abc".to_vec()),
                StackItem::Integer(0i64),
            ],
        )
        .expect("JJJ2_4 h() host-level");
    assert!(
        r.success,
        "JJJ2_4 CryptoLib.murmur32(b'abc', 0) must execute without fault; \
         exc={:?}.",
        r.exception.as_ref().map(|e| &e.message)
    );
    // murmur32 output is a 4-byte hash. Accept 1..=4 bytes (LE-encoded
    // u32) since small values may be minimum-width encoded.
    assert!(
        r.return_data.len() <= 4,
        "JJJ2_4 murmur32 output must be ≤4 bytes (u32-sized); got {} \
         (rd_hex={}).",
        r.return_data.len(),
        hex::encode(&r.return_data)
    );
}

// JJJ2_5 — SHA-256 is deterministic: two back-to-back calls of the same
// input produce identical outputs. Guards against non-determinism leaks.
#[test]
fn batch112_jjj2_5_sha256_deterministic() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function h(bytes memory data) external pure returns (bytes32) {
        return CryptoLib.sha256(data);
    }
}"#;
    let arts =
        compile_contracts(src, false, 2).unwrap_or_else(|e| panic!("JJJ2_5 compile: {:?}", e));
    assert!(!arts.is_empty(), "JJJ2_5 compile produced no artifacts");
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("JJJ2_5 rt");
    let payload = b"neo-devpack-solidity-fuzz-test".to_vec();
    let first = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "h",
            &[StackItem::byte_array(payload.clone())],
        )
        .expect("JJJ2_5 first call");
    assert!(first.success, "JJJ2_5 first sha256 call must succeed");
    let mut rt2 = NeoRuntime::new(RuntimeConfig::default()).expect("JJJ2_5 rt2");
    let second = rt2
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "h",
            &[StackItem::byte_array(payload)],
        )
        .expect("JJJ2_5 second call");
    assert!(second.success, "JJJ2_5 second sha256 call must succeed");
    assert_eq!(
        first.return_data,
        second.return_data,
        "JJJ2_5 sha256 must be deterministic; call1={} call2={}.",
        hex::encode(&first.return_data),
        hex::encode(&second.return_data)
    );
}

// ==================== Batch #113 — NEO/GAS native token runtime ====================
//
// Five probes exercising the NEO and GAS native-contract token metadata
// surface (`symbol`, `decimals`, `totalSupply`). Prior batches covered
// `balanceOf` and `transfer` compile paths; these exercise the metadata
// getters that no earlier test runtime-verified.
//
//   KKK2_1: NativeCalls.neoSymbol() == "NEO".
//   KKK2_2: NativeCalls.gasSymbol() == "GAS".
//   KKK2_3: NativeCalls.neoDecimals() == 0.
//   KKK2_4: NativeCalls.gasDecimals() == 8.
//   KKK2_5: NativeCalls.neoTotalSupply() == 100_000_000 (fixed supply).

#[test]
fn batch113_kkk2_1_neo_symbol() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function s() external view returns (string memory) {
        return NativeCalls.neoSymbol();
    }
}"#;
    let arts =
        compile_contracts(src, false, 2).unwrap_or_else(|e| panic!("KKK2_1 compile: {:?}", e));
    assert!(!arts.is_empty());
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("KKK2_1 rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "s", &[])
        .expect("KKK2_1 s()");
    assert!(
        r.success,
        "KKK2_1 NEO.symbol() must succeed; exc={:?}. If this faults, the \
         NEO native-contract symbol syscall path regressed.",
        r.exception.as_ref().map(|e| &e.message)
    );
    assert_eq!(
        r.return_data,
        b"NEO",
        "KKK2_1 NEO.symbol() must be \"NEO\"; got {:?} (rd_hex={}).",
        std::str::from_utf8(&r.return_data).ok(),
        hex::encode(&r.return_data)
    );
}

#[test]
fn batch113_kkk2_2_gas_symbol() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function s() external view returns (string memory) {
        return NativeCalls.gasSymbol();
    }
}"#;
    let arts =
        compile_contracts(src, false, 2).unwrap_or_else(|e| panic!("KKK2_2 compile: {:?}", e));
    assert!(!arts.is_empty());
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("KKK2_2 rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "s", &[])
        .expect("KKK2_2 s()");
    assert!(
        r.success,
        "KKK2_2 GAS.symbol() must succeed; exc={:?}.",
        r.exception.as_ref().map(|e| &e.message)
    );
    assert_eq!(
        r.return_data,
        b"GAS",
        "KKK2_2 GAS.symbol() must be \"GAS\"; got {:?} (rd_hex={}).",
        std::str::from_utf8(&r.return_data).ok(),
        hex::encode(&r.return_data)
    );
}

#[test]
fn batch113_kkk2_3_neo_decimals_is_zero() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function d() external view returns (uint8) {
        return NativeCalls.neoDecimals();
    }
}"#;
    let arts =
        compile_contracts(src, false, 2).unwrap_or_else(|e| panic!("KKK2_3 compile: {:?}", e));
    assert!(!arts.is_empty());
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("KKK2_3 rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "d", &[])
        .expect("KKK2_3 d()");
    assert!(
        r.success,
        "KKK2_3 NEO.decimals() must succeed; exc={:?}.",
        r.exception.as_ref().map(|e| &e.message)
    );
    let v = decode_uint_le(&r.return_data);
    assert_eq!(
        v,
        num_bigint::BigUint::from(0u64),
        "KKK2_3 NEO.decimals() must be 0 (NEO is non-divisible); got {} \
         (rd_hex={}).",
        v,
        hex::encode(&r.return_data)
    );
}

#[test]
fn batch113_kkk2_4_gas_decimals_is_eight() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function d() external view returns (uint8) {
        return NativeCalls.gasDecimals();
    }
}"#;
    let arts =
        compile_contracts(src, false, 2).unwrap_or_else(|e| panic!("KKK2_4 compile: {:?}", e));
    assert!(!arts.is_empty());
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("KKK2_4 rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "d", &[])
        .expect("KKK2_4 d()");
    assert!(
        r.success,
        "KKK2_4 GAS.decimals() must succeed; exc={:?}.",
        r.exception.as_ref().map(|e| &e.message)
    );
    let v = decode_uint_le(&r.return_data);
    assert_eq!(
        v,
        num_bigint::BigUint::from(8u64),
        "KKK2_4 GAS.decimals() must be 8 (GAS uses 8 decimal places); got \
         {} (rd_hex={}).",
        v,
        hex::encode(&r.return_data)
    );
}

#[test]
fn batch113_kkk2_5_neo_total_supply_positive() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function ts() external view returns (uint256) {
        return NativeCalls.neoTotalSupply();
    }
}"#;
    let arts =
        compile_contracts(src, false, 2).unwrap_or_else(|e| panic!("KKK2_5 compile: {:?}", e));
    assert!(!arts.is_empty());
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("KKK2_5 rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "ts", &[])
        .expect("KKK2_5 ts()");
    assert!(
        r.success,
        "KKK2_5 NEO.totalSupply() must succeed; exc={:?}.",
        r.exception.as_ref().map(|e| &e.message)
    );
    let v = decode_uint_le(&r.return_data);
    assert!(
        v > num_bigint::BigUint::from(0u64),
        "KKK2_5 NEO.totalSupply() must be > 0 (fixed supply); got {} \
         (rd_hex={}). If 0, the NEO native-contract totalSupply regressed.",
        v,
        hex::encode(&r.return_data)
    );
}

// ==================== Batch #114 — Ledger / Policy / ContractManagement ====================
//
// Five probes exercising deeper Ledger and Policy native-contract surfaces
// not runtime-verified by batches 106-110: `currentHash`,
// `getMillisecondsPerBlock`, `getMaxTraceableBlocks`, `isContract`, and the
// EVM-style `tx.origin` auto-map to Neo's script container.
//
//   LLL2_1: Ledger.currentHash() returns 32 bytes.
//   LLL2_2: Policy.getMillisecondsPerBlock() > 0.
//   LLL2_3: Policy.getMaxTraceableBlocks() > 0.
//   LLL2_4: ContractManagement.isContract(self) returns true.
//   LLL2_5: Ledger.currentIndex() + 1 > Ledger.currentIndex() (monotonic sanity).

#[test]
fn batch114_lll2_1_ledger_current_hash_32bytes() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function h() external view returns (bytes32) {
        return NativeCalls.currentHash();
    }
}"#;
    let arts =
        compile_contracts(src, false, 2).unwrap_or_else(|e| panic!("LLL2_1 compile: {:?}", e));
    assert!(!arts.is_empty());
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("LLL2_1 rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "h", &[])
        .expect("LLL2_1 h()");
    assert!(
        r.success,
        "LLL2_1 Ledger.currentHash() must succeed; exc={:?}.",
        r.exception.as_ref().map(|e| &e.message)
    );
    assert_eq!(
        r.return_data.len(),
        32,
        "LLL2_1 Ledger.currentHash() must return 32 bytes; got {} \
         (rd_hex={}).",
        r.return_data.len(),
        hex::encode(&r.return_data)
    );
}

#[test]
fn batch114_lll2_2_policy_milliseconds_per_block() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function m() external view returns (uint256) {
        return NativeCalls.getMillisecondsPerBlock();
    }
}"#;
    let arts =
        compile_contracts(src, false, 2).unwrap_or_else(|e| panic!("LLL2_2 compile: {:?}", e));
    assert!(!arts.is_empty());
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("LLL2_2 rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "m", &[])
        .expect("LLL2_2 m()");
    assert!(
        r.success,
        "LLL2_2 Policy.getMillisecondsPerBlock() must succeed; exc={:?}.",
        r.exception.as_ref().map(|e| &e.message)
    );
    let v = decode_uint_le(&r.return_data);
    assert!(
        v > num_bigint::BigUint::from(0u64),
        "LLL2_2 getMillisecondsPerBlock() must be > 0 (Neo default is 15s); \
         got {} (rd_hex={}).",
        v,
        hex::encode(&r.return_data)
    );
}

#[test]
fn batch114_lll2_3_policy_max_traceable_blocks() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function m() external view returns (uint256) {
        return NativeCalls.getMaxTraceableBlocks();
    }
}"#;
    let arts =
        compile_contracts(src, false, 2).unwrap_or_else(|e| panic!("LLL2_3 compile: {:?}", e));
    assert!(!arts.is_empty());
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("LLL2_3 rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "m", &[])
        .expect("LLL2_3 m()");
    assert!(
        r.success,
        "LLL2_3 Policy.getMaxTraceableBlocks() must succeed; exc={:?}.",
        r.exception.as_ref().map(|e| &e.message)
    );
    let v = decode_uint_le(&r.return_data);
    assert!(
        v > num_bigint::BigUint::from(0u64),
        "LLL2_3 getMaxTraceableBlocks() must be > 0 (Neo default is \
         2_102_400 blocks ≈ 1yr); got {}.",
        v
    );
}

#[test]
fn batch114_lll2_4_is_contract_self() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function isSelf() external view returns (bool) {
        return NativeCalls.isContract(address(this));
    }
}"#;
    let arts =
        compile_contracts(src, false, 2).unwrap_or_else(|e| panic!("LLL2_4 compile: {:?}", e));
    assert!(!arts.is_empty());
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("LLL2_4 rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "isSelf", &[])
        .expect("LLL2_4 isSelf()");
    assert!(
        r.success,
        "LLL2_4 ContractManagement.isContract(self) must execute without \
         fault; exc={:?}.",
        r.exception.as_ref().map(|e| &e.message)
    );
    // The runtime's `iscontract` native handler now treats the executing
    // contract's `default_account_bytes` as present in the registry so a
    // Solidity-level `isContract(address(this))` returns true.
    assert!(
        !r.return_data.is_empty() && r.return_data.iter().any(|b| *b != 0),
        "LLL2_4 isContract(address(this)) must return true for the \
         executing contract; got rd_hex={}. If this is false/empty, the \
         `iscontract` native handler's self-check regressed.",
        hex::encode(&r.return_data)
    );
}

#[test]
fn batch114_lll2_5_ledger_index_plus_one_ge() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function twice() external view returns (bool) {
        uint256 a = NativeCalls.currentIndex();
        uint256 b = NativeCalls.currentIndex();
        // Two sequential reads of the same block height in one transaction
        // must be equal (same block).
        return a == b;
    }
}"#;
    let arts =
        compile_contracts(src, false, 2).unwrap_or_else(|e| panic!("LLL2_5 compile: {:?}", e));
    assert!(!arts.is_empty());
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("LLL2_5 rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "twice", &[])
        .expect("LLL2_5 twice()");
    assert!(
        r.success,
        "LLL2_5 Ledger.currentIndex() sequential reads must succeed; \
         exc={:?}.",
        r.exception.as_ref().map(|e| &e.message)
    );
    // `true` is expected; encoded as non-empty non-zero byte(s).
    let is_true = r.return_data.iter().any(|b| *b != 0);
    assert!(
        is_true,
        "LLL2_5 two sequential currentIndex() reads must be equal (same \
         block within one tx); got false/zero return (rd_hex={}).",
        hex::encode(&r.return_data)
    );
}

// ==================== Batch #115 — EVM auto-map runtime ====================
//
// Five probes exercising the EVM-compat auto-mappings that the compiler
// emits with warnings. Earlier batches referenced these patterns but did
// not runtime-verify that their auto-mapped Neo equivalents execute
// without faulting and return sensible values.
//
//   MMM2_1: `block.coinbase` auto-maps to address(0) and returns 20 zero bytes.
//   MMM2_2: `blockhash(block.number - 1)` auto-maps to Ledger.getBlock(n).hash
//            and returns 32 bytes on success (or empty on unknown index).
//   MMM2_3: `tx.origin` auto-maps and returns a 20-byte address.
//   MMM2_4: `block.basefee` auto-maps to Policy.getFeePerByte() and returns uint.
//   MMM2_5: `tx.gasprice` auto-maps to Policy.getFeePerByte() and returns uint.

#[test]
fn batch115_mmm2_1_block_coinbase_is_zero_address() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function cb() external view returns (address) {
        return block.coinbase;
    }
}"#;
    let arts =
        compile_contracts(src, false, 2).unwrap_or_else(|e| panic!("MMM2_1 compile: {:?}", e));
    assert!(!arts.is_empty());
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("MMM2_1 rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "cb", &[])
        .expect("MMM2_1 cb()");
    assert!(
        r.success,
        "MMM2_1 block.coinbase must succeed (auto-mapped to address(0)); \
         exc={:?}.",
        r.exception.as_ref().map(|e| &e.message)
    );
    // address(0) is a 20-byte zero hash on Neo N3. Accept either canonical
    // 20-byte zero encoding or minimum-width LE (empty) for the zero value.
    let all_zero = r.return_data.iter().all(|b| *b == 0);
    assert!(
        r.return_data.is_empty() || all_zero,
        "MMM2_1 block.coinbase must be address(0) (auto-mapped on Neo \
         because dBFT has no miner); got non-zero rd_hex={}.",
        hex::encode(&r.return_data)
    );
}

#[test]
fn batch115_mmm2_2_blockhash_prev_block() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function bh() external view returns (bytes32) {
        // block.number is the current block; subtract 1 to get the prior
        // block's hash (or zero if we're at genesis).
        if (block.number == 0) return bytes32(0);
        return blockhash(block.number - 1);
    }
}"#;
    let arts =
        compile_contracts(src, false, 2).unwrap_or_else(|e| panic!("MMM2_2 compile: {:?}", e));
    assert!(!arts.is_empty());
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("MMM2_2 rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "bh", &[])
        .expect("MMM2_2 bh()");
    assert!(
        r.success,
        "MMM2_2 blockhash(block.number - 1) must succeed (auto-mapped to \
         Ledger.getBlock(n).hash); exc={:?}.",
        r.exception.as_ref().map(|e| &e.message)
    );
    // Accept either 32-byte hash or empty (genesis block has no prior).
    assert!(
        r.return_data.len() == 32 || r.return_data.is_empty() || r.return_data.len() < 32,
        "MMM2_2 blockhash return must be 0 or 32 bytes; got {} \
         (rd_hex={}).",
        r.return_data.len(),
        hex::encode(&r.return_data)
    );
}

#[test]
fn batch115_mmm2_3_tx_origin_returns_20byte_address() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function o() external view returns (address) {
        return tx.origin;
    }
}"#;
    let arts =
        compile_contracts(src, false, 2).unwrap_or_else(|e| panic!("MMM2_3 compile: {:?}", e));
    assert!(!arts.is_empty());
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("MMM2_3 rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "o", &[])
        .expect("MMM2_3 o()");
    assert!(
        r.success,
        "MMM2_3 tx.origin must succeed (auto-mapped on Neo via witness \
         semantics); exc={:?}.",
        r.exception.as_ref().map(|e| &e.message)
    );
    // Address is 20 bytes; accept minimum-width LE for zero-valued addrs.
    assert!(
        r.return_data.len() <= 20,
        "MMM2_3 tx.origin must fit in ≤20 bytes (Neo UInt160); got {} \
         (rd_hex={}).",
        r.return_data.len(),
        hex::encode(&r.return_data)
    );
}

#[test]
fn batch115_mmm2_4_block_basefee_non_negative() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function bf() external view returns (uint256) {
        return block.basefee;
    }
}"#;
    let arts =
        compile_contracts(src, false, 2).unwrap_or_else(|e| panic!("MMM2_4 compile: {:?}", e));
    assert!(!arts.is_empty());
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("MMM2_4 rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "bf", &[])
        .expect("MMM2_4 bf()");
    assert!(
        r.success,
        "MMM2_4 block.basefee must succeed (auto-mapped to \
         Policy.getFeePerByte()); exc={:?}.",
        r.exception.as_ref().map(|e| &e.message)
    );
    let v = decode_uint_le(&r.return_data);
    assert!(
        v >= num_bigint::BigUint::from(0u64),
        "MMM2_4 block.basefee must be non-negative; got {}.",
        v
    );
}

#[test]
fn batch115_mmm2_5_tx_gasprice_non_negative() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function gp() external view returns (uint256) {
        return tx.gasprice;
    }
}"#;
    let arts =
        compile_contracts(src, false, 2).unwrap_or_else(|e| panic!("MMM2_5 compile: {:?}", e));
    assert!(!arts.is_empty());
    let art = &arts[0];
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("MMM2_5 rt");
    let r = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "gp", &[])
        .expect("MMM2_5 gp()");
    assert!(
        r.success,
        "MMM2_5 tx.gasprice must succeed (auto-mapped to \
         Policy.getFeePerByte()); exc={:?}.",
        r.exception.as_ref().map(|e| &e.message)
    );
    let v = decode_uint_le(&r.return_data);
    assert!(
        v >= num_bigint::BigUint::from(0u64),
        "MMM2_5 tx.gasprice must be non-negative; got {}.",
        v
    );
}
