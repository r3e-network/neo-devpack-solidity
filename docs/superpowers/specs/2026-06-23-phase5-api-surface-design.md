# Phase 5 — Public API Surface Tightening

**Status:** Approved 2026-06-23
**Scope:** Tighten lib.rs exports — 4 modules to `pub(crate)`, 4 to
`#[doc(hidden)] pub`, runtime to `#[doc(hidden)] pub` as interim measure.
Feature-gating runtime deferred to follow-up.

## Context

The README documents exactly one Rust symbol: `cli::compile_contracts`.
The other 10 `pub mod` declarations are either internal (`solidity`,
`semantic_model`, `type_system`, `utils`), test-facing (`ir`,
`storage_key`, `frontend`, `interop`), tooling-facing (`neo`), or a
dev/test simulator (`runtime`). None is documented as public API.

The crate is not published to crates.io (distribution is the `neo-solc`
binary). The only external consumers are `tests/` and `fuzz/`, which are
outside the crate tree.

## Changes

### PR1 — Tighten lib.rs (single PR, all one-liners)

**Tier A — `pub(crate)` (0 external refs, pure internal):**
```rust
pub(crate) mod solidity;
pub(crate) mod semantic_model;
pub(crate) mod type_system;
pub(crate) mod utils;
```

**Tier B — `#[doc(hidden)] pub` (1 test ref each, non-breaking):**
```rust
#[doc(hidden)] pub mod ir;
#[doc(hidden)] pub mod storage_key;
#[doc(hidden)] pub mod frontend;
#[doc(hidden)] pub mod interop;
```

**Tier C — `#[doc(hidden)] pub` (interim for runtime):**
```rust
#[doc(hidden)] pub mod runtime;
```

**Keep `pub` (documented/tooling API):**
```rust
pub mod cli;   // README-documented
pub mod neo;   // 9 test files + 4 fuzz targets — de-facto NEF toolkit
```

### Deferred (not in Phase 5)

- **Runtime feature-gate** (`#[cfg(feature = "runtime")]`) — requires
  extracting `runtime::spec` and adding `--features runtime` to all test
  invocations. Invasive; defer to Phase 5b or a dedicated phase.
- **`neo` documentation** — keep `pub` but add rustdocs in a follow-up.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

The `pub(crate)` changes must not break tests (the 4 modules have 0
external refs). The `#[doc(hidden)]` changes are non-breaking (visibility
unchanged, just hidden from docs).

## Success Criteria

1. 4 modules are `pub(crate)`.
2. 5 modules are `#[doc(hidden)] pub`.
3. 2 modules (`cli`, `neo`) remain `pub`.
4. All tests pass.
5. `cargo doc` no longer shows internal/test-only modules.
