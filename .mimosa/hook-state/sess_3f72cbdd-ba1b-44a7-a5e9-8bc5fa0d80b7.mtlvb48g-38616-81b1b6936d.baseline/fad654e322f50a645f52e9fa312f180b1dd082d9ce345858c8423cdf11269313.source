//! Property-based tests for `StateManager::execute_batch`.
//!
//! Targets `src/runtime/state/impl/batch.rs` — the atomic vs. non-atomic
//! batched state-mutation API. This is a correctness-critical primitive:
//! buggy rollback or buggy partial-apply would corrupt persisted state.
//!
//! ## Canonical contract for `atomic = false`
//!
//! After resolving the wave-#17 ambiguity: the canonical semantic is
//! **"skip-the-invalid-change-and-continue"** (best-effort apply, like
//! Ethereum's `Multicall3.tryAggregate(requireSuccess = false, ...)`).
//!
//! Rationale:
//!   1. The original implementation comment in `batch.rs` is explicit:
//!      `// Continue on error for non-atomic batch`. This is the
//!      documented author intent from commit 163606d.
//!   2. `atomic: true` already covers the "all-or-nothing" semantic with
//!      snapshot rollback. If `atomic: false` also stopped on the first
//!      error (without rollback), it would be a degenerate, rarely-useful
//!      mode — neither all-or-nothing nor best-effort.
//!   3. Best-effort batches are the standard ecosystem reading of
//!      "non-atomic" (cf. `Multicall3.tryAggregate(false)`, SQL
//!      `COMMIT` with `ON ERROR CONTINUE` UDFs, etc.). Solidity's
//!      transaction-level all-or-nothing is preserved by `atomic: true`.
//!
//! Three proptests:
//!   - `state_batch_atomic_rollback_on_error`            — atomic=true must
//!     restore pre-batch state EXACTLY when any change errors.
//!   - `state_batch_non_atomic_continues_past_invalid`   — atomic=false must
//!     apply EVERY well-formed change (including those after the invalid
//!     one) and silently skip ONLY the malformed change.
//!   - `state_batch_non_atomic_returns_ok_and_skips_only_invalid` — atomic=false
//!     must return Ok(()) and the post-state must differ from pre-state by
//!     exactly the well-formed changes (no spurious mutations from the
//!     malformed change).

#![allow(clippy::uninlined_format_args)]

use neo_devpack_solidity::runtime::state::{StateBatch, StateManager};
use neo_devpack_solidity::runtime::types::{StateChange, StateChangeType};
use neo_devpack_solidity::runtime::RuntimeConfig;
use proptest::prelude::*;

// ---------- helpers ----------

/// Build a deterministic 0x-prefixed 20-byte address from an index.
fn addr(seed: u8) -> String {
    // 40 hex chars = 20 bytes. Vary one byte by `seed` so distinct seeds give
    // distinct accounts.
    let mut hex = String::from("0x");
    for i in 0..20u8 {
        hex.push_str(&format!("{:02x}", i.wrapping_add(seed)));
    }
    hex
}

/// A valid balance-change record (well-formed 8-byte little-endian u64).
fn valid_balance_change(account: String, new_balance: u64) -> StateChange {
    StateChange {
        change_type: StateChangeType::BalanceChange,
        account,
        key: None,
        old_value: None,
        new_value: new_balance.to_le_bytes().to_vec(),
    }
}

/// An invalid balance-change record: `new_value` is the wrong number of bytes,
/// so `apply_change` will fail in `u64::from_le_bytes(... try_into ...)`.
fn invalid_balance_change(account: String) -> StateChange {
    StateChange {
        change_type: StateChangeType::BalanceChange,
        account,
        key: None,
        old_value: None,
        // 7 bytes, not 8 — will fail try_into::<[u8; 8]>().
        new_value: vec![1, 2, 3, 4, 5, 6, 7],
    }
}

