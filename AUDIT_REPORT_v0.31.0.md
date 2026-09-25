# Comprehensive Audit, Optimization & Refactor Report — v0.31.0

**Date:** 2026-09-03  
**Scope:** Rust compiler/runtime, C# runtime, DevPack Solidity libraries, CLI, CI and documentation.  
**Method:** Four parallel source audits, manual verification of all reported findings, targeted regression tests, full Rust workspace gates, and .NET runtime tests.

## Executive summary

This pass addressed the deferred simulator-fidelity and compiler-safety items from v0.30.3, plus newly verified CLI, DevPack, C# runtime, and CI defects.

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` | **pass** |
| `cargo clippy --all-targets --all-features -- -D warnings` | **pass** |
| `cargo test --workspace --all-features` | **pass** — 994 fuzz tests + all other workspace tests, 0 failures |
| `dotnet build src/Neo.Sol.Runtime/Neo.Sol.Runtime.csproj -c Release` | **pass** |
| `dotnet test tests/Neo.Sol.Runtime.Tests/Neo.Sol.Runtime.Tests.csproj -c Release` | **pass** — 48 tests |
| New audit regression tests | **pass** — 31 tests |

Five pre-existing fuzz fixtures that exercise known unsupported Yul/BLS paths are now explicitly `#[ignore]` with a reason; they are visible in the test summary rather than silently omitted by cargo target discovery.

## Changes implemented

### NeoVM simulator and runtime safety

- Implemented `NEWARRAY_T` immediate-operand consumption and typed default element construction; instruction pointer now advances by two bytes.
- Changed `NUMEQUAL`/`NUMNOTEQUAL` (and equality halves of `LE`/`GE`) to numeric BigInteger coercion; non-numeric compound operands fault instead of being structurally compared.
- Rejected `SETITEM` on immutable ByteString; mutable Buffer writes remain supported.
- Rejected circular Array/Map mutations to prevent host stack overflow during serialization, JSON conversion, or recursive equality.
- Fixed `i64::MIN` modulus handling in MODMUL/MODPOW by widening before `abs()`.
- Range-checked signed POW exponents instead of truncating with `as u32`.
- Sign-extended narrow negative ByteString integer coercions.
- Removed an unreachable duplicate empty-buffer branch in CONVERT.
- Saturated synthetic ledger arithmetic for extreme host-controlled block heights.
- Made reads past the allocated memory high-water mark zero-fill, while retaining configured memory-limit checks.
- Clamped the calldata word helper for configurations with a memory limit below 32 bytes.

### Compiler / IR

- Derived indexed-event parameter canonical types and dynamic/static classification from resolved `NeoType`, keeping them consistent with topic-0 signatures for contract types, enums, and structs.
- Added a visited set to selector constant resolution and wired the existing constant-resolution guard into ordinary variable lowering, preventing cyclic initializer stack overflows.
- Rejected signed exponent expressions at IR lowering to prevent non-terminating negative-exponent loops.
- Bound the power expression type once instead of repeating an inference relied upon by a later `expect`.
- Corrected the low-level `.call` view/pure diagnostic so it no longer mentions delegatecall in the wrong handler.

### CLI

- Standard JSON now returns a status flag and exits non-zero when emitted JSON contains error-severity diagnostics, while preserving the JSON output.
- `-O/--optimize` is validated by clap to the documented range 0–3.
- Batch compilation continues after a failed input, reports all failed files, and exits non-zero at the end; single-file mode remains fail-fast.
- Human-mode diagnostics now include stable `NSH-XXXX` codes.
- Analyze findings use `BTreeMap` ordering for deterministic output.
- NEF/JSON source paths strip Windows `\\?\` verbatim prefixes and normalize separators.

### DevPack

- Fixed the invalid `NativeContracts.ROLE_CONTRACT` reference in `NativeRole.sol`.
- Corrected `CompleteNEP11NFT._updateFloorPrice` to clamp the moving-window start and avoid underflow on fresh long-duration listings.
- Corrected NEP-22 `manifest` from `string` to `bytes` in both interface and base implementation.
- Repaired the split `SyscallsContract.sol` library by removing invalid qualified function declarations and added the missing contract helpers.
- Added split-library aliases/utilities (`getCurrentRandom`, script-hash/address helpers, contract existence/script helpers) so split and monolith surfaces are aligned.

### C# runtime

- Added decoding for compressed storage values after cache eviction/restart; extracted and tested the pure `ExpandStoredValue` path.
- Changed EVM memory expansion-cost arithmetic to 64-bit math to prevent quadratic-cost overflow.
- Added bounds validation for payload-controlled ABI dynamic bytes pointers and lengths.
- Added local gas-consumption accounting to `GasContext.ConsumeGas`.

### CI, dependencies and tests

- Registered `tests/fuzz_tests/mod.rs` as the `fuzz_tests` Cargo integration target, making the documented nightly command real.
- Updated staged StdLib Base58, memoryCompare, memorySearch, and serialization assertions to differential/current BinarySerializer contracts.
- Updated the conditional-jump oracle for signed ByteString semantics.
- Removed ignored Cargo profile tables from `.cargo/config.toml`; profiles remain authoritative in `Cargo.toml`.
- Unified the direct `thiserror` dependency on 2.0.
- Fixed the performance workflow's cross-step `LINES_GOV` variable usage.
- Corrected README references to the actual nightly showcase workflow and .NET 10.

## Remaining known gaps

The previously identified gaps were addressed in the follow-up pass:

1. Yul mstore/mload/reference fixtures now materialize mutable Buffers before byte-order reversal; all three fixtures are active and pass.
2. BLS12-381 CryptoLib resolver, syscall mapping, and differential pairing paths are active and pass.
3. Event/error registries now use canonical signatures and argument-type matching, including same-arity overloads.
4. Standard-JSON warning suppression/promotion is wired through processing; source-closure construction is cached.
5. Output prefix collisions are rejected before writing artifacts.
6. Analyze-mode failures are surfaced as explicit findings instead of being silently replaced with an empty report.

The remaining non-blocking limitations are broader feature work rather than verified correctness defects: complete source-location threading through every `IrDiagnostic`, and unsupported standard-JSON settings that are intentionally reported as warnings.

## Verdict

The v0.31.0 tree is materially safer and more faithful: malformed runtime inputs now fault rather than panic or silently diverge, compiler paths reject cyclic/non-terminating constructs, CLI status and artifacts are deterministic, DevPack split libraries compile consistently, and the C# runtime handles persisted compressed values and large memory calculations correctly. All active Rust and C# gates are green.
