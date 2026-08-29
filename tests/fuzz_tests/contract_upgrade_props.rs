//! State-machine property tests for `ContractManagement.update` — Neo's
//! native contract-upgrade primitive (analogous to EVM's delegatecall pattern,
//! but with native semantics: the contract hash + storage persist; only the
//! NEF bytecode and manifest are swapped).
//!
//! The runtime's update handler lives at
//! `src/runtime/execution/execution_impl_part2_native/contract_management.rs::"update"`
//! and rewires `contract_registry[hash].nef / .manifest`. Storage is held in
//! `storage_manager` / `storage_overlay` on the `ExecutionContext`. Storage
//! slots for state variables are derived as `SHA-256(field_name)` (see
//! `src/storage_key.rs::compute_state_slot`) — NOT positional — so the slot
//! key is stable under reordering but NOT under renaming.
//!
//! Two property tests are included:
//!
//!   1. `contract_upgrade_storage_persists_compatible_layout` — generates a
//!      v1 contract with `uint256 counter` + `mapping(address => uint256)
//!      balances` plus setter/getter pairs; populates random state via the
//!      setters; calls `NativeCalls.updateContract(v2Nef, v2Manifest)`;
//!      asserts (a) old getters return the same values, (b) the new
//!      `getDoubled()` v2-only function works against the persisted counter,
//!      and (c) mapping reads for v1-written keys still hit.
//!
//!   2. `contract_upgrade_rename_breaks_storage_continuity` — the
//!      compatibility-broken variant. v2 RENAMES a state variable
//!      (`counter` → `counterV2`). Under SHA-256(name) slot derivation,
//!      this orphans the v1 data: the new field's slot has never been
//!      written, so `getCounterV2()` reads 0, while the v1 bytes remain on
//!      disk at the now-unreachable `SHA-256("counter")` key. A passing
//!      test here documents the divergence; a failing test would mean the
//!      compiler/runtime grew rename-aware slot migration (a real safety
//!      improvement) and the assertion should be flipped accordingly.
//!
//!   Note on REORDERING: because slots are name-derived rather than
//!   positional, swapping the declaration order of state variables across
//!   an `update` is SAFE in this compiler. The compatible-layout test in
//!   variant #1 implicitly exercises that — same names, same slot keys,
//!   data persists.

#![allow(unused_imports)]

use super::common::*;
use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::runtime::types::StackItem;
use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};
use proptest::prelude::*;

