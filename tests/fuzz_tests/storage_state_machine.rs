//! Storage state-machine proptest.
//!
//! Drives a small Solidity contract with a `mapping(uint256 => uint256)` and a
//! `uint256[]` through random sequences of write / read / delete / iterate
//! operations, maintaining a Rust-side reference model and asserting the
//! contract's observed values match the model after every step.
//!
//! State-machine testing is the gold standard for surfacing storage bugs
//! (slot collisions, non-determinism, off-by-one length tracking, iterator
//! corruption). The sibling `storage_props.rs` exercises individual storage
//! ops; this module sequences them and verifies model equivalence — which is
//! what catches "interaction" bugs that single-op tests can never observe.
//!
//! Pitfalls (encoded as test rules):
//!
//!   * `pop` on an empty Solidity array is a Panic(0x31). The reference model
//!     skips the contract call when its `Vec` is empty so we don't intentionally
//!     trip the panic shape; PopArr on a non-empty array exercises the real
//!     length-decrement + slot-clearing path.
//!   * Mapping reads on absent keys must return zero. Both the contract and
//!     the reference model agree on this: the model's `HashMap::get(k)`
//!     defaults to `&0u64` via `.copied().unwrap_or(0)`.
//!   * State persistence: a single `NeoRuntime` is reused across every call
//!     within a test case. `call_method` flushes the storage overlay on halt
//!     so subsequent calls observe the previous writes.
//!   * Push values are narrowed to `u32` so they always fit inside `i64`
//!     (the `StackItem::Integer(i64)` envelope) and so summing pushes can
//!     never overflow the model's `u64`.

#![allow(unused_imports)]

use super::common::*;
use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::runtime::types::StackItem;
use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};
use proptest::prelude::*;
use std::collections::HashMap;

/// Operations driven by the state machine.
#[derive(Debug, Clone)]
enum Op {
    /// Set `m[k] = v`.
    SetM(u32, u32),
    /// Delete `m[k]` (Solidity `delete m[k]`).
    DelM(u32),
    /// `arr.push(v)`.
    PushArr(u32),
    /// `arr.pop()`. No-op when the reference array is empty (skips the
    /// contract call too, to avoid the Panic(0x31) under-flow shape).
    PopArr,
}

fn op_strategy() -> impl Strategy<Value = Op> {
    // Restrict the key space to a handful of values so the generated
    // sequences exercise the same key repeatedly (set → del → re-set), which
    // is the most likely locus of slot-bookkeeping bugs.
    let key = 0u32..8;
    prop_oneof![
        (key.clone(), any::<u32>()).prop_map(|(k, v)| Op::SetM(k, v)),
        key.prop_map(Op::DelM),
        any::<u32>().prop_map(Op::PushArr),
        Just(Op::PopArr),
    ]
}

const SOURCE: &str = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract State {
    mapping(uint256 => uint256) public m;
    uint256[] public arr;
    function setM(uint256 k, uint256 v) external { m[k] = v; }
    function delM(uint256 k) external { delete m[k]; }
    function getM(uint256 k) external view returns (uint256) { return m[k]; }
    function pushArr(uint256 v) external { arr.push(v); }
    function popArr() external { arr.pop(); }
    function lenArr() external view returns (uint256) { return arr.length; }
    function getArr(uint256 i) external view returns (uint256) { return arr[i]; }
}"#;

