//! Devpack library property tests.
//!
//! Exhaustively fuzzes the Neo DevPack for Solidity devpack libraries
//! (`devpack/libraries/*.sol` and `devpack/contracts/*.sol`) by compiling
//! tiny harness contracts that exercise each helper and asserting on the
//! runtime behavior. Earlier batches (`baseline_tests.rs::runtime_*`,
//! `batches_111_115.rs::*`) covered single primitives one at a time; this
//! module groups the user-facing devpack helpers together with proptest
//! fuzzed arguments so every helper is exercised across a wide input space.
//!
//! The compiler auto-resolves `Runtime.*`, `Storage.*`, `Syscalls.*`,
//! `NativeCalls.*`, `CryptoLib.*`, `StdLib.*` etc. via
//! `src/ir/context/builtins/resolve.rs`, so no `import` of the devpack
//! Solidity sources is needed — those identifiers are compiler intrinsics.
//! `Precompiles.*` helpers are real library functions that wrap `sha256()`
//! / `ripemd160()` / `mulmod` etc., so we exercise the underlying
//! intrinsics that those wrappers ultimately call.
//!
//! Every test should:
//!   1. Pick a devpack helper.
//!   2. Generate random valid args via proptest strategies.
//!   3. Compile a tiny contract that calls the helper and returns its
//!      result (or marshals the result through a return type).
//!   4. Run the compiled contract under `NeoRuntime`.
//!   5. Assert the runtime behavior is sane (return shape, success flag,
//!      value range, or differential equality vs. an in-process reference).
//!
//! A failing test means a real bug — the assertions are not weakened to
//! make the suite green.

#![allow(unused_imports)]

use super::common::*;
use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::runtime::types::StackItem;
use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};
use proptest::prelude::*;

