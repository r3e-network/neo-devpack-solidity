# Comprehensive Audit, Optimization & Refactor Report — neo-devpack-solidity v0.30.3

**Date:** 2026-08-29
**Scope:** Full codebase — 510 Rust source files (~91.1K LOC `src/`), ~97K LOC `tests/`, 35 DevPack Solidity libraries.
**Method:** Fresh baseline (build / `clippy --all-targets --all-features` / `test --workspace`), re-verification of the prior `audit-report-v0.28.0.md` against the *current* working tree, plus four parallel deep-subsystem scans (`ir`, `runtime`+`opcode`, `cli`/`frontend`/`codegen`/`manifest`/…, DevPack+toolchain). Every finding below was verified by reading the code, not inferred from the older reports.

---

## 0. Executive summary

The v0.28.0-era reports in this repo are **stale** — the working tree already contained fixes for most of their open P2 items (nested-`EQUAL` type-strictness, the `storage_ops` dead functions, the `NativeTypes.ContractState` field types, five of six old clippy warnings, the DevPack missing-handler/wrapper gaps). Those are now marked **RESOLVED** below.

This pass found the codebase to be in **excellent health** — no P0/P1 compiler-safety defects (a compiler must never panic on malformed input; it does not). The genuine issues that remained were a handful of P1/P2 **cleanup, parity and performance** items, which were fixed here and gated green:

| Dimension | Before | After |
|-----------|--------|-------|
| `cargo check --lib --all-features` | clean | clean |
| `cargo clippy --all-targets --all-features` | 1 warning | **0 warnings** |
| `cargo fmt --all -- --check` | **dirty** (pre-existing drift) | **clean** |
| `cargo test --workspace --all-features` | ~965 pass | **1,019 pass / 0 fail** |
| `src/` LOC | 92,843 | 91,116 (**−1,727**) |
| Never-invoked `VMBridge` dispatch | 23 files / ~1.7K LOC live-in-tree | 7 files (dead cluster deleted) |

---

## 1. Changes made this pass (all test-gated)

### 1.1 [P1 · behavior] Import-resolution parity — CLI vs `--standard-json`
`import_aliases()` existed as **two private, drifted copies**: the on-disk CLI resolver (`src/cli/cli_parts/cli_run/imports.rs`) applied Foundry version-pin aliases (`@scope/pkg@x.y.z/path`), while the standard-JSON resolver (`src/cli/standard_json/standard_json_process/imports.rs`) did **not**. Result: an identical Foundry-pinned project resolved imports via the CLI but failed via `--standard-json`.

**Fix:** lifted one canonical `pub(crate) fn import_aliases()` (base + OpenZeppelin prefix + `version_pin_aliases`) into the CLI module and routed **both** resolvers through it. Prevents the entire class of future drift.

### 1.2 [P2 · cleanup] Dead-code removal — the `VMBridge` dispatch cluster (~1.7K LOC, 16 files)
The runtime's `VMBridge` built an `instruction_mapping: HashMap<u8, fn>` and a `system_calls: HashMap<String, fn>`, populated in `new()` and consumed only by `handle_instruction()` / `call_system_function()` — **both of which have zero callers**. The live execution path is `VMBridge::new → execute → ExecutionContext::step()`, and `ExecutionContext` already implements all 196 opcodes + 35 syscalls. The entire parallel dispatch was dead.

**Deleted:** `bridge_helpers.rs`, `bridge_impl_syscalls.rs`, `bridge_impl_arithmetic/`, `logic/`, `stack/`, the arithmetic/bitwise/comparison/shifts files under `bridge_impl_stack_items/`, and `bridge_impl_core/{initialize,instruction,system_function}.rs`. The `instruction_mapping`/`system_calls` fields and `InstructionHandler`/`SystemCall` aliases were removed from `bridge_types.rs`; `new()` no longer builds them. `VMBridge`, `VMBridgeError` and `execute` (the public surface, pinned by `tests/fix_runtime_modules_tests.rs`) are preserved. The 1,019-test suite passing unchanged confirms the runtime semantics were untouched.

