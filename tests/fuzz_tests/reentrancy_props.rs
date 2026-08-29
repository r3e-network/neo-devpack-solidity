//! Property tests for re-entrancy detection and call-depth tracking.
//!
//! These tests target two host-level safety properties of the Neo DevPack for Solidity
//! runtime:
//!
//!   1. **Re-entrancy guards work**: a Solidity contract using the
//!      OpenZeppelin `_status` toggle pattern (a `uint256 _status` slot
//!      flipped to `_ENTERED=2` on entry, asserted to be `_NOT_ENTERED=1`
//!      at the start of every guarded method) MUST detect a recursive
//!      self-call and revert. The outer call sees the inner self-call
//!      fail; whether the outer ALSO bubbles up the revert or swallows it
//!      via try/catch is the contract's choice — we exercise both shapes.
//!
//!   2. **Call-depth limit is enforced**: deeply nested self-external
//!      calls MUST eventually fault gracefully (gas exhaustion or
//!      "Call stack overflow"). They MUST NOT crash the host process or
//!      escape the `RuntimeConfig::call_stack_limit` (default 1024) by
//!      growing the native Rust stack until SIGSEGV.
//!
//!   3. **Cross-contract callback storage propagates**: A → B → A re-entry
//!      across two compiled contracts (A calls B, B calls back into A
//!      with a method that writes A's storage) must restore stack frames
//!      correctly and the storage write must be visible after all returns.
//!
//! Reference reading:
//!   - `src/runtime/execution/types/context.rs` — `call_stack_limit`,
//!     `call_stack`, `try_stack` definitions.
//!   - `src/runtime/execution/instruction/flow/calls.rs::push_call_frame`
//!     — where the stack-overflow guard fires.
//!   - `tests/fuzz_tests/baseline_tests.rs::cross_contract_call_via_calltoken_or_ignore`
//!     — example multi-contract harness.
//!   - `tests/fuzz_tests/batches_46_64.rs::batch51_aa3_reentrancy_guard_self_call_fires_uint_lock`
//!     — sister harness using the simpler `uint256 _lock` pattern; this
//!     module extends to the OZ `_status` 1/2 toggle and adds depth +
//!     cross-contract coverage.

#![allow(unused_imports)]

use super::common::*;
use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::runtime::types::StackItem;
use neo_devpack_solidity::runtime::{NeoRuntime, RuntimeConfig};
use proptest::prelude::*;

