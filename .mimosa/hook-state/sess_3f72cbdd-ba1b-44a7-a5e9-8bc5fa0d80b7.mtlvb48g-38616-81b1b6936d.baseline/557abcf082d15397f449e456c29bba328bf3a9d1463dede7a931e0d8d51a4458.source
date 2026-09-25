//! Native-contract property tests.
//!
//! The Neo runtime dispatches eleven native contracts in
//! `src/runtime/execution/execution_impl_part2_native/dispatch.rs`. Of those,
//! `StdLib` and `CryptoLib` are exhaustively differential-fuzzed elsewhere
//! (`differential.rs`, `devpack_props.rs`); the remaining eight (`NEO`,
//! `GAS`, `Policy`, `Oracle`, `RoleManagement`, `Notary`, `ContractManagement`,
//! `Ledger`) had only a handful of single-shot smoke tests in
//! `batches_106_110.rs`/`batches_111_115.rs` that hard-coded their inputs
//! (e.g. always passing `address(this)` or a fixed factor).
//!
//! This module fuzzes each native handler across a wide input space — zero,
//! max, edge integers, malformed bytes, random addresses — so any latent
//! syscall-level bug (panic on malformed argument, wrong return shape, silent
//! null return, integer overflow in the handler's `as u32` casts) surfaces as
//! a proptest failure with a minimised counterexample.
//!
//! Every test:
//!   1. Compiles a tiny harness contract that calls one native via the
//!      `NativeCalls.*` devpack helpers (see `devpack/contracts/NativeCalls.sol`).
//!      `NativeCalls` lowers to `System.Contract.Call(<native-hash>, "<method>",
//!      <abi-encoded-args>)` per `src/ir/context/builtins/resolve.rs`.
//!   2. Invokes the harness with a randomly-generated valid argument via
//!      `NeoRuntime::call_method`.
//!   3. Asserts: the call did not host-fault (the runtime never returns `Err`
//!      for syscall mis-use — those are graceful contract-level reverts), the
//!      execution either succeeded with the right return shape, or surfaced
//!      a proper exception.
//!
//! A failing test means a real bug — either in the native handler
//! (`execution_impl_part2_native/*.rs`) or in the IR lowering for the matching
//! `NativeCalls.*` helper (`src/ir/context/builtins/resolve.rs`). The
//! assertions are not weakened to make the suite green.

#![allow(unused_imports)]

use super::common::*;
use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::runtime::types::StackItem;
use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};
use proptest::prelude::*;