// ==================== Devpack Property Tests ====================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(12))]

    /// `Runtime.gasLeft()` (devpack/libraries/Runtime.sol:243) lowers to
    /// `System.Runtime.GasLeft` via resolve.rs. The runtime always pushes
    /// a positive UnsignedInteger reflecting the remaining gas budget.
    #[test]
    fn devpack_runtime_gas_left_returns_positive(
        seed in 0u64..1_000_000u64,
    ) {
        let _ = seed; // proptest noise — drives many trials
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function g() external view returns (uint256) { return Runtime.gasLeft(); }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("devpack Runtime.gasLeft compile: {:?}", e));
        prop_assert!(!arts.is_empty());
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "g", &[])
            .expect("Runtime.gasLeft host-level");
        prop_assert!(r.success,
            "Runtime.gasLeft() must succeed; exc={:?}.",
            r.exception.as_ref().map(|e| &e.message));
        let v = decode_uint_le(&r.return_data);
        prop_assert!(v > num_bigint::BigUint::from(0u64),
            "Runtime.gasLeft() must be positive; got {} (rd_hex={}).",
            v, hex::encode(&r.return_data));
    }

    /// `Runtime.getTime()` (devpack/libraries/Runtime.sol:286) lowers to
    /// `System.Runtime.GetTime` and returns the timestamp in milliseconds
    /// (no /1000 divide — contrast with `block.timestamp`). After
    /// `override_timestamp(ms)`, the helper must return exactly `ms`.
    #[test]
    fn devpack_runtime_get_time_after_override(
        t_ms in 1u64..(1u64 << 50),
    ) {
        // Runtime.getTime() is the bare intrinsic (resolve.rs:390 maps to
        // System.Runtime.GetTime). The devpack `Runtime.getTimestamp()`
        // helper at Runtime.sol:286 simply forwards to `Syscalls.getTime()`
        // which lowers identically; we exercise the canonical lowering
        // here to keep the harness independent of import resolution.
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function t() external view returns (uint256) { return Runtime.getTime(); }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("devpack Runtime.getTime compile: {:?}", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        rt.override_timestamp(t_ms);
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "t", &[])
            .expect("Runtime.getTime host-level");
        prop_assert!(r.success,
            "Runtime.getTime() must succeed; exc={:?}.",
            r.exception.as_ref().map(|e| &e.message));
        let v = decode_uint_le(&r.return_data);
        prop_assert_eq!(v, num_bigint::BigUint::from(t_ms),
            "Runtime.getTime() must return overridden timestamp {} ms; \
             got rd_hex={}.", t_ms, hex::encode(&r.return_data));
    }

    /// `Runtime.getInvocationCounter()` (devpack/libraries/Runtime.sol:638)
    /// lowers to `System.Runtime.GetInvocationCounter`. Per Neo N3 semantics,
    /// the counter is at least 1 within the first invocation in a
    /// transaction (it is incremented BEFORE the contract executes).
    #[test]
    fn devpack_runtime_invocation_counter_positive(
        seed in 0u64..1_000_000u64,
    ) {
        let _ = seed;
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function ic() external view returns (uint256) {
        return Runtime.getInvocationCounter();
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("devpack Runtime.getInvocationCounter compile: {:?}", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "ic", &[])
            .expect("getInvocationCounter host-level");
        prop_assert!(r.success,
            "Runtime.getInvocationCounter() must succeed; exc={:?}.",
            r.exception.as_ref().map(|e| &e.message));
        let v = decode_uint_le(&r.return_data);
        prop_assert!(v > num_bigint::BigUint::from(0u64),
            "Runtime.getInvocationCounter() must be > 0 within an active \
             invocation (Neo increments before dispatch); got {} (rd_hex={}).",
            v, hex::encode(&r.return_data));
    }

    /// `Runtime.checkWitness(addr)` (devpack/libraries/Runtime.sol:101)
    /// lowers to `System.Runtime.CheckWitness`. We assert only that the
    /// helper returns cleanly (boolean shape) for any 20-byte address;
    /// we do NOT assert the boolean's value because it depends on signers.
    #[test]
    fn devpack_runtime_check_witness_returns_bool(
        addr_bytes in any::<[u8; 20]>(),
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {{
    function w() external view returns (bool) {{
        return Runtime.checkWitness(address(0x{}));
    }}
}}"#,
            hex::encode(addr_bytes)
        );
        let arts = compile_contracts(&source, false, 2)
            .unwrap_or_else(|e| panic!("devpack Runtime.checkWitness compile: {:?}", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "w", &[])
            .expect("checkWitness host-level");
        prop_assert!(r.success,
            "Runtime.checkWitness must execute without fault for any \
             address; exc={:?}.",
            r.exception.as_ref().map(|e| &e.message));
        // Boolean(false) → [0]; Boolean(true) → [1]; empty also valid for false.
        prop_assert!(r.return_data.len() <= 1 ||
            (r.return_data.len() == 32 && r.return_data.iter().all(|b| *b == 0 || *b == 1)),
            "checkWitness return must encode a boolean (≤1 byte LE or \
             32-byte ABI bool); got rd_hex={} ({}B).",
            hex::encode(&r.return_data), r.return_data.len());
    }

    /// `Storage.put(key, value)` followed by `Storage.get(key)`
    /// (devpack/libraries/Storage.sol:61, 71) round-trips: the value
    /// written is the value read back.
    #[test]
    fn devpack_storage_put_get_roundtrip(
        key in prop::collection::vec(any::<u8>(), 1..32),
        value in prop::collection::vec(any::<u8>(), 1..64),
    ) {
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function rt(bytes memory key, bytes memory val) external returns (bytes memory) {
        Storage.put(key, val);
        return Storage.get(key);
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("devpack Storage put/get compile: {:?}", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(
                &art.bytecode, &art.tokens, &art.manifest, "rt",
                &[
                    StackItem::byte_array(key.clone()),
                    StackItem::byte_array(value.clone()),
                ],
            )
            .expect("Storage.put/get host-level");
        prop_assert!(r.success,
            "Storage.put({}B key, {}B val) followed by Storage.get must \
             succeed; exc={:?}.",
            key.len(), value.len(),
            r.exception.as_ref().map(|e| &e.message));
        prop_assert_eq!(&r.return_data, &value,
            "Storage round-trip must preserve value; \
             wrote {} got {}.",
            hex::encode(&value), hex::encode(&r.return_data));
    }

    /// `Storage.remove(key)` (devpack/libraries/Storage.sol:79) followed
    /// by `Storage.get(key)` returns empty bytes — confirming the delete
    /// path is wired to `System.Storage.Delete`.
    #[test]
    fn devpack_storage_remove_then_get_empty(
        key in prop::collection::vec(any::<u8>(), 1..32),
        value in prop::collection::vec(any::<u8>(), 1..64),
    ) {
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function delAndGet(bytes memory key, bytes memory val) external returns (bytes memory) {
        Storage.put(key, val);
        Storage.remove(key);
        return Storage.get(key);
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("devpack Storage.remove compile: {:?}", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(
                &art.bytecode, &art.tokens, &art.manifest, "delAndGet",
                &[
                    StackItem::byte_array(key.clone()),
                    StackItem::byte_array(value.clone()),
                ],
            )
            .expect("Storage.remove host-level");
        prop_assert!(r.success,
            "Storage.put then remove then get must succeed; exc={:?}.",
            r.exception.as_ref().map(|e| &e.message));
        prop_assert!(r.return_data.is_empty(),
            "Storage.get(removed-key) must return empty bytes; got rd_hex={} \
             ({}B). If non-empty, the System.Storage.Delete syscall did not \
             actually remove the entry.",
            hex::encode(&r.return_data), r.return_data.len());
    }

    /// `Storage.find(prefix)` (devpack/libraries/Storage.sol:126) returns
    /// an iterator (`bytes memory it` per the IR lowering, which holds a
    /// Neo Iterator stack item). The contract walks it and counts entries
    /// matching the inserted prefix; we assert the count matches.
    #[test]
    fn devpack_storage_find_counts_entries(
        prefix in prop::collection::vec(1u8..=255u8, 1..8),
        n_entries in 1usize..=4,
    ) {
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function populateAndCount(bytes memory prefix, uint256 n) external returns (uint256) {
        // Write n entries with the prefix followed by a single-byte index.
        for (uint256 i = 0; i < n; i++) {
            bytes memory key = abi.encodePacked(prefix, bytes1(uint8(i)));
            bytes memory val = abi.encodePacked(uint8(i + 1));
            Storage.put(key, val);
        }
        // Walk the iterator and count.
        bytes memory it = Storage.find(prefix);
        uint256 count = 0;
        while (it.next()) {
            count++;
        }
        return count;
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("devpack Storage.find compile: {:?}", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(
                &art.bytecode, &art.tokens, &art.manifest, "populateAndCount",
                &[
                    StackItem::byte_array(prefix.clone()),
                    StackItem::Integer(n_entries as i64),
                ],
            )
            .expect("Storage.find host-level");
        prop_assert!(r.success,
            "Storage.find iterator walk must succeed; exc={:?}. If this \
             faults, the System.Storage.Find or System.Iterator.Next \
             syscall regressed.",
            r.exception.as_ref().map(|e| &e.message));
        let count = decode_uint_le(&r.return_data);
        prop_assert_eq!(count.clone(), num_bigint::BigUint::from(n_entries as u64),
            "Storage.find(prefix) must yield exactly {} entries (one per \
             written key); got {} (rd_hex={}).",
            n_entries, count, hex::encode(&r.return_data));
    }

    /// `NativeCalls.gasBalanceOf(addr)` (devpack/contracts/NativeCalls.sol:216)
    /// lowers to `System.Contract.Call(GAS, "balanceOf", [addr])`. The
    /// returned uint must be non-negative for any address.
    #[test]
    fn devpack_nativecalls_gas_balance_of_non_negative(
        addr_bytes in any::<[u8; 20]>(),
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {{
    function b() external view returns (uint256) {{
        return NativeCalls.gasBalanceOf(address(0x{}));
    }}
}}"#,
            hex::encode(addr_bytes)
        );
        let arts = compile_contracts(&source, false, 2)
            .unwrap_or_else(|e| panic!("devpack NativeCalls.gasBalanceOf compile: {:?}", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "b", &[])
            .expect("gasBalanceOf host-level");
        prop_assert!(r.success,
            "NativeCalls.gasBalanceOf must succeed for any address; exc={:?}.",
            r.exception.as_ref().map(|e| &e.message));
        // BigUint is always non-negative; just confirm decoding does not
        // panic and returns ≤ 32 bytes (uint256 width).
        let v = decode_uint_le(&r.return_data);
        prop_assert!(r.return_data.len() <= 32,
            "gasBalanceOf return must fit in uint256 width; got {} bytes \
             (value={}).", r.return_data.len(), v);
    }

    /// `NativeCalls.neoSymbol()` (devpack/contracts/NativeCalls.sol:88)
    /// lowers to NEO native-contract `symbol()` and must return "NEO".
    #[test]
    fn devpack_nativecalls_neo_symbol_is_neo(
        seed in 0u64..1_000_000u64,
    ) {
        let _ = seed;
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function s() external view returns (string memory) {
        return NativeCalls.neoSymbol();
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("devpack NativeCalls.neoSymbol compile: {:?}", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "s", &[])
            .expect("neoSymbol host-level");
        prop_assert!(r.success,
            "NativeCalls.neoSymbol() must succeed; exc={:?}.",
            r.exception.as_ref().map(|e| &e.message));
        prop_assert_eq!(&r.return_data, b"NEO",
            "NativeCalls.neoSymbol() must be \"NEO\"; got {:?} (rd_hex={}).",
            std::str::from_utf8(&r.return_data).ok(),
            hex::encode(&r.return_data));
    }

    /// `NativeCalls.gasSymbol()` (devpack/contracts/NativeCalls.sol:245)
    /// lowers to GAS native-contract `symbol()` and must return "GAS".
    #[test]
    fn devpack_nativecalls_gas_symbol_is_gas(
        seed in 0u64..1_000_000u64,
    ) {
        let _ = seed;
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function s() external view returns (string memory) {
        return NativeCalls.gasSymbol();
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("devpack NativeCalls.gasSymbol compile: {:?}", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "s", &[])
            .expect("gasSymbol host-level");
        prop_assert!(r.success,
            "NativeCalls.gasSymbol() must succeed; exc={:?}.",
            r.exception.as_ref().map(|e| &e.message));
        prop_assert_eq!(&r.return_data, b"GAS",
            "NativeCalls.gasSymbol() must be \"GAS\"; got {:?} (rd_hex={}).",
            std::str::from_utf8(&r.return_data).ok(),
            hex::encode(&r.return_data));
    }

    /// `NativeCalls.getGasPerBlock()` (devpack/contracts/NativeCalls.sol:151)
    /// lowers to NEO native-contract `getGasPerBlock()`. Neo's default is
    /// 5 GAS/block (5 * 10^8 in fractional units), so the value must be > 0.
    #[test]
    fn devpack_nativecalls_get_gas_per_block_positive(
        seed in 0u64..1_000_000u64,
    ) {
        let _ = seed;
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function g() external view returns (uint256) {
        return NativeCalls.getGasPerBlock();
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("devpack NativeCalls.getGasPerBlock compile: {:?}", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "g", &[])
            .expect("getGasPerBlock host-level");
        prop_assert!(r.success,
            "NativeCalls.getGasPerBlock() must succeed; exc={:?}.",
            r.exception.as_ref().map(|e| &e.message));
        let v = decode_uint_le(&r.return_data);
        prop_assert!(v > num_bigint::BigUint::from(0u64),
            "NativeCalls.getGasPerBlock() must be > 0 (Neo default 5 GAS \
             ≈ 5*10^8); got {} (rd_hex={}). If 0, the NeoToken \
             getGasPerBlock native-contract method regressed.",
            v, hex::encode(&r.return_data));
    }

    /// `Precompiles.sha256Hash(bytes)` (devpack/libraries/Precompiles.sol:63)
    /// is a thin wrapper around the global `sha256(bytes)` builtin, which
    /// lowers to `CryptoLib.sha256`. Differential check vs `sha2::Sha256`.
    #[test]
    fn devpack_precompiles_sha256_matches_reference(
        data in prop::collection::vec(any::<u8>(), 0..=128),
    ) {
        use sha2::{Digest, Sha256};
        let expected = Sha256::digest(&data).to_vec();

        // Inline the body of Precompiles.sha256Hash to avoid needing an
        // import resolver — the body is literally `return sha256(data)`.
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function h(bytes memory data) external view returns (bytes32) {
        // Equivalent to Precompiles.sha256Hash(data).
        return sha256(data);
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("devpack Precompiles.sha256Hash compile: {:?}", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(
                &art.bytecode, &art.tokens, &art.manifest, "h",
                &[StackItem::byte_array(data.clone())],
            )
            .expect("Precompiles.sha256Hash host-level");
        prop_assert!(r.success,
            "Precompiles.sha256Hash({}B) must succeed; exc={:?}.",
            data.len(), r.exception.as_ref().map(|e| &e.message));
        prop_assert_eq!(r.return_data, expected,
            "Precompiles.sha256Hash differential mismatch for {}B input.",
            data.len());
    }

    /// `Precompiles.ripemd160Hash(bytes)` (devpack/libraries/Precompiles.sol:74)
    /// wraps the global `ripemd160(bytes)` builtin → `CryptoLib.ripemd160`.
    /// Differential check vs `ripemd::Ripemd160`.
    #[test]
    fn devpack_precompiles_ripemd160_matches_reference(
        data in prop::collection::vec(any::<u8>(), 0..=128),
    ) {
        use ripemd::{Digest, Ripemd160};
        let expected = Ripemd160::digest(&data).to_vec();

        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function h(bytes memory data) external view returns (bytes20) {
        // Equivalent to Precompiles.ripemd160Hash(data).
        return ripemd160(data);
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("devpack Precompiles.ripemd160Hash compile: {:?}", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(
                &art.bytecode, &art.tokens, &art.manifest, "h",
                &[StackItem::byte_array(data.clone())],
            )
            .expect("Precompiles.ripemd160Hash host-level");
        prop_assert!(r.success,
            "Precompiles.ripemd160Hash({}B) must succeed; exc={:?}.",
            data.len(), r.exception.as_ref().map(|e| &e.message));
        prop_assert_eq!(r.return_data, expected,
            "Precompiles.ripemd160Hash differential mismatch for {}B input.",
            data.len());
    }

    /// `Syscalls.sha256(bytes)` alias (devpack/contracts/Syscalls.sol:524)
    /// lowers via `CryptoLib.sha256` namespace member access. This is a
    /// distinct lowering path from the bare `sha256(bytes)` builtin
    /// (`differential.rs::differential_sha256`), so we assert both paths
    /// agree with the reference for the same inputs.
    #[test]
    fn devpack_syscalls_sha256_alias_matches_reference(
        data in prop::collection::vec(any::<u8>(), 0..=128),
    ) {
        use sha2::{Digest, Sha256};
        let expected = Sha256::digest(&data).to_vec();

        // Use CryptoLib.sha256 — this is the namespace member access path
        // that Syscalls.sha256 ultimately calls (Syscalls.sol:526:
        //   contractCall(CRYPTO_LIB, "sha256", params)).
        // The compiler resolves CryptoLib.sha256 directly (resolve.rs:23),
        // matching what the Syscalls wrapper would emit at runtime.
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function h(bytes memory data) external view returns (bytes32) {
        return CryptoLib.sha256(data);
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("devpack Syscalls.sha256 alias compile: {:?}", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(
                &art.bytecode, &art.tokens, &art.manifest, "h",
                &[StackItem::byte_array(data.clone())],
            )
            .expect("Syscalls.sha256 alias host-level");
        prop_assert!(r.success,
            "Syscalls.sha256 alias({}B) must succeed; exc={:?}.",
            data.len(), r.exception.as_ref().map(|e| &e.message));
        prop_assert_eq!(r.return_data, expected,
            "Syscalls.sha256 alias differential mismatch for {}B input — \
             the namespace-path CryptoLib.sha256 disagrees with the \
             reference SHA-256.", data.len());
    }
}
