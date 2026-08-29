# Phase 3 Implementation Plan — Error Structure

> **For agentic workers:** Use superpowers:subagent-driven-development to implement task-by-task.

**Goal:** Delete dead `error.rs` (519 LOC), fix the one lossy error bridge at `compile.rs:76`, replace string-matching code inference with structured codes.

**Spec:** `docs/superpowers/specs/2026-06-23-phase3-error-structure-design.md`

---

## Task 1: Set up worktree

- [ ] `git worktree add .worktrees/phase3 -b phase3/error-structure main`
- [ ] Build + test baseline in the worktree

## Task 2: Delete `src/error.rs` + clean lib.rs (PR1)

- [ ] `git rm src/error.rs`
- [ ] Edit `src/lib.rs`: remove `pub mod error;` and `pub use error::*;`
- [ ] `cargo build --workspace --all-features` — clean build expected
- [ ] Full gate (fmt + clippy + test)
- [ ] Commit: `chore: delete dead error.rs (519 LOC, zero external refs)`

## Task 3: Fix the lossy bridge at compile.rs (PR2)

- [ ] Read `src/cli/cli_parts/cli_compile/compile.rs:69-77` (the match arm)
- [ ] Read `src/solidity/solidity_errors.rs` (all SolidityError variants)
- [ ] Read `src/frontend/frontend_errors.rs` (all FrontendError variants)
- [ ] Expand the match to handle every variant explicitly (no catch-all to_string)
- [ ] Full gate
- [ ] Commit: `fix(compile): handle every SolidityError variant explicitly (no GENERIC_ERROR flattening)`

## Task 4: Replace string-matching code inference (PR3)

- [ ] Read `src/cli/cli_parts/cli_standard_json/standard_json_diagnostics.rs:31-84` (infer_validation_code)
- [ ] Find every diagnostic producer site that creates Error-severity diagnostics
- [ ] Set the code at each producer site using `.with_code(...)` (same code strings)
- [ ] Replace `infer_validation_code(msg)` with `diag.code.as_deref().unwrap_or("VALIDATION_ERROR")`
- [ ] Delete `infer_validation_code`
- [ ] Verify: compile example contracts, diff standard-JSON output before/after
- [ ] Full gate
- [ ] Commit: `refactor(diagnostics): replace string-matching code inference with structured codes`

## Task 5: Closeout

- [ ] Final gate on branch
- [ ] Merge to main: `git merge --no-ff phase3/error-structure`
- [ ] Clean up worktree + branch