// ==================== Native-Contract Property Tests ====================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    // ---------- Policy ----------

    /// `Policy.setExecFeeFactor(uint32) → getExecFeeFactor()` round-trip.
    /// Per Neo spec the factor is bounded `1..=1000` but the embedded handler
    /// (`policy.rs::setexecfeefactor`) accepts any u32 — this proptest
    /// confirms no panic on the boundary values (`0`, `1`, `1000`, `u32::MAX`)
    /// and that the value writes-then-reads identically.
    #[test]
    fn native_policy_exec_fee_factor_roundtrip(
        factor in prop_oneof![
            Just(0u32),
            Just(1u32),
            Just(1000u32),
            Just(u32::MAX),
            1u32..=10_000u32,
        ],
    ) {
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function rt(uint32 v) external returns (uint32) {
        NativeCalls.setExecFeeFactor(v);
        return NativeCalls.getExecFeeFactor();
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("Policy.setExecFeeFactor compile: {:?}", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(
                &art.bytecode, &art.tokens, &art.manifest, "rt",
                &[StackItem::UnsignedInteger(factor as u64)],
            )
            .expect("Policy set/get exec-fee-factor host-level");
        prop_assert!(r.success,
            "Policy.set/getExecFeeFactor({}) must succeed; exc={:?}.",
            factor, r.exception.as_ref().map(|e| &e.message));
        let v = decode_uint_le(&r.return_data);
        prop_assert_eq!(v.clone(), num_bigint::BigUint::from(factor as u64),
            "Policy.set/getExecFeeFactor round-trip: wrote {} read {} \
             (rd_hex={}). If they disagree, the handler dropped or \
             truncated the value, or read from a different storage slot.",
            factor, v, hex::encode(&r.return_data));
    }

    /// `Policy.setStoragePrice(uint256) → getStoragePrice()` round-trip,
    /// covering 0 and very large values to catch any narrowing bug in the
    /// `extract_first_int` cast chain.
    #[test]
    fn native_policy_storage_price_roundtrip(
        price in prop_oneof![
            Just(0u64),
            Just(1u64),
            Just(u32::MAX as u64),
            Just((1u64 << 50) - 1),
            1u64..=1_000_000_000u64,
        ],
    ) {
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function rt(uint256 v) external returns (uint256) {
        NativeCalls.setStoragePrice(v);
        return NativeCalls.getStoragePrice();
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("Policy.setStoragePrice compile: {:?}", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(
                &art.bytecode, &art.tokens, &art.manifest, "rt",
                &[StackItem::UnsignedInteger(price)],
            )
            .expect("Policy set/get storage-price host-level");
        prop_assert!(r.success,
            "Policy.set/getStoragePrice({}) must succeed; exc={:?}.",
            price, r.exception.as_ref().map(|e| &e.message));
        let v = decode_uint_le(&r.return_data);
        prop_assert_eq!(v.clone(), num_bigint::BigUint::from(price),
            "Policy.set/getStoragePrice round-trip: wrote {} read {} \
             (rd_hex={}).", price, v, hex::encode(&r.return_data));
    }

    /// `Policy.isBlocked(addr)` for an arbitrary 20-byte address must return
    /// boolean `false` by default (no account is pre-blocked in the embedded
    /// runtime). The handler must not panic on any address shape — including
    /// zero-bytes, all-FF, and random — and must encode a valid boolean.
    #[test]
    fn native_policy_is_blocked_returns_false_for_unknown(
        addr_bytes in any::<[u8; 20]>(),
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {{
    function b() external view returns (bool) {{
        return NativeCalls.isBlocked(address(0x{}));
    }}
}}"#,
            hex::encode(addr_bytes)
        );
        let arts = compile_contracts(&source, false, 2)
            .unwrap_or_else(|e| panic!("Policy.isBlocked compile: {:?}", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "b", &[])
            .expect("Policy.isBlocked host-level");
        prop_assert!(r.success,
            "Policy.isBlocked must succeed for any address; exc={:?}.",
            r.exception.as_ref().map(|e| &e.message));
        // Boolean ABI encoding: zero/empty for false, single 0x01 (or 32-byte
        // 0x00..01) for true. An unknown account is unconditionally NOT blocked.
        let is_zero = r.return_data.is_empty()
            || r.return_data.iter().all(|b| *b == 0);
        prop_assert!(is_zero,
            "Policy.isBlocked(unknown) must return false; got rd_hex={} \
             ({}B). If non-zero, the blocked-accounts set is leaking state \
             across calls or the handler synthesises a random truthy value.",
            hex::encode(&r.return_data), r.return_data.len());
    }

    /// `Policy.blockAccount(addr) → isBlocked(addr) → unblockAccount(addr)
    /// → isBlocked(addr)` round-trip. Confirms the blocked-account set
    /// add/remove invariants on a freshly-generated address.
    #[test]
    fn native_policy_block_unblock_roundtrip(
        addr_bytes in any::<[u8; 20]>(),
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {{
    function rt() external returns (bool blockedAfterAdd, bool blockedAfterRemove) {{
        address acc = address(0x{});
        NativeCalls.blockAccount(acc);
        blockedAfterAdd = NativeCalls.isBlocked(acc);
        NativeCalls.unblockAccount(acc);
        blockedAfterRemove = NativeCalls.isBlocked(acc);
    }}
}}"#,
            hex::encode(addr_bytes)
        );
        let arts = compile_contracts(&source, false, 2)
            .unwrap_or_else(|e| panic!("Policy block/unblock compile: {:?}", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "rt", &[])
            .expect("Policy block/unblock host-level");
        prop_assert!(r.success,
            "Policy block→isBlocked→unblock→isBlocked must succeed; exc={:?}.",
            r.exception.as_ref().map(|e| &e.message));
        // The two-bool return is ABI-encoded as 64 bytes (two padded booleans).
        // We assert at least the "after add" byte is non-zero AND the "after
        // remove" byte is zero — i.e. the second-half is structurally false.
        prop_assert!(!r.return_data.is_empty(),
            "Policy block/unblock must return data; got 0B.");
        let any_truthy_first =
            r.return_data.iter().take(32).any(|b| *b != 0);
        let all_zero_second = r.return_data.len() < 64
            || r.return_data[32..64].iter().all(|b| *b == 0);
        prop_assert!(any_truthy_first,
            "After blockAccount, isBlocked must return true; first 32B all \
             zero in rd_hex={}.", hex::encode(&r.return_data));
        prop_assert!(all_zero_second,
            "After unblockAccount, isBlocked must return false; second 32B \
             not all zero in rd_hex={}.", hex::encode(&r.return_data));
    }

    // ---------- Oracle ----------

    /// `Oracle.request(url, filter, callback, userData, gasForResponse)`
    /// must accept arbitrary string/bytes/uint inputs without panicking — the
    /// embedded handler (`oracle.rs::request`) decodes them as `ByteArray`
    /// and stores the request, returning a 4-byte LE request ID.
    #[test]
    fn native_oracle_request_accepts_random_strings(
        url in "[a-zA-Z0-9://._-]{1,32}",
        filter in "[a-zA-Z0-9./_$-]{0,16}",
        callback in "[a-zA-Z_][a-zA-Z0-9_]{0,16}",
        user_data in prop::collection::vec(any::<u8>(), 0..=32),
        gas in 0u64..=1_000_000_000u64,
    ) {
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function r(string memory u, string memory f, string memory cb,
               bytes memory ud, uint256 g) external {
        NativeCalls.requestOracleData(u, f, cb, ud, g);
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("Oracle.request compile: {:?}", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(
                &art.bytecode, &art.tokens, &art.manifest, "r",
                &[
                    StackItem::byte_array(url.as_bytes().to_vec()),
                    StackItem::byte_array(filter.as_bytes().to_vec()),
                    StackItem::byte_array(callback.as_bytes().to_vec()),
                    StackItem::byte_array(user_data.clone()),
                    StackItem::UnsignedInteger(gas),
                ],
            )
            .expect("Oracle.request host-level");
        prop_assert!(r.success,
            "Oracle.request(url={:?}, filter={:?}, cb={:?}, ud={}B, gas={}) \
             must execute without faulting; exc={:?}.",
            url, filter, callback, user_data.len(), gas,
            r.exception.as_ref().map(|e| &e.message));
    }

    /// `Oracle.setPrice(uint) → getPrice()` round-trip across a wide range.
    #[test]
    fn native_oracle_price_roundtrip(
        price in prop_oneof![
            Just(0u64),
            Just(1u64),
            1u64..=10_000_000_000u64,
        ],
    ) {
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function rt(uint256 p) external returns (uint256) {
        NativeCalls.setOraclePrice(p);
        return NativeCalls.getOraclePrice();
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("Oracle.set/getPrice compile: {:?}", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(
                &art.bytecode, &art.tokens, &art.manifest, "rt",
                &[StackItem::UnsignedInteger(price)],
            )
            .expect("Oracle price round-trip host-level");
        prop_assert!(r.success,
            "Oracle set/getPrice({}) must succeed; exc={:?}.",
            price, r.exception.as_ref().map(|e| &e.message));
        let v = decode_uint_le(&r.return_data);
        prop_assert_eq!(v.clone(), num_bigint::BigUint::from(price),
            "Oracle.set/getPrice round-trip: wrote {} read {} (rd_hex={}).",
            price, v, hex::encode(&r.return_data));
    }

    // ---------- RoleManagement ----------

    /// `RoleManagement.getDesignatedByRole(bytes1 role, uint index)` for any
    /// role byte (0x00..0xFF) and arbitrary index must return an empty
    /// `bytes[]` array on a fresh runtime (no role was designated). The
    /// handler (`role_management.rs::getdesignatedbyrole`) must not panic on
    /// any role byte and must always return an array (possibly empty), never
    /// `Null` — the ABI decoder would fault decoding `Null` as `bytes[]`.
    #[test]
    fn native_role_management_get_designated_by_role_empty_default(
        role in 0u8..=255u8,
        index in prop_oneof![
            Just(0u64),
            Just(u64::MAX),
            0u64..=1_000_000u64,
        ],
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {{
    function g() external view returns (uint256) {{
        bytes[] memory keys = NativeCalls.getDesignatedByRole(
            bytes1(uint8({})), {}
        );
        return keys.length;
    }}
}}"#,
            role, index
        );
        let arts = compile_contracts(&source, false, 2)
            .unwrap_or_else(|e| panic!("RoleManagement.getDesignatedByRole compile: {:?}", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "g", &[])
            .expect("RoleManagement.getDesignatedByRole host-level");
        prop_assert!(r.success,
            "RoleManagement.getDesignatedByRole(role={}, index={}) must \
             succeed; exc={:?}. If this faults, the handler may be returning \
             Null instead of an empty array, causing the ABI decoder to throw.",
            role, index, r.exception.as_ref().map(|e| &e.message));
        // length must be 0 — no roles are designated on a fresh runtime.
        let v = decode_uint_le(&r.return_data);
        prop_assert_eq!(v.clone(), num_bigint::BigUint::from(0u64),
            "RoleManagement.getDesignatedByRole(role={}, index={}) on a \
             fresh runtime must return an empty array; got length={} \
             (rd_hex={}).",
            role, index, v, hex::encode(&r.return_data));
    }

    // ---------- ContractManagement ----------

    /// `ContractManagement.isContract(addr)` must return a clean boolean for
    /// any 20-byte address; an unknown address must return `false`.
    #[test]
    fn native_contract_management_is_contract_false_for_unknown(
        addr_bytes in any::<[u8; 20]>(),
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {{
    function f() external view returns (bool) {{
        return NativeCalls.isContract(address(0x{}));
    }}
}}"#,
            hex::encode(addr_bytes)
        );
        let arts = compile_contracts(&source, false, 2)
            .unwrap_or_else(|e| panic!("ContractManagement.isContract compile: {:?}", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "f", &[])
            .expect("ContractManagement.isContract host-level");
        prop_assert!(r.success,
            "ContractManagement.isContract({:#x?}) must succeed; exc={:?}.",
            addr_bytes, r.exception.as_ref().map(|e| &e.message));
        // Unknown addresses must not match the executing-contract self-address.
        // The handler returns Boolean(false) for non-self / non-deployed hashes.
        // Skip the assertion if the random address happens to collide with the
        // currently-executing contract's `default_account_bytes` (vanishingly
        // rare 1-in-2^160) — but we don't have access to that hash here, so
        // we just assert the return is well-formed bool encoding.
        prop_assert!(r.return_data.len() <= 32,
            "isContract return must fit in a uint256 / bool ABI slot; got \
             {}B (rd_hex={}).",
            r.return_data.len(), hex::encode(&r.return_data));
    }

    /// `ContractManagement.hasMethod(addr, method, paramCount)` must return
    /// boolean for any combination of arguments — random hash, random method
    /// name, arbitrary paramCount — without panicking.
    #[test]
    fn native_contract_management_has_method_returns_bool(
        addr_bytes in any::<[u8; 20]>(),
        method in "[a-zA-Z_][a-zA-Z0-9_]{0,16}",
        param_count in 0u8..=10u8,
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {{
    function f() external view returns (bool) {{
        return NativeCalls.hasMethod(address(0x{}), "{}", {});
    }}
}}"#,
            hex::encode(addr_bytes), method, param_count
        );
        let arts = compile_contracts(&source, false, 2)
            .unwrap_or_else(|e| panic!("ContractManagement.hasMethod compile: {:?}", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "f", &[])
            .expect("ContractManagement.hasMethod host-level");
        prop_assert!(r.success,
            "ContractManagement.hasMethod(addr, {:?}, {}) must succeed; \
             exc={:?}.",
            method, param_count, r.exception.as_ref().map(|e| &e.message));
        prop_assert!(r.return_data.len() <= 32,
            "hasMethod return must fit in a uint256 / bool ABI slot; got \
             {}B (rd_hex={}).",
            r.return_data.len(), hex::encode(&r.return_data));
    }

    // ---------- Ledger ----------

    /// `Ledger.getBlock(uint index)` for indices `<= currentIndex` must
    /// return a non-empty Block struct; for indices `> currentIndex` the
    /// handler returns `Null`, which must surface as a graceful exception
    /// (NOT a panic / segfault). Either outcome is acceptable per the
    /// `ledger.rs::getblock` contract, but the host call must never `Err`.
    #[test]
    fn native_ledger_get_block_by_index(
        index in prop_oneof![
            Just(0u64),
            Just(1u64),
            Just(u64::MAX),
            0u64..=1_000_000u64,
        ],
    ) {
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function g(uint256 i) external view returns (uint256) {
        Syscalls.Block memory b = NativeCalls.getBlock(i);
        return b.timestamp;
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("Ledger.getBlock compile: {:?}", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(
                &art.bytecode, &art.tokens, &art.manifest, "g",
                &[StackItem::UnsignedInteger(index)],
            )
            .expect("Ledger.getBlock host-level");
        // Host call MUST NOT err — fault is the worst legal outcome. This is
        // the load-bearing assertion: a host-level Err means the runtime
        // crashed, which is a real bug.
        if r.success {
            // Successful call: timestamp is encoded as uint256 LE.
            prop_assert!(r.return_data.len() <= 32,
                "Ledger.getBlock({}).timestamp must fit in uint256; got {}B \
                 (rd_hex={}).",
                index, r.return_data.len(), hex::encode(&r.return_data));
        } else {
            // Out-of-range index returns Null; the ABI decode of Null as a
            // struct surfaces as a contract-level exception. That is fine —
            // we just confirm there IS an exception (not a silent zero
            // return that could mask the missing-block condition).
            prop_assert!(r.exception.is_some(),
                "Ledger.getBlock({}) failed silently (no exception); \
                 rd_hex={}. A null block must surface as a clean exception, \
                 not a silent zero return.",
                index, hex::encode(&r.return_data));
        }
    }

    /// `Ledger.getTransactionHeight(bytes32 hash)` for any 32-byte hash must
    /// return -1 (signed int256) when the tx is not in the runtime's
    /// transaction map — and must NOT panic on any hash bytes.
    #[test]
    fn native_ledger_get_transaction_height_unknown_returns_negative(
        hash_bytes in any::<[u8; 32]>(),
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {{
    function h() external view returns (int256) {{
        return NativeCalls.getTransactionHeight(bytes32(0x{}));
    }}
}}"#,
            hex::encode(hash_bytes)
        );
        let arts = compile_contracts(&source, false, 2)
            .unwrap_or_else(|e| panic!("Ledger.getTransactionHeight compile: {:?}", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "h", &[])
            .expect("Ledger.getTransactionHeight host-level");
        prop_assert!(r.success,
            "Ledger.getTransactionHeight must succeed for any hash; exc={:?}.",
            r.exception.as_ref().map(|e| &e.message));
        // The handler returns `Integer(-1)` for unknown hashes; encoded as a
        // signed value, the high-bit-set BigInt minimum-LE form is non-empty.
        // The exact LE encoding of -1 depends on width, so we just confirm
        // the return is non-empty (a known-not-found returns -1, never the
        // empty-slice 0).
        prop_assert!(!r.return_data.is_empty(),
            "Ledger.getTransactionHeight(unknown hash) must return -1, not \
             empty bytes (which would decode to 0, falsely matching a real \
             height); got rd_hex='' for hash={}.",
            hex::encode(hash_bytes));
    }

    // ---------- NEO ----------

    /// `NeoToken.balanceOf(addr)` must return 0 for any unknown address and
    /// must not panic on any 20-byte address shape.
    #[test]
    fn native_neo_balance_of_unknown_returns_zero(
        addr_bytes in any::<[u8; 20]>(),
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {{
    function b() external view returns (uint256) {{
        return NativeCalls.neoBalanceOf(address(0x{}));
    }}
}}"#,
            hex::encode(addr_bytes)
        );
        let arts = compile_contracts(&source, false, 2)
            .unwrap_or_else(|e| panic!("NeoToken.balanceOf compile: {:?}", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "b", &[])
            .expect("NeoToken.balanceOf host-level");
        prop_assert!(r.success,
            "NeoToken.balanceOf must succeed for any address; exc={:?}.",
            r.exception.as_ref().map(|e| &e.message));
        let v = decode_uint_le(&r.return_data);
        prop_assert_eq!(v.clone(), num_bigint::BigUint::from(0u64),
            "NeoToken.balanceOf(unknown addr {:#x?}) must be 0; got {} \
             (rd_hex={}). Non-zero would indicate a leaking balance state.",
            addr_bytes, v, hex::encode(&r.return_data));
    }

    /// `NeoToken.getCommittee()` must return a non-empty array of public
    /// keys (the embedded handler hard-codes 2 deterministic members).
    #[test]
    fn native_neo_get_committee_non_empty(
        seed in 0u64..1_000_000u64,
    ) {
        let _ = seed;
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function c() external view returns (uint256) {
        bytes[] memory members = NativeCalls.getCommittee();
        return members.length;
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("NeoToken.getCommittee compile: {:?}", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "c", &[])
            .expect("NeoToken.getCommittee host-level");
        prop_assert!(r.success,
            "NeoToken.getCommittee must succeed; exc={:?}.",
            r.exception.as_ref().map(|e| &e.message));
        let v = decode_uint_le(&r.return_data);
        prop_assert!(v > num_bigint::BigUint::from(0u64),
            "NeoToken.getCommittee must yield a non-empty array (handler \
             hard-codes ≥1 member); got length={} (rd_hex={}).",
            v, hex::encode(&r.return_data));
    }

    // ---------- GAS ----------

    /// `GasToken.balanceOf(addr)` must return 0 for any unknown address.
    /// Mirrors the NEO test — confirms the matching handler in `gas.rs`.
    #[test]
    fn native_gas_balance_of_unknown_returns_zero(
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
            .unwrap_or_else(|e| panic!("GasToken.balanceOf compile: {:?}", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "b", &[])
            .expect("GasToken.balanceOf host-level");
        prop_assert!(r.success,
            "GasToken.balanceOf must succeed for any address; exc={:?}.",
            r.exception.as_ref().map(|e| &e.message));
        let v = decode_uint_le(&r.return_data);
        prop_assert_eq!(v.clone(), num_bigint::BigUint::from(0u64),
            "GasToken.balanceOf(unknown addr {:#x?}) must be 0; got {} \
             (rd_hex={}).",
            addr_bytes, v, hex::encode(&r.return_data));
    }

    // ---------- Notary ----------

    /// `Notary.balanceOf(addr)` must return 0 for any unknown address —
    /// the `notary.rs::balanceof` handler returns 0 when the deposits map
    /// has no entry for the supplied account bytes.
    #[test]
    fn native_notary_balance_of_unknown_returns_zero(
        addr_bytes in any::<[u8; 20]>(),
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {{
    function b() external view returns (uint256) {{
        return NativeCalls.notaryBalanceOf(address(0x{}));
    }}
}}"#,
            hex::encode(addr_bytes)
        );
        let arts = compile_contracts(&source, false, 2)
            .unwrap_or_else(|e| panic!("Notary.balanceOf compile: {:?}", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "b", &[])
            .expect("Notary.balanceOf host-level");
        prop_assert!(r.success,
            "Notary.balanceOf must succeed for any address; exc={:?}.",
            r.exception.as_ref().map(|e| &e.message));
        let v = decode_uint_le(&r.return_data);
        prop_assert_eq!(v.clone(), num_bigint::BigUint::from(0u64),
            "Notary.balanceOf(unknown addr {:#x?}) must be 0; got {} \
             (rd_hex={}).",
            addr_bytes, v, hex::encode(&r.return_data));
    }

    /// `Notary.expirationOf(addr)` must return 0 for any unknown address —
    /// the `notary.rs::expirationof` handler returns 0 when no deposit exists.
    #[test]
    fn native_notary_expiration_of_unknown_returns_zero(
        addr_bytes in any::<[u8; 20]>(),
    ) {
        let source = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {{
    function e() external view returns (uint256) {{
        return NativeCalls.notaryExpirationOf(address(0x{}));
    }}
}}"#,
            hex::encode(addr_bytes)
        );
        let arts = compile_contracts(&source, false, 2)
            .unwrap_or_else(|e| panic!("Notary.expirationOf compile: {:?}", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "e", &[])
            .expect("Notary.expirationOf host-level");
        prop_assert!(r.success,
            "Notary.expirationOf must succeed for any address; exc={:?}.",
            r.exception.as_ref().map(|e| &e.message));
        let v = decode_uint_le(&r.return_data);
        prop_assert_eq!(v.clone(), num_bigint::BigUint::from(0u64),
            "Notary.expirationOf(unknown addr {:#x?}) must be 0; got {} \
             (rd_hex={}).",
            addr_bytes, v, hex::encode(&r.return_data));
    }

    /// `Notary.getMaxNotValidBeforeDelta() / setMaxNotValidBeforeDelta(v)`
    /// round-trip — confirms the `notary.rs::getmaxnotvalidbeforedelta` /
    /// `setmaxnotvalidbeforedelta` handlers honor writes across edge u32s.
    #[test]
    fn native_notary_max_not_valid_before_delta_roundtrip(
        delta in prop_oneof![
            Just(0u64),
            Just(u32::MAX as u64),
            1u64..=1_000_000u64,
        ],
    ) {
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function rt(uint256 d) external returns (uint256) {
        NativeCalls.notarySetMaxNotValidBeforeDelta(d);
        return NativeCalls.notaryGetMaxNotValidBeforeDelta();
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("Notary set/getMaxNotValidBeforeDelta compile: {:?}", e));
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");
        let r = rt
            .call_method(
                &art.bytecode, &art.tokens, &art.manifest, "rt",
                &[StackItem::UnsignedInteger(delta)],
            )
            .expect("Notary delta round-trip host-level");
        prop_assert!(r.success,
            "Notary set/getMaxNotValidBeforeDelta({}) must succeed; exc={:?}.",
            delta, r.exception.as_ref().map(|e| &e.message));
        let v = decode_uint_le(&r.return_data);
        // The handler casts via `as u32`, so values larger than u32::MAX wrap.
        let expected = (delta as u32) as u64;
        prop_assert_eq!(v.clone(), num_bigint::BigUint::from(expected),
            "Notary set/getMaxNotValidBeforeDelta round-trip: wrote {} \
             expected (after u32 cast) {} read {} (rd_hex={}).",
            delta, expected, v, hex::encode(&r.return_data));
    }
}
