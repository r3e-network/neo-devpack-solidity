# Phase 6 — Performance: Benchmarks + Parallel Compilation

**Status:** Approved 2026-06-23
**Scope:** Add criterion benchmarks (measurement harness), then parallelize
per-contract compilation with rayon. Report whether it matters.

## Context

The exploration found:
- **Zero benchmarks exist** — `criterion` is a dev-dep but has no bench files
- **Rayon is a dormant dependency** — declared in Cargo.toml but never used
- **Parallel compilation is safe** — types are Send+Sync, `Arc<SelectorRegistry>` is read-only
- **No O(n²) patterns** — all passes are linear scans
- **No quick wins** — codebase already disciplined (with_capacity, format! off hot paths)
- **Amdahl ceiling** — parsing + analysis stay sequential; only the compile tail is parallelizable

## Deliverables

### PR1 — Criterion benchmark harness

Create `benches/compile.rs` with criterion benchmarks over representative inputs:
- 1-contract compile (SimpleToken)
- 5-contract compile (batch)
- Large contract (ERC20Token or GovernanceToken)
- O0 vs O3 comparison

This is the **blocking prerequisite** — without it, we can't measure whether PR2 matters.

### PR2 — Parallelize per-contract compilation

Replace the sequential `map` at `compile.rs:116-119`:
```rust
metadatas
    .into_iter()
    .map(|metadata| compile_metadata(metadata, verbose, options.clone()))
    .collect()
```
with:
```rust
use rayon::prelude::*;
metadatas
    .into_par_iter()
    .map(|metadata| compile_metadata(metadata, verbose, options.clone()))
    .collect::<Result<Vec<_>, _>>()
```

Handle two caveats:
1. **Verbose output interleaving** — gate verbose `-v` output behind a note that it may interleave in parallel mode (acceptable for a debug knob)
2. **Error determinism** — `par_iter` short-circuits on some error non-deterministically; accept this for the common (success) path, document it

Then re-run benchmarks to measure the actual speedup.

## Out of Scope

- Parallelizing the analysis loop (`solidity_analyse.rs:969`) — second candidate,
  defer until benchmarks show it's the bigger slice
- In-place optimizer passes (no verified benefit)
- Clone reduction (mostly necessary)
- Runtime simulator perf (test-only)

## Success Criteria

1. `benches/compile.rs` exists and runs via `cargo bench`
2. Per-contract compilation uses `rayon::into_par_iter`
3. Benchmark numbers reported (before vs after parallelization)
4. All tests pass
