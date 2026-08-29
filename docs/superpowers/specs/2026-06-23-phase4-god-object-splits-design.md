# Phase 4 — God-Object File Splits

**Status:** Approved 2026-06-23
**Scope:** Split 5 large files into focused modules via `include!` extraction.
All splits are mechanical (cut/paste + add `include!` in parent). Zero
behavior change, zero visibility change, zero call-site change.

## Context

The initial exploration identified 5 files over 1000 LOC as "god-objects."
Detailed analysis revealed all 5 are `include!` textual includes — they
compile into one giant module each (`ir` or `solidity`). The existing
codebase already uses multi-include splitting extensively (`build/module.rs`:
5 includes, `statements/assignments.rs`: 5, `solidity/analyse/inheritance.rs`:
4). Following this pattern is the lowest-risk way to split.

## The 5 Splits (in execution order)

### PR1: `solidity_analyse.rs` (1848 → ~900 + ~820 + ~120)

**Lowest risk, highest value.** Extract the sibling-merge AST walkers
(820 LOC of pure functions with zero shared state) and merge helpers.

New files (added as `include!` in `solidity.rs`):
- `solidity/analyse/sibling_merge_walkers.rs` (~820 LOC) — `collect_new_contract_refs*`, `collect_interface_casts_*`, `collect_low_level_call_method_refs_*`, `extract_static_*`, `collect_direct_sibling_contract_refs`
- `solidity/analyse/merge_helpers.rs` (~120 LOC) — `normalize_state_type_for_merge`

`solidity_analyse.rs` keeps: `analyse_source`, `analyse_all_sources` (~900 LOC).

### PR2: `lowering_context.rs` (1138 → ~400 + 5 impl files)

Split the `impl LoweringContext` methods across files. The struct
definition + constructor stays in place; methods fan out into
additional `impl<'a> LoweringContext<'a> { ... }` blocks.

New files (added as `include!` in `ir_context.rs`):
- `context/ctx_locals_scopes.rs` (~240 LOC) — locals, scopes, loops, labels, scratch pools, storage aliases
- `context/ctx_overloads.rs` (~260 LOC) — resolve_overload, using_*, FunctionOverloadTable
- `context/ctx_signatures.rs` (~220 LOC) — state_*, event_*, error_signature, LibraryStorageBody
- `context/ctx_type_utils.rs` (~130 LOC) — free fns: normalize_solidity_like_type_signature, is_implicitly_convertible

`lowering_context.rs` keeps: struct + constructor + simple accessors + diagnostics (~400 LOC).

### PR3: `binary.rs` (1540 → ~150 + 3 files)

Extract pure predicates, u256 softarith routines, and overflow guards.

New files (added as `include!` in `dispatch.rs`):
- `dispatch/binary_predicates.rs` (~350 LOC) — `is_*_operand`, `narrow_*_bits`, all `should_*` gates
- `dispatch/binary_u256_softarith.rs` (~670 LOC) — `emit_u256_*` family
- `dispatch/binary_overflow_guards.rs` (~370 LOC) — `emit_checked_arith_guard*`, `emit_widen_*`, `emit_truncate_*`

`binary.rs` keeps: `lower_binary_expr`, `emit_arith_with_overflow_ladder`, `lower_bytes_eq_hex_number_literal` (~150 LOC).

### PR4: `helpers.rs` (2426 → ~280 × 2 + encode + decode + misc)

The biggest file. Split ABI codec into logical layers.

New files (added as `include!` in `builtins.rs`):
- `builtins/abi_predicates.rs` (~280 LOC) — type predicates
- `builtins/abi_buffers.rs` (~280 LOC) — buffer/slot primitives (shared by encode + decode)
- `builtins/abi_encode.rs` (~700 LOC) — encode lowering
- `builtins/abi_decode.rs` (~850 LOC) — decode lowering
- `builtins/abi_structs_notify.rs` (~180 LOC) — struct flatten + Runtime.notify
- `builtins/native_serialize.rs` (~50 LOC)

### PR5: `return_revert.rs` (1818 → ~620 + ~640 + ~260 + ~120)

Split return lowering from revert lowering.

New files (added as `include!` in `dispatch.rs`):
- `dispatch/return_lower.rs` (~620 LOC) — `lower_return_statement`, implicit return, array-return wrapping
- `dispatch/revert_lower.rs` (~640 LOC) — `lower_revert_statement`, custom-error selectors, Error(string) envelope
- `dispatch/return_revert_slots.rs` (~260 LOC) — ABI slot encoders
- `dispatch/fixed_array_shape.rs` (~120 LOC) — `parse_nested_fixed_array_shape`

## Per-PR Verification

Every PR must pass:
```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

Since these are pure `include!` extractions (code moves, no logic change),
test pass = success. No behavior can change.

## Risk

**Effectively zero per PR.** The `include!` mechanism means the code is
textually identical before and after — same module scope, same visibility,
same call sites. The compiler will catch any accidental reference breakage.

The only real risk is **merge conflicts between PRs** — but since each PR
touches a different parent file's `include!` list, there are no conflicts
if landed sequentially.

## Success Criteria

1. All 5 files split into focused modules.
2. No file in the split set exceeds ~900 LOC (down from 1138–2426).
3. All tests pass identically.
4. The parent files' `include!` lists follow the existing convention.
