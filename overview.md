# v0.30.2 — Continued Code Quality Fixes

## What was done

Completed the remaining code quality fixes from the team capability improvement plan. All production `unwrap()` now have documented invariants, and CLI fatal-error patterns are centralized via a macro.

## Fixes Applied

### 1. Regex literal `unwrap()` → `expect()` (10 instances)

All 10 `Regex::new(...).unwrap()` calls in `solidity/upgrade.rs` converted to `.expect("valid regex pattern")` with a module-level comment explaining these are compile-time constant patterns.

### 2. Range-guarded `expect()` documentation (2 instances)

- `runtime/execution/syscalls/contract.rs`: Added `# Invariant` comment
- `neo/contract_hash.rs`: Added `# Invariant` comment

### 3. CLI fatal-error pattern centralization (12 instances)

Added `fatal_error!` macro in `cli_defs.rs` and replaced 12 `eprintln!` + `exit(1)` patterns across 3 files:
- `cli_run/single_file.rs` (7)
- `cli_run/standard_json.rs` (5)
- `cli_analyze.rs` (1)

## Remaining Production unwrap/expect (8 instances)

All are documented invariants:

| File | Count | Type |
|------|-------|------|
| `resolve.rs` | 1 | Hardcoded table lookup |
| `input.rs` | 1 | 32-byte read with memory_limit guarantee |
| `manifest/build.rs` | 1 | Non-empty iterator |
| `literals.rs` | 1 | Hardcoded exponent |
| `ctx_locals_scopes.rs` | 1 | `checked_add` with overflow comment |
| `power.rs` | 1 | Sign-tracking variable |
| `constant_folding.rs` | 2 | In-range arithmetic |

## Verification

- `cargo check`: 0 errors, 0 warnings
- `cargo clippy`: 0 warnings
- `cargo test`: **965 tests passed, 0 failed** — zero regressions
