//! Property-based tests verifying that iterative Solidity workloads
//! converge cleanly without leaking memory or unbounded state across
//! iterations on the same `NeoRuntime` instance.
//!
//! These tests are designed to surface subtle bugs that single-call
//! tests cannot detect:
//!
//!   * `vec` growth past `memory_limit` inside a long-lived loop body
//!     (e.g., a per-iteration allocation that never reclaims).
//!   * `storage_overlay` leaks across `call_method` invocations on the
//!     same `NeoRuntime` (the overlay should be drained on every
//!     successful halt — see `bridge::execute` → `apply_storage_overlay`).
//!   * State that doesn't get reset between method invocations on the
//!     same runtime (gas tracker, execution context, iterator handles).
//!
//! Each subtest pins a different facet of "convergence":
//!
//!   a. `loop_convergence_with_break`         — early-break in a tight
//!      loop produces the exact requested return value.
//!   b. `repeated_storage_writes_stable_gas`  — gas-per-call is stable
//!      across N identical calls (no per-call linear-growth fixed cost).
//!   c. `empty_loop_terminates`               — Gauss-formula closed
//!      form for `sum(0..n)` matches the loop accumulator.
//!   d. `memory_overlay_drains_on_call_end`   — 50 sequential
//!      `call_method` invocations on the same runtime each succeed and
//!      gas-used stays bounded.
//!   e. `recursive_returns_stable`            — `this.fib(10)` self-
//!      external recursion yields 55 and tears down 177 frames cleanly.

#![allow(unused_imports)]

use super::common::*;
use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::runtime::types::StackItem;
use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};
use proptest::prelude::*;

// ---------- (a) loop_convergence_with_break ----------

const SOURCE_LOOP_UNTIL: &str = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function loopUntil(uint256 target) external pure returns (uint256) {
        for (uint256 i = 0; i < 100000; i++) {
            if (i >= target) return i;
        }
        return 0xfffffff;
    }
}
"#;

// ---------- (b) repeated_storage_writes_stable_gas ----------

const SOURCE_BUMP_N: &str = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    uint256 public counter;
    function bumpN(uint256 n) external {
        for (uint256 i = 0; i < n; i++) {
            counter += 1;
        }
    }
    function getCounter() external view returns (uint256) {
        return counter;
    }
}
"#;

// ---------- (c) empty_loop_terminates ----------

const SOURCE_SPIN: &str = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function spin(uint256 n) external pure returns (uint256) {
        uint256 acc = 0;
        for (uint256 i = 0; i < n; i++) {
            acc += i;
        }
        return acc;
    }
}
"#;

// ---------- (d) memory_overlay_drains_on_call_end ----------
//
// Each call writes a *unique* key (the `idx` argument) so 50 calls leave
// 50 distinct entries in storage. This contract is intentionally simple:
// the assertion is about runtime state hygiene across the call boundary,
// not about Solidity-side correctness.

const SOURCE_PER_CALL_KEY: &str = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    mapping(uint256 => uint256) public m;
    function writeKey(uint256 idx) external {
        m[idx] = idx + 1;
    }
}
"#;

// ---------- (e) recursive_returns_stable ----------
//
// `this.fib(...)` routes each recursive call through the self-external
// dispatch path (System.Contract.Call back into the same script with
// the `fib` method offset), so `fib(10)` exercises 177 frame setup/
// teardown sequences (fib call count = 2 * fib(n+1) - 1 = 2*89 - 1 = 177).

const SOURCE_FIB: &str = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function fib(uint256 n) external view returns (uint256) {
        if (n < 2) return n;
        return this.fib(n - 1) + this.fib(n - 2);
    }
}
"#;

// ---------- helpers ----------

fn compile_one(src: &str, label: &str) -> neo_devpack_solidity::cli::CompilationArtifacts {
    let arts = compile_contracts(src, false, 2)
        .unwrap_or_else(|e| panic!("convergence_props {} compile: {:?}", label, e));
    assert!(
        !arts.is_empty(),
        "convergence_props {} produced no artifacts",
        label
    );
    arts.into_iter().next().unwrap()
}

