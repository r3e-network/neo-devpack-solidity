//! Property tests for Solidity `fallback()` and `receive()` dispatch.
//!
//! Solidity dispatch rules (Solidity 0.8.x spec, "Receive Ether Function"
//! and "Fallback Function"):
//!
//! 1. **Empty calldata**           → `receive()` if defined, else
//!    `fallback()` if defined, else revert.
//! 2. **Calldata < 4 bytes**       → `fallback()` if defined, else revert.
//!    (NOT `receive`: Solidity requires the
//!    *empty* calldata distinction.)
//! 3. **Calldata ≥ 4, selector ✓** → the matching public method.
//! 4. **Calldata ≥ 4, selector ✗** → `fallback()` if defined, else revert.
//!
//! There is an existing one-shot single-selector test
//! (`batch102_zzz2_fallback_dispatch_returns_fallback`) that pins the
//! "unknown selector hits fallback" path with a fixed `0xDEADBEEF` selector.
//! This module adds *coverage breadth* across:
//!
//!  a. `fallback_dispatched_on_unknown_selector` — randomised selectors and
//!     random 0..=8-byte payloads, asserting fallback's distinguishing return
//!     value (0xfa11) is observed across the entire shape space.
//!  b. `receive_dispatched_on_empty_calldata` — verifies that receive and
//!     fallback have *separate* lowering paths (in Neo N3 a Solidity
//!     `receive()` is exposed as `onNEP17Payment` in the manifest), and that
//!     calling each routes to the body that updates the matching `lastCall`
//!     marker.
//!  c. `known_selector_overrides_fallback` — calling the auto-generated
//!     `lastCall()` getter returns the storage value, never the fallback's
//!     side effect.
//!  d. `revert_when_no_fallback_no_receive` — a contract with neither hook
//!     must surface a host-level error (or runtime fault) for unknown
//!     selectors and empty calldata; this verifies the absence of a silent
//!     pass-through.
//!
//! The harness drives `NeoRuntime::call_method`, which dispatches by
//! manifest method name (mirroring how the embedded VM would route an
//! incoming call once selector→method demux has happened). The breadth is
//! over selectors and payload lengths the runtime would feed into the
//! demux. End-to-end calldata-driven dispatch (i.e. a raw byte stream
//! threaded through the synthetic ABI dispatcher head) is owned by the
//! `differential` harness, not these manifest-level invariants.

#![allow(unused_imports)]
#![allow(clippy::uninlined_format_args)]

use super::common::*;
use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::runtime::types::StackItem;
use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};
use proptest::prelude::*;

/// Compute keccak256("known()")[..4] — the canonical Solidity selector for
/// the `known()` external function used by case (a).
fn known_selector() -> [u8; 4] {
    use sha3::{Digest, Keccak256};
    let mut h = Keccak256::new();
    h.update(b"known()");
    let d = h.finalize();
    [d[0], d[1], d[2], d[3]]
}

