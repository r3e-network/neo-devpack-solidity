# Library External Support Implementation Plan

**Goal:** Allow user-defined Solidity library functions declared `external` to compile on Neo by lowering them as inlined internal helpers.

**Architecture:** The compiler already merges non-builtin libraries into consuming contracts and normalizes merged library functions to internal visibility during analysis. The remaining blocker is an early validation error in the library validator. The change should replace that hard error with a warning and prove the merged lowering path works through focused library tests and one end-to-end showcase expectation.

**Tech Stack:** Rust compiler/frontend/IR pipeline, Rust unit/e2e tests, markdown support-matrix docs.

## Task 1: Capture the failing behavior

**Files:**
- Modify: `src/cli/tests/libraries.rs`
- Modify: `tests/e2e_compilation_tests.rs`

**Step 1: Write the failing test**

Add a focused unit test that compiles a user-defined library with an `external` function and a contract that calls it. Update the e2e expectation for `examples/new/LibraryExternalError.sol` from compile-failure to compile-success.

**Step 2: Run test to verify it fails**

Run: `cargo test libraries:: -- --nocapture`
Expected: FAIL because validation still reports `external library functions are not supported`.

**Step 3: Write minimal implementation**

Update library validation so `external` library functions no longer hard-fail. Emit a warning describing the Neo lowering model instead.

**Step 4: Run test to verify it passes**

Run: `cargo test libraries:: -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/cli/tests/libraries.rs tests/e2e_compilation_tests.rs src/solidity/validate/contract/library.rs
git commit -m "feat: support external library helpers on Neo"
```

## Task 2: Align docs with actual behavior

**Files:**
- Modify: `README.md`
- Modify: `FEATURE_MATRIX.md`
- Modify: `docs/SOLIDITY_SUPPORT_MATRIX.md`
- Modify: `docs/solidity/feature-support.md`

**Step 1: Write the failing documentation expectation**

Document that user-defined library functions are merged/inlined on Neo, and that `external` library visibility is accepted with normalization/warning rather than rejected.

**Step 2: Run targeted verification**

Run: `rg -n "external library functions are not supported|user-defined libraries partially supported|Libraries.*External" README.md FEATURE_MATRIX.md docs/SOLIDITY_SUPPORT_MATRIX.md docs/solidity/feature-support.md`
Expected: outdated wording found before edits

**Step 3: Write minimal documentation updates**

Update the support matrix rows and notes to reflect the new behavior precisely without claiming deployable external libraries.

**Step 4: Re-run verification**

Run: `rg -n "external library functions are not supported|user-defined libraries partially supported" README.md FEATURE_MATRIX.md docs/SOLIDITY_SUPPORT_MATRIX.md docs/solidity/feature-support.md`
Expected: outdated wording removed or narrowed

**Step 5: Commit**

```bash
git add README.md FEATURE_MATRIX.md docs/SOLIDITY_SUPPORT_MATRIX.md docs/solidity/feature-support.md
git commit -m "docs: update library support notes"
```

## Task 3: Re-verify affected compiler surfaces

**Files:**
- Test: `src/cli/tests/libraries.rs`
- Test: `tests/e2e_compilation_tests.rs`

**Step 1: Run focused Rust tests**

Run: `cargo test libraries:: -- --nocapture`
Expected: PASS

**Step 2: Run end-to-end library showcase tests**

Run: `cargo test test_library_ --test e2e_compilation_tests -- --nocapture`
Expected: PASS

**Step 3: Run broader frontend/compiler confidence check**

Run: `cargo test frontend::tests -- --nocapture`
Expected: PASS

**Step 4: Summarize residual risks**

Note that this change only covers inlined user-defined libraries; deployable library bytecode and true external linking semantics remain out of scope for Neo.

**Step 5: Commit**

```bash
git add .
git commit -m "test: verify external library lowering support"
```
