# Phase 2 — Dead Code & Scaffolding Removal

**Status:** Approved 2026-06-23
**Scope:** Delete 7 verified-dead scaffolding modules (~640 LOC) + refactor
`semantic_model.rs` (called for side effects only). Second phase of the
7-phase risk-ascending refactor.
**Out of scope:** Error/diagnostic consolidation (Phase 3), god-object
splits (Phase 4), public API curation beyond removing dead modules
(Phase 5).

## Context

Phase 1's pre-plan validation taught us to verify before trusting the
initial exploration. This phase's verification was rigorous: for each
module, `rg -n "<every exported item>" --type rust src/ tests/` was run
excluding the module itself. **All 7 modules have zero references outside
their own file.** The conformance suite's `TestResult` in
`tests/conformance/infrastructure.rs` is a separate struct, not an import
from `src/testing.rs`.

## Verification Summary

| Module | LOC | Exported items | External refs | Verdict |
| --- | --- | --- | --- | --- |
| `security.rs` | 92 | `SecurityChecker`, `SecurityIssue`, `SecuritySeverity` | **0** | DELETE |
| `docs.rs` | 43 | `DocItem`, `DocKind`, `DocParam` | **0** | DELETE |
| `testing.rs` | 60 | `TestSuite`, `TestResult` | **0** (conformance has its own) | DELETE |
| `codegen_helpers.rs` | 38 | `encode_small_int`, `encode_varint` | **0** | DELETE |
| `validation.rs` | 67 | `InputValidator` | **0** | DELETE |
| `warning.rs` | 120 | `WarningCollector`, `WarningConfig` | **0** (real warnings use `Diagnostic`) | DELETE |
| `types.rs` | 220 | `CompilerConfig`, `CompilerConfigBuilder`, `OutputFormat`, `GasModel`, `ConfigError` | **0** (all 5 items dead) | DELETE |
| `semantic_model.rs` | 320 | `build_semantic_model`, `SemanticModel`, symbols | called at `compile.rs:135` but return value dropped | REFACTOR |

**Total pure deletion:** ~640 LOC across 7 modules.
**Refactor:** ~320 LOC module simplified to drop the unused return type.

## Deliverables

### PR1 — Delete 6 pure-dead modules + clean lib.rs

Delete entirely:
- `src/security.rs`
- `src/docs.rs`
- `src/testing.rs`
- `src/codegen_helpers.rs`
- `src/validation.rs`
- `src/warning.rs`

In `src/lib.rs`, remove the corresponding `pub mod` declarations:
```rust
pub mod codegen_helpers;  // remove
pub mod docs;              // remove
pub mod security;          // remove
pub mod testing;           // remove
pub mod validation;        // remove
pub mod warning;           // remove
```

Keep the remaining `pub mod` declarations unchanged.

### PR2 — Delete `types.rs` (all items dead)

`src/types.rs` (220 LOC) defines `CompilerConfig`, `CompilerConfigBuilder`,
`OutputFormat`, `GasModel`, `ConfigError`. All 5 items have zero external
references. The live compiler uses `CompileOptions`
(`cli/cli_parts/cli_compile/types.rs`) instead.

In `src/lib.rs`:
- Remove `pub mod types;`
- Remove `pub use types::*;`

**Risk check:** the `pub use types::*` glob re-export means anything in
types.rs is available at the crate root. After deletion, any downstream
code referencing `neo_devpack_solidity::CompilerConfig` etc. would break.
Since the items have zero references even within the workspace, this is
safe. If there are external consumers (not in this repo), they'd be
referencing undocumented API — acceptable breakage for a 0.x release.

### PR3 — Simplify `semantic_model.rs`

`build_semantic_model` at `src/semantic_model.rs` returns
`Result<SemanticModel, Vec<Diagnostic>>`. The single caller at
`src/cli/cli_parts/cli_compile/compile.rs:135` is:
```rust
if let Err(diags) = build_semantic_model(&metadata) {
    // emit diagnostics
}
```

The `Ok(SemanticModel)` is constructed and immediately dropped. Two
options:

**Option A (simpler):** Change the return type to
`Result<(), Vec<Diagnostic>>` and delete the `SemanticModel` struct + all
its supporting types (`FunctionSymbol`, `StateVariableSymbol`, etc.).
The function becomes a pure validator.

**Option B (preserves future use):** Keep the return type but add a
`// TODO: use the SemanticModel for cross-contract type checking` comment.

**Recommended: Option A.** YAGNI — the model has been unused since it was
written. If cross-contract type checking is needed later, it can be
rebuilt with the correct architecture. The current model is a 320-LOC
artifact that serves no purpose.

After the refactor, `semantic_model.rs` should be ~50-80 LOC (just the
validation logic that produces diagnostics).

## Sequencing

```
PR1 (delete 6 modules + lib.rs cleanup) — independent
PR2 (delete types.rs + lib.rs cleanup) — independent
PR3 (refactor semantic_model.rs) — independent
```

All three are independent. PR1 and PR2 both touch `lib.rs` but in
different lines (different `pub mod` declarations), so they can land in
any order. If landed sequentially in one branch, no conflicts.

## Per-PR Verification Gate

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

Every PR must pass all three. Since these are pure deletions of dead
code, tests should be unaffected (any test that referenced the deleted
items would be a reference the verification missed).

## Risks

1. **A reference was missed.** *Mitigation:* the verification searched
   `src/` and `tests/` for every exported item name. If `cargo build`
   fails after deletion, the reference exists but wasn't found — restore
   and investigate.
2. **External consumer breakage.** *Mitigation:* 0.x release, items were
   never documented as public API, and the README's API reference section
   doesn't mention them. Acceptable.
3. **`semantic_model.rs` refactor introduces a behavior change.**
   *Mitigation:* Option A preserves the exact same diagnostic output;
   only the unused return value construction is removed. The existing
   tests for the diagnostic behavior remain unchanged.

## Success Criteria

Phase 2 is done when:

1. 7 modules deleted (~640 LOC removed).
2. `semantic_model.rs` simplified (SemanticModel struct + supporting
   types removed, return type changed to `Result<(), Vec<Diagnostic>>`).
3. `lib.rs` cleaned (7 `pub mod` declarations removed, `pub use types::*`
   removed).
4. `cargo fmt --check && cargo clippy -D warnings && cargo test` all green.
5. No behavior change in compiler output (all tests pass identically).