proptest! {
    // Compile-and-run is multi-second per case; cap the case count and
    // let PROPTEST_CASES override when a regression is being narrowed.
    #![proptest_config(ProptestConfig::with_cases(8))]

    /// (a) loopUntil(target): the inner loop runs up to 100_000 iters
    ///     but must early-break at `i == target`. The function returns
    ///     `i` (= `target`) on break, or the sentinel `0xfffffff` on the
    ///     fall-through path. We assert `target` is observed for every
    ///     `target ∈ 0..=1000`, pinning that the break branch is reached
    ///     and the loop accumulator is not corrupted by surrounding state.
    #[test]
    fn loop_convergence_with_break(target in 0u64..=1000) {
        let art = compile_one(SOURCE_LOOP_UNTIL, "loopUntil");
        let mut rt = NeoRuntime::new(RuntimeConfig::default())
            .expect("loopUntil runtime");
        let r = rt.call_method(
            &art.bytecode, &art.tokens, &art.manifest,
            "loopUntil",
            &[StackItem::Integer(target as i64)],
        ).expect("loopUntil host-level");
        prop_assert!(
            r.success,
            "loopUntil({}) faulted: {:?}",
            target,
            r.exception.as_ref().map(|e| e.message.clone()),
        );
        let v = decode_uint_le(&r.return_data);
        prop_assert_eq!(
            v.clone(),
            num_bigint::BigUint::from(target),
            "loopUntil({}) must return {} (early-break); got {} (rd_hex={})",
            target, target, v, hex::encode(&r.return_data),
        );
    }

    /// (c) spin(n): closed-form Gauss `sum(0..n) = n*(n-1)/2`. We pin
    ///     this for n ∈ 0..=200 to assert the accumulator does not drift
    ///     across iterations and the loop terminates at exactly `n` iters.
    #[test]
    fn empty_loop_terminates(n in 0u64..=200) {
        let art = compile_one(SOURCE_SPIN, "spin");
        let mut rt = NeoRuntime::new(RuntimeConfig::default())
            .expect("spin runtime");
        let r = rt.call_method(
            &art.bytecode, &art.tokens, &art.manifest,
            "spin",
            &[StackItem::Integer(n as i64)],
        ).expect("spin host-level");
        prop_assert!(
            r.success,
            "spin({}) faulted: {:?}",
            n,
            r.exception.as_ref().map(|e| e.message.clone()),
        );
        let observed = decode_uint_le(&r.return_data);
        // Closed form. `n*(n-1)/2` is exact for n <= 200 (max 19900).
        let expected = num_bigint::BigUint::from(n * n.saturating_sub(1) / 2);
        prop_assert_eq!(
            observed.clone(), expected.clone(),
            "spin({}) must return n*(n-1)/2 = {}; got {} (rd_hex={})",
            n, expected, observed, hex::encode(&r.return_data),
        );
    }
}

// ---------- non-proptest convergence checks ----------
//
// (b), (d), (e) don't need a generator — they're convergence assertions
// over a fixed sequence of calls on a single `NeoRuntime`. Implemented
// as plain `#[test]` fns so they always run (rather than being subject
// to proptest case shrinking).

/// (b) Three consecutive `bumpN(50)` calls on the same `NeoRuntime`:
///
///     - The terminal `counter` value must be exactly 150 (50 + 50 + 50).
///       Any deviation indicates either storage isn't persisting across
///       calls or the overlay is being double-applied.
///     - Per-call gas-used must be approximately equal across the three
///       calls. A linearly growing per-call cost would surface a
///       state-leak (e.g., a `Vec` in the execution context that grows
///       on each call without being reset on `initialize`).
#[test]
fn repeated_storage_writes_stable_gas() {
    let art = compile_one(SOURCE_BUMP_N, "bumpN");
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("bumpN runtime");

    let mut gas_per_call: [u64; 3] = [0; 3];
    for (idx, slot) in gas_per_call.iter_mut().enumerate() {
        let r = rt
            .call_method(
                &art.bytecode,
                &art.tokens,
                &art.manifest,
                "bumpN",
                &[StackItem::Integer(50)],
            )
            .expect("bumpN host-level");
        assert!(
            r.success,
            "bumpN call #{} faulted: {:?}. If only call #2/#3 fault, the \
             storage overlay is leaking writes from the previous call \
             (e.g. drain_dirty_storage_overlay isn't running on commit), \
             or the gas tracker isn't being reset between calls.",
            idx + 1,
            r.exception.as_ref().map(|e| &e.message),
        );
        *slot = r.gas_used;
    }

    // (a) Counter must be 150.
    let r_get = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "getCounter", &[])
        .expect("getCounter host-level");
    assert!(
        r_get.success,
        "getCounter faulted: {:?}",
        r_get.exception.as_ref().map(|e| &e.message),
    );
    let counter_val = decode_uint_le(&r_get.return_data);
    assert_eq!(
        counter_val,
        num_bigint::BigUint::from(150u64),
        "counter must equal 150 after 3 × bumpN(50); got {} (rd_hex={}). \
         If lower than 150, storage writes did not persist across calls. \
         If higher, the overlay is being committed multiple times.",
        counter_val,
        hex::encode(&r_get.return_data),
    );

    // (b) Gas-per-call stability: each call's gas should be within
    //     ±20% of the median (or, equivalently, the maximum delta from
    //     the minimum should be ≤ 20% of the minimum). 20% is a
    //     deliberately generous bound that still flags a clear
    //     linear-growth leak (e.g., `gas[2] - gas[0]` ≫ noise floor).
    //     The 3-call window keeps drift hard to mask: a doubling on
    //     the second call would already trip a 20% bound.
    let g_min = *gas_per_call.iter().min().unwrap();
    let g_max = *gas_per_call.iter().max().unwrap();
    let delta = g_max.saturating_sub(g_min);
    // Avoid div-by-zero on a contrived zero-gas profile.
    let g_min_for_pct = g_min.max(1);
    let pct_drift = (delta as f64) / (g_min_for_pct as f64);
    assert!(
        pct_drift <= 0.20,
        "bumpN(50) gas-per-call drifted >20% across 3 sequential calls \
         on the SAME NeoRuntime: gas={:?}, min={}, max={}, drift={:.2}%. \
         A drift >20% indicates per-call linear-growth fixed costs — most \
         commonly: storage_overlay accumulating across calls, or the gas \
         tracker not being fully reset between invocations.",
        gas_per_call,
        g_min,
        g_max,
        pct_drift * 100.0,
    );
}

