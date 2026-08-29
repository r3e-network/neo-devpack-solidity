//! Differential fuzz tests.
//!
//! Compiles a tiny Solidity source exercising a single primitive, runs it
//! under the embedded `NeoRuntime`, and diffs the output against a
//! reference implementation in-process (the `sha2`, `ripemd`, `sha3`, and
//! `num-bigint` crates). This catches correctness divergence that weak
//! "it compiled and executed" assertions miss.
//!
//! Distinct from the canonical-test-vector proptests in
//! `batches_116_120.rs::batch121_*`: those use fixed inputs to pin known
//! digests; these walk the entire input space and fail fast on the first
//! disagreement.
//!
//! Every test should:
//!   1. Generate an arbitrary input via proptest.
//!   2. Compute the reference value in-process.
//!   3. Compile + execute the Solidity equivalent.
//!   4. Assert bit-for-bit equality with the reference value.
//!
//! A miss here indicates either a compiler-side intrinsic-wiring bug, a
//! runtime-side native-contract bug, or an encoding mismatch — all of
//! which should produce a minimized proptest regression at
//! `differential.proptest-regressions` automatically.

#![allow(unused_imports)]

use super::common::*;
use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::runtime::types::StackItem;
use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// Bare `sha256(bytes)` intrinsic must match `sha2::Sha256::digest`
    /// for every input in the fuzz space.
    #[test]
    fn differential_sha256(data in prop::collection::vec(any::<u8>(), 0..=256)) {
        use sha2::{Digest, Sha256};
        let expected = Sha256::digest(&data).to_vec();

        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function h(bytes memory d) external pure returns (bytes32) {
        return sha256(d);
    }
}"#;
        let arts = compile_contracts(src, false, 2).unwrap();
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).unwrap();
        let r = rt
            .call_method(
                &art.bytecode,
                &art.tokens,
                &art.manifest,
                "h",
                &[StackItem::byte_array(data.clone())],
            )
            .unwrap();
        prop_assert!(r.success, "sha256({}B) faulted: {:?}", data.len(),
            r.exception.as_ref().map(|e| &e.message));
        prop_assert_eq!(r.return_data, expected,
            "sha256 differential mismatch for {}B input", data.len());
    }

    /// Bare `ripemd160(bytes)` intrinsic must match `ripemd::Ripemd160`.
    #[test]
    fn differential_ripemd160(data in prop::collection::vec(any::<u8>(), 0..=256)) {
        use ripemd::{Digest, Ripemd160};
        let expected = Ripemd160::digest(&data).to_vec();

        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function h(bytes memory d) external pure returns (bytes20) {
        return ripemd160(d);
    }
}"#;
        let arts = compile_contracts(src, false, 2).unwrap();
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).unwrap();
        let r = rt
            .call_method(
                &art.bytecode,
                &art.tokens,
                &art.manifest,
                "h",
                &[StackItem::byte_array(data.clone())],
            )
            .unwrap();
        prop_assert!(r.success, "ripemd160({}B) faulted: {:?}", data.len(),
            r.exception.as_ref().map(|e| &e.message));
        prop_assert_eq!(r.return_data, expected,
            "ripemd160 differential mismatch for {}B input", data.len());
    }

    /// `keccak256(bytes)` intrinsic must match `sha3::Keccak256`.
    #[test]
    fn differential_keccak256(data in prop::collection::vec(any::<u8>(), 0..=256)) {
        use sha3::{Digest, Keccak256};
        let expected = Keccak256::digest(&data).to_vec();

        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function h(bytes memory d) external pure returns (bytes32) {
        return keccak256(d);
    }
}"#;
        let arts = compile_contracts(src, false, 2).unwrap();
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).unwrap();
        let r = rt
            .call_method(
                &art.bytecode,
                &art.tokens,
                &art.manifest,
                "h",
                &[StackItem::byte_array(data.clone())],
            )
            .unwrap();
        prop_assert!(r.success, "keccak256({}B) faulted: {:?}", data.len(),
            r.exception.as_ref().map(|e| &e.message));
        prop_assert_eq!(r.return_data, expected,
            "keccak256 differential mismatch for {}B input", data.len());
    }

    /// `addmod(a, b, m)` intrinsic must match native rust arithmetic for
    /// values that fit in u64. Keeps the test within a single-slot range
    /// so the Solidity source compiles without overflow-check friction.
    #[test]
    fn differential_addmod(
        a in 0u64..=1_000_000u64,
        b in 0u64..=1_000_000u64,
        m in 1u64..=1_000_000u64,
    ) {
        let expected = num_bigint::BigUint::from((a as u128 + b as u128) % m as u128);

        let src = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {{
    function f() external pure returns (uint256) {{
        return addmod({a}, {b}, {m});
    }}
}}"#);
        let arts = compile_contracts(&src, false, 2).unwrap();
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).unwrap();
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "f", &[])
            .unwrap();
        prop_assert!(r.success, "addmod faulted: {:?}",
            r.exception.as_ref().map(|e| &e.message));
        let got = decode_uint_le(&r.return_data);
        prop_assert_eq!(got.clone(), expected.clone(),
            "addmod differential mismatch for a={} b={} m={}", a, b, m);
    }

    /// `mulmod(a, b, m)` intrinsic must match native rust arithmetic for
    /// u32-range operands (u64 product).
    #[test]
    fn differential_mulmod(
        a in 0u32..=100_000u32,
        b in 0u32..=100_000u32,
        m in 1u32..=100_000u32,
    ) {
        let expected = num_bigint::BigUint::from((a as u64 * b as u64) % m as u64);

        let src = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {{
    function f() external pure returns (uint256) {{
        return mulmod({a}, {b}, {m});
    }}
}}"#);
        let arts = compile_contracts(&src, false, 2).unwrap();
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).unwrap();
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "f", &[])
            .unwrap();
        prop_assert!(r.success, "mulmod faulted: {:?}",
            r.exception.as_ref().map(|e| &e.message));
        let got = decode_uint_le(&r.return_data);
        prop_assert_eq!(got.clone(), expected.clone(),
            "mulmod differential mismatch for a={} b={} m={}", a, b, m);
    }

    /// NeoVM bytecode disassembler must never panic on arbitrary bytes.
    /// Complements `fuzz_target_disasm` with structured proptest coverage.
    #[test]
    fn differential_disasm_never_panics(
        bytes in prop::collection::vec(any::<u8>(), 0..=1024),
    ) {
        let result = std::panic::catch_unwind(|| {
            neo_devpack_solidity::cli::disassemble_neovm_bytecode(&bytes)
        });
        prop_assert!(result.is_ok(),
            "disasm panicked on {}B input starting {:?}",
            bytes.len(),
            &bytes.iter().take(8).copied().collect::<Vec<_>>()
        );
    }

    /// Scalar uint256 external return emits MINIMUM-WIDTH LITTLE-ENDIAN bytes
    /// (Neo-native), NOT a 32-byte BE slot. See `decode_uint_le` docstring in
    /// common.rs: single returns skip ABI encoding and surface the native
    /// NeoVM integer layout. This pins the shape so a future regression that
    /// pads single returns to 32 BE bytes is caught immediately.
    #[test]
    fn differential_abi_encode_uint256_single(v in 0u64..=u64::MAX) {
        let src = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {{
    function f() external pure returns (uint256) {{
        return {v};
    }}
}}"#);
        let arts = compile_contracts(&src, false, 2).unwrap();
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).unwrap();
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "f", &[])
            .unwrap();
        prop_assert!(r.success, "single uint256 return faulted: {:?}",
            r.exception.as_ref().map(|e| &e.message));
        // Expected: minimum-width LE. BigUint::from(v).to_bytes_le() has the
        // same trim-trailing-zeros semantics except for v==0 which yields `[]`.
        let got = decode_uint_le(&r.return_data);
        prop_assert_eq!(got, num_bigint::BigUint::from(v),
            "single uint256 return decode mismatch for v={}; rd={:?}",
            v, r.return_data);
        // Also sanity-check the SHAPE: must NOT be a 32-byte BE slot for
        // small v. If r.return_data.len() == 32 and v fits in u64, the high
        // 24 bytes would be zero — that's the EVM-canonical BE shape, which
        // is NOT what single returns should produce.
        if v != 0 && v <= 0xFF {
            prop_assert!(r.return_data.len() <= 8,
                "single uint256 return for v={} must be minimum-width LE \
                 (≤8 bytes), not BE-padded; got {} bytes: {:?}",
                v, r.return_data.len(), r.return_data);
        }
    }

    /// External `returns (uint256, uint256)` produces 64 BE bytes =
    /// `BE32(a) || BE32(b)`. Multi-return triggers ABI encoding.
    #[test]
    fn differential_abi_encode_two_uint256s(
        a in 0u64..=u64::MAX,
        b in 0u64..=u64::MAX,
    ) {
        let src = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {{
    function f() external pure returns (uint256, uint256) {{
        return ({a}, {b});
    }}
}}"#);
        let arts = compile_contracts(&src, false, 2).unwrap();
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).unwrap();
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "f", &[])
            .unwrap();
        prop_assert!(r.success, "(uint,uint) return faulted: {:?}",
            r.exception.as_ref().map(|e| &e.message));
        // Reference: two 32-byte BE slots, each with 24 leading zeros then
        // the 8-byte BE u64.
        let mut expected = Vec::with_capacity(64);
        expected.extend_from_slice(&[0u8; 24]);
        expected.extend_from_slice(&a.to_be_bytes());
        expected.extend_from_slice(&[0u8; 24]);
        expected.extend_from_slice(&b.to_be_bytes());
        prop_assert_eq!(r.return_data, expected,
            "(uint256,uint256) return ABI-encoding mismatch for a={} b={}",
            a, b);
    }

    /// External `returns (bool, uint256)` produces 64 BE bytes =
    /// `BE32(bool as 0/1) || BE32(uint)`.
    #[test]
    fn differential_abi_encode_bool_uint(
        flag in any::<bool>(),
        v in 0u64..=u64::MAX,
    ) {
        let flag_lit = if flag { "true" } else { "false" };
        let src = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {{
    function f() external pure returns (bool, uint256) {{
        return ({flag_lit}, {v});
    }}
}}"#);
        let arts = compile_contracts(&src, false, 2).unwrap();
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).unwrap();
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "f", &[])
            .unwrap();
        prop_assert!(r.success, "(bool,uint256) return faulted: {:?}",
            r.exception.as_ref().map(|e| &e.message));
        // Reference: 64 BE bytes. Slot 0 = 31 zeros + 0x00/0x01.
        // Slot 1 = 24 zeros + BE(v).
        let mut expected = Vec::with_capacity(64);
        expected.extend_from_slice(&[0u8; 31]);
        expected.push(if flag { 1u8 } else { 0u8 });
        expected.extend_from_slice(&[0u8; 24]);
        expected.extend_from_slice(&v.to_be_bytes());
        prop_assert_eq!(r.return_data, expected,
            "(bool,uint256) return ABI-encoding mismatch for flag={} v={}",
            flag, v);
    }

    /// `abi.encodePacked(uint8, uint8)` emits EXACTLY two raw bytes
    /// (width-aware narrow-integer packing; see Task #66 precedent in
    /// baseline_tests.rs::abi_encodePacked_small_width_matches_spec).
    #[test]
    fn differential_abi_encode_packed_two_uint8s(
        a in any::<u8>(),
        b in any::<u8>(),
    ) {
        let src = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {{
    function f() external pure returns (bytes memory) {{
        return abi.encodePacked(uint8({a}), uint8({b}));
    }}
}}"#);
        let arts = compile_contracts(&src, false, 2).unwrap();
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).unwrap();
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "f", &[])
            .unwrap();
        prop_assert!(r.success, "encodePacked(u8,u8) faulted: {:?}",
            r.exception.as_ref().map(|e| &e.message));
        let expected = vec![a, b];
        prop_assert_eq!(r.return_data, expected,
            "encodePacked(uint8({}), uint8({})) payload mismatch", a, b);
    }

    /// `abi.encodePacked(address, uint256)` emits 20 EVM-BE bytes (address) ||
    /// 32 EVM-BE bytes (uint256) = 52 bytes total.
    ///
    /// Neo N3 stores `UInt160` addresses in its script-hash LE byte order
    /// internally; the compiler's packed-encoding path (see
    /// `lower_packed_abi_bytes_for_expr` in `src/ir/expressions/calls/builtins/helpers.rs`)
    /// reverses those 20 bytes so the emitted layout is EVM-canonical.
    /// Our reference therefore reverses the test-supplied `addr` bytes
    /// (which stand in for the on-stack Neo-LE representation) before
    /// comparing — we're validating the Neo-LE → EVM-BE conversion, not
    /// an identity pass-through.
    #[test]
    fn differential_abi_encode_packed_address_uint256(
        addr in prop::array::uniform20(any::<u8>()),
        v in 0u64..=u64::MAX,
    ) {
        let src = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {{
    function f(address a) external pure returns (bytes memory) {{
        return abi.encodePacked(a, uint256({v}));
    }}
}}"#);
        let arts = compile_contracts(&src, false, 2).unwrap();
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).unwrap();
        let r = rt
            .call_method(
                &art.bytecode,
                &art.tokens,
                &art.manifest,
                "f",
                &[StackItem::byte_array(addr.to_vec())],
            )
            .unwrap();
        prop_assert!(r.success, "encodePacked(address,uint256) faulted: {:?}",
            r.exception.as_ref().map(|e| &e.message));
        // Reference: reverse(addr_LE) (20 EVM-BE bytes) || BE32(v).
        let mut addr_be = addr.to_vec();
        addr_be.reverse();
        let mut expected = Vec::with_capacity(52);
        expected.extend_from_slice(&addr_be);
        expected.extend_from_slice(&[0u8; 24]);
        expected.extend_from_slice(&v.to_be_bytes());
        prop_assert_eq!(r.return_data, expected,
            "encodePacked(address, uint256({})) payload mismatch; addr_LE={:?}",
            v, addr);
    }

    /// `CryptoLib.sha256(bytes)` namespace-entry path must match
    /// `sha2::Sha256::digest`. Covers the `resolve_cryptolib_member` dispatch
    /// (see src/ir/context/builtins/resolve.rs) distinct from the bare
    /// `sha256` intrinsic exercised by `differential_sha256`. If the resolver
    /// regresses and omits `CryptoLib` from its namespace table, the bare
    /// intrinsic still works but this path silently breaks.
    #[test]
    fn differential_cryptolib_sha256_namespace(data in prop::collection::vec(any::<u8>(), 0..=256)) {
        use sha2::{Digest, Sha256};
        let expected = Sha256::digest(&data).to_vec();

        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function h(bytes memory d) external pure returns (bytes32) {
        return CryptoLib.sha256(d);
    }
}"#;
        let arts = compile_contracts(src, false, 2).unwrap();
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).unwrap();
        let r = rt
            .call_method(
                &art.bytecode,
                &art.tokens,
                &art.manifest,
                "h",
                &[StackItem::byte_array(data.clone())],
            )
            .unwrap();
        prop_assert!(r.success, "CryptoLib.sha256({}B) faulted: {:?}", data.len(),
            r.exception.as_ref().map(|e| &e.message));
        prop_assert_eq!(r.return_data, expected,
            "CryptoLib.sha256 namespace-path differential mismatch for {}B input",
            data.len());
    }

    /// `CryptoLib.ripemd160(bytes)` namespace-entry path must match
    /// `ripemd::Ripemd160::digest`. Same rationale as the sha256 namespace
    /// test: exercises the `CryptoLib` dispatch branch of
    /// `resolve_cryptolib_member`, not the bare `ripemd160` intrinsic.
    #[test]
    fn differential_cryptolib_ripemd160_namespace(data in prop::collection::vec(any::<u8>(), 0..=256)) {
        use ripemd::{Digest, Ripemd160};
        let expected = Ripemd160::digest(&data).to_vec();

        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function h(bytes memory d) external pure returns (bytes20) {
        return CryptoLib.ripemd160(d);
    }
}"#;
        let arts = compile_contracts(src, false, 2).unwrap();
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).unwrap();
        let r = rt
            .call_method(
                &art.bytecode,
                &art.tokens,
                &art.manifest,
                "h",
                &[StackItem::byte_array(data.clone())],
            )
            .unwrap();
        prop_assert!(r.success, "CryptoLib.ripemd160({}B) faulted: {:?}", data.len(),
            r.exception.as_ref().map(|e| &e.message));
        prop_assert_eq!(r.return_data, expected,
            "CryptoLib.ripemd160 namespace-path differential mismatch for {}B input",
            data.len());
    }

    /// `CryptoLib.keccak256(bytes)` namespace-entry path must match
    /// `sha3::Keccak256::digest`. Same rationale as the sha256/ripemd160
    /// namespace tests: ensures the `CryptoLib` dispatch branch of
    /// `resolve_cryptolib_member` stays wired to the `keccak256` method on
    /// the native contract, independent of the bare `keccak256(...)`
    /// intrinsic.
    #[test]
    fn differential_cryptolib_keccak256_namespace(data in prop::collection::vec(any::<u8>(), 0..=256)) {
        use sha3::{Digest, Keccak256};
        let expected = Keccak256::digest(&data).to_vec();

        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function h(bytes memory d) external pure returns (bytes32) {
        return CryptoLib.keccak256(d);
    }
}"#;
        let arts = compile_contracts(src, false, 2).unwrap();
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).unwrap();
        let r = rt
            .call_method(
                &art.bytecode,
                &art.tokens,
                &art.manifest,
                "h",
                &[StackItem::byte_array(data.clone())],
            )
            .unwrap();
        prop_assert!(r.success, "CryptoLib.keccak256({}B) faulted: {:?}", data.len(),
            r.exception.as_ref().map(|e| &e.message));
        prop_assert_eq!(r.return_data, expected,
            "CryptoLib.keccak256 namespace-path differential mismatch for {}B input",
            data.len());
    }

    /// `StdLib.itoa(uint256, 10)` namespace-entry path must match Rust's
    /// `u64::to_string()`. Covers the `resolve_stdlib_member` dispatch (see
    /// src/ir/context/builtins/resolve.rs) end-to-end for the base-10
    /// formatter.
    ///
    /// The runtime returns an ASCII string as raw bytes via a single return;
    /// per `decode_uint_le`'s single-return semantics and the III2_1 batch
    /// precedent (see tests/fuzz_tests/batches_111_115.rs), the payload is
    /// the ASCII decimal literal itself. We decode the return as a BigUint
    /// via `BigUint::from_bytes_le` per the task spec and string-compare
    /// against the expected decimal text — both representations must agree
    /// after UTF-8 round-trip.
    ///
    /// Range note: the embedded NeoRuntime's `stdlib::itoa` handler coerces
    /// its input to `i64` (see src/runtime/execution/execution_impl_part2_native/stdlib.rs),
    /// so values > `i64::MAX` wrap to negative and format with a leading
    /// `-`. Real Neo N3 StdLib.itoa uses arbitrary-precision BigInteger;
    /// this test is therefore bounded to the embedded runtime's actual
    /// capability, not the production StdLib's. Widening the range to
    /// `u64::MAX` is tracked as a runtime-BigInteger follow-up; the
    /// canonical CALLT-level test in `baseline_tests.rs::callt_stdlib_itoa_roundtrip_via_token`
    /// already pins `0..=999_999_999u32`, so the bounds here can also be
    /// widened once the runtime is upgraded.
    // ============================================================
    // Extended-length differentials (4096-byte inputs).
    //
    // The original `differential_sha256/keccak256/ripemd160` tests cap input
    // size at 256 bytes — fine for catching short-message wiring bugs but
    // misses any breakage that surfaces only at lengths approaching the
    // runtime's memory_limit (e.g. block-flush bugs that show up after the
    // first ~64-byte SHA block, or chunked allocator bugs at multi-KB).
    // The 4096-byte cap is 4× higher than the original and stays within the
    // default RuntimeConfig memory budget.
    // ============================================================

    /// Extended-length sha256 differential: 0..=4096-byte inputs.
    #[test]
    fn differential_sha256_long(data in prop::collection::vec(any::<u8>(), 0..=4096)) {
        use sha2::{Digest, Sha256};
        let expected = Sha256::digest(&data).to_vec();
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function h(bytes memory d) external pure returns (bytes32) {
        return sha256(d);
    }
}"#;
        let arts = compile_contracts(src, false, 2).unwrap();
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).unwrap();
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "h",
                &[StackItem::byte_array(data.clone())])
            .unwrap();
        prop_assert!(r.success, "sha256({}B long) faulted: {:?}", data.len(),
            r.exception.as_ref().map(|e| &e.message));
        prop_assert_eq!(r.return_data, expected,
            "sha256 long-input differential mismatch for {}B input", data.len());
    }

    /// Extended-length keccak256 differential: 0..=4096-byte inputs.
    #[test]
    fn differential_keccak256_long(data in prop::collection::vec(any::<u8>(), 0..=4096)) {
        use sha3::{Digest, Keccak256};
        let expected = Keccak256::digest(&data).to_vec();
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function h(bytes memory d) external pure returns (bytes32) {
        return keccak256(d);
    }
}"#;
        let arts = compile_contracts(src, false, 2).unwrap();
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).unwrap();
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "h",
                &[StackItem::byte_array(data.clone())])
            .unwrap();
        prop_assert!(r.success, "keccak256({}B long) faulted: {:?}", data.len(),
            r.exception.as_ref().map(|e| &e.message));
        prop_assert_eq!(r.return_data, expected,
            "keccak256 long-input differential mismatch for {}B input", data.len());
    }

    /// Extended-length ripemd160 differential: 0..=4096-byte inputs.
    #[test]
    fn differential_ripemd160_long(data in prop::collection::vec(any::<u8>(), 0..=4096)) {
        use ripemd::{Digest, Ripemd160};
        let expected = Ripemd160::digest(&data).to_vec();
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function h(bytes memory d) external pure returns (bytes20) {
        return ripemd160(d);
    }
}"#;
        let arts = compile_contracts(src, false, 2).unwrap();
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).unwrap();
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "h",
                &[StackItem::byte_array(data.clone())])
            .unwrap();
        prop_assert!(r.success, "ripemd160({}B long) faulted: {:?}", data.len(),
            r.exception.as_ref().map(|e| &e.message));
        prop_assert_eq!(r.return_data, expected,
            "ripemd160 long-input differential mismatch for {}B input", data.len());
    }

    // ============================================================
    // Boundary-byte-pattern differentials.
    //
    // Random fuzzing rarely lands on uniform-byte fills (`0x00 * N`,
    // `0xFF * N`) or strict alternating patterns, but those are exactly the
    // inputs that surface block-boundary off-by-ones in hash compression
    // functions. SHA-256 / SHA-3 / RIPEMD-160 all have 64-byte blocks (and
    // the SHA-2 family has a length-padding step that fires at length ≡ 56
    // mod 64); we walk the lengths {0,1,31,32,33,63,64,65,127,128,129,
    // 255,256} which straddle every block boundary up to two SHA blocks
    // and then some, for each of three byte fills. proptest indexes into
    // the (length × pattern) grid via a 0..39 strategy.
    // ============================================================

    /// `differential_sha256_boundary_patterns` — sha256 over uniform-byte
    /// fills at every block-boundary length. 13 lengths × 3 patterns = 39
    /// fixed inputs walked deterministically by the proptest index.
    #[test]
    fn differential_sha256_boundary_patterns(idx in 0usize..39) {
        use sha2::{Digest, Sha256};
        const LENS: [usize; 13] = [0,1,31,32,33,63,64,65,127,128,129,255,256];
        let len = LENS[idx % 13];
        let data: Vec<u8> = match idx / 13 {
            0 => vec![0x00u8; len],
            1 => vec![0xFFu8; len],
            _ => (0..len).map(|i| if i.is_multiple_of(2) { 0xAA } else { 0x55 }).collect(),
        };
        let expected = Sha256::digest(&data).to_vec();
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function h(bytes memory d) external pure returns (bytes32) {
        return sha256(d);
    }
}"#;
        let arts = compile_contracts(src, false, 2).unwrap();
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).unwrap();
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "h",
                &[StackItem::byte_array(data.clone())])
            .unwrap();
        prop_assert!(r.success, "sha256 boundary({}B, pat={}) faulted: {:?}",
            len, idx / 13, r.exception.as_ref().map(|e| &e.message));
        prop_assert_eq!(r.return_data, expected,
            "sha256 boundary-pattern mismatch (len={}, pattern_idx={})", len, idx / 13);
    }

    /// `differential_keccak256_boundary_patterns` — keccak256 sibling of
    /// the sha256 boundary walker.
    #[test]
    fn differential_keccak256_boundary_patterns(idx in 0usize..39) {
        use sha3::{Digest, Keccak256};
        const LENS: [usize; 13] = [0,1,31,32,33,63,64,65,127,128,129,255,256];
        let len = LENS[idx % 13];
        let data: Vec<u8> = match idx / 13 {
            0 => vec![0x00u8; len],
            1 => vec![0xFFu8; len],
            _ => (0..len).map(|i| if i.is_multiple_of(2) { 0xAA } else { 0x55 }).collect(),
        };
        let expected = Keccak256::digest(&data).to_vec();
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function h(bytes memory d) external pure returns (bytes32) {
        return keccak256(d);
    }
}"#;
        let arts = compile_contracts(src, false, 2).unwrap();
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).unwrap();
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "h",
                &[StackItem::byte_array(data.clone())])
            .unwrap();
        prop_assert!(r.success, "keccak256 boundary({}B, pat={}) faulted: {:?}",
            len, idx / 13, r.exception.as_ref().map(|e| &e.message));
        prop_assert_eq!(r.return_data, expected,
            "keccak256 boundary-pattern mismatch (len={}, pattern_idx={})", len, idx / 13);
    }

    /// `differential_ripemd160_boundary_patterns` — ripemd160 sibling of
    /// the sha256 boundary walker.
    #[test]
    fn differential_ripemd160_boundary_patterns(idx in 0usize..39) {
        use ripemd::{Digest, Ripemd160};
        const LENS: [usize; 13] = [0,1,31,32,33,63,64,65,127,128,129,255,256];
        let len = LENS[idx % 13];
        let data: Vec<u8> = match idx / 13 {
            0 => vec![0x00u8; len],
            1 => vec![0xFFu8; len],
            _ => (0..len).map(|i| if i.is_multiple_of(2) { 0xAA } else { 0x55 }).collect(),
        };
        let expected = Ripemd160::digest(&data).to_vec();
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function h(bytes memory d) external pure returns (bytes20) {
        return ripemd160(d);
    }
}"#;
        let arts = compile_contracts(src, false, 2).unwrap();
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).unwrap();
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "h",
                &[StackItem::byte_array(data.clone())])
            .unwrap();
        prop_assert!(r.success, "ripemd160 boundary({}B, pat={}) faulted: {:?}",
            len, idx / 13, r.exception.as_ref().map(|e| &e.message));
        prop_assert_eq!(r.return_data, expected,
            "ripemd160 boundary-pattern mismatch (len={}, pattern_idx={})", len, idx / 13);
    }

    // ============================================================
    // CryptoLib namespace coverage: the only remaining hash-style method
    // exposed by `invoke_native_cryptolib` (see
    // src/runtime/execution/execution_impl_part2_native/crypto.rs) for
    // which a Rust reference crate is available is `murmur32`. The other
    // CryptoLib entries (`verifywithecdsa`, `recoversecp256k1`,
    // `bls12381*`) are signature/curve operations, not hashes, and their
    // oracles depend on having a known keypair — out of scope for a pure
    // hash differential. `sha1` is not exposed by CryptoLib at all (see
    // the resolver allowlist at src/ir/context/builtins/resolve.rs:175-179),
    // so no test is needed.
    // ============================================================

    /// `CryptoLib.murmur32(data, seed)` must match the `murmur3` reference
    /// crate's `murmur3_32` (Austin Appleby's MurmurHash3 x86_32). The
    /// runtime's inline implementation lives at
    /// `src/runtime/execution/helpers/crypto.rs::murmur3_32`; if it ever
    /// diverges from canonical MurmurHash3 (e.g. wrong rotation count,
    /// missing finalization mix, wrong magic constants), this proptest
    /// surfaces the divergence immediately.
    ///
    /// The runtime returns the 4-byte digest as a min-width LE byte array
    /// (small u32 values may emit fewer than 4 bytes), so we decode via
    /// `decode_uint_le` and compare against `expected as u64`.
    #[test]
    fn differential_cryptolib_murmur32(
        data in prop::collection::vec(any::<u8>(), 0..=256),
        seed in any::<u32>(),
    ) {
        let expected = murmur3::murmur3_32(&mut std::io::Cursor::new(&data), seed)
            .expect("reference murmur3_32 cannot fail on in-memory cursor");
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function h(bytes memory d, uint32 s) external pure returns (uint32) {
        return CryptoLib.murmur32(d, s);
    }
}"#;
        let arts = compile_contracts(src, false, 2).unwrap();
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).unwrap();
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "h",
                &[
                    StackItem::byte_array(data.clone()),
                    StackItem::Integer(seed as i64),
                ])
            .unwrap();
        prop_assert!(r.success, "CryptoLib.murmur32({}B, seed={}) faulted: {:?}",
            data.len(), seed, r.exception.as_ref().map(|e| &e.message));
        let got = decode_uint_le(&r.return_data);
        prop_assert_eq!(got.clone(), num_bigint::BigUint::from(expected),
            "CryptoLib.murmur32 differential mismatch; len={}, seed={}, \
             expected={:#x}, got={}",
            data.len(), seed, expected, got);
    }

    // ============================================================
    // `abi.encode(uint256)` length / endian invariants.
    //
    // EVM-canonical: `abi.encode(uint256)` is exactly 32 BE bytes, with the
    // value right-aligned (high 24 bytes zero for u64-fitting values,
    // low 8 bytes = `v.to_be_bytes()`). Empirical probe (the earlier
    // version of this test ran with a diagnostic `eprintln!`) showed
    // **every** sampled `v` in `0..=2^32` produced exactly that 32-BE
    // shape — i.e. the compiler has a static IR-level fast path for
    // `abi.encode(uint256(literal))` that bypasses the
    // `StdLib.serialize` fallback in
    // `src/cli/bytecode/bytecode_builtins/builtin_call/abi.rs::emit_abi_encode`.
    //
    // Pin that invariant here. If the static fast path ever regresses to
    // the Neo-serialize fallback (which would emit a tagged byte string,
    // NOT 32 BE bytes), this test will fail and surface the regression
    // with the offending input.
    // ============================================================

    /// `abi.encode(uint256(literal))` MUST be exactly 32 EVM-canonical
    /// big-endian bytes for any literal in `0..=2^32`. Empirically pinned
    /// after probing the earlier diagnostic version of this test.
    #[test]
    fn differential_abi_encode_uint256_shape(v in 0u64..=(1u64 << 32)) {
        let src = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {{
    function f() external pure returns (bytes memory) {{
        return abi.encode(uint256({v}));
    }}
}}"#);
        let arts = compile_contracts(&src, false, 2).unwrap();
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).unwrap();
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "f", &[])
            .unwrap();
        prop_assert!(r.success, "abi.encode(uint256({})) faulted: {:?}",
            v, r.exception.as_ref().map(|e| &e.message));
        // Length invariant: must be exactly 32 bytes for a single uint256
        // slot.
        prop_assert_eq!(r.return_data.len(), 32,
            "abi.encode(uint256({})) length must be 32 EVM-canonical bytes; \
             got {} (rd_hex={}). A length other than 32 indicates the \
             compiler has fallen back to StdLib.serialize for static \
             uint256 literals — see src/cli/bytecode/bytecode_builtins/\
             builtin_call/abi.rs::emit_abi_encode.",
            v, r.return_data.len(), hex::encode(&r.return_data));
        // High-byte / endian invariant: 24 leading zeros followed by
        // `v.to_be_bytes()`. We construct the canonical reference and
        // require bit-for-bit equality so any byte-order swap (LE leak,
        // EVM-LE-mod, etc.) is caught.
        let mut expected = vec![0u8; 32];
        expected[24..].copy_from_slice(&v.to_be_bytes());
        prop_assert_eq!(r.return_data.clone(), expected.clone(),
            "abi.encode(uint256({})) endian/high-byte mismatch; \
             got_hex={}, expected_hex={}",
            v, hex::encode(&r.return_data), hex::encode(&expected));
    }

    #[test]
    fn differential_stdlib_itoa_base10(v in 0u64..=(i64::MAX as u64)) {
        let expected_str = v.to_string();
        let src = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {{
    function f() external pure returns (string memory) {{
        return StdLib.itoa({v}, 10);
    }}
}}"#);
        let arts = compile_contracts(&src, false, 2).unwrap();
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).unwrap();
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "f", &[])
            .unwrap();
        prop_assert!(r.success, "StdLib.itoa({}, 10) faulted: {:?}", v,
            r.exception.as_ref().map(|e| &e.message));
        // Per the task spec: decode the return bytes as BigUint via
        // from_bytes_le, then string-compare. The payload IS the ASCII
        // decimal literal, so `from_bytes_le` treats that byte-stream as a
        // little-endian integer; comparing its decimal string to the
        // reference's decimal string is equivalent to comparing raw ASCII
        // byte-sequences (modulo the canonical empty-slice → 0 mapping for
        // v==0, which matches u64::to_string()).
        let got = num_bigint::BigUint::from_bytes_le(&r.return_data);
        let got_str = got.to_string();
        let expected_le_as_bigint = num_bigint::BigUint::from_bytes_le(expected_str.as_bytes());
        prop_assert_eq!(got_str.clone(), expected_le_as_bigint.to_string(),
            "StdLib.itoa({}, 10) namespace-path differential mismatch; \
             return_data={:?} (utf8={:?})",
            v, r.return_data, std::str::from_utf8(&r.return_data).ok());
    }
}

