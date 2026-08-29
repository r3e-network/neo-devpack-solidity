# Phase 2 Implementation Plan — Dead Code Removal

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement task-by-task.

**Goal:** Delete 7 verified-dead scaffolding modules (~640 LOC) + refactor `semantic_model.rs` to drop its unused return type.

**Architecture:** PR1 deletes 6 modules + `types.rs` (all zero-ref) and cleans `lib.rs`. PR2 refactors `semantic_model.rs`. Pure deletion — no behavior change.

**Spec:** `docs/superpowers/specs/2026-06-23-phase2-dead-code-removal-design.md`

---

## Task 1: Set up worktree

- [ ] **Step 1:** `git worktree add .worktrees/phase2 -b phase2/dead-code-removal main`
- [ ] **Step 2:** `cd .worktrees/phase2 && cargo build --workspace --all-features 2>&1 | tail -3`
- [ ] **Step 3:** Verify clean baseline: `cargo test --workspace --all-features 2>&1 | grep FAILED | head -5` (expect no output)

## Task 2: Delete 6 pure-dead modules + types.rs + clean lib.rs (PR1+PR2 combined)

**Files to delete:**
- `src/security.rs` (92 LOC)
- `src/docs.rs` (43 LOC)
- `src/testing.rs` (60 LOC)
- `src/codegen_helpers.rs` (38 LOC)
- `src/validation.rs` (67 LOC)
- `src/warning.rs` (120 LOC)
- `src/types.rs` (220 LOC)

**File to modify:**
- `src/lib.rs` — remove 7 `pub mod` declarations + `pub use types::*;`

- [ ] **Step 1:** `git rm src/security.rs src/docs.rs src/testing.rs src/codegen_helpers.rs src/validation.rs src/warning.rs src/types.rs`
- [ ] **Step 2:** Edit `src/lib.rs` — remove these lines:
  - `pub mod codegen_helpers;`
  - `pub mod docs;`
  - `pub mod security;`
  - `pub mod testing;`
  - `pub mod types;`
  - `pub mod validation;`
  - `pub mod warning;`
  - `pub use types::*;`
  - Keep: `pub mod error;` and `pub use error::*;` (error module is live)
- [ ] **Step 3:** Build: `cargo build --workspace --all-features 2>&1 | tail -10`
  - Expected: clean build. If it fails, a reference was missed — read the error and restore the needed item.
- [ ] **Step 4:** Full gate: `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings 2>&1 | tail -5 && cargo test --workspace --all-features 2>&1 | tail -10`
  - Expected: all green.
- [ ] **Step 5:** Commit:
```bash
git add -A
git commit -m "chore: delete 7 dead scaffolding modules (~640 LOC)

Verified zero external references for every exported item:
- security.rs (SecurityChecker — no detectors wired)
- docs.rs (DocItem stubs)
- testing.rs (TestSuite — conformance has its own TestResult)
- codegen_helpers.rs (encode_small_int/varint — inline elsewhere)
- validation.rs (InputValidator — cli_run does its own checks)
- warning.rs (WarningCollector — real warnings use Diagnostic)
- types.rs (CompilerConfig/Builder/OutputFormat/GasModel/ConfigError
  — CLI uses CompileOptions instead)

lib.rs: removed 7 pub mod declarations + pub use types::*"
```

## Task 3: Refactor semantic_model.rs (PR3)

**File:** `src/semantic_model.rs` (320 LOC → ~50-80 LOC)
**File:** `src/cli/cli_parts/cli_compile/compile.rs:135` (update call site)

The `build_semantic_model` function returns `Result<SemanticModel, Vec<Diagnostic>>` but the caller only uses the `Err` path. Change the return type to `Result<(), Vec<Diagnostic>>` and delete all the unused `SemanticModel` scaffolding (`FunctionSymbol`, `StateVariableSymbol`, etc.).

- [ ] **Step 1:** Read `src/semantic_model.rs` to understand what validation logic it performs (the part that produces diagnostics).
- [ ] **Step 2:** Read `src/cli/cli_parts/cli_compile/compile.rs:130-140` to see the exact call site.
- [ ] **Step 3:** Refactor `build_semantic_model`:
  - Change return type from `Result<SemanticModel, Vec<Diagnostic>>` to `Result<(), Vec<Diagnostic>>`
  - Replace `Ok(model)` with `Ok(())`
  - Delete the `SemanticModel` struct and all its fields/methods
  - Delete `FunctionSymbol`, `StateVariableSymbol`, and any other types only used by `SemanticModel`
  - Keep all the diagnostic-producing validation logic intact
- [ ] **Step 4:** The call site at `compile.rs:135` already uses `if let Err(diags) = build_semantic_model(...)` — no change needed there.
- [ ] **Step 5:** Build: `cargo build --workspace --all-features 2>&1 | tail -10`
- [ ] **Step 6:** Full gate: `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings 2>&1 | tail -5 && cargo test --workspace --all-features 2>&1 | tail -10`
- [ ] **Step 7:** Commit:
```bash
git add -A
git commit -m "refactor(semantic_model): drop unused SemanticModel return type

build_semantic_model was called at compile.rs:135 but only for its
Err path — the Ok(SemanticModel) was constructed and immediately
dropped. Changes the return type to Result<(), Vec<Diagnostic>> and
deletes the SemanticModel struct + supporting types (FunctionSymbol,
StateVariableSymbol, etc.). The validation logic that produces
diagnostics is unchanged."
```

## Task 4: Closeout

- [ ] **Step 1:** `git diff main --stat` — verify net deletion is ~900+ LOC
- [ ] **Step 2:** Final gate: `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --workspace --all-features`
- [ ] **Step 3:** Merge to main: `git checkout main && git merge --no-ff phase2/dead-code-removal -m "Phase 2: dead code & scaffolding removal"`
- [ ] **Step 4:** Clean up: `git worktree remove .worktrees/phase2 && git branch -d phase2/dead-code-removal`
