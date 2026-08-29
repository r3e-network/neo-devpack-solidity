# Phase 7 — include! → mod Migration (Top-Level Boundaries)

**Status:** Approved 2026-06-23
**Scope:** Convert the top-level `include!` calls in each root module to
proper `mod` declarations. Nested `include!`s within sub-aggregators stay
as-is. Items crossing the new boundary get `pub(crate)` per the `runtime/`
module's proven convention.

## Context

The codebase has 283 `include!` calls across 53 aggregator files. A full
migration (Option C) would require ~1,100 visibility edits across 5 nesting
levels — high churn for zero behavior change. Option B converts only the
top-level boundary in each of 8 root modules, capturing most of the
rustdoc/IDE navigation benefit at a fraction of the risk.

The `runtime/` module already uses proper `mod` declarations with blanket
`pub(crate)` — this is our reference. The compiler guides every visibility
edit: convert one root, `cargo check`, fix the errors it reports.

## Deliverables

One PR per root, smallest first (practice the pattern, then scale):

### PR1 — Small roots: `type_system.rs`, `frontend.rs`

**`type_system.rs`** (2 includes → 2 mods):
```rust
mod types;
mod parse;
```

**`frontend.rs`** (6 includes → 6 mods):
```rust
mod frontend_parse;
mod frontend_convert;
mod frontend_ir;
mod frontend_errors;
mod frontend_diagnostics;
mod frontend_guarded_parse;
```

For each: the compiler reports which items need `pub(crate)`. Apply blanket
`pub(crate)` to everything the compiler flags.

### PR2 — `solidity.rs` root

Convert the top-level includes (solidity_errors, solidity_docs,
solidity_metadata, solidity_analyse, solidity_convert, solidity_validate,
upgrade) to `mod` declarations. The nested includes inside sub-aggregators
(convert/, validate/, analyse/) stay as-is.

### PR3 — `ir.rs` root

Convert the 6 top-level includes (ir_types, ir_build, ir_context,
ir_statements, ir_expressions, ir_deploy) to `mod` declarations.

### PR4 — `cli/mod.rs` + `cli/bytecode.rs` + `standard_json.rs`

Convert the top-level includes in each CLI sub-root to `mod` declarations.

## Method (per root)

1. Read the root file, find all `include!(...)` lines
2. Replace each `include!("file.rs");` with `mod file;` (Rust resolves
   `mod file;` to `file.rs` in the same directory)
3. If the included file has a different name than the module should have,
   use `#[path = "actual_name.rs"] mod desired_name;`
4. Run `cargo check` — the compiler lists every private item that's now
   cross-module
5. Add `pub(crate)` to every item the compiler reports
6. Run full gate (fmt + clippy + test)

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

## Success Criteria

1. All 8 root modules use `mod` declarations for their direct children
2. Nested `include!`s within sub-aggregators remain (acceptable intermediate)
3. Cross-boundary items are `pub(crate)` per the `runtime/` convention
4. All tests pass
5. `cargo doc` shows navigable module tree for top-level modules