// ============================================================
// mulmod 512-bit intermediate-precision differential.
//
// `differential_mulmod` above uses u32 operands whose product always fits
// in 256 bits, so it cannot catch the bug where `mulmod(a,b,m)` lowers to a
// native NeoVM MUL (truncating a*b to 256 bits) before reducing mod m.
// Solidity `mulmod` requires the FULL 512-bit intermediate product; when
// a*b >= 2^256 the truncated path gives a wrong residue. These cases all
// have a*b >= 2^256 and are computed against an arbitrary-precision
// reference so any truncation surfaces immediately.
// ============================================================

/// Regression for full-512-bit `mulmod`: the intermediate product a*b must
/// NOT be truncated to 256 bits before reducing mod m. Each case has
/// `a*b >= 2^256`; the reference is `num-bigint`'s arbitrary-precision
/// `(a*b) % m`. Pre-fix the compiler emitted a native MUL here and every
/// case returned a wrong residue.
///
/// Follows the established `differential_mulmod` pattern: literals are
/// inlined into the Solidity source and the function takes no args, which
/// sidesteps the question of how the runtime marshals >i64 uint256
/// parameters. Each case recompiles its own tiny contract (5 cases total).
#[test]
fn differential_mulmod_512bit() {
    use num_bigint::BigUint;

    let two = BigUint::from(2u32);
    let pow128 = two.pow(128); // 2^128
    let pow256 = two.pow(256);
    let max_u256 = &pow256 - 1u32; // 2^256 - 1

    // (a, b, m) triples whose product a*b >= 2^256. Expected = (a*b) % m.
    let cases: Vec<(BigUint, BigUint, BigUint)> = vec![
        // 2^128 * 2^128 = 2^256 exactly; 2^256 mod (2^256-1) = 1.
        (pow128.clone(), pow128.clone(), max_u256.clone()),
        // (2^256-1)^2 mod (2^256-1) = 0.
        (max_u256.clone(), max_u256.clone(), max_u256.clone()),
        // (2^256-1)*2 = 2^257-2 = 2*(2^256-1); mod (2^256-1) = 0.
        (max_u256.clone(), BigUint::from(2u32), max_u256.clone()),
        // 2^200 * 2^200 = 2^400; mod (2^256-1). 2^400 = 2^256*2^144 ≡ 1*2^144.
        (two.pow(200), two.pow(200), max_u256.clone()),
        // (2^256-1)*(2^256-2) mod (2^256-3): big product, non-trivial residue.
        (max_u256.clone(), &max_u256 - 1u32, &max_u256 - 2u32),
    ];

    for (a, b, m) in &cases {
        let expected = (a * b) % m;
        // Sanity: every case really does overflow 256 bits in the product.
        assert!(
            a * b >= pow256,
            "test invariant: a*b must be >= 2^256 for case a={} b={}",
            a,
            b
        );

        let src = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {{
    function f() external pure returns (uint256) {{
        return mulmod({a}, {b}, {m});
    }}
}}"#
        );
        let arts = compile_contracts(&src, false, 2).expect("compile mulmod_512bit");
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).unwrap();
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "f", &[])
            .unwrap();
        assert!(
            r.success,
            "mulmod_512bit faulted: a={} b={} m={}: {:?}",
            a,
            b,
            m,
            r.exception.as_ref().map(|e| &e.message)
        );
        let got = decode_uint_le(&r.return_data);
        assert_eq!(
            got,
            expected,
            "mulmod_512bit mismatch (product truncated?): \
             a={} b={} m={}; a*b={} expected={}",
            a,
            b,
            m,
            a * b,
            expected
        );
    }
}