/// Look up a method entry by name in the compiled manifest. Returns true
/// iff `manifest.abi.methods` contains an entry with this exact name.
fn manifest_has_method(manifest: &serde_json::Value, name: &str) -> bool {
    manifest
        .get("abi")
        .and_then(|a| a.get("methods"))
        .and_then(|m| m.as_array())
        .map(|arr| {
            arr.iter()
                .any(|m| m.get("name").and_then(serde_json::Value::as_str) == Some(name))
        })
        .unwrap_or(false)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// (a) Unknown 4-byte selectors with random short payloads must route
    /// to `fallback()`. The fallback body returns `0xfa11`; `known()`
    /// returns `1`. We assert the fallback marker — never `1` — comes back
    /// regardless of which (non-`known`) selector or payload was supplied.
    #[test]
    fn fallback_dispatched_on_unknown_selector(
        sel in any::<[u8; 4]>(),
        payload in prop::collection::vec(any::<u8>(), 0..=8),
    ) {
        // Skip the rare collision with `known()`'s real selector.
        prop_assume!(sel != known_selector());

        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function known() external pure returns (uint256) { return 1; }
    fallback() external returns (uint256) { return 0xfa11; }
}"#;
        let arts = compile_contracts(src, false, 2)
            .map_err(|e| TestCaseError::fail(format!("compile failed: {:?}", e)))?;
        prop_assert_eq!(arts.len(), 1);
        let art = &arts[0];

        // Manifest must expose both `known` (named external) and `fallback`
        // (catch-all). If `fallback` is missing, the runtime would not have
        // an offset to route to, and call_method would fail at lookup time
        // — so we surface that as a richer assertion *before* invoking.
        prop_assert!(
            manifest_has_method(&art.manifest, "fallback"),
            "compiled manifest is missing the `fallback` method entry; \
             abi.methods={:?}",
            art.manifest.get("abi").and_then(|a| a.get("methods"))
        );
        prop_assert!(
            manifest_has_method(&art.manifest, "known"),
            "compiled manifest is missing the `known` method entry"
        );

        let mut rt = NeoRuntime::new(RuntimeConfig::default())
            .map_err(|e| TestCaseError::fail(format!("rt new: {:?}", e)))?;

        // Concatenate selector + payload as a single calldata byte_array.
        // The fallback signature takes no params, so any extra args are
        // ignored by the prologue; this matches the pattern used by the
        // existing batch102_zzz2 fallback test.
        let mut cd = Vec::with_capacity(4 + payload.len());
        cd.extend_from_slice(&sel);
        cd.extend_from_slice(&payload);

        let r = rt
            .call_method(
                &art.bytecode,
                &art.tokens,
                &art.manifest,
                "fallback",
                &[StackItem::byte_array(cd.clone())],
            )
            .map_err(|e| TestCaseError::fail(format!("call_method fallback: {:?}", e)))?;

        prop_assert!(
            r.success,
            "fallback dispatch must succeed for sel={} payload={}; exc={:?}",
            hex::encode(sel),
            hex::encode(&payload),
            r.exception.as_ref().map(|e| &e.message)
        );

        // Fallback returns 0xfa11; `known()` would return 1. A correct
        // dispatch never produces 1 here because the manifest method
        // `fallback` is invoked directly.
        let v = decode_uint_le(&r.return_data);
        prop_assert_eq!(
            v.clone(),
            num_bigint::BigUint::from(0xfa11u32),
            "fallback must return 0xfa11; got {} (rd_hex={}, sel={}, payload={})",
            v,
            hex::encode(&r.return_data),
            hex::encode(sel),
            hex::encode(&payload)
        );
    }
}

/// (b) Receive vs fallback routing.
///
/// In Neo N3 the Solidity `receive() external payable` lowers to
/// `onNEP17Payment` in the manifest (see
/// `src/cli/tests/integration/receive_hooks.rs`), while `fallback()` keeps
/// its own offset entry. Both bodies write distinct values to `lastCall`
/// so we can confirm the dispatch table preserves the two paths
/// independently — i.e. the fallback offset is *not* aliased onto the
/// receive offset, and vice versa.
///
/// This is a deterministic test (no randomised inputs would change its
/// shape: we are pinning a static lowering invariant), so it lives outside
/// the proptest! block, alongside the other dispatch invariants.
#[test]
fn receive_dispatched_on_empty_calldata() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    uint256 public lastCall;
    receive() external payable { lastCall = 1; }
    fallback() external { lastCall = 2; }
}"#;
    let arts =
        compile_contracts(src, false, 2).unwrap_or_else(|e| panic!("compile failed: {:?}", e));
    assert_eq!(arts.len(), 1);
    let art = &arts[0];

    // Both lowerings must be present in the manifest; receive() is
    // remapped to `onNEP17Payment` on Neo N3 (no other Solidity hook
    // shares that slot), while fallback() keeps its own entry.
    assert!(
        manifest_has_method(&art.manifest, "onNEP17Payment"),
        "expected `receive()` to be lowered as `onNEP17Payment`; abi.methods={:?}",
        art.manifest.get("abi").and_then(|a| a.get("methods"))
    );
    assert!(
        manifest_has_method(&art.manifest, "fallback"),
        "expected `fallback` method entry; abi.methods={:?}",
        art.manifest.get("abi").and_then(|a| a.get("methods"))
    );

    // Drive each path on its own runtime so storage is fresh per assertion.
    // (Sharing one runtime would let an earlier write pollute the later
    // observation.)

    // 1. Empty calldata → receive (onNEP17Payment) → lastCall == 1.
    {
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt(receive)");
        // onNEP17Payment takes (from, amount, data); pass dummy values.
        let r = rt
            .call_method(
                &art.bytecode,
                &art.tokens,
                &art.manifest,
                "onNEP17Payment",
                &[
                    StackItem::byte_array(vec![0u8; 20]),
                    StackItem::Integer(0),
                    StackItem::Null,
                ],
            )
            .expect("call onNEP17Payment");
        assert!(
            r.success,
            "onNEP17Payment (receive lowering) must succeed; exc={:?}",
            r.exception.as_ref().map(|e| &e.message)
        );
        let r2 = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "lastCall", &[])
            .expect("call lastCall after receive");
        assert!(
            r2.success,
            "lastCall getter must succeed; exc={:?}",
            r2.exception.as_ref().map(|e| &e.message)
        );
        let v = decode_uint_le(&r2.return_data);
        assert_eq!(
            v,
            num_bigint::BigUint::from(1u32),
            "after receive() lastCall must be 1; got {} (rd_hex={}). \
             If 2, the fallback body fired instead of the receive body — \
             receive→onNEP17Payment lowering is aliased onto fallback.",
            v,
            hex::encode(&r2.return_data)
        );
    }

    // 2. Non-empty / unknown-selector calldata → fallback → lastCall == 2.
    {
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt(fallback)");
        // 1-byte calldata (< 4 bytes): Solidity rule (2) — fallback only.
        let r = rt
            .call_method(
                &art.bytecode,
                &art.tokens,
                &art.manifest,
                "fallback",
                &[StackItem::byte_array(vec![0xAB])],
            )
            .expect("call fallback (1-byte)");
        assert!(
            r.success,
            "fallback (1-byte calldata) must succeed; exc={:?}",
            r.exception.as_ref().map(|e| &e.message)
        );
        let r2 = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "lastCall", &[])
            .expect("call lastCall after fallback(1B)");
        assert!(r2.success);
        let v = decode_uint_le(&r2.return_data);
        assert_eq!(
            v,
            num_bigint::BigUint::from(2u32),
            "after fallback() (1-byte calldata) lastCall must be 2; got {} (rd_hex={}). \
             If 1, fallback was aliased to receive — the < 4-byte calldata \
             rule must NOT route to receive.",
            v,
            hex::encode(&r2.return_data)
        );
    }

    // 3. 4-byte unknown selector calldata → fallback → lastCall == 2.
    {
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt(fallback4)");
        let r = rt
            .call_method(
                &art.bytecode,
                &art.tokens,
                &art.manifest,
                "fallback",
                &[StackItem::byte_array(vec![0xDE, 0xAD, 0xBE, 0xEF])],
            )
            .expect("call fallback (4-byte unknown selector)");
        assert!(
            r.success,
            "fallback (4-byte unknown selector) must succeed; exc={:?}",
            r.exception.as_ref().map(|e| &e.message)
        );
        let r2 = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest, "lastCall", &[])
            .expect("call lastCall after fallback(4B)");
        assert!(r2.success);
        let v = decode_uint_le(&r2.return_data);
        assert_eq!(
            v,
            num_bigint::BigUint::from(2u32),
            "after fallback() (4-byte unknown selector) lastCall must be 2; got {} (rd_hex={}). \
             If 1, an unknown selector incorrectly fell through to receive.",
            v,
            hex::encode(&r2.return_data)
        );
    }
}