/// Snapshot the relevant observable fields of a StateManager into a comparable
/// map: address -> (balance, nonce). We intentionally limit to the fields
/// affected by our generated changes so the diff is precise.
fn observe(state: &StateManager, addresses: &[String]) -> Vec<(String, u64, u64)> {
    addresses
        .iter()
        .map(|a| {
            let bal = state.get_balance(a).unwrap_or(0);
            let nonce = state.get_nonce(a).unwrap_or(0);
            (a.clone(), bal, nonce)
        })
        .collect()
}

/// Strategy for a single valid balance change. Reuses one of N seed addresses
/// so multiple changes can target overlapping accounts.
fn valid_change_strategy() -> impl Strategy<Value = StateChange> {
    (0u8..8u8, 0u64..1_000_000u64).prop_map(|(seed, bal)| valid_balance_change(addr(seed), bal))
}

// ---------- proptest 1: atomic rollback ----------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Atomic batches must roll back ALL changes when any single change fails.
    #[test]
    fn state_batch_atomic_rollback_on_error(
        valid in proptest::collection::vec(valid_change_strategy(), 1..=10),
        insert_at in 0usize..=10usize,
        bad_seed in 0u8..8u8,
    ) {
        let cfg = RuntimeConfig::default();
        let mut state = StateManager::new(&cfg).expect("StateManager::new");

        // Pre-seed every potentially-touched account with a deterministic
        // baseline so rollback has a non-trivial state to restore.
        for s in 0u8..8u8 {
            state.create_account(&addr(s), 100).expect("seed create_account");
        }

        // The set of addresses we'll observe before & after the batch.
        let touched: Vec<String> = (0u8..8u8).map(addr).collect();
        let pre = observe(&state, &touched);

        // Build the batch: insert the invalid change at `insert_at` (clamped).
        let mut changes = valid.clone();
        let pos = insert_at.min(changes.len());
        changes.insert(pos, invalid_balance_change(addr(bad_seed)));
        let batch = StateBatch { changes, atomic: true };

        let result = state.execute_batch(batch);
        prop_assert!(result.is_err(), "atomic batch with an invalid change must Err");

        let post = observe(&state, &touched);
        prop_assert_eq!(
            &pre,
            &post,
            "atomic rollback must restore pre-batch observable state EXACTLY"
        );
    }
}