### 1.3 [P2 · perf] IR lowering hot paths
- `src/ir/ir_build/panic.rs`: `emit_panic` re-ran `keccak256("Panic(uint256)")` on every call (~41 lowering sites) to produce a constant `0x4e487b71`. Hoisted to a `const`, with a new unit test (`panic_selector_matches_keccak`) that recomputes the digest so the hoisted value can never silently drift.
- `src/ir/ir_context/lowering_context.rs`: `is_externally_callable_fn` allocated a `String` on every lookup just to hash it; replaced with a borrowing `.iter().any(...)`.

### 1.4 [lint] `cargo clippy`
Fixed the last warning (`manual_is_multiple_of` in `runtime/.../stdlib.rs` hex-decode) → clippy now 0 warnings across all targets/features.

### 1.5 [P2/P3 · DevPack]
- Removed six unreachable `bls12381G1Add/G1Mul/G1Neg/G2Add/G2Mul/G2Neg` wrappers from the monolith `devpack/contracts/Syscalls.sol` (the on-chain `CryptoLib` has no G1/G2-suffixed methods; the builtin whitelist in `resolve.rs` already rejects these names with an "unsupported builtin library call" diagnostic — guarded by `tests/fuzz_tests/batches_116_120.rs::UUU2_2`). Brings the monolith in sync with `syscalls/SyscallsCrypto.sol`, which had already dropped them.
- Removed an unused `import "./SyscallsTypes.sol";` in `devpack/contracts/syscalls/SyscallsOracle.sol`.

### 1.6 [hygiene] `cargo fmt --all` normalized pre-existing formatting drift in `ir_expressions/dispatch/binary.rs` and `binary_u256_softarith.rs` (CI runs `fmt --check`, so the tree was not actually gate-clean). `RELEASE_NOTES.md` refreshed (was pinned to v0.27.0, three releases behind `Cargo.toml`).

---

## 2. Prior-report items — re-verification (current tree)

| Old finding | Status now |
|-------------|-----------|
| P2-1 Nested `EQUAL` not type-strict (`stack.rs`) | **RESOLVED** — `PartialEq` now compares `type_tag` before bytes |
| P2-2 `NativeTypes.ContractState` field types | **RESOLVED** — now `int256 id` / `uint256 updateCounter` / `address hash`, consistent with runtime + `SyscallsTypes` |
| P2-3 ~20 missing runtime handlers (StdLib base58/base64url, NEO candidates, ContractMgmt, ed25519) | **LARGELY RESOLVED** — `bs58`/`ed25519-dalek` now in `Cargo.toml`, native handlers present |
| P2-4 Dead `build_storage_entries`/`allocate_iterator` | **RESOLVED** — no longer present |
| P2-5 Dead `VMBridge` dispatch | **RESOLVED this pass** (see §1.2) |
| P3-1 old clippy warnings | **RESOLVED** — 0 warnings now |
| P3-4 `compat/*.sol` pragma `^0.8.20` | **RESOLVED** — all `.sol` are `^0.8.19` |

---

## 3. Remaining findings — deliberately NOT changed (recommendations)

These are real but either risk regressions on a production compiler, are team-policy decisions, or are higher-effort/lower-reward. Listed with owner-relevant detail.

### 3.1 NeoVM simulator fidelity (correctness, needs targeted tests + on-chain differential)
- **`NUMEQUAL`/`NUMNOTEQUAL` are structural, not numeric** — `src/runtime/execution/instruction/arithmetic/comparison.rs:20-37` compares via `stack_items_equal` instead of coercing operands to BigInteger the way real NeoVM does. Divergences (reachable by a hand-crafted/fuzzed NEF): `Integer(5)` vs `ByteString[0x05]`, `[0x05]` vs `[0x05,0x00]`, and Arrays (which NeoVM *faults* on). → Add a BigInt-coercing numeric path and `FAULT` on non-numeric Array/Map. **Deferred:** changes observable simulator behavior; should land with new differential tests, not silently.
- **`SETITEM` ignores `type_tag`** — `src/runtime/execution/collections/indexing.rs:102-139` mutates a `ByteArray` regardless of `ByteString` vs `Buffer`; real NeoVM faults on immutable `ByteString`. → Reject when `type_tag == ByteString` (confirm the compiler always emits `CONVERT→0x30` first). **Deferred** for the same reason.

