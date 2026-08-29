# Production Readiness Pass Implementation Plan

**Goal:** Align the verified quality gates and public production-readiness claims for the Neo DevPack for Solidity compiler, then close the concrete failures uncovered by baseline validation.

**Architecture:** Keep this pass narrow and evidence-driven. Fix the CI-breaking Rust/Clippy policy mismatch first, then update stale quality metrics/docs so the repository’s “production-ready” claims match what the verified gates actually prove.

**Tech Stack:** Rust, Cargo, Clippy, Markdown docs, GitHub Actions CI

## Task 1: Align Rust/Clippy Policy With Documented Support

**Files:**
- Modify: `Cargo.toml`
- Verify: `.github/workflows/ci.yml`

**Step 1: Reproduce the failing gate**

Run: `cargo clippy --all-targets --all-features -- -D warnings`
Expected: FAIL with `clippy::manual_is_multiple_of` on `src/runtime/execution/helpers/interop.rs` and `src/runtime/types/types_wrappers.rs`

**Step 2: Apply the minimal policy fix**

Update `Cargo.toml` so the package’s Rust-version/MSRV contract is explicit and Clippy evaluates lints against that contract instead of the host toolchain’s newest conveniences.

**Step 3: Re-run the failing gate**

Run: `cargo clippy --all-targets --all-features -- -D warnings`
Expected: PASS

## Task 2: Make Production Claims Match Verified Evidence

**Files:**
- Modify: `README.md`
- Modify: `docs/index.md`
- Modify: `docs/workflows/production.md`

**Step 1: Derive current evidence**

Run: `cargo test --workspace`
Run: `cargo test --workspace -- --list`
Run: `bash examples/test_strict_compatibility_sweep.sh`
Expected: tests pass, the source-controlled test inventory is reported by the commands above, and the strict sweep passes with zero failures and zero unexpected-warning contracts

**Step 2: Replace stale, fragile counts**

Update the public-facing docs to use current, resilient language such as “layered Rust, fuzz, E2E, conformance, and Neo-Express validation” instead of stale exact test totals, and ensure the production-gate description reflects the commands actually verified in this pass.

**Step 3: Sanity check docs consistency**

Run: `rg -n "666 Tests|660\\+ tests|620\\+ tests|700\\+ tests" README.md docs`
Expected: no remaining stale production-count claims in maintained docs

## Task 3: Re-Verify the Production Readiness Slice

**Files:**
- No source edits expected

**Step 1: Run formatting gate**

Run: `cargo fmt --all -- --check`
Expected: PASS

**Step 2: Run core compiler validation**

Run: `cargo test --workspace`
Run: `cargo build --release`
Expected: PASS

**Step 3: Run strict compatibility validation**

Run: `bash examples/test_strict_compatibility_sweep.sh`
Expected: PASS with `strict_sweep_failures=0` and `strict_sweep_unexpected_warning_contracts=0`

**Step 4: Summarize residual risk**

Document any gates not run in this session, especially Neo-Express deploy smokes and optional tooling/runtime suites, so completion claims stay evidence-based.