// ---------- proptest 2 & 3: non-atomic best-effort apply ----------
//
// Both proptests below lock in the canonical contract: a non-atomic batch
// applies every well-formed change exactly once (regardless of position
// relative to malformed changes) and silently skips only the malformed
// change(s). They approach the invariant from two angles to catch different
// classes of regression:
//
//   * `state_batch_non_atomic_continues_past_invalid` — direct equality
//     against an "expected" StateManager built by replaying just the valid
//     changes via the public API. Catches both missing applies and ordering
//     bugs.
//
//   * `state_batch_non_atomic_returns_ok_and_skips_only_invalid` — diff-based
//     check: post-state must differ from pre-state ONLY at addresses
//     touched by well-formed changes, and `execute_batch` must return Ok(()).
//     Catches spurious side effects from the malformed change (e.g. a
//     half-applied write that happens to coincide with a later valid
//     write's address).

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Non-atomic batches as currently implemented: every WELL-FORMED change
    /// in the batch is applied, regardless of position relative to a malformed
    /// change. The malformed change itself is silently skipped and Ok(()) is
    /// returned.
    #[test]
    fn state_batch_non_atomic_continues_past_invalid(
        valid in proptest::collection::vec(valid_change_strategy(), 1..=10),
        insert_at in 0usize..=10usize,
        bad_seed in 0u8..8u8,
    ) {
        let cfg = RuntimeConfig::default();
        let mut state = StateManager::new(&cfg).expect("StateManager::new");

        for s in 0u8..8u8 {
            state.create_account(&addr(s), 100).expect("seed create_account");
        }
        let touched: Vec<String> = (0u8..8u8).map(addr).collect();

        let mut changes = valid.clone();
        let pos = insert_at.min(changes.len());
        changes.insert(pos, invalid_balance_change(addr(bad_seed)));
        let batch = StateBatch { changes: changes.clone(), atomic: false };

        // Build the "expected" state by applying ALL valid changes (i.e.
        // everything except the invalid one), in order, via the public API.
        let mut expected = StateManager::new(&cfg).expect("expected");
        for s in 0u8..8u8 {
            expected.create_account(&addr(s), 100).expect("seed expected");
        }
        for (idx, c) in changes.iter().enumerate() {
            if idx == pos { continue; } // skip the invalid one
            if let StateChangeType::BalanceChange = c.change_type {
                let bytes: [u8; 8] = c.new_value.as_slice().try_into()
                    .expect("valid bytes by construction");
                expected.set_balance(&c.account, u64::from_le_bytes(bytes))
                    .expect("set_balance");
            }
        }
        let expected_post = observe(&expected, &touched);

        let result = state.execute_batch(batch);
        prop_assert!(result.is_ok(), "non-atomic batch must not return Err");

        let post = observe(&state, &touched);
        prop_assert_eq!(
            &expected_post,
            &post,
            "non-atomic batch must apply every well-formed change exactly once"
        );
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Non-atomic batch contract, diff-based view: the malformed change must
    /// produce no observable side effect, and `execute_batch` must return
    /// `Ok(())`. Concretely: every address whose post-state differs from its
    /// pre-state must correspond to at least one well-formed change in the
    /// batch targeting that address; and the malformed change's address, if
    /// not also targeted by a valid change, must be unchanged from the
    /// (well-formed) pre-batch baseline applied via valid changes.
    #[test]
    fn state_batch_non_atomic_returns_ok_and_skips_only_invalid(
        valid in proptest::collection::vec(valid_change_strategy(), 1..=10),
        insert_at in 0usize..=10usize,
        bad_seed in 0u8..8u8,
    ) {
        let cfg = RuntimeConfig::default();
        let mut state = StateManager::new(&cfg).expect("StateManager::new");
        for s in 0u8..8u8 {
            state.create_account(&addr(s), 100).expect("seed create_account");
        }
        let touched: Vec<String> = (0u8..8u8).map(addr).collect();

        let mut changes = valid.clone();
        let pos = insert_at.min(changes.len());
        let bad_addr = addr(bad_seed);
        changes.insert(pos, invalid_balance_change(bad_addr.clone()));
        let batch = StateBatch { changes: changes.clone(), atomic: false };

        // Compute the set of addresses targeted by VALID changes only.
        let valid_targets: std::collections::HashSet<&String> = valid
            .iter()
            .map(|c| &c.account)
            .collect();

        let result = state.execute_batch(batch);
        prop_assert!(result.is_ok(), "non-atomic batch must return Ok(())");

        let post = observe(&state, &touched);

        // For the malformed change's address: if no valid change targets it,
        // its balance must remain at the seeded baseline (100). The malformed
        // change must not have leaked any partial write.
        if !valid_targets.contains(&bad_addr) {
            let bad_post = post.iter().find(|(a, _, _)| a == &bad_addr).unwrap();
            prop_assert_eq!(
                bad_post.1, 100u64,
                "malformed change must produce no side effect at addr {}", bad_addr
            );
        }

        // Cross-check: the final balance for each valid target must match the
        // LAST valid change targeting that account (last-write-wins, which is
        // how `set_balance` semantics compose).
        for target in &valid_targets {
            let last_for_target = valid.iter().rev().find(|c| &c.account == *target);
            if let Some(last) = last_for_target {
                let bytes: [u8; 8] = last.new_value.as_slice().try_into()
                    .expect("valid bytes by construction");
                let expected_bal = u64::from_le_bytes(bytes);
                let actual = post.iter().find(|(a, _, _)| a == *target).unwrap();
                prop_assert_eq!(
                    actual.1, expected_bal,
                    "addr {} must reflect last valid write (last-write-wins)",
                    target
                );
            }
        }
    }
}