### 3.2 Compiler safety hardening (low risk, small wins)
- `src/runtime/execution/execution_impl_part3_offsets/input.rs:67` — `calldataload_word` `.expect()`s a 32-byte read; only reachable if `RuntimeConfig.memory_limit < 32` (non-default, `pub` field). → clamp to `min(32, …)`.
- `src/ir/ir_expressions/power.rs:218` — an `.expect()` relies on two independent `infer_type_from_expression` calls 130 lines apart agreeing. → bind the type once.

### 3.3 CLI behavior (needs exit-code / arg-parsing test review)
- Batch mode `exit(1)`s on the first failing input (`src/cli/cli_parts/cli_run/single_file.rs:242-313`), aborting the remaining file list. → collect per-file errors, continue, exit non-zero at end.
- Invalid `-O`/`optimizer.level` silently falls back to `2` (`single_file.rs:52`, `standard_json.rs:53`, `process.rs:33`). → reject via clap `value_parser`.
- Human-mode `emit_error` drops the `NSH-XXXX` code (`src/cli/cli_parts/cli_diagnostics.rs:69-77`), so users can't feed it to `--Wno/--Werror`.
- `extract_imports` runs a full 256 MiB-stack solang parse **per imported file** just to list imports (`cli_run/imports.rs:128`) → N+1 full parses per compile. → cache by content hash or an AST-lite scan.

### 3.4 Diagnostics quality (largest UX win, largest effort)
`IrDiagnostic` (`src/ir/ir_context/lowering_context.rs`) carries **no source `Loc`**, even though solang provides one on every node — so IR errors print `"function 'X': msg"` with no line/column. Threading `Loc` through diagnostics is the single biggest usability improvement available. Separately, `src/diagnostics/report.rs` is an intentionally-staged formatter (`#![allow(dead_code)]`, unit-tested, "will be wired in P1") whose JSON shape has already **diverged** from `cli_compile/errors.rs` — wire the two together in one change rather than deleting.

### 3.5 Toolchain / CI
- `.github/workflows/security.yml`: `cargo audit` (and npm/dotnet scans) run **non-fatally** — vulnerabilities are written to `security_report.md` but never fail the job. → gate on real advisories (keep network/parse errors non-fatal). **Deferred** because I cannot run `cargo audit` offline here and hard-failing could turn a green pipeline red for reasons outside this diff.
- `Cargo.toml`: direct dep is still `thiserror = "1.0"` while the tree also carries `thiserror 2.0` transitively. Unifiable by bumping the direct dep to `2.0` (mechanical, low risk). Other duplicate versions in `Cargo.lock` (`getrandom`, `rand`, `rand_core`, `hashbrown`, `windows-sys`, `itertools`) are unavoidable transitive splits from pinned parents. No `cargo deny`/`deny.toml` (license policy) exists.
- `clippy::manual_let_else` is set to `allow` in `Cargo.toml`; worth revisiting once diagnostics gain `Loc`.

---

## 4. Verification

Final combined CI-parity gate (see repo chat / `.final_gate.log`):

- `cargo fmt --all -- --check` → **clean**
- `cargo clippy --all-targets --all-features -- -D warnings` → **0 warnings, 0 errors**
- `cargo test --workspace --all-features` → **1,019 passed / 0 failed**

A working backup of the pre-refactor `src/runtime/bridge` tree is retained in the agent workspace (`bridge_backup/`) in case the dead-code removal needs to be revisited.

---

## 5. Verdict

v0.30.3 is in **production-ready health** for the compiler pipeline and in-tree simulator. This pass removed the last substantial dead-code island, fixed a real CLI↔standard-JSON parity bug, eliminated the remaining lint/format debt, and hoisted two hot-path allocations — all with the full suite green. The outstanding items are a small set of simulator-fidelity refinements and CLI/diagnostic UX improvements that each warrant their own change with dedicated tests, plus one CI policy decision (fail-on-advisory). None block production use.
