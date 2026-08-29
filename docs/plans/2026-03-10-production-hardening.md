# Production Hardening Implementation Plan

**Goal:** Restore the strict production quality gate, harden runtime hex/account normalization with regression coverage, and correct high-visibility stale project metadata so the repository's readiness claims match the current codebase.

**Architecture:** Keep the change set intentionally narrow. Fix the clippy failure in the runtime validation paths without introducing APIs newer than the repository's declared Rust toolchain floor, add focused tests around the affected edge cases, and update top-level docs to remove obvious drift discovered during review.

**Tech Stack:** Rust 2021, cargo/clippy/fmt, Rust unit and integration tests, Markdown project docs.

## Task 1: Capture and document the failing quality gate

**Files:**
- Modify: `docs/plans/2026-03-10-production-hardening.md`
- Verify: `Cargo.toml`

**Step 1: Reproduce the failing gate**

Run: `cargo clippy --all-targets --all-features -- -D warnings`
Expected: FAIL with `clippy::manual_is_multiple_of` in `src/runtime/execution/helpers/interop.rs` and `src/runtime/types/types_wrappers.rs`.

**Step 2: Confirm the compatibility constraint**

Run: `rg -n "1\\.88|rust-version|MSRV" README.md Cargo.toml docs/basics/installing-the-compiler.md`
Expected: documentation shows the Rust 1.88+ expectation and `Cargo.toml` pins `rust-version = "1.88"`.

**Step 3: Compare nearby working patterns**

Run: `rg -n "& 1|is_multiple_of|% 2 != 0" src/runtime src/runtime/types`
Expected: existing code already uses stable bitwise odd/even checks elsewhere, providing a clippy-safe and MSRV-safe pattern to follow.

## Task 2: Add focused regression coverage before changing implementation

**Files:**
- Modify: `src/runtime/types/tests.rs`
- Modify: `tests/runtime_account_tests.rs`
- Verify: `src/runtime/types/types_wrappers.rs`
- Verify: `src/runtime/execution/helpers/interop.rs`

**Step 1: Write the failing tests**

Add tests that assert:
- `TransactionHash::new("0xabc".to_string())` returns an error.
- `BlockHash::new("abc".to_string())` returns an error.
- `ExecutionContext::override_caller_account("0123456789abcdef0123456789abcdef01234567")` succeeds without a `0x` prefix.
- `ExecutionContext::override_caller_account("0x123456789abcdef0123456789abcdef01234567")` rejects odd-length hex specifically.

**Step 2: Run the new tests to watch red**

Run: `cargo test test_hash_types_reject_odd_length_hex normalize_account_rejects_odd_length_address normalize_account_accepts_unprefixed_address -- --nocapture`
Historical expected result: at least one test failed before implementation because the new coverage did not exist yet or because the assertion set was incomplete at the start of this plan.

**Step 3: Keep the tests minimal**

Do not broaden the tests into unrelated runtime behavior. The test surface should stay limited to prefix stripping and odd-length validation.

## Task 3: Refactor the runtime validation paths to satisfy clippy without changing the MSRV contract

**Files:**
- Modify: `src/runtime/types/types_wrappers.rs`
- Modify: `src/runtime/execution/helpers/interop.rs`
- Verify: `tests/runtime_account_tests.rs`
- Verify: `src/runtime/types/tests.rs`

**Step 1: Replace the lint-triggering odd-length checks**

Use a stable helper or bitwise even/odd check instead of `% 2 != 0`, and keep the implementation inside the current `rust-version` contract.

**Step 2: Keep behavior unchanged**

Preserve:
- acceptance of both prefixed and unprefixed hex where already supported,
- existing error strings,
- existing length constraints (`40` hex chars for runtime accounts).

**Step 3: Run the focused regression tests**

Run: `cargo test test_hash_types_reject_odd_length_hex normalize_account_rejects_odd_length_address normalize_account_accepts_unprefixed_address -- --nocapture`
Expected: PASS.

## Task 4: Correct high-visibility stale metadata/docs uncovered during review

**Files:**
- Modify: `README.md`
- Modify: `docs/ARCHITECTURE.md`

**Step 1: Fix stale readiness metadata**

Update the README status line so it stops claiming an exact stale test count (`666 Tests`). Replace it with less fragile wording derived from the current suite, such as “Actively Fuzzed” and source-linked test tiers rather than a fixed total.

**Step 2: Fix obvious architecture drift**

Update `docs/ARCHITECTURE.md` so the project structure no longer references the missing `src/runtime/helpers/` path and instead reflects the current `src/runtime/execution`, `src/runtime/bridge`, `src/runtime/state`, and `src/runtime/spec` layout at a high level.

**Step 3: Keep docs edits surgical**

Do not rewrite the entire architecture document. Only correct the clearly stale, high-visibility portions discovered during review.

## Task 5: Verify the hardening pass end-to-end

**Files:**
- Verify: `src/runtime/types/types_wrappers.rs`
- Verify: `src/runtime/execution/helpers/interop.rs`
- Verify: `src/runtime/types/tests.rs`
- Verify: `tests/runtime_account_tests.rs`
- Verify: `README.md`
- Verify: `docs/ARCHITECTURE.md`

**Step 1: Run targeted tests**

Run: `cargo test test_hash_types_reject_odd_length_hex normalize_account_rejects_odd_length_address normalize_account_accepts_unprefixed_address -- --nocapture`
Expected: PASS.

**Step 2: Run the strict quality gate that failed initially**

Run: `cargo clippy --all-targets --all-features -- -D warnings`
Expected: PASS.

**Step 3: Re-run the main Rust suite**

Run: `cargo test --workspace -q`
Expected: PASS.

**Step 4: Check formatting**

Run: `cargo fmt --all -- --check`
Expected: PASS.