/// Decode a `uint256` return from `call_method` into a `u64`. The runtime
/// emits variable-width little-endian bytes (NeoVM `BigInteger` native
/// encoding stripped of trailing zero bytes); empty `return_data` decodes
/// to zero. We bound test inputs to `u32` so every legal model value fits
/// in a `u64` reading window without truncation.
fn decode_u64(bytes: &[u8]) -> u64 {
    let big = decode_uint_le(bytes);
    // u32-bounded inputs never overflow u64; if they do, that's a bug we
    // want to surface — `to_u64_digits` returns zero limbs on `0`, one on
    // values < 2^64, and the test will visibly diverge from the model.
    let digits = big.to_u64_digits();
    match digits.len() {
        0 => 0,
        1 => digits[0],
        _ => panic!(
            "decode_u64: contract returned a value > 2^64 ({:?}); a u32-bounded \
             input set should never produce this — a slot collision or \
             integer-encoding bug is the most likely cause",
            big
        ),
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn storage_state_machine_matches_reference_model(
        ops in prop::collection::vec(op_strategy(), 0..50)
    ) {
        // Compile once per case. Compile failures here are unambiguous
        // regressions in the front-end (the source is a fixed literal); we
        // panic loudly so the failing case isn't masked as "test rejected".
        let arts = compile_contracts(SOURCE, false, 2)
            .unwrap_or_else(|e| panic!("storage_state_machine compile: {:?}", e));
        prop_assert!(!arts.is_empty(), "storage_state_machine: no artifacts");
        let art = &arts[0];

        let mut rt = NeoRuntime::new(RuntimeConfig::default())
            .expect("storage_state_machine runtime");

        // Reference model — `HashMap` for the mapping (`get` defaults to 0)
        // and `Vec` for the dynamic array.
        let mut ref_m: HashMap<u64, u64> = HashMap::new();
        let mut ref_arr: Vec<u64> = Vec::new();

        // Track every key we have ever touched (via `SetM` or `DelM`) so the
        // post-step verification reads include keys that the model has seen
        // erased — those should still observe 0 from the contract.
        let mut touched_keys: Vec<u64> = Vec::new();

        for (step, op) in ops.iter().enumerate() {
            match op {
                Op::SetM(k, v) => {
                    let k64 = *k as u64;
                    let v64 = *v as u64;
                    let r = rt.call_method(
                        &art.bytecode, &art.tokens, &art.manifest,
                        "setM",
                        &[StackItem::Integer(k64 as i64), StackItem::Integer(v64 as i64)],
                    ).expect("setM host-level");
                    prop_assert!(
                        r.success,
                        "step {}: setM({}, {}) faulted: {:?}",
                        step, k64, v64, r.exception.as_ref().map(|e| &e.message)
                    );
                    ref_m.insert(k64, v64);
                    if !touched_keys.contains(&k64) { touched_keys.push(k64); }
                }
                Op::DelM(k) => {
                    let k64 = *k as u64;
                    let r = rt.call_method(
                        &art.bytecode, &art.tokens, &art.manifest,
                        "delM",
                        &[StackItem::Integer(k64 as i64)],
                    ).expect("delM host-level");
                    prop_assert!(
                        r.success,
                        "step {}: delM({}) faulted: {:?}",
                        step, k64, r.exception.as_ref().map(|e| &e.message)
                    );
                    ref_m.remove(&k64);
                    if !touched_keys.contains(&k64) { touched_keys.push(k64); }
                }
                Op::PushArr(v) => {
                    let v64 = *v as u64;
                    let r = rt.call_method(
                        &art.bytecode, &art.tokens, &art.manifest,
                        "pushArr",
                        &[StackItem::Integer(v64 as i64)],
                    ).expect("pushArr host-level");
                    prop_assert!(
                        r.success,
                        "step {}: pushArr({}) faulted: {:?}",
                        step, v64, r.exception.as_ref().map(|e| &e.message)
                    );
                    ref_arr.push(v64);
                }
                Op::PopArr => {
                    if ref_arr.is_empty() {
                        // Skip the contract call too: an empty pop is a
                        // Panic(0x31) and would leave the runtime in a
                        // post-fault state for the next op. Modelling it as
                        // a no-op keeps the state machine tractable.
                        continue;
                    }
                    let r = rt.call_method(
                        &art.bytecode, &art.tokens, &art.manifest,
                        "popArr",
                        &[],
                    ).expect("popArr host-level");
                    prop_assert!(
                        r.success,
                        "step {}: popArr() (len={}) faulted: {:?}",
                        step, ref_arr.len(), r.exception.as_ref().map(|e| &e.message)
                    );
                    ref_arr.pop();
                }
            }

            // ---- Invariant block: read back every observable slot. ----

            // (a) Mapping: every touched key (live or erased) must match
            //     the model. Erased keys read as 0 from both sides.
            for k in &touched_keys {
                let r = rt.call_method(
                    &art.bytecode, &art.tokens, &art.manifest,
                    "getM",
                    &[StackItem::Integer(*k as i64)],
                ).expect("getM host-level");
                prop_assert!(
                    r.success,
                    "step {}: getM({}) faulted: {:?}",
                    step, k, r.exception.as_ref().map(|e| &e.message)
                );
                let observed = decode_u64(&r.return_data);
                let expected = ref_m.get(k).copied().unwrap_or(0);
                prop_assert_eq!(
                    observed, expected,
                    "step {}: getM({}) divergence after op {:?}: contract={}, model={}",
                    step, k, op, observed, expected
                );
            }

            // (b) Array length must match.
            let r_len = rt.call_method(
                &art.bytecode, &art.tokens, &art.manifest,
                "lenArr",
                &[],
            ).expect("lenArr host-level");
            prop_assert!(
                r_len.success,
                "step {}: lenArr() faulted: {:?}",
                step, r_len.exception.as_ref().map(|e| &e.message)
            );
            let observed_len = decode_u64(&r_len.return_data);
            prop_assert_eq!(
                observed_len, ref_arr.len() as u64,
                "step {}: lenArr divergence after op {:?}: contract={}, model={}",
                step, op, observed_len, ref_arr.len()
            );

            // (c) Each in-bounds array index must match the model.
            for (i, expected_v) in ref_arr.iter().enumerate() {
                let r = rt.call_method(
                    &art.bytecode, &art.tokens, &art.manifest,
                    "getArr",
                    &[StackItem::Integer(i as i64)],
                ).expect("getArr host-level");
                prop_assert!(
                    r.success,
                    "step {}: getArr({}) faulted (len={}): {:?}",
                    step, i, ref_arr.len(), r.exception.as_ref().map(|e| &e.message)
                );
                let observed = decode_u64(&r.return_data);
                prop_assert_eq!(
                    observed, *expected_v,
                    "step {}: getArr({}) divergence after op {:?}: contract={}, model={}",
                    step, i, op, observed, expected_v
                );
            }
        }
    }
}