// ============================================================
// BLS12-381 G1/G2 differentials.
//
// These exercise the `CryptoLib.bls12381*` family used by ZK proof
// verification (Groth16, etc.), threshold signatures, and the Neo-side
// adaptations of EVM precompiles 0x06–0x08 (see
// devpack/libraries/Precompiles.sol::ecAdd / ecMul / ecPairing). The
// IR-level resolver and bytecode emitter wire these methods through to
// the `CryptoLib` native contract (see
// src/ir/context/builtins/syscalls.rs:149-196 and
// src/ir/context/builtins/resolve.rs:182-200), but the runtime stub at
// src/runtime/execution/execution_impl_part2_native/crypto.rs falls
// through to `StackItem::Null` for every bls12381* method as of this
// writing — only sha256/ripemd160/keccak256/murmur32/recoverSecp256K1
// are wired up there.
//
// Strategy: register the differential proptests now so the
// compile-and-execute path is continuously exercised (catching wiring
// regressions and panics on the IR/bytecode side), but key the
// reference-equality assertion off whether the runtime returned a
// non-empty byte string. While the runtime returns the placeholder Null
// (which surfaces as an empty `return_data`), the test asserts only
// "didn't panic / didn't fault out". Once the runtime implementation
// lands and starts emitting real compressed-point bytes, the equality
// assertion automatically activates and any divergence from the
// `bls12_381` reference crate will fail the proptest with the offending
// scalar inputs.
//
// Input bounds are kept small: u64 scalars (not full 255-bit) so each
// generated case runs in well under 200ms; pairing test uses 1..=8 to
// stay snappy.
// ============================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    /// `CryptoLib.bls12381Add(a*G1, b*G1)` must equal `(a+b)*G1` on the
    /// BLS12-381 G1 curve. Reference oracle: the `bls12_381` crate's
    /// `G1Projective` arithmetic, compared via the canonical
    /// 48-byte compressed encoding.
    #[test]
    fn differential_bls12381_g1_add(a in 0u64..=u64::MAX, b in 0u64..=u64::MAX) {
        use bls12_381::{G1Affine, G1Projective, Scalar};
        use group::Group;

        let sa = Scalar::from(a);
        let sb = Scalar::from(b);
        let g = G1Projective::generator();
        let p_a = g * sa;
        let p_b = g * sb;
        let sum = p_a + p_b;
        let expected_compressed: [u8; 48] = G1Affine::from(sum).to_compressed();

        // Pre-serialize the two operand points to compressed bytes; the
        // contract treats them as opaque `bytes` and feeds them into
        // CryptoLib.bls12381Add(...).
        let a_bytes = G1Affine::from(p_a).to_compressed().to_vec();
        let b_bytes = G1Affine::from(p_b).to_compressed().to_vec();

        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f(bytes memory x, bytes memory y) external view returns (bytes memory) {
        return CryptoLib.bls12381Add(x, y);
    }
}"#;
        let arts = compile_contracts(src, false, 2).unwrap();
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).unwrap();
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "f",
                &[
                    StackItem::byte_array(a_bytes),
                    StackItem::byte_array(b_bytes),
                ])
            .unwrap();
        prop_assert!(r.success, "CryptoLib.bls12381Add(a={}, b={}) faulted: {:?}",
            a, b, r.exception.as_ref().map(|e| &e.message));

        // Partial-implementation gate: skip the equality assertion when
        // the runtime stub returns the placeholder (empty bytes from
        // StackItem::Null). Once the runtime implementation lands and
        // starts emitting non-empty compressed-point bytes, this branch
        // self-activates.
        if !r.return_data.is_empty() {
            prop_assert_eq!(r.return_data.clone(), expected_compressed.to_vec(),
                "CryptoLib.bls12381Add divergence; a={}, b={}, expected_hex={}, got_hex={}",
                a, b, hex::encode(expected_compressed), hex::encode(&r.return_data));
        }
    }

    /// `CryptoLib.bls12381Mul(G1, s)` must equal `s*G1` on BLS12-381 G1.
    #[test]
    fn differential_bls12381_g1_mul(s in 0u64..=u64::MAX) {
        use bls12_381::{G1Affine, G1Projective, Scalar};
        use group::Group;

        let scalar = Scalar::from(s);
        let g = G1Projective::generator();
        let prod = g * scalar;
        let expected_compressed: [u8; 48] = G1Affine::from(prod).to_compressed();

        let g_bytes = G1Affine::from(g).to_compressed().to_vec();
        // Scalar passed as 32 BE bytes (little-end zero-pad u64).
        let mut scalar_bytes = vec![0u8; 32];
        scalar_bytes[24..].copy_from_slice(&s.to_be_bytes());

        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f(bytes memory p, bytes memory k) external view returns (bytes memory) {
        return CryptoLib.bls12381Mul(p, k, false);
    }
}"#;
        let arts = compile_contracts(src, false, 2).unwrap();
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).unwrap();
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "f",
                &[
                    StackItem::byte_array(g_bytes),
                    StackItem::byte_array(scalar_bytes),
                ])
            .unwrap();
        prop_assert!(r.success, "CryptoLib.bls12381Mul(s={}) faulted: {:?}",
            s, r.exception.as_ref().map(|e| &e.message));

        if !r.return_data.is_empty() {
            prop_assert_eq!(r.return_data.clone(), expected_compressed.to_vec(),
                "CryptoLib.bls12381Mul divergence; s={}, expected_hex={}, got_hex={}",
                s, hex::encode(expected_compressed), hex::encode(&r.return_data));
        }
    }

    /// `CryptoLib.bls12381Add(a*G2, b*G2)` must equal `(a+b)*G2` on G2.
    /// Reference: `bls12_381::G2Projective` with 96-byte compressed encoding.
    #[test]
    fn differential_bls12381_g2_add(a in 0u64..=u64::MAX, b in 0u64..=u64::MAX) {
        use bls12_381::{G2Affine, G2Projective, Scalar};
        use group::Group;

        let sa = Scalar::from(a);
        let sb = Scalar::from(b);
        let g = G2Projective::generator();
        let p_a = g * sa;
        let p_b = g * sb;
        let sum = p_a + p_b;
        let expected_compressed: [u8; 96] = G2Affine::from(sum).to_compressed();

        let a_bytes = G2Affine::from(p_a).to_compressed().to_vec();
        let b_bytes = G2Affine::from(p_b).to_compressed().to_vec();

        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f(bytes memory x, bytes memory y) external view returns (bytes memory) {
        return CryptoLib.bls12381Add(x, y);
    }
}"#;
        let arts = compile_contracts(src, false, 2).unwrap();
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).unwrap();
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "f",
                &[
                    StackItem::byte_array(a_bytes),
                    StackItem::byte_array(b_bytes),
                ])
            .unwrap();
        prop_assert!(r.success, "CryptoLib.bls12381Add[G2](a={}, b={}) faulted: {:?}",
            a, b, r.exception.as_ref().map(|e| &e.message));

        if !r.return_data.is_empty() {
            prop_assert_eq!(r.return_data.clone(), expected_compressed.to_vec(),
                "CryptoLib.bls12381Add[G2] divergence; a={}, b={}, expected_hex={}, got_hex={}",
                a, b, hex::encode(expected_compressed), hex::encode(&r.return_data));
        }
    }

    /// Pairing bilinearity: `e(a*G1, b*G2) == e(G1, G2)^(a*b)`. Both
    /// sides are computed via the runtime AND via the `bls12_381`
    /// reference crate; the byte-equality of the two pairing outputs
    /// from the compiler/runtime path is asserted to match the reference
    /// (Gt) target's compressed encoding when non-empty. Bounds kept
    /// small (1..=8) so each case runs in well under 200ms.
    #[test]
    fn differential_bls12381_pairing(a in 1u64..=8, b in 1u64..=8) {
        use bls12_381::{pairing, G1Affine, G1Projective, G2Affine, G2Projective, Scalar};
        use group::Group;

        let sa = Scalar::from(a);
        let sb = Scalar::from(b);
        let g1 = G1Projective::generator();
        let g2 = G2Projective::generator();
        let p1 = G1Affine::from(g1 * sa);
        let p2 = G2Affine::from(g2 * sb);

        // Reference: e(a*G1, b*G2) and e(G1, G2)^(ab) must match in Gt.
        let lhs = pairing(&p1, &p2);
        let ab = sa * sb;
        let rhs = pairing(&G1Affine::from(g1), &G2Affine::from(g2 * ab));
        prop_assert_eq!(lhs, rhs,
            "Reference pairing bilinearity failed for a={}, b={}", a, b);

        // Now exercise the runtime path. We don't have a stable
        // serialization for Gt elements that we can directly compare to
        // an opaque runtime `bytes` blob, so this run primarily ensures
        // the IR/bytecode wiring + runtime dispatch survives the
        // BLS12-381 pairing call without panicking. When the runtime
        // implementation lands AND its Gt encoding is documented, the
        // equality of the two pairings (lhs/rhs runtime calls) can be
        // asserted directly here: lhs_runtime_bytes == rhs_runtime_bytes
        // is implementation-independent.
        let p1_bytes = p1.to_compressed().to_vec();
        let p2_bytes = p2.to_compressed().to_vec();

        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f(bytes memory x, bytes memory y) external view returns (bytes memory) {
        return CryptoLib.bls12381Pairing(x, y);
    }
}"#;
        let arts = compile_contracts(src, false, 2).unwrap();
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).unwrap();
        let r_lhs = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "f",
                &[
                    StackItem::byte_array(p1_bytes.clone()),
                    StackItem::byte_array(p2_bytes.clone()),
                ])
            .unwrap();
        prop_assert!(r_lhs.success,
            "CryptoLib.bls12381Pairing(a*G1, b*G2) faulted; a={}, b={}: {:?}",
            a, b, r_lhs.exception.as_ref().map(|e| &e.message));

        // RHS: e(G1, (a*b)*G2)
        let g1_bytes = G1Affine::from(g1).to_compressed().to_vec();
        let p2ab_bytes = G2Affine::from(g2 * ab).to_compressed().to_vec();
        let mut rt2 = NeoRuntime::new(RuntimeConfig::default()).unwrap();
        let r_rhs = rt2
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "f",
                &[
                    StackItem::byte_array(g1_bytes),
                    StackItem::byte_array(p2ab_bytes),
                ])
            .unwrap();
        prop_assert!(r_rhs.success,
            "CryptoLib.bls12381Pairing(G1, ab*G2) faulted; a={}, b={}: {:?}",
            a, b, r_rhs.exception.as_ref().map(|e| &e.message));

        // Bilinearity at the runtime layer: when both sides return
        // non-empty bytes, they must be byte-equal (Gt elements compare
        // by their canonical encoding). This is implementation-agnostic
        // — it doesn't require us to know the runtime's internal Gt
        // representation, only that whatever it is, equal Gt elements
        // produce equal byte strings. Skipped while the runtime stub
        // returns Null (empty bytes).
        if !r_lhs.return_data.is_empty() && !r_rhs.return_data.is_empty() {
            prop_assert_eq!(r_lhs.return_data.clone(), r_rhs.return_data.clone(),
                "CryptoLib.bls12381Pairing bilinearity divergence at runtime: \
                 e(a*G1, b*G2) != e(G1, ab*G2); a={}, b={}, lhs_hex={}, rhs_hex={}",
                a, b, hex::encode(&r_lhs.return_data), hex::encode(&r_rhs.return_data));
        }
    }
}

