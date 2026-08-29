# Phase 3 — Error Structure: Delete Dead Enum + Fix Lossy Bridge

**Status:** Approved 2026-06-23
**Scope:** Delete the vestigial `error::CompilerError` (519 LOC dead code),
fix the one structural bridge that flattens meaningful errors to
`GENERIC_ERROR`, and replace the fragile string-matching code inference
with structured codes set at construction time.
**Out of scope:** Merging the 3 live error enums (they're layered by module
ownership — healthy). Adding source location to `solidity::Diagnostic`
(that's a cross-cutting structural change better suited to Phase 4).

## Context

The original Phase 3 framing was "consolidate 4 parallel error enums."
Detailed exploration revealed this is over-engineering: the three live
enums (`FrontendError → SolidityError → CompileError`) are layered by
module ownership (frontend → analysis → CLI), which is a healthy pattern.
The fourth enum (`error::CompilerError`) is entirely dead — 519 LOC of
rich infrastructure (typed `ErrorCode` enum with 40 variants,
`SourceLocation`, `DiagnosticBuilder`) that was never wired into the
pipeline.

The real structural problem is at `compile.rs:76`: every `SolidityError`
variant except `ParseDiagnostics` is flattened to
`CompileError::Message(string)` and renders as `GENERIC_ERROR`. The
downstream renderer (`standard_json_diagnostics.rs:31-84`) then
reconstructs error codes by substring-matching on the message text
(~25 fragile branches like `msg.contains("full wildcard manifest
permissions")` → `"MANIFEST_FULL_WILDCARD"`).

## Verification Summary

| Item | Location | Status |
| --- | --- | --- |
| `error::CompilerError` + all supporting types | `src/error.rs` (519 LOC) | **Zero external references** — dead code |
| `pub mod error;` + `pub use error::*;` in lib.rs | `src/lib.rs` | Exports dead code |
| Lossy bridge `other => CompileError::Message(other.to_string())` | `cli_compile/compile.rs:76` | The one structural wound |
| String-matching code inference | `standard_json_diagnostics.rs:31-84` | ~25 fragile branches |

## Deliverables

### PR1 — Delete `src/error.rs` + clean lib.rs

Delete the entire file (519 LOC). In `src/lib.rs`, remove:
- `pub mod error;`
- `pub use error::*;`

**Risk check:** The `pub use error::*` glob re-export means anything in
error.rs is available at the crate root. The verification confirmed zero
references across `src/` and `tests/`. If external consumers reference
`neo_devpack_solidity::CompilerError`, they'd be referencing undocumented
API — acceptable breakage.

**After deletion:** `From<io::Error>` for `CompilerError` is also removed.
Verify no code relies on it (the verification found none).

### PR2 — Fix the lossy bridge at `compile.rs:76`

The current match at `src/cli/cli_parts/cli_compile/compile.rs:69-77`:

```rust
match result {
    Ok(metadata) => { /* ... */ }
    Err(SolidityError::Frontend(FrontendError::ParseDiagnostics(diags))) => {
        return Err(CompileError::ParseErrors(diags));  // structured ✓
    }
    Err(other) => {
        return Err(CompileError::Message(other.to_string()));  // lossy ✗
    }
}
```

The catch-all flattens 7+ meaningful variants into a string. Fix by
expanding the match to handle each variant explicitly:

```rust
match result {
    Ok(metadata) => { /* ... */ }
    Err(SolidityError::Frontend(FrontendError::ParseDiagnostics(diags))) => {
        return Err(CompileError::ParseErrors(diags));
    }
    Err(SolidityError::Frontend(FrontendError::UnsupportedVersion(v))) => {
        return Err(CompileError::Message(format!("Unsupported Solidity version: {v}")));
        // Note: code is set to PARSE_ERROR by the renderer, which is correct
    }
    Err(SolidityError::Frontend(FrontendError::ImportError { path, reason })) => {
        return Err(CompileError::Message(format!(
            "Failed to resolve import '{path}': {reason}"
        )));
    }
    Err(SolidityError::Frontend(FrontendError::ContractNotFound(name))) => {
        return Err(CompileError::Message(format!(
            "Contract '{name}' not found in source"
        )));
    }
    Err(SolidityError::Frontend(FrontendError::UnsupportedConstruct(kind))) => {
        return Err(CompileError::Message(format!(
            "Unsupported top-level construct: {kind}"
        )));
    }
    Err(SolidityError::Frontend(FrontendError::Parse(msg))) => {
        return Err(CompileError::Message(msg));
    }
    Err(SolidityError::NoContracts) => {
        return Err(CompileError::Message("No contract definitions found in source".into()));
    }
    Err(SolidityError::ContractNotFound(name)) => {
        return Err(CompileError::Message(format!("Contract '{name}' not found")));
    }
    Err(SolidityError::UnsupportedFeature(msg)) => {
        return Err(CompileError::Message(format!("Unsupported feature: {msg}")));
    }
    Err(SolidityError::InheritanceError(msg)) => {
        return Err(CompileError::Message(format!("Inheritance error: {msg}")));
    }
    Err(SolidityError::Analysis(msg)) => {
        return Err(CompileError::Message(format!("Analysis error: {msg}")));
    }
}
```

**Why not add new `CompileError` variants?** That would be the "ideal"
fix, but it would require updating every renderer (`into_errors`,
`emit_error`, `standard_json_diagnostics`) with new branches for each
variant — significant scope. The pragmatic fix is to keep `CompileError`
as-is but make the mapping explicit and documented. The user-facing
output is the same (the message is still formatted), but future
maintainers can see exactly what information is available at each
error site and upgrade specific cases to structured variants when
needed.

**Alternative (out of scope):** Add a `CompileError::Solidity(SolidityError)`
wrapping variant that carries the full structure. This is cleaner but
changes `CompileError`'s public shape. Defer to Phase 5 (API surface).

### PR3 — Replace string-matching code inference with structured codes

The `infer_validation_code` function at
`src/cli/cli_parts/cli_standard_json/standard_json_diagnostics.rs:31-84`
reconstructs error codes by substring-matching on diagnostic messages:

```rust
fn infer_validation_code(msg: &str) -> &'static str {
    let lower = msg.to_lowercase();
    if lower.contains("full wildcard manifest permissions") {
        "MANIFEST_FULL_WILDCARD"
    } else if lower.contains("wildcard contract") {
        "MANIFEST_WILDCARD_CONTRACT"
    } else if lower.contains("wildcard method") {
        "MANIFEST_WILDCARD_METHODS"
    }
    // ... ~22 more branches ...
    else {
        "VALIDATION_ERROR"
    }
}
```

This is fragile (rename a message → silently breaks classification).
Replace by setting the code at construction time in the diagnostic
producer, then reading it directly in the renderer.

**Approach:**
1. In each diagnostic producer site (e.g., `erc_nep_patterns.rs:80`
   constructs a diagnostic with `.with_code("W101")`), the code is
   already set for warnings. Extend the pattern to error-level
   diagnostics.
2. In the renderer, replace `infer_validation_code(msg)` with
   `diag.code.as_deref().unwrap_or("VALIDATION_ERROR")`.
3. Delete `infer_validation_code` and its ~25 branches.

**Scope guard:** This PR only changes HOW codes reach the renderer (set
at source vs inferred from text), not WHAT the codes are. Every code
string stays the same. The user-visible output is identical.

## Sequencing

```
PR1 (delete error.rs + clean lib.rs) — independent, ~1 hour
PR2 (fix lossy bridge) — independent, ~1 hour
PR3 (replace string-matching codes) — independent, ~2-3 hours
```

All three are independent and can land in any order.

## Per-PR Verification Gate

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

Additionally, for PR3: verify that the set of error codes emitted in
standard-JSON output is identical before and after (no classification
changes). A simple way: compile a few example contracts that trigger
different validation errors and diff the JSON output.

## Risks

1. **`pub use error::*` consumers in external crates.** *Mitigation:*
   0.x release, undocumented API, zero internal refs. Acceptable.
2. **PR2 expands the match but output is unchanged.** *Mitigation:* the
   messages are the same strings `to_string()` produced. Test output
   is identical.
3. **PR3 misses a code that was only reachable via string-matching.**
   *Mitigation:* the code strings are the same literal values. Any code
   that was being inferred will now be set explicitly at the producer.
   If a producer is missed, the fallback is still `"VALIDATION_ERROR"`
   (same as before).

## Success Criteria

Phase 3 is done when:

1. `src/error.rs` deleted (519 LOC removed).
2. The `compile.rs` bridge handles every `SolidityError` variant
   explicitly (no catch-all `to_string()`).
3. `infer_validation_code` deleted; codes set at construction time.
4. `cargo fmt --check && cargo clippy -D warnings && cargo test` green.
5. Standard-JSON error output is byte-identical for the same inputs
   (no classification changes).