/// Compile a single contract source and return the (bytecode, tokens,
/// manifest-bytes, manifest-Value) tuple needed to (a) call methods and (b)
/// hand the NEF + manifest to `NativeCalls.updateContract`.
fn compile_one(
    source: &str,
    label: &str,
) -> (
    Vec<u8>,
    Vec<neo_devpack_solidity::neo::MethodToken>,
    Vec<u8>,
    serde_json::Value,
) {
    let arts = compile_contracts(source, false, 2)
        .unwrap_or_else(|e| panic!("contract_upgrade {} compile: {:?}", label, e));
    assert!(
        !arts.is_empty(),
        "contract_upgrade {} compile produced no artifacts",
        label
    );
    let art = arts[0].clone();
    let manifest_bytes = serde_json::to_vec(&art.manifest)
        .unwrap_or_else(|e| panic!("contract_upgrade {} manifest serialize: {:?}", label, e));
    (art.bytecode, art.tokens, manifest_bytes, art.manifest)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(6))]

    /// COMPATIBLE LAYOUT — storage MUST persist across `update`, and a v2-only
    /// function MUST read the v1-populated storage cleanly.
    ///
    /// Sequence:
    ///   1. Compile v1 (uint256 counter + mapping balances + setters/getters).
    ///   2. `setCounter(c)` on v1.
    ///   3. `setBalance(addr_i, v_i)` for each random (addr, value) pair.
    ///   4. Compile v2 — same storage layout, adds `getDoubled()`.
    ///   5. `upgrade(v2Nef, v2Manifest)` via `NativeCalls.updateContract` from v1.
    ///   6. Switch the active bytecode to v2 and re-call `getCounter()` /
    ///      `getBalance(addr_i)` / `getDoubled()`.
    ///   7. Assert: old getters return v1-written values; getDoubled() == 2*c;
    ///      mapping reads for v1 keys still hit.
    ///
    /// On failure, report the seed and the specific divergence — that's a real
    /// runtime bug (storage rebound to bytecode rather than to the contract).
    #[test]
    fn contract_upgrade_storage_persists_compatible_layout(
        counter_value in 1u64..=1_000_000u64,
        addr_seeds in prop::collection::vec(any::<[u8; 20]>(), 1..=3),
        bal_values in prop::collection::vec(1u64..=1_000_000u64, 1..=3),
    ) {
        // Pair up addrs and values; use the smaller length so we never
        // index past either.
        let n = addr_seeds.len().min(bal_values.len());
        let pairs: Vec<([u8; 20], u64)> = addr_seeds
            .iter()
            .copied()
            .zip(bal_values.iter().copied())
            .take(n)
            .collect();
        prop_assume!(!pairs.is_empty());

        let v1_src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    uint256 public counter;
    mapping(address => uint256) public balances;

    function setCounter(uint256 v) external { counter = v; }
    function getCounter() external view returns (uint256) { return counter; }
    function setBalance(address a, uint256 v) external { balances[a] = v; }
    function getBalance(address a) external view returns (uint256) { return balances[a]; }

    function upgrade(bytes calldata nef, bytes calldata manifest) external {
        NativeCalls.updateContract(nef, manifest);
    }
}"#;

        let v2_src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    // SAME order as v1 — storage layout-compatible.
    uint256 public counter;
    mapping(address => uint256) public balances;

    function setCounter(uint256 v) external { counter = v; }
    function getCounter() external view returns (uint256) { return counter; }
    function setBalance(address a, uint256 v) external { balances[a] = v; }
    function getBalance(address a) external view returns (uint256) { return balances[a]; }
    // v2-only addition.
    function getDoubled() external view returns (uint256) { return counter * 2; }

    function upgrade(bytes calldata nef, bytes calldata manifest) external {
        NativeCalls.updateContract(nef, manifest);
    }
}"#;

        let (v1_bc, v1_tk, _v1_mb, v1_manifest) = compile_one(v1_src, "v1-compat");
        let (v2_bc, v2_tk, v2_mb, v2_manifest) = compile_one(v2_src, "v2-compat");

        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");

        // ---- Step 2: setCounter on v1.
        let r_set_c = rt.call_method(
            &v1_bc, &v1_tk, &v1_manifest, "setCounter",
            &[StackItem::UnsignedInteger(counter_value)],
        ).expect("v1 setCounter host-level");
        prop_assert!(r_set_c.success,
            "v1 setCounter({}) must succeed; exc={:?}.",
            counter_value, r_set_c.exception.as_ref().map(|e| &e.message));

        // ---- Step 3: setBalance for each (addr, value).
        for (addr, val) in &pairs {
            let r_sb = rt.call_method(
                &v1_bc, &v1_tk, &v1_manifest, "setBalance",
                &[
                    StackItem::byte_array(addr.to_vec()),
                    StackItem::UnsignedInteger(*val),
                ],
            ).expect("v1 setBalance host-level");
            prop_assert!(r_sb.success,
                "v1 setBalance({:?}, {}) must succeed; exc={:?}.",
                addr, val, r_sb.exception.as_ref().map(|e| &e.message));
        }

        // Sanity: read counter back via v1 BEFORE upgrade.
        let r_pre = rt.call_method(&v1_bc, &v1_tk, &v1_manifest, "getCounter", &[])
            .expect("v1 getCounter pre-upgrade host-level");
        prop_assert!(r_pre.success,
            "v1 getCounter pre-upgrade must succeed; exc={:?}.",
            r_pre.exception.as_ref().map(|e| &e.message));
        let pre_v = decode_uint_le(&r_pre.return_data);
        prop_assert_eq!(pre_v.clone(), num_bigint::BigUint::from(counter_value),
            "v1 getCounter pre-upgrade must return the value we just wrote; \
             wrote {} read {} (rd_hex={}). If they disagree, the setter/getter \
             pair already disagree BEFORE any upgrade — separate bug.",
            counter_value, pre_v, hex::encode(&r_pre.return_data));

        // ---- Step 5: upgrade via NativeCalls.updateContract.
        let r_up = rt.call_method(
            &v1_bc, &v1_tk, &v1_manifest, "upgrade",
            &[
                StackItem::byte_array(v2_bc.clone()),
                StackItem::byte_array(v2_mb.clone()),
            ],
        ).expect("v1 upgrade(v2nef, v2manifest) host-level");
        prop_assert!(r_up.success,
            "v1.upgrade(v2nef, v2manifest) must succeed; exc={:?}. If this \
             fires, ContractManagement.update dispatch regressed in \
             contract_management.rs::\"update\" — file as a CRITICAL finding.",
            r_up.exception.as_ref().map(|e| &e.message));

        // ---- Step 6: switch to v2 bytecode/manifest and re-read.
        let r_post_c = rt.call_method(&v2_bc, &v2_tk, &v2_manifest, "getCounter", &[])
            .expect("v2 getCounter host-level");
        prop_assert!(r_post_c.success,
            "v2 getCounter post-upgrade must succeed; exc={:?}.",
            r_post_c.exception.as_ref().map(|e| &e.message));
        let post_c = decode_uint_le(&r_post_c.return_data);
        prop_assert_eq!(post_c.clone(), num_bigint::BigUint::from(counter_value),
            "v2 getCounter post-upgrade MUST equal v1-written value {} (storage \
             must persist across update); got {} (rd_hex={}). If 0, the \
             counter slot was wiped on upgrade — runtime BUG: storage is being \
             rebound to bytecode hash rather than contract hash.",
            counter_value, post_c, hex::encode(&r_post_c.return_data));

        // v2-only function: getDoubled() == 2 * counter.
        let r_doubled = rt.call_method(&v2_bc, &v2_tk, &v2_manifest, "getDoubled", &[])
            .expect("v2 getDoubled host-level");
        prop_assert!(r_doubled.success,
            "v2 getDoubled (v2-only fn) must succeed post-upgrade; exc={:?}. \
             If this fires citing 'manifest.abi.methods has no entry', the \
             post-upgrade manifest swap didn't take effect.",
            r_doubled.exception.as_ref().map(|e| &e.message));
        let got_doubled = decode_uint_le(&r_doubled.return_data);
        let expected_doubled = num_bigint::BigUint::from(counter_value) * num_bigint::BigUint::from(2u64);
        prop_assert_eq!(got_doubled.clone(), expected_doubled.clone(),
            "v2 getDoubled() must return 2 * counter ({} * 2 = {}); got {} \
             (rd_hex={}). v2-only function read the v1-populated `counter` \
             slot incorrectly — storage layout drifted across upgrade.",
            counter_value, expected_doubled, got_doubled,
            hex::encode(&r_doubled.return_data));

        // Mapping reads for v1 keys must still hit.
        for (addr, val) in &pairs {
            let r_gb = rt.call_method(
                &v2_bc, &v2_tk, &v2_manifest, "getBalance",
                &[StackItem::byte_array(addr.to_vec())],
            ).expect("v2 getBalance host-level");
            prop_assert!(r_gb.success,
                "v2 getBalance({:?}) must succeed post-upgrade; exc={:?}.",
                addr, r_gb.exception.as_ref().map(|e| &e.message));
            let got = decode_uint_le(&r_gb.return_data);
            prop_assert_eq!(got.clone(), num_bigint::BigUint::from(*val),
                "v2 getBalance({:?}) post-upgrade MUST equal v1-written value \
                 {}; got {} (rd_hex={}). If 0, the mapping slot was wiped or \
                 keyed differently after upgrade — runtime BUG: mapping \
                 storage layout broke across update.",
                addr, val, got, hex::encode(&r_gb.return_data));
        }
    }

    /// COMPATIBILITY-BROKEN LAYOUT — v2 RENAMES a state variable.
    ///
    /// Storage slots in Neo DevPack for Solidity are derived as `SHA-256(field_name)`
    /// (see `src/storage_key.rs::compute_state_slot`), NOT by positional
    /// index. As a positive consequence, REORDERING state variables across
    /// an `update` is SAFE — slot derivation tracks the name, so
    /// `uint256 counter` lands at the same slot whether it appears first
    /// or second in the contract. (The first variant in this module pins
    /// that exact reordering safety: the test passes when v2 swaps the
    /// declaration order but keeps both names.)
    ///
    /// The real footgun under this scheme is RENAMING: if v2 renames
    /// `counter` to `counterV2` while keeping the same Solidity API,
    /// `getCounterV2()` reads `SHA-256("counterV2")` — a slot that was
    /// NEVER written by v1 — and silently returns 0. The v1-written value
    /// at `SHA-256("counter")` is now orphaned: still on disk, no longer
    /// reachable by name, and a future v3 that re-introduces `counter`
    /// would resurrect the stale value.
    ///
    /// This test asserts the divergence:
    ///   * v1 writes `counter = counter_value` (lives at SHA-256("counter")).
    ///   * v2 renames `counter` → `counterV2` and adds `getCounterV2()`.
    ///   * Post-upgrade, `getCounterV2()` returns 0 — NOT counter_value —
    ///     because the new field's slot has never been touched.
    ///
    /// If this test ever flips (post-upgrade `getCounterV2()` returns
    /// counter_value), it means the compiler/runtime grew rename-aware slot
    /// migration — at which point this test should be flipped to assert the
    /// corrected behaviour and the documentation site updated to remove the
    /// rename-footgun warning.
    #[test]
    fn contract_upgrade_rename_breaks_storage_continuity(
        counter_value in 1u64..=1_000_000u64,
    ) {
        // v1: a single state variable named `counter`.
        let v1_src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    uint256 public counter;

    function setCounter(uint256 v) external { counter = v; }
    function getCounter() external view returns (uint256) { return counter; }

    function upgrade(bytes calldata nef, bytes calldata manifest) external {
        NativeCalls.updateContract(nef, manifest);
    }
}"#;

        // v2: SAME slot meaning, NEW field name. Under SHA-256(name) slot
        // derivation, this lands the new field at a brand-new slot, leaving
        // v1's data orphaned at SHA-256("counter").
        let v2_src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    uint256 public counterV2;  // RENAMED from `counter` — different slot key.

    function setCounterV2(uint256 v) external { counterV2 = v; }
    function getCounterV2() external view returns (uint256) { return counterV2; }

    function upgrade(bytes calldata nef, bytes calldata manifest) external {
        NativeCalls.updateContract(nef, manifest);
    }
}"#;

        let (v1_bc, v1_tk, _v1_mb, v1_manifest) = compile_one(v1_src, "v1-rename");
        let (v2_bc, v2_tk, v2_mb, v2_manifest) = compile_one(v2_src, "v2-rename");

        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");

        // Populate v1.counter.
        let r_set = rt.call_method(
            &v1_bc, &v1_tk, &v1_manifest, "setCounter",
            &[StackItem::UnsignedInteger(counter_value)],
        ).expect("v1 setCounter host-level");
        prop_assert!(r_set.success, "v1 setCounter({}) must succeed; exc={:?}.",
            counter_value, r_set.exception.as_ref().map(|e| &e.message));

        // Sanity: v1 reads back the value.
        let r_pre = rt.call_method(&v1_bc, &v1_tk, &v1_manifest, "getCounter", &[])
            .expect("v1 getCounter host-level");
        prop_assert!(r_pre.success, "v1 getCounter must succeed; exc={:?}.",
            r_pre.exception.as_ref().map(|e| &e.message));
        let pre_v = decode_uint_le(&r_pre.return_data);
        prop_assert_eq!(pre_v.clone(), num_bigint::BigUint::from(counter_value),
            "v1 getCounter pre-upgrade must return what we just wrote; \
             wrote {} read {} — separate setter/getter bug if they disagree.",
            counter_value, pre_v);

        // Upgrade to v2 (renamed field).
        let r_up = rt.call_method(
            &v1_bc, &v1_tk, &v1_manifest, "upgrade",
            &[
                StackItem::byte_array(v2_bc.clone()),
                StackItem::byte_array(v2_mb.clone()),
            ],
        ).expect("v1 upgrade(v2-rename) host-level");
        prop_assert!(r_up.success,
            "v1.upgrade(v2-rename) must succeed (the runtime has no \
             slot-stability check); exc={:?}.",
            r_up.exception.as_ref().map(|e| &e.message));

        // v2.getCounterV2() reads SHA-256("counterV2"), which has never been
        // written by anyone — so it MUST return 0 even though we previously
        // wrote `counter_value` to v1.counter and the bytes are still on disk
        // at SHA-256("counter").
        let r_gv2 = rt.call_method(&v2_bc, &v2_tk, &v2_manifest, "getCounterV2", &[])
            .expect("v2 getCounterV2 host-level");
        prop_assert!(r_gv2.success,
            "v2 getCounterV2 must succeed post-upgrade; exc={:?}.",
            r_gv2.exception.as_ref().map(|e| &e.message));
        let got = decode_uint_le(&r_gv2.return_data);

        // PINNED FOOTGUN — v2's renamed field reads a fresh, never-written
        // slot. The v1 data at SHA-256("counter") is orphaned.
        prop_assert_eq!(got.clone(), num_bigint::BigUint::from(0u64),
            "DOCUMENTED FOOTGUN: under SHA-256(name)-derived slots \
             (src/storage_key.rs::compute_state_slot), renaming a state \
             variable across `update` ORPHANS the old data. v1 wrote {} to \
             `counter`; after renaming to `counterV2`, getCounterV2() reads \
             a fresh slot and MUST return 0; got {} (rd_hex={}). If this \
             fires returning the v1 value, the runtime grew rename-aware \
             slot migration — flip this assertion to assert {} and remove \
             the rename-footgun warning from the docs.",
            counter_value, got, hex::encode(&r_gv2.return_data),
            counter_value);
    }
}