/// (c) Calling the auto-generated `lastCall()` getter (i.e. a *known*
/// selector) must return the storage value directly — never the fallback's
/// side-effect-only body. This guards against an over-eager fallback
/// route that would shadow named methods.
#[test]
fn known_selector_overrides_fallback() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    uint256 public lastCall;
    receive() external payable { lastCall = 1; }
    fallback() external { lastCall = 2; }
    function setIt(uint256 v) external { lastCall = v; }
}"#;
    let arts =
        compile_contracts(src, false, 2).unwrap_or_else(|e| panic!("compile failed: {:?}", e));
    let art = &arts[0];

    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");

    // Set storage via the explicit setter (not via fallback). After this,
    // lastCall must read 0x1234 — fallback's body must NOT have fired.
    let r = rt
        .call_method(
            &art.bytecode,
            &art.tokens,
            &art.manifest,
            "setIt",
            &[StackItem::Integer(0x1234)],
        )
        .expect("call setIt");
    assert!(
        r.success,
        "setIt must succeed; exc={:?}",
        r.exception.as_ref().map(|e| &e.message)
    );

    // Call the getter — a known selector. Must NOT clobber storage with
    // fallback's value of 2.
    let r2 = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "lastCall", &[])
        .expect("call lastCall getter");
    assert!(
        r2.success,
        "lastCall getter must succeed; exc={:?}",
        r2.exception.as_ref().map(|e| &e.message)
    );
    let v = decode_uint_le(&r2.return_data);
    assert_eq!(
        v,
        num_bigint::BigUint::from(0x1234u32),
        "lastCall() getter must return the stored value (0x1234); got {} \
         (rd_hex={}). If 1 or 2, the named-selector dispatch was \
         overridden by the receive/fallback path.",
        v,
        hex::encode(&r2.return_data)
    );
}