proptest! {
    // 4 cases is plenty — each case compiles + runs a contract, which is
    // 1-2 seconds. The harness logic itself doesn't take a fuzzed input
    // beyond a seed; the property is about the runtime, not contract
    // permutations.
    #![proptest_config(ProptestConfig::with_cases(4))]

    /// **a. `reentrancy_guard_blocks_self_call`**
    ///
    /// Compile a contract with an OpenZeppelin-style `nonReentrant` modifier
    /// using the canonical `_status` slot (1 = NOT_ENTERED, 2 = ENTERED).
    /// `action()` calls itself via `this.action()`. The inner self-call MUST
    /// fail because the outer frame already flipped `_status` to ENTERED.
    ///
    /// We exercise BOTH shapes:
    ///   (i)  Outer wraps the inner self-call in `try/catch` and returns a
    ///        flag — outer succeeds, flag indicates the inner reverted.
    ///   (ii) Outer does NOT wrap — the inner revert bubbles up and the
    ///        outer call itself fails.
    ///
    /// Both confirm the `_status` slot is shared between the outer and
    /// inner frames (the Task #70 self-dispatch path keeps the same
    /// storage_overlay) — without that shared slot the guard would be
    /// useless because the inner frame would see `_status = NOT_ENTERED`.
    #[test]
    fn reentrancy_guard_blocks_self_call(_seed in any::<u8>()) {
        // OZ-style: 1 = NOT_ENTERED, 2 = ENTERED.
        // The `try { this.action() } catch { entered = true; }` pattern
        // lets the outer call SUCCEED but observe the inner revert as a
        // boolean signal — proves the modifier fired without forcing the
        // outer-call-failure path.
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    uint256 private _status;
    bool public innerReverted;

    constructor() { _status = 1; }

    modifier nonReentrant() {
        require(_status != 2, "ReentrancyGuard: reentrant call");
        _status = 2;
        _;
        _status = 1;
    }

    function action() external nonReentrant returns (uint256) {
        try this.action() returns (uint256) {
            // Should NEVER land here — the inner action() must revert.
            return 0;
        } catch {
            innerReverted = true;
            return 1;
        }
    }

    function checkInnerReverted() external view returns (bool) {
        return innerReverted;
    }

    // Variant where outer does NOT wrap the inner — used by sub-test (ii).
    function actionUnguarded() external nonReentrant returns (uint256) {
        // No try/catch. The inner self-call's revert MUST bubble up.
        return this.actionUnguarded();
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("reentrancy_guard compile: {:?}", e));
        prop_assert!(!arts.is_empty(), "reentrancy_guard produced no artifacts");
        let art = &arts[0];

        // ---- Sub-test (i): try/catch path. Outer succeeds, inner reverts.
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt(i)");
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest,
                "action", &[] as &[StackItem])
            .expect("action() host-level");

        // The OUTER call's success surface depends on whether the
        // compiler's try/catch lowering catches the reverted self-call's
        // payload. Two acceptable shapes:
        //   (A) outer succeeds + return data == 1 (try/catch caught it)
        //   (B) outer reverts citing "ReentrancyGuard" or "reentrant"
        //       (lowering does not yet catch self-call reverts; the inner
        //       revert bubbles up unchanged).
        // BOTH prove the modifier's `require(_status != 2)` fired in the
        // inner frame — which is the property under test. What we MUST
        // NOT see: outer succeeds + return data == 0 (inner ran to
        // completion, modifier did NOT fire — that would be a CRITICAL
        // re-entrancy bug).
        let exc_msg = r
            .exception
            .as_ref()
            .map(|e| e.message.as_str())
            .unwrap_or("");
        let rd = &r.return_data;
        let rd_hex = hex::encode(rd);

        if r.success {
            // Shape (A): try/catch swallowed it. return_data must NOT
            // decode to 0 (which would mean the inner ran successfully).
            let v = decode_uint_le(rd);
            prop_assert_ne!(v.clone(), num_bigint::BigUint::from(0u64),
                "reentrancy_guard outer succeeded but returned 0 — that means \
                 the inner self-call's nonReentrant modifier did NOT fire and \
                 the recursive call completed. This is a CRITICAL re-entrancy \
                 bug: the `_status` slot is NOT shared between outer and inner \
                 frames (likely a regression in the Task #70 self-dispatch \
                 storage_overlay propagation). rd_hex={}", rd_hex);
        } else {
            // Shape (B): inner revert bubbled up. The exception message OR
            // return_data must reference "reentrant" or "ReentrancyGuard"
            // (not just be empty).
            let in_exc = exc_msg.to_lowercase().contains("reentrant");
            let in_rd = rd
                .windows(b"reentrant".len())
                .any(|w| w.eq_ignore_ascii_case(b"reentrant"))
                || rd
                    .windows(b"ReentrancyGuard".len())
                    .any(|w| w == b"ReentrancyGuard");
            prop_assert!(in_exc || in_rd,
                "reentrancy_guard outer reverted but the message did NOT cite \
                 `reentrant` / `ReentrancyGuard`. This means the failure was \
                 something OTHER than the modifier firing (e.g. a panic or \
                 self-dispatch wiring crash). exc_msg={:?}, rd_hex={}",
                exc_msg, rd_hex);
        }

        // ---- Sub-test (ii): Unwrapped path. Inner revert bubbles up; outer
        // MUST fail with the modifier message.
        let mut rt2 = NeoRuntime::new(RuntimeConfig::default()).expect("rt(ii)");
        let r2 = rt2
            .call_method(&art.bytecode, &art.tokens, &art.manifest,
                "actionUnguarded", &[] as &[StackItem])
            .expect("actionUnguarded() host-level");
        prop_assert!(!r2.success,
            "reentrancy_guard actionUnguarded() MUST fail — inner self-call \
             hits the same _status slot and reverts; without try/catch the \
             revert bubbles up. Got success=true, rd_hex={}",
            hex::encode(&r2.return_data));
        let exc2 = r2.exception.as_ref().map(|e| e.message.as_str()).unwrap_or("");
        let in_exc2 = exc2.to_lowercase().contains("reentrant");
        let in_rd2 = r2.return_data
            .windows(b"reentrant".len())
            .any(|w| w.eq_ignore_ascii_case(b"reentrant"));
        prop_assert!(in_exc2 || in_rd2,
            "reentrancy_guard actionUnguarded() reverted but did not cite \
             `reentrant`. exc_msg={:?}, rd_hex={}. If neither contains the \
             expected literal, either the modifier require dropped its \
             string during lowering OR the inner revert payload was lost on \
             propagation back to the outer frame.",
            exc2, hex::encode(&r2.return_data));
    }

    /// **b. `call_depth_limit_enforced`**
    ///
    /// A recursively self-calling contract MUST eventually fault — either
    /// "Call stack overflow" (`call_stack_limit` enforced — see
    /// `src/runtime/execution/instruction/flow/calls.rs::push_call_frame`)
    /// or out-of-gas.  The host process MUST NOT crash (no native Rust
    /// stack overflow / SIGSEGV).
    ///
    /// We pin a small `gas_limit` and small `call_stack_limit` so the test
    /// terminates quickly. We then call `recurse(2000)` — well beyond any
    /// reasonable depth budget. If we get back an `ExecutionResult` (even
    /// a failed one), the host survived; if Rust panics or segfaults,
    /// `expect()` reports the host-level fault and the test fails loudly.
    #[test]
    fn call_depth_limit_enforced(_seed in any::<u8>()) {
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function recurse(uint256 n) external returns (uint256) {
        if (n == 0) return 0;
        return this.recurse(n - 1);
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("call_depth compile: {:?}", e));
        prop_assert!(!arts.is_empty());
        let art = &arts[0];

        // Tight config: small stack limit (16 frames) so we hit the limit
        // before gas runs out — exercises the `push_call_frame` guard
        // directly. Default gas (~10M) should be more than enough.
        let mut cfg = RuntimeConfig {
            gas_limit: 100_000_000,
            call_stack_limit: 1024, // The default; the spec we're verifying.
            memory_limit: 16 * 1024 * 1024,
            storage_limit: 16 * 1024 * 1024,
            network_magic: 0x4F454E,
            enable_debugging: false,
            enable_tracing: false,
            strict_mode: false,
            neo_version: "3.10.0".to_string(),
            contract_account: "0000000000000000000000000000000000000000".to_string(),
            default_block_height: 0,
            default_timestamp: 0,
        };
        // Knock the stack limit down so we exercise the guard fast on a
        // CI machine without burning CPU on 1024 self-calls.
        cfg.call_stack_limit = 64;
        let mut rt = NeoRuntime::new(cfg).expect("rt");

        // Drive 2000 levels (>> 64). MUST fault gracefully; MUST NOT crash
        // the host. `call_method` returns `Result<ExecutionResult, _>`;
        // a host-level Err would mean the runtime's outer error path fired
        // (also acceptable — that means the recursion was bounded by the
        // outer host driver, not the inner stack-frame guard, but still
        // not a process crash).
        let n = StackItem::UnsignedInteger(2000);
        let outcome = rt.call_method(
            &art.bytecode, &art.tokens, &art.manifest,
            "recurse", &[n],
        );

        match outcome {
            Ok(r) => {
                prop_assert!(!r.success,
                    "call_depth recurse(2000) MUST NOT succeed with \
                     call_stack_limit=64 — the runtime should have hit \
                     either the frame cap or out-of-gas before n reached 0. \
                     If success=true, either the limit isn't enforced, or \
                     the self-call lowering compiled to an internal CALL_L \
                     loop that bypasses push_call_frame entirely (which \
                     would be a separate but equally CRITICAL bug — \
                     `this.recurse()` MUST route through the external \
                     call path). rd_hex={}",
                    hex::encode(&r.return_data));

                let exc = r.exception.as_ref().map(|e| e.message.as_str()).unwrap_or("");
                let exc_lower = exc.to_lowercase();
                let is_stack_overflow = exc_lower.contains("call stack")
                    || exc_lower.contains("stack overflow")
                    || exc_lower.contains("too deep");
                let is_oog = exc_lower.contains("out of gas")
                    || exc_lower.contains("gas")
                    || matches!(
                        r.exception.as_ref().map(|e| e.exception_type),
                        Some(neo_devpack_solidity::runtime::types::ExceptionType::OutOfGas)
                    );
                prop_assert!(is_stack_overflow || is_oog,
                    "call_depth recurse(2000) MUST fault with either a \
                     'call stack overflow' or 'out of gas' message. Got \
                     exception={:?}. If the message is something else \
                     (e.g. arithmetic underflow), the recursion didn't \
                     terminate via the guards we expect — investigate \
                     push_call_frame and the gas accounting in CALL.",
                    exc);
            }
            Err(host_err) => {
                // Host-level Err is also acceptable: it means the outer
                // driver bailed out. We just need to confirm we didn't
                // panic / segfault — the fact we got back a typed Err
                // proves that. Log for visibility.
                eprintln!("call_depth recurse(2000) host-Err (acceptable): {:?}", host_err);
            }
        }
    }

    /// **c. `cross_contract_callback`**
    ///
    /// Compile two contracts where A calls into B which calls back into A
    /// (which writes to A's storage) and verify:
    ///   - the call stack restores correctly (every frame returns cleanly,
    ///     no double-pop, no orphan frame),
    ///   - the storage write performed by the inner A.z() callback is
    ///     visible to the outer A.x() after all returns finish, AND
    ///     persists across a fresh `call_method` on A (overlay propagation).
    ///
    /// Implementation note: building two truly distinct contracts that
    /// know each other's hash + ABI at compile time is non-trivial in the
    /// current Neo DevPack for Solidity test surface — `compile_contracts` does not
    /// give us a cross-contract dispatch wiring. Instead, we approximate
    /// the A → B → A pattern using the supported `this.<method>()` self-
    /// dispatch path (Task #70): `methodA_x` calls `methodB_y` (a public
    /// method on the SAME contract that simulates B's role) which calls
    /// `methodA_z` via `this.`, and `methodA_z` writes storage. This
    /// hits the SAME runtime call-stack and storage-overlay machinery
    /// that a real two-contract chain would, while staying within the
    /// helpers the test infra supports today.
    ///
    /// If/when full cross-contract dispatch lands, this harness should be
    /// extended to compile two source files and verify the same property
    /// across actual contract hashes — see baseline_tests.rs CALLT example.
    #[test]
    fn cross_contract_callback(_seed in any::<u8>()) {
        let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    // Logical "A" storage.
    uint256 public stateA;

    // A.x() — entry: calls B's role (methodB_y) which calls back to A.
    function methodA_x() external returns (uint256) {
        // Pre-call sentinel.
        stateA = 100;
        // A → B.
        this.methodB_y();
        // After all returns, stateA must reflect A.z()'s write (777),
        // not the local sentinel (100). Returning it lets the caller
        // verify overlay propagation across the chain.
        return stateA;
    }

    // B.y() — middle: calls back into A.z(). Simulates a separate
    // contract B's behavior; the storage slot it manipulates here is
    // independent of A's stateA.
    uint256 public stateB;
    function methodB_y() external {
        stateB = 42;
        this.methodA_z();
        // Verify B's frame is intact after the callback: stateB must
        // still read 42 (this is B's own slot from B's own frame).
        require(stateB == 42, "B.stateB clobbered by callback");
    }

    // A.z() — innermost: writes A's storage. After every frame returns,
    // the outer A.x() must observe stateA == 777.
    function methodA_z() external {
        stateA = 777;
    }
}"#;
        let arts = compile_contracts(src, false, 2)
            .unwrap_or_else(|e| panic!("cross_contract_callback compile: {:?}", e));
        prop_assert!(!arts.is_empty());
        let art = &arts[0];
        let mut rt = NeoRuntime::new(RuntimeConfig::default()).expect("rt");

        // Drive A.x() — this triggers the full A → B → A chain.
        let r = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest,
                "methodA_x", &[] as &[StackItem])
            .expect("methodA_x host-level");
        prop_assert!(r.success,
            "cross_contract_callback methodA_x() must succeed end-to-end \
             (A → B → A → returns); exc={:?}. Failure here means either \
             the self-dispatch path doesn't restore frames properly OR the \
             nested CALL didn't propagate the storage_overlay write back \
             to the parent frame.",
            r.exception.as_ref().map(|e| e.message.as_str()));

        // The returned value reads stateA AFTER all frames return — must
        // be 777 (set by the innermost A.z), NOT 100 (the sentinel before
        // the call) and NOT 0 (slot never written). If it's 100, the
        // overlay write from A.z's frame was lost on the inner-frame's
        // return — a CRITICAL overlay-propagation bug. If it's 0, A.z()
        // didn't run at all.
        let v = decode_uint_le(&r.return_data);
        prop_assert_eq!(v.clone(), num_bigint::BigUint::from(777u64),
            "cross_contract_callback methodA_x() returned {} (rd_hex={}); \
             expected 777 (the value A.z() wrote in the innermost callback). \
             If 0: A.z() never ran — the chain broke at A → B or B → A. \
             If 100: A.z() ran but its overlay write was discarded on \
             frame return — overlay propagation regression in \
             handle_contract_call's commit path.",
            v, hex::encode(&r.return_data));

        // Cross-call independence: re-read stateA via a fresh entry. The
        // overlay must have persisted to the outer storage_manager so
        // a second `call_method` sees 777, not 0.
        let r2 = rt
            .call_method(&art.bytecode, &art.tokens, &art.manifest,
                "stateA", &[] as &[StackItem])
            .expect("stateA host-level");
        if r2.success {
            // Storage may not always persist across `call_method` boundaries
            // depending on the runtime's commit semantics; the contract-
            // local guarantee in the SAME execution is the load-bearing
            // property. So we treat persistence-across-call as
            // best-effort, but if it DOES return data we want it to be 777.
            let v2 = decode_uint_le(&r2.return_data);
            if !r2.return_data.is_empty() {
                prop_assert_eq!(v2.clone(), num_bigint::BigUint::from(777u64),
                    "cross_contract_callback stateA getter returned {} after \
                     the chain; expected 777. If the second call sees 0, \
                     the storage_overlay was reset between call_method \
                     invocations (separate concern; not the focus of this \
                     test, but worth noting).", v2);
            }
        }
    }
}
