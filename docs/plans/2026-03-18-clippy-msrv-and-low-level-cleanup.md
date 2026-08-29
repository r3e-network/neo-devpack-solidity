# Clippy MSRV And Low-Level Cleanup Implementation Plan

**Goal:** Make the repository pass `cargo clippy --all-targets --all-features -- -D warnings` without changing runtime behavior, while preserving the declared Rust `1.88` MSRV.

**Architecture:** Keep the change set narrow and behavior-preserving. Fix the misleading low-level call control flow in-place, replace the eager-lint issue in try/catch lowering with the direct form Clippy expects, and keep odd-length validation checks inside the current Rust `1.88` contract already covered by tests.

**Tech Stack:** Rust, Cargo, Clippy, existing unit/integration test suites

## Task 1: Capture The Failing Verification Surface

**Files:**
- Modify: `none`
- Test: `cargo clippy --all-targets --all-features -- -D warnings`

**Step 1: Run the failing gate**

Run: `cargo clippy --all-targets --all-features -- -D warnings`
Expected: FAIL with `possible_missing_else`, `unnecessary_lazy_evaluations`, and `incompatible_msrv`.

**Step 2: Confirm characterization coverage**

Run: `cargo test --workspace`
Expected: PASS, proving existing behavior is already covered before refactor.

## Task 2: Refactor The Flagged Rust Paths

**Files:**
- Modify: `src/ir/expressions/calls/low_level.rs`
- Modify: `src/ir/statements/dispatch/try_catch.rs`
- Modify: `src/runtime/execution/helpers/interop.rs`
- Modify: `src/runtime/types/types_wrappers.rs`

**Step 1: Clean the low-level call control flow**

Rewrite the `delegatecall` rejection + safe-call block so it is clearly structured and lint-clean, with no hidden same-line `if`.

**Step 2: Replace the unnecessary lazy fallback**

Change `unwrap_or_else(|| (0, ValueType::Any))` to `unwrap_or((0, ValueType::Any))`.

**Step 3: Restore MSRV compatibility**

Keep the runtime hex normalization helpers lint-clean and compatible with the declared Rust `1.88` toolchain floor.

## Task 3: Verify Behavior And Lint Cleanliness

**Files:**
- Test: `tests/runtime_account_tests.rs`
- Test: `src/runtime/types/tests.rs`
- Test: `src/cli/tests/semantics/state_mutability.rs`

**Step 1: Run targeted behavioral tests**

Run: `cargo test runtime_account_tests state_mutability test_hash_types_reject_odd_length_hex --workspace`
Expected: PASS.

**Step 2: Re-run the lint gate**

Run: `cargo clippy --all-targets --all-features -- -D warnings`
Expected: PASS.

**Step 3: Re-run the broader regression suite**

Run: `cargo test --workspace`
Expected: PASS.