/// (d) A contract with NO fallback and NO receive must produce a
/// host-visible error (or runtime fault) when an unknown method name is
/// invoked, rather than silently returning Null/zero. This pins the
/// "graceful failure" semantics for the no-hook configuration.
///
/// Because `NeoRuntime::call_method` routes by manifest method name, the
/// unknown-method path manifests as `Err(_)` from the manifest lookup
/// step (`manifest.abi.methods has no entry named ...`). That is exactly
/// the graceful-failure shape: a host-level error instead of a panic or a
/// fault that crashes the embedding process.
#[test]
fn revert_when_no_fallback_no_receive() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function only() external pure returns (uint256) { return 7; }
}"#;
    let arts =
        compile_contracts(src, false, 2).unwrap_or_else(|e| panic!("compile failed: {:?}", e));
    let art = &arts[0];

    // Manifest must NOT contain fallback / receive / onNEP17Payment hooks.
    assert!(
        !manifest_has_method(&art.manifest, "fallback"),
        "no-hook contract must not synthesize a fallback; abi.methods={:?}",
        art.manifest.get("abi").and_then(|a| a.get("methods"))
    );
    assert!(
        !manifest_has_method(&art.manifest, "receive"),
        "no-hook contract must not synthesize a receive entry"
    );
    assert!(
        !manifest_has_method(&art.manifest, "onNEP17Payment"),
        "no-hook contract must not synthesize an onNEP17Payment entry"
    );

    let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");

    // 1. The known method works: returns 7. (Sanity check.)
    let r_ok = rt
        .call_method(&art.bytecode, &art.tokens, &art.manifest, "only", &[])
        .expect("call only()");
    assert!(
        r_ok.success,
        "only() must succeed; exc={:?}",
        r_ok.exception.as_ref().map(|e| &e.message)
    );
    let v = decode_uint_le(&r_ok.return_data);
    assert_eq!(
        v,
        num_bigint::BigUint::from(7u32),
        "only() must return 7; got {} (rd_hex={})",
        v,
        hex::encode(&r_ok.return_data)
    );

    // 2. Empty-calldata equivalent: a "receive" method name with no hook.
    //    Must surface a graceful host-level error (Err(_)), not a panic.
    let r_recv = rt.call_method(&art.bytecode, &art.tokens, &art.manifest, "receive", &[]);
    assert!(
        r_recv.is_err(),
        "calling `receive` on a no-hook contract must surface Err(_); \
         instead got Ok({:?}). The runtime is silently absorbing empty \
         calldata where Solidity's spec requires a revert.",
        r_recv.as_ref().ok()
    );

    // 3. Unknown 4-byte selector: invoke under a name that doesn't exist.
    //    `call_method` routes by name; an absent method must not silently
    //    fall through to a default — it must surface Err(_).
    let r_unknown = rt.call_method(
        &art.bytecode,
        &art.tokens,
        &art.manifest,
        "fallback",
        &[StackItem::byte_array(vec![0xDE, 0xAD, 0xBE, 0xEF])],
    );
    assert!(
        r_unknown.is_err(),
        "calling `fallback` on a no-hook contract must surface Err(_); \
         instead got Ok({:?}). If a fallback offset materialized despite \
         no source-level fallback, the lowering is over-synthesizing hooks.",
        r_unknown.as_ref().ok()
    );
}

// ============================================================================
// M-FE1 fix — when a contract defines BOTH receive() AND an explicit
// onNEP17Payment(), the receive() becomes dead code: Neo N3 only invokes
// onNEP17Payment for incoming NEP-17 transfers, so the receive() body never
// fires. Before this fix the compiler emitted only a W105 warning, which was
// easy to miss — an author trusting receive() to log/handle deposits would
// ship a silently broken contract. The fix promotes the coexist case to a
// hard error.
// ============================================================================

#[test]
fn fe1_receive_with_explicit_onnep17_is_a_hard_error() {
    let src = r#"
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract Both {
    /// @dev This is intentionally dead: Neo N3 only invokes onNEP17Payment.
    receive() external payable {
        // never fires on Neo
    }

    function onNEP17Payment(address from, uint256 amount, Any data) external {
        // the real callback
    }
}
"#;
    let result = compile_contracts(src, false, 2);
    assert!(
        result.is_err(),
        "M-FE1: a contract defining both receive() and onNEP17Payment must be \
         a hard error (receive() would be silently dead code); compile \
         unexpectedly succeeded"
    );
}

#[test]
fn fe1_receive_without_onnep17_still_compiles() {
    // Regression guard: receive() alone is fine — it gets remapped to a
    // synthetic onNEP17Payment. Only the BOTH case is an error.
    let src = r#"
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract OnlyReceive {
    receive() external payable {}
}
"#;
    let result = compile_contracts(src, false, 2);
    assert!(
        result.is_ok(),
        "receive() without an explicit onNEP17Payment must still compile (it \
         is remapped); got error: {:?}",
        result.err()
    );
}