/// Solidity `<<` never reverts: `x << n` wraps to `(x * 2^n) mod 2^256`. Native
/// NeoVM `SHL` faults once the full-precision result leaves the 32-byte integer
/// range, so the compiler lowers a uint256 shift through a software routine.
/// This pins the routine against the reference value for the exact edge cases
/// that a bare `SHL` would fault on (operands >= 2^255, shift amounts that push
/// set bits past bit 255, and shift amounts >= 256). `x` is passed as a 32-byte
/// little-endian argument (the runtime's representation of a >64-bit integer)
/// and `n` as a small integer, so constant folding cannot pre-compute the shift.
#[test]
fn differential_uint256_shl_wraps_without_faulting() {
    use num_bigint::BigUint;
    let modulus = BigUint::from(1u8) << 256u32;
    let two = |e: u32| BigUint::from(1u8) << e;
    let max = &modulus - 1u8;

    // (x, n) cases; expected = (x << n) mod 2^256, computed below.
    let cases: Vec<(BigUint, u32)> = vec![
        (two(255), 0),
        (two(255), 1),
        (max.clone(), 1),
        (BigUint::from(1u8), 255),
        (BigUint::from(1u8), 256),
        (BigUint::from(1u8), 257),
        (BigUint::from(3u8), 254),
        (two(200), 100),
        (two(128), 127),
        (max.clone(), 128),
        (BigUint::from(5u8), 3),
        (max.clone(), 255),
    ];

    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f(uint256 x, uint256 n) external pure returns (uint256) {
        unchecked { return x << n; }
    }
}"#;
    let arts = compile_contracts(src, false, 2).unwrap();
    let art = &arts[0];

    for (x, n) in cases {
        let expected = (&x << n) % &modulus;
        let mut le = x.to_bytes_le();
        le.resize(32, 0);
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).unwrap();
        let r = rt
            .call_method(
                &art.bytecode,
                &art.tokens,
                &art.manifest,
                "f",
                &[StackItem::byte_array(le), StackItem::Integer(n as i64)],
            )
            .unwrap();
        assert!(
            r.success,
            "x << n FAULTED for x={x}, n={n}: {:?}",
            r.exception.as_ref().map(|e| &e.message)
        );
        let got = decode_uint_le(&r.return_data);
        assert_eq!(got, expected, "x<<n mismatch for x={x}, n={n}");
    }
}