/// (d) 50 separate `call_method` invocations on the same `NeoRuntime`.
///
///     Each call writes a unique key `m[idx] = idx + 1`. We assert:
///       * Every call succeeds (no faults from accumulated state).
///       * `gas_used` per call stays bounded — i.e., the LAST call's
///         gas is not catastrophically larger than the FIRST call's.
///         A hard 5x bound is a deliberately generous ceiling that
///         still flags a per-call O(N) leak (which would push the
///         50th call to ~50x the first).
#[test]
fn memory_overlay_drains_on_call_end() {
    let art = compile_one(SOURCE_PER_CALL_KEY, "writeKey");
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("writeKey runtime");

    const N: usize = 50;
    let mut gas_per_call: Vec<u64> = Vec::with_capacity(N);
    for idx in 0..N {
        let r = rt
            .call_method(
                &art.bytecode,
                &art.tokens,
                &art.manifest,
                "writeKey",
                &[StackItem::Integer(idx as i64)],
            )
            .expect("writeKey host-level");
        assert!(
            r.success,
            "writeKey({}) faulted on call #{}: {:?}. If calls succeed up \
             to some N then start failing, the per-call accumulated state \
             (storage_overlay, evaluation stack, or memory pool) is not \
             being drained on call termination.",
            idx,
            idx + 1,
            r.exception.as_ref().map(|e| &e.message),
        );
        gas_per_call.push(r.gas_used);
    }

    // Bound on growth: the last call's gas should not exceed 5× the
    // first call's gas. Storage iteration over already-committed entries
    // could legitimately add a small linear factor, but 5x is a clear
    // ceiling for a single-key write that itself does no scanning.
    let g_first = gas_per_call.first().copied().unwrap_or(0);
    let g_last = gas_per_call.last().copied().unwrap_or(0);
    let g_first_for_ratio = g_first.max(1);
    let ratio = (g_last as f64) / (g_first_for_ratio as f64);
    assert!(
        ratio <= 5.0,
        "writeKey gas-used grew >5x across 50 sequential calls on the \
         same NeoRuntime: first={}, last={}, ratio={:.2}x. A linear-in-\
         call-count growth signals the storage_overlay (or another \
         per-call cache) is not being drained at the end of each call. \
         Full series: {:?}",
        g_first,
        g_last,
        ratio,
        gas_per_call,
    );

    // Also assert the maximum/min spread isn't pathological.
    let g_min = *gas_per_call.iter().min().unwrap();
    let g_max = *gas_per_call.iter().max().unwrap();
    let g_min_for_ratio = g_min.max(1);
    let max_ratio = (g_max as f64) / (g_min_for_ratio as f64);
    assert!(
        max_ratio <= 5.0,
        "writeKey gas-used max/min ratio across 50 calls = {:.2}x \
         (min={}, max={}). A spread >5x even if the *last* call is \
         cheap suggests an accumulator that grows then shrinks — also a \
         leak shape worth surfacing. Full series: {:?}",
        max_ratio,
        g_min,
        g_max,
        gas_per_call,
    );
}

/// (e) `this.fib(10)` exercises 177 self-external call frames. Each
///     `this.fib(...)` routes through System.Contract.Call back into the
///     same compiled script at the `fib` method offset (Task #70 self-
///     method dispatch table). The result must be 55 (fib(10) = 55).
///
///     Failure modes this test surfaces:
///       * Frame teardown leaking the previous frame's state (would
///         compound and produce a wrong result for fib(n>3)).
///       * Recursion depth limits hit by any per-call stack-growth leak.
#[test]
fn recursive_returns_stable() {
    let art = compile_one(SOURCE_FIB, "fib");
    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("fib rt");
    let r = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "fib",
            &[StackItem::Integer(10)],
        )
        .expect("fib host-level");
    assert!(
        r.success,
        "this.fib(10) faulted: {:?}. If exception mentions 'stack \
         overflow' or 'gas exceeded', a per-frame state leak is \
         compounding across the 177 self-external calls.",
        r.exception.as_ref().map(|e| &e.message),
    );
    let v = decode_uint_le(&r.return_data);
    assert_eq!(
        v,
        num_bigint::BigUint::from(55u64),
        "this.fib(10) must return 55; got {} (rd_hex={}). A non-55 \
         result indicates frame setup/teardown is corrupting the return \
         path on at least one of the 177 self-external invocations.",
        v,
        hex::encode(&r.return_data),
    );
}
