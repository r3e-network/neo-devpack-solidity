# Changelog

All notable changes to the Neo DevPack for Solidity will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.30.3] - 2026-08-11

Release focused on runtime modularization, documentation cleanliness, and
CI-driven hardening of the compiler and standard library.

### Changed

**Runtime `stdlib.rs` modularization**:
- Split `src/runtime/execution/execution_impl_part2_native/stdlib.rs` into
  three focused modules:
  - `stdlib.rs`: Core string/encoding helpers and native dispatch.
  - `stdlib_abi.rs`: ABI-encoded argument helpers (`abi_encode`, `call`, etc.).
  - `stdlib_binary.rs`: Binary buffer helpers (`concat`, `range`, `take`, etc.).

**Compiler analysis pipeline modularization**:
- Split `src/solidity/solidity_analyse.rs` into pipeline-stage modules:
  - `solidity_analyse.rs`: Top-level driver.
  - `solidity_analyse_typecheck.rs`: Type checking and validation.
  - `solidity_analyse_transform.rs`: Source-to-source transformations.

### Fixed

**Documentation warnings**:
- Resolved all `cargo doc` warnings; `cargo doc --no-deps` now exits with
  0 warnings.

**QA gate formatting and clippy**:
- Applied `cargo fmt --all` formatting fixes.
- Applied `cargo clippy --all-targets --all-features` lint fixes.

### Added

**CI audit scripts**:
- `scripts/ci/file_length_audit.py`: Enforces a per-file line-count budget to
  prevent runaway module growth.
- `scripts/ci/unwrap_audit.py`: Audits production `unwrap()` and `expect()`
  usage and verifies each remaining instance is documented with an invariant
  comment.

### Verification
- `cargo fmt --all -- --check`: passed
- `cargo clippy --all-targets --all-features -- -D warnings`: 0 warnings
- `cargo doc --no-deps`: 0 warnings
- `cargo test --workspace --all-features`: passed
- `cargo build --release`: succeeded

## [v0.30.2] - 2026-07-04

Continued code quality fixes: all remaining production `unwrap()` converted
to `expect()` with documented invariants, and CLI fatal-error pattern
centralized via `fatal_error!` macro.

### Fixed

**Regex literal `unwrap()` → `expect()`** (10 instances):
- `solidity/upgrade.rs`: All 10 `Regex::new(...).unwrap()` calls converted
  to `.expect("valid regex pattern")` with a module-level comment explaining
  that these are compile-time constant patterns.

**Range-guarded `expect()` documentation** (2 instances):
- `runtime/execution/syscalls/contract.rs`: Added `# Invariant` comment
  explaining the `1..=16` range guard.
- `neo/contract_hash.rs`: Added `# Invariant` comment explaining the
  `value <= 16` range guard.

**CLI fatal-error pattern centralization** (12 instances):
- Added `fatal_error!` macro in `cli_defs.rs` — standardizes the
  `eprintln!` + `exit(1)` pattern.
- `cli_run/single_file.rs`: 7 fatal-error patterns replaced with macro.
- `cli_run/standard_json.rs`: 5 fatal-error patterns replaced with macro.
- `cli_analyze.rs`: 1 fatal-error pattern replaced with macro.

### Remaining `unwrap()`/`expect()` in production code
All remaining instances (8) are documented invariants:
- `resolve.rs` (1): Hardcoded table lookup with matching arm guarantee
- `input.rs` (1): 32-byte read with documented memory_limit guarantee
- `manifest/build.rs` (1): Non-empty iterator with invariant comment
- `literals.rs` (1): Hardcoded exponent with invariant comment
- `ctx_locals_scopes.rs` (1): `checked_add` with impossible-overflow comment
- `power.rs` (1): Sign-tracking variable with invariant comment
- `constant_folding.rs` (2): In-range arithmetic with invariant comments

### Verification
- `cargo check`: 0 errors, 0 warnings
- `cargo clippy`: 0 warnings
- `cargo test`: 965 tests passed, 0 failed (zero regressions)

## [v0.30.1] - 2026-07-04

Code quality fixes from team capability improvement plan. Addresses error
handling and unsafe code issues identified in the code quality audit.

### Fixed

**Manifest `eprintln!` elimination** (7 calls → 0):
- `manifest/build.rs`: All 7 `eprintln!` warning calls replaced with a
  `warnings: &mut Vec<String>` collector parameter. Warnings are now
  returned to the caller (`compile.rs`) and added to the compilation
  warnings list as `Diagnostic::warning()` entries, instead of being
  printed directly to stderr.

**Runtime `unwrap()` hardening** (3 risky instances → 0):
- `execution_impl_part3_conversion.rs`: Added empty-bytes guard before
  `bytes.last().expect(...)` — previously could panic on empty ByteString.
- `execution_impl_part3_offsets/call_stack.rs`: Changed `unwrap()` to
  `expect()` with documented invariant.
- `execution_impl_part2_native/stdlib.rs`: Changed `unwrap()` to
  `expect()` with documented invariant.

**Unsafe block documentation** (1 block → 2 documented):
- `storage_ops.rs` line 164: Added `# Safety` comment explaining the
  soundness argument (single-threaded execution, exclusive access,
  pointer valid for context lifetime).

### Remaining `unwrap()`/`expect()` in production code
All remaining instances (30+) are documented invariants:
- `resolve.rs` (7): Hardcoded table lookups with matching arms
- `upgrade.rs` (10): `Regex::new()` on compile-time constant patterns
- `ctx_locals_scopes.rs` (1): `checked_add` with impossible-overflow comment
- `power.rs` (1): Sign-tracking variable always set before use
- `contract_hash.rs` (1): Guarded by `value <= 16` check
- `input.rs` (1): 32-byte read with documented memory_limit guarantee

### Verification
- `cargo check`: 0 errors, 0 warnings
- `cargo clippy`: 0 warnings
- `cargo test`: 965 tests passed, 0 failed (zero regressions)

## [v0.30.0] - 2026-07-04

Architecture Phase 4: file refactoring. Splits 3 of the largest monolithic
files (>800 lines) into smaller, domain-focused submodules. Reduces the
count of files exceeding 800 lines from 13 to 10.

### Changed

- `src/ir/ir_statements/assembly.rs` (1440 lines) split into:
  - `assembly.rs` (10 lines) — module dispatcher
  - `assembly/extsload.rs` (190 lines) — extsload/exttload storage access
  - `assembly/yul/mod.rs` (289 lines) — Yul block lowering + state
  - `assembly/yul/dispatch.rs` (562 lines) — Yul statement/expression dispatch
  - `assembly/yul/opcodes.rs` (407 lines) — Individual Yul opcode implementations

- `src/frontend/frontend_parse.rs` (1037 lines) split into:
  - `frontend_parse.rs` (13 lines) — module dispatcher
  - `frontend_parse/parse.rs` (246 lines) — source parsing + file-level injection
  - `frontend_parse/pragma.rs` (312 lines) — pragma validation + feature gates
  - `frontend_parse/semver.rs` (304 lines) — version range types + comparator parsing
  - `frontend_parse/natspec.rs` (184 lines) — NatSpec comment handling

- `src/ir/ir_statements/assignments/lower_assignment.rs` (1140 lines) split into:
  - `lower_assignment.rs` (728 lines) — main assignment lowering
  - `storage_array_ops.rs` (413 lines) — storage array memory↔storage copy + read + external dynamic assignment

### Remaining
10 files still exceed 800 lines — all are monolithic match chains or
dispatchers that require surgical refactoring (not mechanical splitting).
These will be addressed in subsequent releases.

### Verification
- `cargo check`: 0 errors, 0 warnings
- `cargo clippy`: 0 warnings
- `cargo test`: 965 tests passed, 0 failed (zero regressions)

## [v0.29.1] - 2026-07-04

Architecture Phase 1 (continued): manifest extraction. Completes the CLI
decomposition by extracting manifest generation into a first-class module.

### Added
- `src/manifest/` — new top-level module for manifest generation
  (extracted from `src/cli/cli_parts/cli_manifest/`). Contains manifest
  building, permissions inference, standards detection, and the
  `solidity_to_manifest_type` type mapping function. 11 files, ~2,332 LOC.
- `ManifestError` type — manifest-specific error type that doesn't depend
  on `crate::cli`, mapped to `CompileError::Manifest` at the CLI boundary.

### Changed
- Broke bidirectional dependency between `cli_manifest` and `standard_json`:
  - Moved `solidity_to_manifest_type` from `standard_json_output.rs` to
    `src/manifest/mod.rs` (it's a manifest type mapping, belongs in manifest)
  - `standard_json_abi.rs` now imports `manifest_abi_method_name_fn` from
    `crate::manifest` (one-directional: standard_json → manifest)
- Broke `CompileError` ownership coupling:
  - `build_manifest` now returns `Result<Value, ManifestError>` instead of
    `Result<Value, CompileError>`
  - `cli_compile/compile.rs` maps `ManifestError` to `CompileError::Manifest`
    at the boundary
- CLI module reduced from 117 to 106 files (11 files extracted)
- `cli_parts/mod.rs` no longer declares `cli_manifest` submodule

### Verification
- `cargo check`: 0 errors, 0 warnings
- `cargo clippy`: 0 warnings
- `cargo test`: 965 tests passed, 0 failed (zero regressions)
- Public API unchanged

## [v0.29.0] - 2026-07-04

Architecture Phase 1: CLI decomposition. Extracts two major subsystems from
the CLI god module (151 files) into first-class top-level modules.

### Added
- `src/codegen/` — new top-level module for NeoVM bytecode generation
  (extracted from `src/cli/bytecode/`). Contains bytecode assembly, builtins,
  disassembler, and storage helpers. 29 files, ~5,394 LOC.
- `src/optimizer/` — new top-level module for IR optimization passes
  (extracted from `src/cli/ir_optimize/`). Contains constant folding, label
  deduplication, and NeoVM peephole optimization. 5 files, 692 LOC.

### Changed
- `src/cli/mod.rs` — removed `mod bytecode;` and `mod ir_optimize;` declarations.
  CLI module now imports `generate_contract_bytecode` and `optimize_ir` from
  the new top-level modules. CLI file count reduced from 151 to 117 files.
- `src/cli/cli_parts/mod.rs` — imports `generate_contract_bytecode` from
  `crate::codegen` and `optimize_ir` from `crate::optimizer` directly,
  instead of through `crate::cli` re-exports.
- `src/lib.rs` — added `pub mod codegen;` and `pub mod optimizer;` as
  `#[doc(hidden)]` internal modules.
- All `bytecode::` references in `cli_parts/` and `tests/` updated to
  `codegen::` or `crate::codegen::`.

### Deferred
- Manifest extraction (`src/neo/` + `src/cli/cli_parts/cli_manifest/` →
  `src/manifest/`) deferred to a sub-phase. Two blocking couplings identified:
  1. Bidirectional dependency between `cli_manifest` and `standard_json`
  2. `CompileError` ownership (defined in `cli_compile/types.rs`, used by
     `cli_manifest`)

### Verification
- `cargo check`: 0 errors, 0 warnings
- `cargo clippy`: 0 warnings
- `cargo test`: 965 tests passed, 0 failed (zero regressions)
- Public API unchanged: `compile_contracts`, `disassemble_neovm_bytecode`,
  standard-JSON all work identically.

## [v0.28.1] - 2026-07-03

Audit-driven patch release. Fixes all P2 and P3 issues identified in the
comprehensive v0.28.0 audit report — nested EQUAL type-strictness, ContractState
type mismatch, 20 missing runtime handlers, dead code removal, clippy warnings,
DevPack import hygiene, and 14 missing Solidity wrappers for runtime methods.

### Fixed

- **Nested EQUAL type-strictness** (P2-1) — `stack_items_equal()` now recursively
  calls itself for Array and Map element comparison instead of delegating to
  `PartialEq`, which ignored `type_tag`. `EQUAL` on arrays containing
  `ByteString` vs `Buffer` elements with identical bytes now correctly returns
  `false`. The `PartialEq` impl for `ByteArray` also now checks `type_tag`.
- **NativeTypes.ContractState type mismatch** (P2-2) — `hash` changed from
  `bytes32` to `address`, `id` from `uint16` to `int256`, `updateCounter` from
  `uint8` to `uint256`, matching the NeoVM ContractState wire format.
- **20 missing runtime handlers** (P2-3) — implemented:
  - NEO (5): `getAllCandidates`, `getCandidateVote`, `getRegisterPrice`,
    `setRegisterPrice`, `unclaimedGas`
  - ContractManagement (4): `destroyContract`, `getContractById`,
    `listContracts`, `setMinimumDeploymentFee`
  - StdLib (10): `base64UrlEncode`, `base64UrlDecode`, `base58Encode`,
    `base58Decode`, `base58CheckEncode`, `base58CheckDecode`, `memoryCompare`,
    `memorySearch`, `stringSplit`, `strLen`
  - CryptoLib (1): `verifyWithEd25519` (Ed25519 signature verification via
    `ed25519-dalek`)
- **Dead code removal** (P2-4/P2-5) — removed `build_storage_entries` and
  `allocate_iterator` from `storage_ops.rs` (remnants of v0.28.0 streaming
  iterator refactor). Fixed `VMBridge::eq_stack_items` to check `type_tag`.
- **7 clippy warnings** (P3-1) — fixed `redundant_closure`, `absurd_extreme_comparison`,
  `iter_kv_map`, `empty_line_after_doc_comment`, and `borrowed expression` warnings.
  `cargo clippy` now reports 0 warnings.

### Changed

- **DevPack import hygiene** (P3-2) — removed 10 unused imports:
  - `SyscallsTypes.sol` from `SyscallsCrypto.sol`, `SyscallsPolicy.sol`,
    `SyscallsRole.sol`
  - `NativeTypes.sol` from `NativeOracle.sol`, `NativeNotary.sol`,
    `NativeGAS.sol`, `NativeLedger.sol`, `NativeTreasury.sol`,
    `NativePolicy.sol`, `NativeRole.sol`
- **Removed 9 unused `using Syscalls for *;` directives** (P3-3) — all 9 native
  library files used qualified `Syscalls.contractCall(...)` calls, making the
  `using` directive unnecessary.
- **Pragma consistency** (P3-4) — `compat/EVMContractFactory.sol`,
  `EVMFallbackDispatcher.sol`, `EVMNativeAssetAdapter.sol` changed from
  `^0.8.20` to `^0.8.19` to match the rest of the codebase.
- **Fixed compile-time bug** — removed dead `CONTRACTMGMT_CONTRACT` constant in
  `NativeContractMgmt.sol` that referenced a non-existent constant name
  (`NativeContracts.CONTRACTMGMT_CONTRACT`); methods use
  `NativeContracts.CONTRACT_MANAGEMENT` directly.

### Added

- **14 missing Solidity wrappers** (P3-5) — added wrappers for runtime methods
  that had Rust handlers but no Solidity interface:
  - CryptoLib (7): `sha1`, `bls12381G1Add`, `bls12381G1Mul`, `bls12381G1Neg`,
    `bls12381G2Add`, `bls12381G2Mul`, `bls12381G2Neg`
  - Oracle (1): `getOracleRequest(uint256 requestId)`
  - Syscalls (6): `getMsgValue`, `getNetwork`, `getAddressVersion`,
    `getInvocationCounter`, `getRandom`, `burnGas`
  - Added to both domain libraries and the `Syscalls.sol` aggregate.
- **New runtime dependencies** — `bs58 = "0.5"` and `ed25519-dalek = "2.1"`
  added as optional dependencies under the `runtime` feature for base58 and
  Ed25519 handler implementations.

## [v0.28.0] - 2026-07-03

Precision & correctness release. Four deep architecture improvements from a
systematic P3 audit — enriching the embedded NeoVM simulator's type system to
distinguish `ByteString` from `Buffer`, adding arithmetic operand-size gas
scaling, implementing lazy streaming storage iteration, and enriching Oracle
`getrequest` responses to match the real NeoVM wire format.

### Added

- **ByteString / Buffer type distinction** — `StackItem::ByteArray` now carries
  a `type_tag` (`ByteString = 0x28` / `Buffer = 0x30`). `NEWBUFFER`, `CAT`,
  `SUBSTR`, `LEFT`, `RIGHT` emit `Buffer`-tagged byte arrays; `CONVERT` tags
  output by the target type code. This brings the simulator's type model in
  line with the NeoVM, where `Buffer ≠ ByteString` even with identical bytes.
  `StackItem::byte_array()` (ByteString-tagged) and `StackItem::buffer()`
  (Buffer-tagged) constructors added; 40+ match arms updated.
- **Type-strict EQUAL** — `stack_items_equal()` now compares `type_tag` before
  comparing bytes. `ByteArray(a)` `EQUAL` `ByteArray(a)` returns **false** when
  the tags differ, matching NeoVM semantics.
- **ISTYPE split** — `ISTYPE(0x28)` now correctly distinguishes ByteString
  from `ISTYPE(0x30)` Buffer; the two type codes were previously conflated.
- **REVERSEITEMS restriction** — ByteString-tagged arrays are rejected
  (only Buffer / Array arrays may be reversed), matching NeoVM behaviour.
- **MEMCPY restriction** — requires a Buffer-tagged destination; a
  ByteString-tagged target is rejected.
- **Arithmetic operand-size gas scaling** — `MUL`, `DIV`, `MOD`, `POW`,
  `MODMUL`, `MODPOW` now charge `max(byte_len(a), byte_len(b)) × 3` extra gas
  (`ARITH_PER_BYTE_GAS = 3`) on top of the base opcode cost, reflecting
  big-integer computational complexity.
- **Streaming storage iterator** — `IteratorState` redesigned with a
  `StreamingCursor` (prefix, options, `last_key`, `exhausted`, `page_size=50`)
  for lazy batch-on-demand fetching. `System.Storage.Find` now allocates a
  small pre-fetched buffer; `Iterator.Next` triggers `refill_iterator_buffer()`
  when the buffer is exhausted. Per-page overlay merging (`shape_raw_entries`)
  computes the correct `last_key` after merge, enabling accurate cursor
  pagination across overlay-modified key ranges.
- **Oracle `getrequest` response** — added a `"getrequest"` match arm in
  `invoke_native_oracle` that returns a structured NeoVM Map with 7 fields
  (`OriginalTxid`, `GasForResponse`, `Url`, `Filter`, `CallbackContract`,
  `CallbackMethod`, `UserData`), matching the real Oracle contract's wire
  format.

### Changed

- **Simplified `consume_gas` signature** — changed from
  `consume_gas(name, amount: Option<u64>)` to `consume_gas(name, amount: u64)`.
  All 26 bridge call sites batch-updated (`None` → `3`; `Some(N)` → `N`).
- **Removed dead `named_operation_cost()`** — the 9-arm EVM-style cost table
  was never used; the `operation_cost()` path is always taken.
- **`StorageQuery` cursor support** — added `start_after_key: Option<Vec<u8>>`
  field for cursor-based pagination; `StorageManager::query()` sorts results
  before applying the cursor filter.

### Fixed

- **Buffer-producing opcodes** — `NEWBUFFER`, `CAT`, `SUBSTR`, `LEFT`, `RIGHT`
  now correctly emit `Buffer`-tagged `ByteArray` items (previously emitted
  `ByteString`-tagged), matching NeoVM semantics.
- **CONVERT type tagging** — the output of `CONVERT` is now tagged by the
  target type code (`0x28` → ByteString, `0x30` → Buffer) instead of always
  emitting ByteString.

## [v0.27.0] - 2026-07-02

Real-world-compatibility & performance release. Driven by two systematic
audits — a ~600-contract famous-contract compilation evaluation (full
dependency trees; DeFi / NFT / GameFi / zero-knowledge / infrastructure)
and a 168-probe **runtime-semantic** audit of the latest Solidity 0.8.x
features — every reproducible, deployment-blocking gap found was fixed,
and a 63-repro validation matrix re-verified the entire campaign with
**zero regressions**. The in-tree NeoVM interpreter is ~**1.9× faster**
on compute-heavy workloads. Compiler / CLI / workspace and devpack stay
version-aligned at **v0.27.0**.

### Added

- **Famous-contract sample corpus** — 33 self-contained, real-world
  contracts (WETH9, UniswapV2ERC20, DAI, ERC721A, BoredApeYachtClub,
  Synthetix StakingRewards, MasterChef, PLONK/Semaphore verifiers,
  ETHTornado, Multicall3, TimelockController, …) vendored under
  `third_party/famous-contracts/samples/` across
  `defi/nft/gamefi/zkp/infra-dao/patterns`, each compiling hermetically
  (no include paths). `tests/famous_samples_compile.rs` asserts **100%**.
- **`W_USER_DEFINED_OPERATOR`** — user-defined operators
  (`using {add as +} for T global`, Solidity 0.8.19) are accepted but NOT
  dispatched (the bound function is not called); the compiler now warns
  loudly instead of silently emitting raw arithmetic.
- **`W_TRANSIENT_PERSISTED`** — `transient` state variables
  (Solidity 0.8.28, EIP-1153) compile to regular PERSISTENT storage
  (NeoVM has no transient store); the compiler now warns loudly instead
  of silently persisting.
- README famous-contract compilation coverage table (~87% across ~600
  contracts; zero-knowledge 100%, solmate 100%, solady 99%, Uniswap-v3
  93%, OpenZeppelin 89%, Aave v3 85%).

### Fixed

- **Famous-contract merge compatibility** — a `library`'s own event
  declarations are carried into the consuming contract (Aave v3
  `ReserveLogic.ReserveDataUpdated`); a concrete contract holding an
  `abstract contract`-typed field is no longer forced to implement that
  abstract's virtuals (Compound v2 `PriceOracle`/`InterestRateModel`
  fields); the sibling-merge carries a sibling's INHERITED
  (interface/base) events through cross-contract `new` (USDC FiatToken,
  SushiSwap MasterChef, ENS PublicResolver); `new X{salt: s}()`
  CREATE2-style salted creation is accepted (salt ignored with a warning
  — Neo N3 has no CREATE2). Aave v3 compilation coverage: 71% → **85%**,
  with every deployable implementation (Pool / PoolConfigurator / AToken)
  compiling.
- **`bytesN` shifts** — `bytesN << k` / `>> k` now operate on the
  BIG-endian face value with EVM truncation. Previously the BE ByteString
  reached native NeoVM SHL/SHR, which rejected it ("Invalid operands",
  N < 32) or read it little-endian (`bytes32(0x100) >> 8` returned
  2^232). Includes the shift-expression type-inference arm so
  `uint256(b32 >> 8)` applies the cast's byte-reversal.
- **Unchecked `int256` arithmetic** — `unchecked` int256 Add/Sub/Mul now
  wraps two's-complement mod 2^256 (`type(int256).max + 1 ==
  type(int256).min`); previously the unbounded-BigInt native op produced
  an unrepresentable 33-byte value. Checked mode still Panics(0x11).
- **Fixed-size storage arrays** — `T[N].length` returns the declared
  compile-time bound N; previously it read the (never-written) base slot
  and returned 0, silently no-op'ing `for (i = 0; i < a.length; i++)`
  loops and always-reverting `require(a.length == N)`.
- **Free-function overloads** — file-scope free-function overloads are
  all injected (dedup by name+arity); previously every overload after the
  first was dropped and calls aborted "has no compiled body".

### Performance

- **Interpreter hot path ~1.9× faster** (compute-heavy neo-test: 1.91s →
  0.99s): `step()` no longer clones the entire eval stack nor allocates
  an opcode-name String per executed instruction (debug-tracing only,
  now gated); per-instruction opcode gas/name lookups use static
  `[_; 256]` tables instead of a `HashMap<u8, _>`; slot getters
  (`LDLOC`/`LDARG`/`LDSFLD`) no longer eagerly format error Strings;
  `GasTracker` no longer allocates a 9-String map per construction.
- `infer_type_from_expression` no longer re-descends the same subscript
  base up to 4× per fall-through arm (exponential in `a[i][j][k]`
  nesting depth).
- Sibling-merge transitive closure memoizes each contract's
  direct-reference walk — O(primaries × edges) full-AST walks reduced to
  one walk per contract (byte-identical output verified on a 24-contract
  unit).

### Validation

- 63-repro fix-validation matrix re-ran every issue identified across the
  bug-hunt backlog, feature audit, and famous-contract evaluation against
  HEAD: **54 verified fixed, 0 regressions**; the 8 deferred items fail
  exactly as documented (all root-caused in-tree for follow-up), and the
  known simulator merged-context artifact reproduces as documented.
- Full suite green: **1,964 tests across 55 targets** (integration + lib
  unit + cargo-fuzz/proptest), clippy `-D warnings` + fmt clean.

### Changed

- Compiler / CLI / workspace and devpack (`@neo-devpack-solidity/contracts`)
  version-aligned at **v0.27.0**.

## [v0.26.0] - 2026-07-01

Dev-environment & deep-correctness release. Ships a native **`neo-test`**
Foundry-style Solidity test runner (executing directly on the in-tree
NeoVM) with Foundry cheatcodes, plus a large wave of neo-test-dogfooded
and fuzz-found correctness fixes across arithmetic, `bytesN`, storage,
ABI, low-level calls, events, inheritance and `try/catch`. Compiler /
CLI / workspace and devpack stay version-aligned at **v0.26.0**. Full
suite green: **1952 tests across 54 targets** (integration + lib unit +
cargo-fuzz/proptest), clippy + fmt clean.

### Added

- **`neo-test` — native Foundry-style Solidity test runner.** Executes
  `test*` / `testFail*` / `setUp()` directly on the in-tree NeoVM with
  per-test state isolation, cross-contract `new`, decoded `revert` /
  `Panic` reasons, `console.log`, and gas reporting. Bundles
  `neo-std/{Test,console,Vm}.sol`. Wired into `neo-forge test`; see
  `docs/TESTING.md`.
- **Foundry cheatcodes** (via the HEVM address, no compiler change):
  `vm.prank` / `startPrank` / `stopPrank` / `warp` / `roll` / `deal` /
  `label` / `assume` and `vm.expectRevert()`.
- **Value-rich assertion failures** — `assertEq failed: 3 != 5` style
  messages carrying the actual operands.
- **Expanded structured-Solidity fuzz grammar** — loops, locals,
  mappings, arrays and devpack-framework calls (`Runtime.*` / `Storage.*`)
  in the `structured_sol` generator.

### Fixed

- **Arithmetic** — unchecked `-type(intN).min` / `0 - min` now WRAP
  two's-complement instead of yielding the out-of-range `+2^(N-1)`
  (which also faulted NeoVM's 256-bit integer) [bug-hunt #2/#25/#31];
  `addmod(a,b,0)` / `mulmod(a,b,0)` Panic(0x12) instead of returning 0;
  `addmod` full-width carry; `int256 **` / `uint256 **` use soft-arith
  magnitude (catchable Panic / wrap, no 33-byte fault); `uint128` mul
  overflow; `mapping(K => uintN)` value width (was silently dropping the
  checked-overflow Panic and the mod-2^N wrap).
- **`bytesN` / byte semantics** — integer-backed `bytesN` literals are
  canonicalized to big-endian `ByteString` at EVERY binding site
  (scalar / struct field / array push / mapping key / call argument /
  return / `constant` / multi-return tuple / indexed event topic);
  `bytesN` bitwise `& | ^` read back through `uint256(...)` correctly
  [bug-hunt #14]; byte index `b[i]` is a 1-byte `bytes1`; `intN(bytesN)`
  endianness; `bytesN` constant comparison (fixes a byte-reversed
  access-control check).
- **Storage** — a storage `struct` passed by reference to an internal
  function now ALIASES the slot (field writes persist) [bug-hunt #8];
  `T[] memory m = storageArr` deep-copies [bug-hunt #10]; storage
  `bytes` element write `data[i] = v` persists [bug-hunt #11/#13];
  storage `bytes` push/pop.
- **ABI / calls** — a plain internal call to a public array/`bytes`/
  `string`-returning function decodes the ABI blob instead of binding
  the raw bytes [bug-hunt #9]; low-level `.call` returndata is the raw
  EVM ABI envelope (no Neo serialize framing) and a no-reason revert
  yields empty returndata; `(bool ok, ) = addr.call(...)` reports
  `ok = false` for a reverting call [bug-hunt #21/#28/#29]; `abi.decode`
  type fidelity (EQUAL is type-strict); `abi.encode` / `abi.encodePacked`
  return a proper `ByteString`; partial tuple destructure from a
  cross-contract call selects the right slot.
- **Events / inheritance / merge** — a contract's own event declarations
  survive when another contract references it by type [bug-hunt #23];
  a `new`-deployed contract's public method that calls its own concrete
  internal helper now reaches a real body (was dropped → `PUSH0` → the
  write vanished) [bug-hunt #18/#20]; cross-contract `msg.sender` for
  `new`-deployed callees.
- **`try/catch` & control flow** — bound `TRY` nesting depth (fixes a VM
  DoS) and make `ExecutionEngineLimits` faults uncatchable; `catch Panic`
  / `catch Error` match real-node call faults (Buffer vs ByteString);
  the NeoVM `TRY` frame stays balanced across every catch exit; modifier
  / base-constructor arguments are evaluated exactly once.
- **Type resolution** — overload resolution lets a positive integer
  literal match any integer parameter; `address.balance` / `.code` /
  `.codehash` inference; NeoVM `EQUAL` type-strictness (ternary `bytesN`
  compare, `bytesN` mapping-key slot).
- **devpack** — authorize `multiSigMint`, settle stake rewards, unblock
  NEP-11 oracle / curation and minter-royalty paths.

### Changed

- Compiler / CLI / workspace and devpack (`@neo-devpack-solidity/contracts`)
  version-aligned at **v0.26.0**.

## [v0.25.0] - 2026-06-28

Single-source-of-truth release: a new `pub mod opcode` (top-level
`crate::opcode::OpCode` `#[repr(u8)]` enum) replaces the previous
200-line match-table inside `runtime::spec::opcodes` as the canonical
Neo N3 opcode definition. Compiler and devpack stay version-aligned at
**v0.25.0**. 1488 tests green (up from 1885 in v0.24.0 — the proptest
harness was split into 998 individual cases), clippy + fmt clean,
**zero behavior change** in the emitted bytecode.

### Added

- **New public module `pub mod opcode` (`crate::opcode::OpCode`)**.
  One variant per Neo N3 opcode — fixed opcodes (`ADD`, `JMP`, `JMP_L`,
  `RET`, `SYSCALL`, `NEWBUFFER`, `CAT`, `SUBSTR`, `EQUAL`, `DUP`,
  `SWAP`, `ISNULL`, `CONVERT`, `ABORTMSG`, …) plus every member of the
  indexed families (`PUSH0..PUSH16`, `LDLOC0..LDLOC6` + `LDLOC`,
  `LDSFLD0..LDSFLD6` + `LDSFLD`, `LDARG0..LDARG6` + `LDARG`,
  `STLOC0..STLOC6` + `STLOC`, `STSFLD0..STSFLD6` + `STSFLD`,
  `STARG0..STARG6` + `STARG`).
- **`OpCode::byte()`** — the raw byte (single inline `as u8` thanks
  to `#[repr(u8)]`).
- **`OpCode::name()`** — canonical spec string (`"JMP_L"`,
  `"PUSH0"`, …) for disassembler / debugging.
- **`OpCode::gas()`** — default gas cost.
- **`OpCode::push_small(n)` / `ldloc(n)` / `stloc(n)` / `ldarg(n)` /
  `starg(n)` / `ldsfld(n)` / `stsfld(n)`** — `const fn` constructors
  for the indexed opcode families.
- **`OpCode::push_data(len)` / `push_int(value)`** — `const fn` size
  selectors for the `PUSHDATA1/2/4` and `PUSHINT8/16/32/64`
  families.
- **`TryFrom<u8> for OpCode`** — full reverse map; returns `Err(())`
  for the 23 unassigned bytes in the spec (0x06, 0x07, 0x42, 0x44,
  0x47, 0x4C, 0x4F, 0x8A, 0xA7, 0xAD, 0xB2, 0xBC, 0xBD, 0xD5..=0xD7,
  0xDA, 0xDC, 0xDD, 0xDE, 0xDF, 0xE2..=0xFF).
- **6 new unit tests** in `src/opcode.rs`: round-trip through
  `TryFrom`, unassigned-byte rejection, indexed-constructor byte
  layout, `push_data` / `push_int` selection, and `name()`
  stability.

### Changed

- **411 hardcoded `0xXX` byte literals replaced** across 32 files with
  `OpCode::XXX.byte()` (or `OpCode::XXX`) references. The previous
  `runtime::spec::opcodes::OPCODES` `HashMap<u8, OpcodeSpec>` is now
  derived from the enum at runtime — the spec module dropped from
  248 lines to 25.
- **`src/runtime/spec/opcodes.rs`**: 248 → 25 lines. The remaining
  file is a 12-line `Lazy<HashMap>` derived by walking 0x00..=0xFF
  and calling `OpCode::try_from` on each byte; the legacy 200-line
  `op!()` macro match table is gone.
- **All 19 files under `src/runtime/execution/instruction/`** now
  use `const` aliases like `const PUSHINT256: u8 = OpCode::PUSHINT256.byte();`
  inside their `match opcode { ... }` dispatch functions, with range
  patterns kept as `u8` ranges (e.g. `PUSH0..=PUSH16`,
  `LDSFLD0..=LDSFLD6`). The match scrutinee is still `opcode: u8` —
  no `try_from` conversion in the hot path, zero performance cost.
- **All 8 files under `src/cli/bytecode/bytecode_builtins/builtin_call/`**
  plus `src/cli/bytecode/{bytecode_emit_ir,bytecode_core,bytecode_builtins/{data,events,syscalls}}.rs`
  and 8 helper files now reference `OpCode::XXX.byte()` at every
  emission site.
- **`src/neo/contract_hash.rs`**: the PUSHDATA1/2/4, PUSHDATA-length,
  PUSH0/PUSHn, PUSHINT8/16/32/64 emitters all route through
  `OpCode::push_data(len)`, `OpCode::push_small(n)`, and the
  explicit `PUSHINT8..64` variants.
- **`src/runtime/execution/syscalls/contract.rs`**:
  `append_pushdata` / `append_push_int` use the new constructors.
- **`src/runtime/tests.rs`** + **`src/cli/bytecode/tests/helpers.rs`**:
  test fixtures and expected-byte assertions reference the
  named constants; the `emit_binary_op` test now iterates over
  `(operator, OpCode::XXX)` pairs.

### What is **not** an opcode (and stays as a raw byte)

The 7 remaining `0xXX` literal sites are explicit non-opcodes with
code comments:

- `0xFD` / `0xFE` / `0xFF` in `src/neo/encoding.rs`,
  `src/neo/build.rs`, `src/neo/tests.rs::read_varint` — Bitcoin
  CompactSize varint length markers in the NEF serialization.
- `0x0A` / `0x09` / `0x0D` / `0x22` / `0x27` / `0x5C` / `0x00` in
  `src/ir/build/selectors.rs::unescape_solidity_string` —
  C-string escape characters (`\n \t \r \" \' \\ \0`).
- `0x00` / `0x01` / `0x02` / `0x03` / `0x40` / `0x80` in
  `src/runtime/execution/execution_impl_part2_native/stdlib.rs` —
  StackItem type tags in the Neo N3 BinarySerializer wire format.
- `0x21` / `0x28` in `src/cli/bytecode/bytecode_builtins/builtin_call/crypto.rs`
  and `src/cli/bytecode/bytecode_helpers/storage/state.rs` — same
  StackItem type tags used as `CONVERT` / `ISTYPE` operands.
- `0x11` in `src/runtime/execution/helpers/arithmetic/basic_ops.rs`
  — Solidity Panic code value (`Panic(uint256) 0x11`).
- `0x14` in `src/cli/bytecode/bytecode_helpers/array_runtime.rs` —
  PUSHDATA1 length operand (20 bytes, for the ContractManagement
  native-hash comparison).
- `0xFF` / `0xff` — two's-complement sign-extension fill bytes
  (`PUSHINT128/256`) and the ContractManagement native-hash data
  in `array_runtime.rs`.

### Notes for contributors

- **Match arms on `opcode: u8`** in the runtime simulator keep
  using `u8` constants; converting to `OpCode::try_from(opcode)`
  was deliberately not done to avoid the `Result` unwrap in the
  hot dispatch path. Range patterns like `PUSH0..=PUSH16` work
  as `u8` ranges thanks to the `const` aliases at the top of
  each `match` block.
- **`#![allow(non_camel_case_types)]`** is set on `src/opcode.rs`
  so Neo N3's `JMP_L`, `JMPIF_L`, `NEWARRAY_T`, `LDSFLD0`..6,
  `PUSH0`..16 names stay spelled as in the spec rather than being
  mangled to upper-camel-case.
- The enum is `#[repr(u8)]`, `Copy`, and `Hash`-able, so it
  composes naturally with existing `HashMap`/`HashSet`/match
  code without any adapter types.

## [v0.24.0] - 2026-06-27

Hardening & real-node verification release: 5 correctness fixes surfaced by
the new real-Node differential harness and the famous-contracts runtime
smoke, plus a focused dedup/refactor pass. Compiler and devpack stay
version-aligned at **v0.24.0**. 1885 tests green (up from 1844 in v0.23.0),
clippy + fmt clean, differential harness **14/14 PASS** on real
Neo-Express 3.9.1.

### 🔴 Correctness fixes (real-node oracle-discovered)

- **`a ** b` on-chain fault fixed.** The previous overflow check compared
  the loop's accumulated product against a 33-byte `2^256` literal, which
  pushed a 33-byte `ByteArray` and then coerced to `Integer` —
  faulting on-chain with `MaxSize of Integer is exceeded: 33/32`. Real
  NeoVM rejected `2 ** 10` on a fresh deployment. Replaced with
  `(result >> 255) >= 2` (32-byte shift that real NeoVM accepts, plus a
  signed-overflow discriminator: `2^255 >> 255 = 1`, `2^256 >> 255 = 2`).
  Post-truncate result also now `Instruction::Convert { Integer }`-ed so
  the new CONVERT fidelity gate (see below) catches it locally next time.
  Diff harness: `pow_test([2, 10])` was FAULT → HALT(1024).
- **S6 manifest permission gate fully wired (compiler + runtime).**
  Before, the manifest permission derivation only saw IR-level
  `BuiltinCall::NativeCall` markers and missed the codegen paths that
  emit native calls directly to bytecode — most notably the
  `keccak256` / `serialize` storage-key-derivation helpers reached via
  `StoreState(computed_slot)` for fixed-size array elements inside
  structs. Those contracts emitted valid IR but faulted on real nodes
  with `no permission to call ...`. Added
  `collect_bytecode_native_permissions` (scans both `CALLT` 0x37 method
  tokens and `System.Contract.Call` 0x41 syscall sites, extracting
  `(hash_le, method)` by walking the operand stack) and wired it into
  `infer_permissions → build_manifest → compile.rs`. Runtime side got a
  matching `manifest_permits(target_hash_le, method)` check that fires
  before `invoke_native_contract`. Diff harness: `StructFixed` struct
  storage contract was `5 × FAULT` → HALT.
- **S6 CallFlags propagation.** Runtime now declares the full Neo N3
  `CallFlags` bitmask (ReadStates / WriteStates / AllowCall /
  AllowNotify / All). Every mutating syscall gates on its bit
  (`AllowNotify` for `Runtime.Notify` + `Runtime.Log`, `AllowCall` for
  `System.Contract.Call`, etc.) and the caller's flag set is saved per
  `CallFrame` and restored on return + on exception unwind so nested
  calls don't leak permissions across frames. Plus compiler-emitted
  flags are now validated to fit in `0x0F`.
- **M-IR2 logical operators normalize.** `||` and `&&` right operands
  were not always coerced to `Boolean` after short-circuit evaluation,
  producing a `Integer`-typed result when the LHS was already a
  boolean. Added `Instruction::Convert { Boolean }` after each
  right-operand evaluation; +1 structural + behavioral test.
- **M-DEV1 NEP-11 `mint` to self now reverts with `InvalidReceiver`**
  when the contract's own `INEP11Receiver.onNEP11Received` reverts.
  Previously un-deferred `mintToSelf` silently succeeded and the
  contract ended up owning its own token. New test
  (`m_dev1_nep11_mint_to_self_succeeds_and_contract_owns_token`)
  uses a reverting receiver as the failure-mode discriminator.

### 🟢 Safety hardening (silent → loud)

- **`LoweringContext::allocate_local` no longer silently collides**
  on the (impossible) u16 overflow. The previous
  `checked_add(1).unwrap_or(self.local_count)` returned the SAME index
  for two distinct locals when the function hit 65 536 locals, which
  would have them share a slot and corrupt IR state silently. Now
  `.expect("...exceeds u16::MAX (65 536) locals")` panics with an
  actionable message instead. `next_label` got the same treatment for
  its `usize` counter.
- **`CONVERT`-to-Integer errors on `bytes.len() > 32`**, matching real
  NeoVM behavior. Previously the simulator wrapped a >32-byte ByteArray
  as `ByteArray` silently — a divergence that hid
  `[2^255, 2^256-1]`-class lowering bugs. Two new
  `convert_to_integer_*` tests pin the new behavior.
- **`unwrap()` hardening:** `return_lower.rs::wrap_external_single_array_return_value`
  had two `return_types.first().unwrap()` sites after a `len() == 1`
  guard. Replaced with an explicit `first_ret_type = &return_types[0]`
  binding right after the guard.

### 🟡 Real-node test infrastructure (new)

- **Neo-Express differential harness** (audit gap #1 closed). Compiles
  Solidity → runs bytecode in BOTH the in-tree simulator AND a real
  Neo-Express 3.9.1 node → diffs results. 14 probes across 7 pure
  methods (POW, XOR, SHL, nested MOD, complex bitwise, DIV,
  pow_wide, mul_wide). 14/14 PASS. Found 2 of the correctness bugs
  above (`**` and S6 manifest). Gated behind
  `#![cfg(feature = "neoxp-diff")]` + `#[ignore]`; runs in the
  dedicated `neoxp-diff` CI job.
- **Famous-contracts runtime smoke** (`famous_corpus_runtime_smoke`).
  For every vendored .sol in `third_party/famous-contracts/sources/`
  (92 contracts from OpenZeppelin, Uniswap V2/V4, Aave V3, Chainlink,
  Safe, ENS, MakerDAO), compile → deploy to Neo-Express → invoke
  a representative read-only method (`name`, `symbol`, `decimals`,
  `totalSupply`, `owner`, `paused`, `get`, `view`) → write a markdown
  report. First-run: **WETH9** is the only contract that fully passed
  (compile + deploy + `name()` returned "Wrapped Ether"). The other
  6 deploy-pass contracts are abstract / library base classes with
  no zero-arg reads; 2 deploy-faults are constructor-required
  contracts (VRFConsumerBaseV2, SafeProxy — both fault on no-arg
  deploy as expected). The 85 compile-failures are missing transitive
  deps (IERC20.sol, IERC721.sol, …) — leaf-only OZ vendor by design,
  documented in `famous_contracts_compile.rs`. Per-contract results
  written to `third_party/famous-contracts/RUNTIME_REPORT.md`.
- **S7 e2e revert-rollback test** (genuine regression guard — verified
  by removing the storage-snapshot restore).

### 🔵 Architecture & refactor (Round 4, zero behavior change)

- **−25 LOC net** across 19 files; 5 dedup passes collapsed what were
  3-5 near-identical definitions into single sources of truth:
  - `canonical_param_type` (3 duplicates → `crate::utils::canonical_param_type`)
  - `method_name_from_signature` (5 inline implementations → one helper with doctest)
  - `BUILTIN_LIBRARY_BASES` (3 inline `matches!` → the canonical `pub(crate)` const in `ir_context`)
  - `MAX_CLIMB = 16` (2 local consts 36 lines apart → 1 module-level)
  - `MAX_DECIMAL_EXPONENT = 1024` (now also reused by `power.rs` — was `MAX_LITERAL_POW_EXP`)
- **`OutputConfig::nef_source()` method** replaces 3 inline
  `config.nef_source_override.unwrap_or(config.input_file)` fallbacks.
- **Dead code removed**: orphaned `src/ir/expressions/calls/builtins/helpers.rs`
  (no `mod helpers;` declaration, zero callers), 2 unused `check_*`
  validators in `erc_nep_patterns.rs`, dead
  `SolidityError::is_recoverable`.
- **Other minor cleanups**: removed 4 dead npm deps + add lean-build
  CI gate (`e8787df`, `7c733bf`); hardened production `unreachable!`
  /`expect` to recoverable errors (`2473e5f`); B2+C2+P3 fix batch
  (restore `from_contract` for tests, fix `multiSigTransfer` API,
  dedup p256 dep listing — `30d6301`); NEP compliance docs verified
  (`77aec5a`).

### v0.23 audit validation (back-port pass)

The 10 v0.23 audit items claimed fixed in v0.23.0 all remain green:
S7, M-DEV1, M-IR2, M-IR3 (deliberately omitted — documented why),
M-TEST1, M-TEST3, M-INT4, M-INT6, S5, S6. Diff-harness B2+C2
corrections landed in `30d6301`.

---

## [v0.23.0] - 2026-06-24

Deep-refactor & correctness release: a systematic 7-phase codebase review
plus 5 audit follow-up fixes. Compiler and devpack are now version-aligned
at **v0.23.0**. Every change is TDD'd and the full workspace stays green
(49 test suites, 0 failures; clippy + fmt clean).

### 🔴 Correctness fixes

- **`mulmod(a, b, m)` now uses full 512-bit intermediate precision.** The
  previous lowering emitted a native NeoVM `MUL` for `a*b`, truncating to
  256 bits. When `a*b ≥ 2^256`, the result was silently wrong. New
  `emit_u256_mulmod_512bit_ir` computes the full 512-bit product via 8-column
  schoolbook multiplication, then reduces mod m via bit-serial shift-subtract
  (512 iterations). Differential test:
  `mulmod(2^128, 2^128, 2^256-1) == 1`.
- **`CheckSig`/`CheckMultisig` default flipped — synthetic hash removed.**
  The fallback `SHA256(bytecode‖account‖counter)` hash matched no real
  signature, making uninjected CheckSig results meaningless. Without
  `override_signing_hash()`, CheckSig/CheckMultisig now deterministically
  return `false` (reject) — the honest default. Meaningful verification
  requires explicit hash injection.
- **`parse_source` no longer silently drops unrecognized `SourceUnitPart`
  variants** (audit L-FE1). The catch-all `_ => {}` is replaced with an
  explicit enumeration of all 13 current variants + a hard error for future
  parser additions. Prevents silent empty-contract compilation when
  solang-parser adds new grammar constructs.
- **`getGovernanceInfo()` in `CompleteNEP17Token.sol` now correctly
  enumerates proposals** (audit L-DEV). The previous `Storage.find("proposal")`
  iterator never matched the Solidity keccak-keyed mapping slot, so
  `activeProposals`/`executedProposals` were always 0. Fixed via a parallel
  `bytes32[] _proposalIds` index.

### 🟡 Testing improvements

- **Optimizer differential extended to cover storage + events** (audit
  M-TEST3). The O0↔O3 semantic-equivalence proptest previously compared only
  `return_data`. A peephole pass that reorders `PUT` and `Notify` (observable
  on-chain) was invisible. Now compares `state_changes` + `logs` in order.
- **BLS12-381 Gt encoding stability lock** (audit S5). The non-canonical
  `Debug`-format encoding used for differential pairing tests is now pinned
  by a `#[cfg(test)]` guard so a future change can't silently break it.
- **First-ever criterion benchmarks.** Baseline: 460µs (simple contract),
  2.6ms (ERC20Token.sol 442 LOC, O2), 3.1ms (O3).
- **Neo-Express CI gate expanded.** 16 compiler-behavior scripts now run
  per-PR via a 4-way parallel matrix (15-min timeout), up from 5. The 10
  DeFi/showcase scripts stay nightly.

### 🔵 Architecture & refactor (7 phases, zero behavior change)

- **−2391 LOC net** removed across dead code + dormant files.
- **8 dead modules deleted** (security.rs, docs.rs, testing.rs,
  codegen_helpers.rs, validation.rs, warning.rs, types.rs, error.rs) +
  1491-LOC dormant `uint256_ops.rs` — all verified zero external references.
- **5 god-object files split** into 19 focused modules. Largest was 2426 LOC;
  no production file exceeds ~1024 LOC now.
- **Error pipeline restructured.** The lossy `SolidityError → CompileError`
  bridge (catch-all `to_string()` → `GENERIC_ERROR`) replaced with 11
  explicit arms. 25-branch `infer_validation_code` string-matching replaced
  with codes set at construction time.
- **All 186 production `include!` calls converted to proper `mod`
  declarations.** Every module in `src/` now uses Rust's native module system.
  Only 55 test-only `include!`s remain.
- **Public API surface tightened.** 9 internal modules are `#[doc(hidden)]`;
  only `cli` and `neo` are the documented public contract.
- **Runtime simulator feature-gated** behind `#[cfg(feature = "runtime")]`
  (default-on). `cargo build --no-default-features --bin neo-solc` skips the
  17K-LOC simulator and 4 heavy deps (bls12_381, group, p256, dashmap).
- **Parallel per-contract compilation** via `rayon::into_par_iter` (rayon
  was already a dormant dependency; types verified Send+Sync).

### v0.22 audit validation

A full validation pass confirmed all 26 v0.21 audit findings claimed fixed
in v0.22: **19 YES, 7 PARTIAL, 0 NO.** See
`docs/audits/AUDIT_v0.22_validation.md`.

## [v0.22.0] - 2026-06-17

Runtime-fidelity & audit-fix release: a full systemic audit (22 findings,
all fixed) closes the gap between "passes in the embedded simulator" and
"runs correctly on a real Neo N3 node", and the toolchain target is
aligned to Neo N3 **v3.10.0**. Every fix was TDD'd (test first, watch it
fail, implement, watch it pass) and the full workspace stays green
(1881 tests, 0 failures; clippy + fmt clean).

### 🔴 Runtime simulator — 7 critical fidelity fixes

The embedded NeoVM runtime is the oracle every compiler test trusts; these
were the highest-impact divergences from a real node.

- **`StdLib.serialize` now emits the Neo N3 BinarySerializer wire format**
  (type-tagged little-endian), not JSON. The previous `serde_json` output
  round-tripped inside the simulator but was byte-incompatible on-chain —
  storage keys derived from serialized values, length checks, and
  inter-contract interop all silently diverged. `jsonSerialize` stays JSON
  for callers that explicitly want it. New `neo_binary_serialize`/
  `deserialize` helpers with Neo's 7-bit-group big-endian varint.
- **`CheckSig`/`CheckMultisig` verify against an injectable transaction
  signing hash.** The previous synthetic hash (`SHA256(bytecode‖account‖
  counter)`) matched no real signature, so signature verification was
  effectively untestable. New `override_signing_hash()` host API; default
  preserves the synthetic hash for backward compatibility.
- **`CreateMultisigAccount` builds the real verification script** (`PUSHINT
  m / PUSHDATA pk_i / PUSHINT n / SYSCALL CheckMultisig`) and returns
  `RIPEMD160(SHA256(script))`, instead of the wrong `SHA256(m‖pubkeys)
  [..20]`. Shares a `append_pushdata` helper with the (already-correct)
  `CreateStandardAccount`.
- **Storage gas aligned to Neo N3 mainnet: 100_000/byte** (was 100/byte,
  ~1000× too cheap). `RuntimeConfig::default().gas_limit` raised 10M → 1B
  (≈ `MaxTransactionSystemFee`) so realistic contracts don't OOG.
- **CallFlags enforced: `Storage.Put`/`Delete` FAULT without `WriteStates`.**
  `GetCallFlags` returns the active flags instead of a hard-coded `0x0F`.
  New `override_call_flags()` host API lets tests simulate a `staticcall`-
  shaped read-only context — the staticcall-could-write-storage trap is
  now impossible.
- **Storage snapshot/rollback on inner-call revert.** A faulting callee's
  dirty writes are now discarded to the call-boundary snapshot
  (`CallFrame.storage_snapshot` + `dispatch_exception` unwind), matching
  Neo N3. Previously they leaked into the caller's overlay and got
  committed at top-level halt.
- **`StdLib.deserialize` decodes the binary format** (inverse of the new
  `serialize`); `GetNotifications` returns real notifications (was empty);
  `revert`-vs-`fault` is discriminated by the `revert_payload` marker, not
  by substring-matching `"THROW"` in the rendered error.

### 🟡 Compiler / IR / devpack — correctness fixes

- **`mulmod`/`addmod` modulus step routed through the uint256 software
  divmod** — native signed `MOD` gave wrong residues for moduli ≥ 2^255.
- **`receive()` + explicit `onNEP17Payment` coexistence is now a hard
  error (E105)** — the `receive()` body was dead code (Neo only invokes
  `onNEP17Payment`); `fallback()` stays a loud W105 warning.
- **NEP-11 `_transfer`/`_mint` self-escrow short-circuit** (`to !=
  address(this)`) — matches NEP-17; unblocks NFT custody flows.
- **Bytecode emission hardened**: 3 unchecked slice writes → `Result`;
  unresolved call target → hard error (was `eprintln!` + zero-byte operand
  = on-chain infinite loop); `u16` token-index overflow + unregistered
  CALLT patches → hard errors.
- **Removed the unsound `x == true → x` optimizer rewrite** (wrong for
  non-boolean operands: `5 == true` must be `false`, not `5`).
- **`MethodToken::serialize` returns `Result`** instead of `assert!`-
  panicking on oversize method names.
- **onNEP17Payment detection is case-insensitive**; **ETHER_UNIT_RE
  requires a digit/`)` prefix** (no more false-positive on `// whether` or
  `uint ether`); **inheritance type-conflict is a hard error (E122)**.
- **NEP-24 `royaltyInfo` tokenId** `bytes32` → `bytes` (matches NEP-11/26).

### 🔧 Toolchain & CI

- **Target Neo N3 v3.10.0** (Gorgon-hardfork preparation; v3.10.0 activates
  NO hardfork, so opcode/syscall/gas/NEF stay consensus-compatible).
  `neo_version` 3.5.0 → 3.10.0; Neo / Neo.SmartContract.Framework NuGet
  3.7.4 → 3.10.0 (49 csproj files); TFM net8.0 → net10.0; CI dotnet
  8.0.x → 10.0.x. Neo.Express kept at 3.9.1 (latest; no 3.10.x exists).
- **Neo-Express on-chain smoke is now a CI gate.** `ci.yml` runs a 6-script
  subset per PR; `fuzz.yml` runs the full 28-script suite + 6 DeFi
  showcases nightly. This is the only oracle that catches "passes in
  simulator, fails on-chain" regressions.

## [v0.21.0] - 2026-06-16

Deep correctness-and-conformance release: software 256-bit arithmetic
lands end to end so `uint256` values `>= 2^255` behave like a real Neo
node, plus two full adversarial review passes — a 25-defect production
audit (24 fixed) and a 12-subsystem systemic best-practice review (33
findings: 32 fixed, 1 surfaced as a loud compile-time warning). Net
effect is EVM/solc and Neo N3 conformance across integer arithmetic, the
ABI codec (now including nested-dynamic and dynamic-struct encode/decode),
runtime VM fidelity (`MaxStackSize`/`MaxItemSize`/storage limits),
dispatch correctness (overload resolution, fail-loud member calls,
function-pointer-local CALLA), selector/manifest canonicalization, tooling
output, and devpack fund-safety (NEP-17 self-escrow, escrow-stranding).

### Added

- **Software 256-bit limb arithmetic for `uint256`.** Schoolbook routines
  implement add/sub (two 128-bit limbs with carry/borrow folded through the
  limb boundary) and multiply (four 64-bit limbs), each computing
  `mod 2^256` so no intermediate exceeds NeoVM's 32-byte integer limit — a
  native ADD/MUL of large `uint256` values would form a rejected 33-byte
  result. Checked variants detect unsigned overflow (carry out of bit 256),
  underflow (final borrow / signed high limb `< 0`), and product `>= 2^256`
  without forming a 33-byte intermediate, raising a Solidity Panic(0x11) —
  replacing the `GetSize > 32` heuristic that a two's-complement wrap
  defeats.
- **Unsigned 256-bit comparison, division, modulo, and shifts for
  `uint256`.** The comparison family (`<`, `<=`, `>`, `>=`) is computed on
  the 32-byte two's-complement form via the order-preserving XOR-`2^255`
  map using only `<= 32`-byte operations, so values `>= 2^255` compare
  correctly (`type(uint256).max < 5 == false`). Division/modulo use the
  Hacker's Delight reduction to one signed DIV/MOD on the
  provably-non-negative `(a>>1, b)` with a correction step (or a 0/1
  quotient by unsigned compare when `b >= 2^255`); the limb-unsafe steps
  call the validated add/sub helpers, and `b == 0` throws Panic(0x12).
  Logical right shift uses `((a>>1) & (2^255-1)) >> (n-1)` (native NeoVM
  SHR is arithmetic/sign-extending), and wrapping left shift masks each
  128-bit limb to its low `128-k` bits before shifting and recombines as
  two's-complement (native NeoVM SHL does not wrap, so `1 << 255` would
  form a rejected 33-byte integer).
- **Nested-dynamic ABI encode/decode** — `string[]`, `bytes[]`, `T[][]`
  (arrays whose elements are themselves dynamic) now use the full recursive
  EVM head/tail layout (`[length, n offsets, n element tails]`, each
  element encoded/decoded recursively) on both the encode and decode sides,
  instead of falling back to `StdLib.serialize`, which emitted Neo-native
  bytes no EVM decoder could read. Cross-contract calldata and return-data
  for these types is now byte-exact with EVM/solc. Locals draw from a
  depth-indexed reusable scratch pool so many encode sites in one function
  stay within NeoVM's 255-slot limit.
- **Dynamic-struct ABI encode/decode** — structs containing any dynamic
  field are now encoded as the EVM tuple of their fields with the proper
  head/tail layout (static fields inline, dynamic fields as offset words
  plus recursive tails) and decoded symmetrically, replacing the
  `StdLib.serialize` fallback. `abi_value_type_is_dynamic` now follows
  solc's tuple rule (a struct is dynamic iff any field is dynamic); all-static
  structs are unchanged. Recursion covers nested shapes such as struct arrays
  (`D[]`), a dynamic struct in a tuple position, and a struct whose dynamic
  field is itself a `string[]`/struct.
- **Global NeoVM `MaxStackSize` ref-count enforcement** — a
  `live_stack_item_count` traversal walks every root (eval stack, indexed
  slots, saved call frames), recursing into arrays/maps and de-duplicating
  shared array/map/bytestring objects by pointer identity. Wired into
  `NEWARRAY`, `APPEND`, `PACK`, and `PACKMAP`, it faults when total live
  items exceed 2048, catching the case where many separate collections —
  each individually under the limit — collectively blow the global limit
  and revert on a real node (the per-collection guards alone could not see
  this).
- **Same-arity overload dispatch by argument type.** A call to one of two
  overloads sharing a `(name, arity)` key (Solidity overloading by parameter
  *type*, e.g. `f(uint256)` vs `f(address)`) is now resolved by matching the
  call's inferred argument types against each overload's parameter types
  (integers by signedness, everything else exactly) instead of collapsing to
  whichever overload was inserted last. Previously one overload silently
  executed the other's body on-chain; ambiguous/unmatched cases now fail
  loud rather than mis-dispatching.
- **Function-pointer local dispatch via CALLA.** Calls to a
  `function(...) internal` *local* (not just a parameter) now register a
  function-pointer binding at declaration and emit `CallIndirect`/`CALLA`.
  Previously such a call fell through to a fallback that dropped its
  arguments and pushed `0`; a never-assigned (zero-init) fp local now
  reverts cleanly on call (CALLA on a Null slot faults) matching Solidity's
  revert-on-zero-init-function-pointer behavior.
- **Narrow-integer checked arithmetic** — `uintN`/`intN` (N<256) add/sub/mul/`<<`/`**`,
  compound assignment, and `++`/`--` now revert on overflow and wrap mod-`2^N`
  inside `unchecked` blocks via a shared overflow ladder, even when one
  operand is an untyped literal (which previously defaulted to `uint256` and
  routed past the narrow guard). Mixed-width operations (e.g. `uint256 + uint32`)
  stay 256-bit; narrow signed division now panics on `intN.min / -1`. Matches
  solc 0.8 semantics.
- **`CryptoLib.verifyWithECDsa`** — implemented real signature verification
  for secp256r1 (p256) and secp256k1 with SHA256/Keccak256 message hashing
  selected by the NamedCurveHash byte (22/23/122/123), replacing the
  always-false stub so the test oracle matches a real Neo node; secp256k1
  accepts both high- and low-S forms.
- **uint256 literal representation warning** — the compiler now warns when
  an integer literal (e.g. `type(uint256).max`, all-ones masks) needs more
  than NeoVM's 32-byte signed-integer limit, surfacing the on-chain fault
  risk at compile time instead of failing silently after deploy.
- **`CALLT` disassembly** — the disassembler now decodes the `CALLT`
  (`0x37`) 2-byte token operand, keeping all following instructions
  correctly aligned.
- **Owner-managed multisig signer set** (devpack `NEP17`) —
  `setMultisigSigner`/`setMultisigThreshold` (onlyOwner) so the contract's
  escrowed pool can only be moved by an explicitly authorized signer quorum,
  with the threshold required to be `>= 2`.
- **Native-signature oracle callback** (devpack) — `CompleteNEP17Token`
  and `CompleteNEP11NFT` register and expose
  `onNativeOracleResponse(string url, bytes userData, int code, bytes result)`
  for direct Oracle-native invocation, while retaining the
  `IOracleServiceReceiver` method for the OracleService-forwarded path.

### Changed

- **`uint256` values in `[2^255, 2^256-1]` are now represented as 32-byte
  two's-complement throughout**, matching how a real Neo node stores them:
  literals (`type(uint256).max`, `1 << 255`, ERC-20 max approval,
  keccak-derived values) emit a two's-complement PUSHINT256 instead of a
  33-byte literal that faults on-chain; ordered comparison and
  checked/unchecked add/sub/mul/div/mod route through the software 256-bit
  routines, gated to genuinely-typed `uint256` so signed `int256` keeps
  native operations; `int256(x)` at 256 bits is a no-op bit reinterpret;
  and `abi.decode` reads the slot directly as two's-complement (dropping the
  old positive-magnitude `0x00` sign-byte append) so
  `abi.decode(abi.encode(x)) == x`.
- **Runtime simulator made faithful to a 32-byte two's-complement NeoVM** —
  negatives serialize to a 32-byte two's-complement word so a `uint256`
  stays a distinguishable 256-bit value, narrow add/sub/mul promote to
  BigInt on i64 overflow, and right-shift sign-extends narrow negatives,
  letting it validate `uint256 >= 2^255` behavior that the old
  unsigned-magnitude model could not.
- **Yul/inline-assembly opcodes now use EVM-unsigned 256-bit semantics** —
  `not` → `x XOR (2^256-1)` (was NeoVM INVERT's arbitrary-precision
  `-x-1`, so `not(0)` gave `-1`), `lt`/`gt` → unsigned 256-bit compare
  (was signed, so `lt(sub(0,1),5)` wrongly returned 1), `shr` → logical
  shift (was arithmetic), and `div`/`mod` → unsigned divmod (were signed),
  with the div-by-zero → 0 guard preserved.
- **Top-level dynamic `abi.decode` honors the encoded head offset** — a
  single top-level dynamic decode now dereferences head slot 0 to obtain the
  tail offset at runtime instead of assuming the canonical `0x20`, so valid
  but non-canonically-placed tails decode correctly, matching EVM decoders
  that follow the encoded pointer. The superseded constant-offset tail
  decoders were removed in favor of the runtime-offset path.
- **Canonical struct/enum ABI types in standard-JSON output** —
  `evm.methodIdentifiers` and `neo.methodMap` now build signature keys with
  the same tuple-expanded canonicalizer used to compute the selector (e.g.
  `f(MyStruct)` keys under `f((uint256,bool))`), so an SDK recomputing
  `keccak256(key)[..4]` matches the emitted selector. The top-level `abi`
  array now emits struct params as `tuple`/`tuple[]` with a recursively-built
  `components` array (enums as `uint8`) instead of an undecodable
  `"type":"MyStruct"`, so ethers/web3/Foundry can decode them. The devpack
  `type Any is bytes` placeholder now canonicalizes to its underlying
  `bytes` in selector and event-topic hashes (manifest still displays
  `Any`), correcting NEP-17/NEP-11 transfer/onNEPxxPayment selectors.
- **ABI-canonical selectors and event topics** — function selectors, the
  `.selector` registry, and event `topic0` are now computed from
  EVM-ABI-canonical types (struct → tuple, enum → `uint8`,
  `uint` → `uint256`), unified across the manifest and event paths to match
  Ethereum tooling.
- **Indexed array/struct event topics hash `abi.encode(value)`** — an
  `indexed` dynamic-array or dynamic-struct event parameter now hashes
  `keccak256(abi.encode(value))` through the conformant ABI encoder,
  matching ethers' `keccak256(abiCoder.encode([type],[value]))`. Previously
  it hashed the raw NeoVM Array stack item (a serde_json blob), which
  matched no off-chain decoder and faulted on a real node since
  `CryptoLib.keccak256` requires a ByteString; `string`/`bytes` still hash
  their raw bytes per the Solidity rule.
- **Return-tuple arity mismatches promoted to hard errors** — unambiguous,
  explicitly-written tuple/return-count mismatches (a single declared return
  given `return (a, b)`, or a multi-return given an explicit tuple of the
  wrong length) are now compile errors instead of warnings, so invalid
  Solidity no longer produces a NEF whose declared ABI return contradicts
  the body. Cases with false-positive risk (`return;` with named returns,
  public-getter synthesis, heuristic call-arity inference) deliberately stay
  warnings.
- **`>255` local-slot functions rejected** — a function exceeding NeoVM's
  255 local-slot limit now returns a clear compile error instead of silently
  truncating the `INITSLOT` count and `LDLOC`/`STLOC` indices into the
  `0..=255` byte range, which previously miscompiled slot indices.
- **Internal function pointers lowered via `PUSHA` + `CALLA`** — internal
  function pointers now emit a relative `PUSHA` Pointer consumed by `CALLA`
  instead of a raw `PUSHINT32`, so they execute on a real Neo node.
- **Per-collection NeoVM `MaxStackSize` (2048) guard** — `NEWARRAY` and
  `APPEND` now fault when a single array exceeds 2048 elements, modeling
  `ExecutionEngineLimits.MaxStackSize` (which counts every contained item).
  Previously the simulator allowed unbounded arrays, hiding an on-chain
  `MaxStackSize exceeded` fault and reporting success where a real node
  reverts.
- **NeoVM `MaxItemSize` (65535) enforcement on `NEWBUFFER`/`CAT`** — both
  now fault when the requested or concatenated length exceeds
  `ExecutionEngineLimits.MaxItemSize` instead of succeeding on a buffer a
  real node rejects.
- **Neo N3 storage size limits on `Storage.Put`** — keys past
  `MaxStorageKeySize` (64) and values past `MaxStorageValueSize` (65535)
  now fault; previously an oversized dynamic value (e.g. a `>64 KB` string)
  silently succeeded in the simulator while reverting on-chain.
- **Conformant minimal `CONVERT` Integer→ByteString/Buffer encoding** — the
  `0x28`/`0x30` CONVERT handler now emits the minimal two's-complement
  little-endian encoding of an integer (zero yields an empty span) instead
  of a fixed 8-byte word, so a contract that converts an integer to bytes
  and inspects its length, hashes it, or returns it observes the same widths
  as a real node. Scoped to the Solidity-observable CONVERT path; the
  internal storage/map-key byte helper is unchanged. `CONVERT` of an
  Array/Map to Integer now faults as real NeoVM does, instead of silently
  producing a degenerate integer.
- **`MODPOW` exponent `-1` computes the modular multiplicative inverse** —
  exponent `== -1` now returns `base.ModInverse(modulus)` in `[0, |m|)` via
  the extended Euclidean algorithm (faulting only when no inverse exists),
  matching NeoVM, instead of rejecting all negative exponents. `MODMUL`/`MODPOW`
  now use C#-style truncated (`%`) remainders whose sign follows the operand,
  matching NeoVM's `BigInteger.ModPow`/`%` for negative inputs rather than
  the Euclidean non-negative form.
- **`PACKMAP` pop order corrected** — now pops the key first (top of stack)
  then the value, matching NeoVM's `key = Pop(); value = Pop()`; the order
  was previously inverted, building maps with keys and values swapped.
- **`msg.value` lowers to a conformant `0` (PUSH0)** instead of a fabricated
  `System.Runtime.GetMsgValue` syscall. Neo N3 has no EVM-style attached
  call value and no such interop, so the prior lowering FAULTed on a real
  node (unknown interop service). Received amounts on Neo arrive as the
  `amount` argument of `onNEP17Payment`/`onNEP11Payment`, not an ambient
  `msg.value`.
- **Auto-generated struct getters omit mapping/array members** — public
  getters for struct state variables now drop mapping and array fields
  (`bytes`/`string` retained), matching solc and avoiding an invalid
  ABI/manifest that referenced non-returnable members.
- **NEP-11 explicit-standards validation tightened** — declaring
  `supportedstandards: ["NEP-11"]` now requires `totalSupply` and
  `tokensOf` (matching the auto-detection set), so a manifest can no longer
  advertise NEP-11 while missing spec-mandatory methods wallets/indexers
  depend on.
- **`abi.decode` accepts over-length buffers** — the static-size guard now
  reverts only on under-length input and ignores trailing bytes, matching
  solc/ethers/foundry decoding.
- **`bytesN` width validation** — `bytes0` and `bytes33`+ are now rejected
  at parse time instead of being silently accepted and mis-encoded
  downstream (only `bytes1`..`bytes32` are valid).
- **Yul `div`/`mod` by zero yields 0** — inline-assembly `div`/`mod` now
  return 0 on a zero divisor per EVM/Yul semantics; high-level Solidity
  division/modulo still Panics `0x12`.
- **Standard-JSON parse errors are now one structured entry per diagnostic**
  — a parse failure previously collapsed every parser diagnostic into a
  single opaque `type:"Generic"` error with no byte offsets; it now emits
  one `type:"ParseError"` object per diagnostic, each with a
  `sourceLocation:{file,start,end}` byte range, so Hardhat/Foundry-style
  tooling can enumerate individual errors and position source markers as it
  expects.
- **Aliased imports now emit a loud `IMPORT_ALIAS_BY_NAME` warning** —
  symbol-rename (`import {A as B}`) and global-symbol (`import * as NS`)
  aliases resolve by the underlying symbol name rather than the alias
  binding, so a name collision in the import closure could bind to the wrong
  declaration; every aliased import now warns instead of silently performing
  a bare-name bind (resolution and codegen unchanged).
- **Zero-amount transfers now conform to NEP-17** (devpack) — the canonical
  4-arg `transfer` no longer reverts when `amount == 0`; a zero-value
  transfer is processed normally (emits `Transfer`, runs the receiver
  callback, returns true), matching the NEP-17 spec and a real Neo node. The
  `validAmount` guard remains on mint/burn.
- **Self-escrow skips the receiver callback** (devpack) — `_transfer` no
  longer invokes `onNEP17Payment` when `to == address(this)`. Escrowing to
  the contract itself (timelock, conditional, staking, scheduled transfers)
  is internal bookkeeping; the base contract does not implement the
  receiver, so the prior self-call faulted and reverted the entire
  escrow-in leg. This also avoids spurious self-reentrancy.
- **NEP-26 `onNEP11Payment` tokenId is now `bytes`** (devpack) —
  `INEP26Receiver` takes a dynamic `bytes tokenId` instead of `bytes32`,
  matching `INEP11Receiver` and the ByteString token IDs the NEP-11 base
  actually passes; a non-32-byte id no longer mis-encodes.

### Fixed

- **`uint256` `**` (exponentiation) overflow and wrap** — checked `uint256`
  `**` now Panics(0x11) when the true result exceeds `2^256-1` and unchecked
  `**` wraps mod `2^256`; previously only narrow widths were handled, so
  `uint256` `**` overflow faulted with the wrong reason and unchecked never
  wrapped.
- **Unchecked `uint256` compound assignment (`+=`, `-=`, `*=`, `++`)** —
  routed through the software limb routines so operands `>= 2^255` wrap mod
  `2^256` instead of faulting on a 33-byte native intermediate; the old
  workaround left a Buffer that broke index reuse (e.g.
  `for(...; i++) a[i]`).
- **Width-correct bitwise NOT (`~`)** — `~x` now complements within the
  operand width: `~uint8(0)` returns 255 (not `-1`) via re-truncation after
  INVERT, and `~uint256(x)` uses `x XOR (2^256-1)` instead of truncating to
  u64; `int256` keeps the full-width `-x-1`.
- **`uint256` right-shift is now a logical shift** — a `uint256` at or above
  `2^255` (stored 32-byte two's-complement) no longer sign-extends:
  `type(uint256).max >> 1` gives `2^255-1` instead of `2^256-1` — while
  `int256` retains the arithmetic shift.
- **Narrow unchecked post-increment/decrement old-value recovery** —
  `old = new -/+ 1` is re-truncated to the operand width, so a `uint8` wrap
  recovers the width-bounded old value (255) instead of `-1`.
- **Shift-count encoding for counts `>= 128`** — emitted via PUSHINT16
  instead of the signed PUSHINT8 (where `0x80` decoded to `-128` and faulted
  the shift).
- **Post-increment/decrement evaluates the lvalue exactly once** — `x++`/`x--`
  on an indexed or keyed lvalue (`m[f()]++`, `arr[g()]++`) previously lowered
  the lvalue twice, running a side-effecting index expression twice (wrong
  slot plus duplicate effects). The compound now runs once (single index
  evaluation, leaving the new value on the stack) and the old value is
  recovered as `new -/+ 1`, using the `uint256` limb routine so a wrapped
  `>= 2^255` result does not fault on a 33-byte intermediate.
- **`ErrorName.selector` hashes the parametrized signature** — a custom
  error with parameters now computes `keccak("Name(t1,t2,...)")[..4]`
  instead of the bare `Name()` form, so `ErrorName.selector` matches solc
  and the 4-byte selector the error's own `revert` emits.
- **Sign-extension of negative signed integers in static multi-value
  returns** — the static-ABI return fast path zero-filled the 32-byte slot
  before copying an integer's minimal two's-complement bytes, so a negative
  signed value zero-extended its high bytes (e.g. `int256(-1)` became
  `0x00..00FF` = 255) rather than the EVM-canonical `0xFF..FF`. Signed
  integers are now masked to 256 bits before the slot encode, producing the
  correct sign-extended two's-complement; applied to both the
  `return_revert.rs` and `function.rs` static-slot encoders, with
  unsigned/positive paths unchanged.
- **Array-of-struct expansion in the fallback selector canonicalizer** —
  `canonical_param_type_with_structs` (used when NeoType resolution fails)
  only expanded a struct when the whole token was the struct name, so
  `P[]`/`P[3]` passed through verbatim and produced a selector disagreeing
  with `keccak256("(uint256,bool)[]")`. It now peels the trailing array
  suffix, canonicalizes the element type recursively, and re-appends it.
- **Same-ABI-shape struct-overload selector collisions detected** — two
  overloads taking different structs with the same ABI tuple shape
  (`struct A{uint256 x}` vs `struct B{uint256 y}`) now error, since both
  keccak-hash to the identical `f((uint256))` 4-byte selector and cannot be
  dispatched apart on-chain. The duplicate-signature check now canonicalizes
  each struct parameter to its tuple shape (from `metadata.structs`) instead
  of its bare name; distinct-shape overloads still compile.
- **Rejection of non-canonical length/offset slots in `abi.decode`** — the
  dynamic-decode slot readers consumed only the low 8 bytes of each 32-byte
  length/offset word, so a crafted payload encoding a value in
  `[2^64, 2^192)` with small low bits was silently truncated, causing the
  decoder to read the wrong region instead of reverting. A high-bits guard
  now faults (Panic `0x41`) whenever any of the high 24 bytes is set, on
  both the compile-time-offset and runtime-offset readers; conformant slots
  always have zero high bytes, so valid input is never faulted.
- **Unresolved member calls fail loud** — a member call
  `inner.member(args)` matching no resolution branch (builtin, library,
  using-directive, iterator op, push/pop) now emits a hard diagnostic
  instead of dropping the receiver and arguments, pushing `0`, and reporting
  success. A typo'd method, missing library function, or member access on an
  unsupporting type no longer silently compiles into a function that returns
  0 and discards side-effecting arguments, matching solc's "member not
  found" rejection.
- **Fixed-size arrays classified as Array in the manifest** — a fixed-size
  array type such as `uint256[3]` (ends with `]` but not `[]`) is now
  correctly classified as an Array rather than falling through to the
  uint-prefix branch and being labelled Integer, which previously produced a
  spurious "expected Integer, got Array" error for fixed-array event
  parameters. Manifest type detection now keys on any trailing `]`.
- **Mixed unsigned/negative integer comparison via BigInt** — comparing an
  `UnsignedInteger` against a negative `Integer` previously faulted on the
  u64-coercion path; both operands are now promoted to BigInt so `<`/`>`/equality
  yield the value a real node computes (NeoVM integers are arbitrary-precision
  with no signed/unsigned tag).
- **`SUBSTR` integer-overflow guard** — the bounds check now uses
  `checked_add` for `index + count`; crafted operands could overflow `usize`
  and wrap past the guard into a Rust slice-index panic (DoS) instead of a
  clean VM fault.
- **modexp (`0x05`) precompile result padding** — the MODPOW result is now
  left-padded to exactly 32 bytes (prepend 32 zeros, keep the rightmost 32)
  so every result width including zero yields a stable 32-byte big-endian
  slot; the prior prepend-31-zeros approach produced 39-byte output under
  the old fixed-8-byte CONVERT and would have produced 31 bytes for a zero
  result under minimal encoding.
- **0x05 modexp precompile now faults on unsupported operand widths** —
  the 1-byte-operand variant read fixed offsets 96/97/98 and ignored the
  EIP-198 length headers, mis-reading any wider input; it now asserts the
  low byte of each length word (`base_len@31`, `exp_len@63`, `mod_len@95 == 1`)
  and FAULTs on any unsupported shape, while the supported 1-byte path (e.g.
  `3^2 mod 7 = 2`) is unchanged.
- **Disassembler now decodes the 1-byte operand of `CONVERT` (`0xDB`) and
  `ISTYPE` (`0xD9`)** — both fell into the no-operand catch-all, so their
  mandatory StackItemType operand was re-decoded as the next opcode (the
  compiler emits `0xDB 0x21` for nearly every storage-int load), silently
  misaligning every following instruction in the listing; the operand is now
  consumed and rendered as `type=0x..`. Debug-listing surface only; emitted
  bytecode is unaffected.
- **Dropped unsound `x * 0 → PUSH 0` peephole** — the constant-fold no
  longer rewrites `<x>; PUSH 0; MUL` to `<x>; PUSH 0`, which leaked the
  multiplicand on the evaluation stack and could eventually fault on
  MAXSTACKSIZE inside a loop.
- **`PUSHA` offset interpretation** — the runtime now reads the `PUSHA`
  operand as a signed offset relative to the opcode (`ip + operand`),
  mirroring NeoVM's Pointer semantics, instead of treating it as an absolute
  u32 address.
- **Pragma concat-feature gate now respects identifier boundaries** — the
  `string.concat`/`bytes.concat` version checks (`>=0.8.12` / `>=0.8.4`)
  require a non-identifier character before the match, so a user variable
  like `myString.concat(...)` no longer falsely trips the gate and gets the
  compile rejected.
- **Fixed-array length preserved through the type system for correct ABI
  selectors** — `NeoType::Array` now carries the fixed length `N`; the
  parser records it and `canonical_abi_type`/`type_name` emit `T[N]` instead
  of collapsing to `T[]`. A function with a fixed-size array parameter (e.g.
  `uint256[3]`) now computes the correct keccak selector, `interfaceId`, and
  `abi.encodeWithSelector` payload, matching solc/ethers.
- **`bytes20` parses as a distinct fixed-bytes type, not `address`** — it
  now maps to `ByteArray { fixed_len: Some(20) }` → canonical `bytes20`
  rather than `address`. Solidity treats the two as different ABI types, so
  `f(bytes20)` previously got the wrong (`f(address)`) selector and
  `interfaceId`.
- **Deterministic Natspec attachment keyed by declaration start** — each
  accumulated doc block is now keyed at the first non-whitespace byte after
  it (the start of the declaration it documents), and a doc run breaks when
  a non-whitespace token sits between two doc comments. `find_preceding_doc`
  becomes an exact, distance-independent lookup at the declaration start,
  replacing a fixed backward `0..100`-byte scan that dropped doc blocks
  separated by more whitespace and could cross-attach a comment onto the
  wrong (next) declaration. Manifest documentation correctness only; no
  bytecode impact.
- **`NEP17` witness-authorized transfer no longer underflows the allowance**
  (devpack) — allowance is now consumed only on the approval (spender)
  path; an owner/witness-authorized transfer with zero allowance no longer
  computes `allowance - amount` and reverts with Panic `0x11`.
- **`NEP17` constructor enforces the max-supply cap on the initial mint**
  (devpack) — the initial supply is now checked against a non-zero
  `maxSupply` (the internal `_mint` does not enforce the cap), preserving
  the supply invariant.
- **Oracle conditional-transfer callback decoded the wrong arguments**
  (devpack) — `conditionalTransferCallback` declared its parameters in the
  wrong order, so the Oracle native (which calls
  `(string url, bytes userData, int code, bytes result)`) caused `userData`
  (carrying the local request id) to be read from the wrong slot, leaving
  escrowed `conditionalTransfer` funds permanently locked. Corrected to the
  native signature.
- **Conditional-transfer callback could strand escrow on a short oracle
  result** (devpack) — the callback consumed the pending-request record and
  then ran `abi.decode(result, (bool))` on the raw filtered oracle body; a
  body shorter than a 32-byte ABI word hit Panic (`0x41`) and reverted after
  the request was consumed, permanently locking the tokens. Now guarded with
  `result.length >= 32` so unparsable/short results fall through to the
  existing refund branch.
- **Unauthorized accounts could drain the escrowed pool via
  `multiSigTransfer`** (devpack) — the pool transfer now fails closed when
  the multisig set/threshold is unconfigured and requires every declared
  signer to be an owner-authorized signer; previously any caller could
  supply two arbitrary self-witnessed accounts and drain the contract's
  escrow.
- **`multiSigBurn` could burn a third party's balance** (devpack) —
  `CompleteNEP17Token.multiSigBurn` now requires the holder's own witness
  (`checkWitness(from)`), preventing arbitrary colluding signers from
  burning an account they do not control.
- **`Syscalls.scriptHashToAddress` preserves all 20 bytes** (devpack) —
  script-hash-to-address conversion now maps the 20 bytes straight to
  `uint160` instead of left-aligning into `bytes32` and discarding the top
  12 bytes.
- **VaultPattern shares minted against the pre-deposit balance** (devpack)
  — share minting now divides by the vault's assets before the incoming
  deposit (which already lands in the live balance), so depositors are no
  longer diluted.
- **`CompleteNEP11NFT` royalties paid by the buyer** (devpack) — the royalty
  leg now transfers from the witnessed buyer (`msg.sender`) instead of the
  seller, whose witness is never present in a buyer-initiated purchase, so
  the payment no longer always fails; the buyer's payment is split between
  seller proceeds and royalty.

### Internal

- **Functions exceeding NeoVM's 255 local-slot limit now raise a compile
  error** — INITSLOT encodes the local count in one byte and LDLOC/STLOC
  index `0..=255`, so a function with `>255` locals previously truncated
  silently and miscompiled slot indices; `generate_contract_bytecode` now
  returns a clear error instead.

## [v0.20.0] - 2026-06-11

Correctness-and-conformance release: a full adversarial review pass (26
verified findings fixed), ecosystem conformance for tokens (native NEP
Transfer notifications, NEP-11 iterators/ByteString token IDs), devpack
surface honesty (uncallable/unfaithful APIs removed), and large structural
cleanups (dead Yul frontend deleted, `src/runtime` modularized). Everything
below is verified by 1,758 tests plus 23 neo-express on-chain smoke tests.

### Added

- **NEP-11 deep conformance** (devpack `NEP11.sol` + `CompleteNEP11NFT.sol`):
  token IDs are now dynamic `bytes` (NEP-11 ByteString, ≤ 64 bytes,
  manifest `ByteArray`) instead of `bytes32`/`Hash256`, and
  `tokensOf`/`tokens` return NeoVM storage iterators (manifest returntype
  `InteropInterface`, matching the C# devpack) backed by a raw-storage token
  index scanned with `FindOptions.KeysOnly | RemovePrefix`. Public methods
  may now declare `Syscalls.Iterator` returns: the raw iterator stack item
  is the NeoVM return value (no ABI re-encoding) and the manifest type
  mappers emit `InteropInterface`. The bundled runtime's
  `System.Storage.Find` now honours Neo N3 `FindOptions`
  (KeysOnly/RemovePrefix/ValuesOnly/DeserializeValues/PickField0/1/
  Backwards, with C#-node combination validation) instead of always
  yielding `[key, value]` structs. The NEP-11 `transfer`/`onNEP11Payment`
  `data` parameter is now typed with the devpack `Any` alias so the
  manifest declares `Any` per spec — the devpack NEP-11 base now passes
  neo-express's strict NEP-11 deploy-time standard check, validated
  on-chain (deploy + mint + native Transfer notification +
  `InteropInterface` iterator traversal). Remaining documented deviation:
  `properties` returns serialized `bytes` (manifest `ByteArray`) — Solidity
  has no construct that produces a NeoVM Map stack item return
  (STANDARDS_MAPPING.md).
- `ir::builtin_intrinsic_surface()` / `ir::BUILTIN_LIBRARY_BASES`:
  introspection over the builtin-library intrinsic whitelist, plus a
  regression sweep (`tests/gap_hasrole_tests.rs`) that compiles a probe call
  for every whitelisted member and fails if any advertised member has no
  lowering — whitelist-without-lowering intrinsics can no longer reappear.

### Changed

- **Standards auto-detection requires conformance before claiming a NEP.**
  NEP-17 needs the five methods plus a 4-parameter `transfer` and a
  3-parameter `Transfer` event; NEP-11 needs the full mandatory method set
  (`symbol`, `decimals`, `totalSupply`, `balanceOf`, `tokensOf`, `ownerOf`,
  `transfer`) plus a 3-parameter `transfer` and 4-parameter `Transfer`
  event; NEP-24 is only detected for a 3-parameter `royaltyInfo` (`tokenURI`
  no longer triggers it). Near-misses emit warnings instead of false
  manifest claims. Explicitly declared NEP-17/NEP-11 standards additionally
  hard-require the spec `transfer` arity.
- **Distinct-arity overloads keep their original Solidity name in the
  manifest** (Neo dispatches on name + parameter count, like native
  `ContractManagement.deploy`). Only true same-arity collisions fall back to
  the mangled `name(type,...)` form, with one deterministic primary keeping
  the clean name. The standard-json `methodMap` now reports the
  manifest-visible (callable) names.
- **NEP-17 / NEP-11 `Transfer` events are now emitted in NATIVE Neo
  notification shape.** When an event declaration matches a NEP standard
  Transfer signature — `Transfer(address, address, uint256)` (NEP-17) or
  `Transfer(address, address, uint256, bytes32|bytes)` (NEP-11), any
  indexed-ness — the compiler emits `Notify("Transfer", [from, to,
  amount(, tokenId)])` with no EVM topic0: `from`/`to` are 20-byte
  ByteStrings with the zero address mapped to `Null` at runtime (NEP
  mint/burn convention), `amount` is CONVERTed to Integer. The manifest
  declares the native shape (`from: Hash160, to: Hash160, amount: Integer
  (, tokenId)`), so wallets, indexers and NEP trackers can read token
  transfers. All other events keep the EVM log shape. Detection is shared
  between the lowering, the manifest builder and the standards checks
  (`ir::native_transfer_standard`).
- **Manifests now declare the truthful EVM wire shape for non-NEP
  events** — `[topic0: ByteArray, <one ByteArray per indexed param>,
  data: ByteArray]` (anonymous events drop the topic0 slot). Neo nodes
  >= 3.6 (HF_Basilisk) fault notifications whose state-item count
  mismatches the manifest declaration, so the previous declared-parameter
  shape made every `emit` fault on-chain.
- **Anonymous events Notify under their declared Solidity name** instead
  of an empty `""` event name (which faults on Neo nodes >= 3.6 because
  the name is not declared in the manifest). `anonymous` now only
  suppresses the EVM signature-hash topic0 in the state payload; the
  Neo-level event name and the manifest declaration use the declaration
  name.
- **Standards validation understands native Transfer shape**: explicitly
  declared NEP-17/NEP-11 (`@custom:neo.manifest.supportedstandards`) now
  hard-fails when the `Transfer` event's parameter types don't match the
  standard signature (it would be emitted in EVM shape, unreadable by NEP
  trackers); auto-detected standards emit a warning instead.

### Fixed

- **`abi.decode` of multi-slot static types (structs, arrays of structs) now
  decodes canonical ABI bytes** instead of silently falling back to
  `StdLib.deserialize`, which faulted at runtime on real ABI payloads.
- **`abi.decode` of `uint256` values `>= 2^255` no longer reinterprets them
  as negative.** The decoder appends a sign byte when the high bit is set,
  and the bundled runtime's wide bitwise/shift results follow the same
  value model.
- **Custom error selectors are computed from the declared `error`
  signature**, not from inferred argument-expression types. Named arguments
  are reordered into declaration order before encoding.
- **`require(cond, msg)` with a non-literal message now throws the canonical
  `Error(string)` envelope**, matching `revert(msg)` so `catch Error(string)`
  works on both paths.
- **`abi.encodePacked` sign-extends negative signed integers** instead of
  zero-padding them.
- **Mapping keys are canonicalized (`CONVERT`) before serialization**, so
  `Buffer` vs `ByteString` representations of the same value (e.g. a
  `bytes.concat` result vs an ABI parameter) derive the same storage slot on
  real Neo N3.
- **`delete` on storage arrays clears every element.** Fixed-size arrays
  zero all element slots; dynamic arrays clear elements and length. Struct
  field array subscripts now get the same `Panic(0x32)` bounds guard as
  top-level arrays, so deleted data is no longer silently readable.
- **`delete` on a struct containing a mapping member no longer emits a
  `Storage.Put` with a Null value** (which faults on real Neo N3); mapping
  members are skipped per Solidity semantics.
- **Sibling-contract merge rejects same-named state variables with
  conflicting types** instead of silently aliasing one storage slot.
- **The optimizer no longer aborts the process on huge literal shifts or
  multiplies** (`1 << 2**64-1` previously OOM-aborted at the default `-O2`);
  oversized folds now fall through to runtime ops, bounded by
  `MAX_FOLDED_LITERAL_BITS`.
- **Deeply nested expressions no longer overflow the parser stack.** All
  `solang_parser::parse` entry points run on a worker thread with a large
  bounded stack and report failures as ordinary diagnostics.
- **Devpack APIs that could never work on Neo N3 are now hard compile
  errors instead of silent miscompiles or on-chain FAULTs**: the
  `Storage.*Local` family lowered to fictional `System.Storage.Local.*`
  syscalls, and `Storage.batchPut/batchGet/batchDelete/count/exists/
  clearPrefix/findValues/findKeys/getUsage` lowered to single raw syscalls
  that dropped arguments. `devpack/libraries/{Runtime,Storage,Neo}.sol` are
  pruned to the intrinsic surface the compiler actually lowers (111
  documented-but-uncallable functions removed), with a probe test keeping
  the libraries and the intrinsic whitelist in lockstep.
- **Manifest permissions now cover contract calls inside `catch` blocks.**
  Permission inference previously never modeled the `Try → catch` exception
  edge, so a contract call performed in a catch handler shipped without a
  manifest permission and FAULTed on-chain when the error-recovery path ran.
  Any contract-call site left unreached by the dataflow walk now degrades to
  an explicit wildcard permission instead of silent under-permission.
- **Manifest event declarations now describe the actual `System.Runtime.Notify`
  payload** (`topic0` + one `ByteArray` per indexed parameter + `data`)
  instead of the Solidity-declared parameter list. Neo N3 nodes since 3.6
  (HF_Basilisk) validate every notification against the manifest ABI by
  state-item count and type, so the previous declarations made every
  non-anonymous `emit` fault on real networks.
- **`view`/`pure` validation now follows function-pointer calls.** Taking a
  function's address adds a call-graph edge, so a `view` method that writes
  storage through an indirect call is rejected at compile time instead of
  being advertised `"safe": true` in the manifest.
- **The devpack `Any` user-defined value type (`type Any is bytes;`) now
  reaches the manifest as `Any`** (e.g. the NEP-17 `transfer` `data`
  parameter), matching the NEP specs and external standard validators. The
  special case is scoped to aliases whose underlying type is exactly
  `bytes`; `type Any is bytes32;` keeps its alias semantics.
- Deduplicated `getCurrentBlock` in the `Neo` builtin whitelist (it appeared
  twice in the "supported intrinsics" diagnostic).

### Removed

- **Dead Yul frontend** (`src/lexer`, `src/parser`, `src/semantic`,
  `src/optimizer`, AST codegen): these implemented a Yul pipeline that the
  real solang-based compile path never invoked. The live `interop_id_bytes`
  helper moved to `src/interop.rs`. Public modules
  `neo_devpack_solidity::{lexer,parser,semantic,optimizer,codegen}` no
  longer exist.
- `.dead_modules/` (15 parked, never-integrated analysis passes).
- The unused `storage_key::derive_mapping_slot`/`KeyFragment` API, which
  documented a mapping-slot scheme the compiler does not emit; module docs
  now describe the production scheme.
- **`hasRole` pruned from the devpack/intrinsic surface.** Neo N3's native
  RoleManagement contract only exposes `getDesignatedByRole(role, index)` and
  `designateAsRole`; neither real Neo N3 nor the official C# devpack provides
  a generic role-membership check. The dead `"hasRole" => None` arm in
  `resolve_syscalls_member` (an advertised-but-uncallable intrinsic) was
  removed, and `devpack/libraries/Runtime.sol` no longer declares `hasRole`
  — the old helper was uncallable (builtin libraries are compiler intrinsics
  with no lowering for it) and a security footgun: it silently ignored its
  `role` argument and degraded to `checkWitness(account)`. Calls to
  `Syscalls.hasRole` / `Runtime.hasRole` now fail with the standard targeted
  "unsupported builtin library call" diagnostic; use
  `Syscalls.getDesignatedByRole(...)` (and scan the returned node list) or
  `Runtime.checkWitness(...)` instead. User-defined contract-level `hasRole`
  methods (OpenZeppelin AccessControl pattern) are unaffected.
- **Fictional `System.Storage.Local.*` syscalls** (`Get`/`Put`/`Delete`/`Find`)
  removed everywhere — these syscalls never existed on Neo N3. The bundled
  runtime no longer registers or executes them (a script invoking them now
  FAULTs as an unknown syscall, matching real-node behavior); the compiler
  intrinsics (`Syscalls.storage*Local`, `Storage.getLocal`/`putLocal`/
  `removeLocal`/`findLocal*`/`countLocal`) were removed so such calls are
  rejected at compile time; the devpack wrappers and the runtime-spec /
  syscall docs were pruned accordingly.
- **Silent legacy `emit` fallback removed.** Emitting an event with no
  resolved declaration, with a mismatched argument count, or whose
  arguments fail to lower is now a compile error (previously it emitted a
  raw `Notify(name, args...)` whose shape mismatched the manifest and
  faulted on post-Basilisk nodes — or silently emitted nothing).

### Internal

- **`src/runtime` is real Rust modules now**: all 156 `include!()` fragments
  converted to `mod`/`pub use` with explicit visibility; keyword `impl/`
  directories renamed; no behavior change, public API paths preserved.
- The C# `src/Neo.Sol.Runtime` README and root README now state explicitly
  that the C# library is a standalone experimental EVM-emulation layer, not
  used by `neo-solc` and not shipped in releases.

## [v0.19.0] - 2026-05-19

Compatibility-focused release: this is the "any existing Solidity contract
should compile to NeoVM" pass. Three rounds of refactoring across the import
resolver, type system, inheritance/merge pipeline, and IR lowering. Before
this release, only 7 of the 40 OpenZeppelin contracts in the famous-contracts
corpus compiled even with a full npm install — most blocked by a pragma
combinator bug, missing `using`-directive propagation, opaque `address.call`
hard-rejection, and qualified-struct name collisions. After this release,
**40/40 OpenZeppelin contracts compile** and the full famous-contracts
corpus (OZ + Uniswap V2/V4 + Safe + Aave + Chainlink + solmate) reaches
**88/88 unique contracts compiling end-to-end** to valid Neo N3 NEF3 +
manifest. No regressions in the 67 internal examples or 1248 integration
tests.

### Added

- **Foundry-style import remapping** with full Foundry-equivalent semantics:
  - `--remap PREFIX=PATH` CLI flag (repeatable).
  - `--remappings FILE` CLI flag to load Foundry's `remappings.txt` format.
  - Auto-discovery of `remappings.txt` from every ancestor of the entry
    file AND from every package inside reachable `node_modules/`
    directories. Uniswap V4 periphery's `permit2/=lib/permit2/`,
    `forge-std/=lib/forge-std/src/`, etc. now register automatically.
  - Auto-registration of `<pkg>/lib/<dep>/=lib/<dep>/` mappings for any
    package shipping a Foundry-style vendored layout — covers Uniswap V4
    and every forge-installed contract suite without manual configuration.
  - Foundry inline-version-pin syntax (`@openzeppelin/contracts@4.8.3/...`)
    now resolves with three fallbacks in order: dash-form
    (`contracts-4.8.3`), then unpinned (`contracts`), then the original
    pinned spelling. Matches both Foundry's pinning syntax and the npm
    aliased-package convention used by Chainlink.
  - String-prefix substitution (was `Path::join`, which silently broke
    remappings whose suffix begins with `/`).
- **node_modules auto-discovery** from every entry file AND every
  `--include-path`. Walks up to 16 ancestors looking for `node_modules/`
  and adds each discovered directory as an implicit include path. Mirrors
  solc / hardhat / foundry's npm-style import resolution.
- **Cross-package virtual-path resolution** for relative imports. When a
  vendored file at `vendor/@openzeppelin/contracts/token/ERC20/ERC20.sol`
  does `import "./IERC20.sol"` and `IERC20.sol` isn't vendored alongside,
  the resolver now finds the equivalent file under any include path that
  exposes a parallel package layout.
- **Qualified-struct-name tracking** through the IR. A struct defined
  inside `library Pool { struct SwapParams { … } }` now keeps its scope
  qualifier (`Pool.SwapParams`) all the way through library merging and
  IR lowering, so it can coexist with a same-named file-level
  `struct SwapParams` from a different scope (Uniswap V4 PoolOperation
  ships exactly that pair). `from_solidity` does qualified-exact match
  first, falls back to short-name suffix match. Internal references
  inside the owning scope are auto-qualified during frontend conversion.
- **Foundry-compatible regression test**: `tests/famous_contracts_compile.rs`
  enforces two compile-pass floors — a hermetic vendor-only floor (5)
  and a strict OZ-install floor (35, currently hitting 40/40). Picks up
  `NEO_SOL_OZ_INSTALL_DIR` env var or any ancestor `node_modules/` to
  decide whether to run the install-required pass.

### Changed

- **Opaque `address.call(<bytes>)` and `delegatecall` are now runtime
  traps, not compile errors.** Previously the compiler hard-rejected any
  contract whose compiled methods transitively included these patterns,
  which made every contract that imports OZ `Address.sol` (every
  transparent proxy, Multicall, VestingWallet, TimelockController, …)
  fail compilation even when the offending paths were dead code reached
  through inheritance. The compiler now emits a warning at compile time
  and an `ABORTMSG` instruction at the call site — contracts deploy
  normally, and only the specific opaque/delegate-call code path traps if
  execution ever reaches it. Manifest permission analysis is correct
  (no `System.Contract.Call` instruction is emitted, so no spurious
  wildcard permissions are claimed).
- **Multi-file pragma combinator now uses MAX (intersection) instead of
  MIN.** When the compiled source unit contains multiple `pragma solidity`
  directives — typical of import chains — the effective compiler version
  must satisfy every file's constraint. Taking the MIN (the old behaviour)
  incorrectly lowered the effective version below feature gates, causing
  legitimate uses of `string.concat` / `bytes.concat` in the entry
  contract to fail when one transitively imported file declared a broad
  `>=0.4.16` pragma (used by many OZ utility files).
- **Sibling-merge no longer clobbers bodied overrides with bodyless
  declarations.** When a derived contract's inheritance MRO places an
  abstract-interface declaration AFTER a concrete abstract-contract
  override (Chainlink FunctionsCoordinator: OCR2Base implements
  `latestConfigDetails` with body, IFunctionsCoordinator declares it
  bodyless, both are inherited), the flattener now keeps the bodied
  version.
- **Unresolved-overload internal call** (typical of sibling-merged bodies
  referencing abstract functions in the original scope) is now a warning
  + runtime trap rather than a hard error.
- Library `external`→`internal` visibility normalization now runs BEFORE
  validation, so Aave's `EModeLogic.executeSetUserEMode(mapping storage,
  ...)` no longer trips the "external function may not use storage" check.

### Fixed

- **Library `using` directive merge.** When a library is inlined into a
  host contract, its own `using L for T;` directives now merge too. OZ
  Strings.sol declares `using SafeCast for *;` then calls
  `someBool.toUint()` inside helpers; the inlined helper now resolves
  correctly inside the consuming contract.
- **Ancestor `using` directive merge.** Inheritance flattening now pulls
  ancestor `using` directives into the descendant's scope. Repro: Aave
  AToken inheriting from IncentivizedERC20 — the parent's
  `using SafeCast for uint256;` is what makes the inherited `transfer`
  body's `amount.toUint128()` typecheck.
- **Sibling `using` directive merge.** Sibling-merged external bodies
  now carry their owning contract's `using` directives — e.g. Gnosis
  Safe's `using SafeMath for uint256;` is in scope when
  CompatibilityFallbackHandler triggers a sibling merge.
- **Sibling modifier definition merge.** Sibling-merge now pulls in
  modifier definitions from the original contract AND from its base
  classes. Repro: OZ TransparentUpgradeableProxy ↔ ProxyAdmin cycle,
  where ProxyAdmin's `onlyOwner` is inherited from Ownable.
- **Private state-variable shadowing across inheritance.** Solidity
  allows a derived contract to redeclare a private state variable with
  the same name as an ancestor (typical of OZ Governor's
  `string private _name` shadowing EIP712's `ShortString private
  immutable _name`). The flattener now renames the ancestor's slot to
  `__<Ancestor>__<name>` and rewrites identifier references in the
  ancestor's own function bodies. Each scope keeps its own storage.
- **`function_first_param_types` keeps every overload's first-param
  type.** Solidity allows overloading by parameter type; the previous
  single-entry map collapsed `toInt128(int256)` and `toInt128(uint256)`
  (Uniswap V4 SafeCast) so receiver-type matching against
  `using SafeCast for int256;` failed for the wrong-arity surviving
  entry.
- **Type alias resolution in directives.** Library-form `using L for
  Currency;` (with `type Currency is address;`) now resolves the alias
  to its underlying type for receiver matching, so the directive applies
  to address-typed values. Function-list form `using {add as +} for
  BalanceDelta;` deliberately does NOT resolve (operator-overload scope
  is restricted to the alias only — broadening it would over-capture
  every same-underlying receiver).
- **Interface/contract targets in `using L for IInterface`** now resolve
  to `address` for receiver matching. OZ's `using SafeERC20 for IERC20;`
  pattern works again with `IERC20(addr).safeTransfer(...)` call sites.
- **Public state-variable getter deduplication.** Auto-synthesized
  getters no longer collide with existing inherited functions of the
  same name+arity (Aave AToken's `IPool public immutable POOL`).
- **`__ctor__<Sibling>` mangled sibling constructors** can now write to
  immutable state vars (the original constructor's semantics carry over
  to the mangled form). Also handles `__super___ctor__*` preservation
  for inherited base constructors (Chainlink AutomationRegistry).
- **Receive→onNEP17Payment dedup.** When multiple `receive()` declarations
  surface via independent inheritance paths, the synthetic
  `onNEP17Payment(address,uint256,Any)` no longer produces duplicate
  ABI entries.
- **Struct field type-alias resolution.** `type_aliases` now thread
  through nested type resolution (struct fields, array elements, mapping
  values), not just at the top of the parser. Uniswap V4
  `struct State { Slot0 slot0; … }` (with `type Slot0 is bytes32;`) now
  types `state.slot0` as `bytes32` instead of `Any`.
- **Cross-library struct visibility.** Library validation now pre-merges
  peer-library struct/enum pools, so a library function signed against
  `OtherLibrary.NestedStruct` typechecks regardless of which library is
  being validated first.

### Statistics

- **Famous-contracts corpus pass rate**: 7/92 → 88/88 unique
  (4 dup base names in the 92).
- **OpenZeppelin contracts pass rate**: 7/40 → **40/40**.
- **Lib tests**: 511 → 516 (added 5 regression tests).
- **Integration tests**: 1244 → 1248.
- **Internal examples**: 67/71 unchanged (4 intentional Error showcases).

## [v0.18.1] - 2026-05-05

Compatibility and release-readiness follow-up to v0.18.0. This patch release
adds explicit devpack adapters for the most common EVM-only migration gaps,
tightens Hardhat/devpack support metadata, and refreshes validation gates so
Solidity contracts can be compiled, deployed, and exercised on Neo N3 with
clearer guidance.

### Added

- **EVM compatibility adapters** in `devpack/contracts/compat/`:
  - `EVMNativeAssetAdapter` maps payable / `msg.value`-style flows to
    NEP-17 `onNEP17Payment(address,uint256,Any)` callbacks.
  - `EVMFallbackDispatcher` exposes explicit `dispatch(bytes4,bytes)`
    selector routing for contracts that previously relied on EVM fallback
    dispatch.
  - `EVMContractFactory` wraps Neo ContractManagement deploy/update/destroy
    paths for CREATE/CREATE2/selfdestruct-style lifecycle migrations.
- **Neo-Express compatibility smoke test**:
  `examples/test_neoxp_evm_compat_smoke.sh` compiles, deploys, transfers GAS
  into the adapter contract, reads the recorded payment amount, and invokes
  public fallback dispatch on a local Neo-Express chain.
- **EVM Compatibility Layer documentation** under
  `docs/additional-material/neo-devpack/evm-compatibility-layer.md`, including
  supported workarounds and features that intentionally remain Neo-specific.
- **Showcase contract**: `examples/new/EVMCompatibilityShowcase.sol`.

### Changed

- Hardhat plugin peer ranges and install docs now explicitly target Hardhat
  `>=2.28.6 <3`; Hardhat 3 remains a separate migration because of ESM,
  task API, and network schema changes.
- Node engine ranges are aligned across packages to current LTS lines
  (`^20.19.0 || ^22.12.0 || ^24.0.0`).
- Devpack docs now use `Syscalls.getCallingScriptHash()` as the reliable
  NEP-17 callback token source instead of `msg.sender`.
- Security and CI wording now treat high-severity audit findings as release
  blockers while leaving low/moderate Hardhat 2 legacy advisories documented.

### Fixed

- `devpack/hardhat.config.js` now uses Solidity `0.8.34`, matching the
  compiler/devpack examples and avoiding stale `0.8.28` integration failures.
- Hardhat deployer tests no longer import undeclared `@cityofzion/neon-js`
  helpers for basic hash/primitive assertions.

### Validation

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features`
- `cargo build --release`
- `dotnet test tests/Neo.Sol.Runtime.Tests/Neo.Sol.Runtime.Tests.csproj --configuration Release`
- `npm --prefix tooling run build && npm --prefix tooling run lint && npm --prefix tooling run typecheck && npm --prefix tooling test`
- `npm --prefix devpack test`
- `npm --prefix devpack run test:integration`
- `NEO_SOLC="$PWD/target/release/neo-solc" make test-deploy-evm-compat-smoke`
- `NEO_SOLC="$PWD/target/release/neo-solc" STRICT_SWEEP_FAIL_ON_UNEXPECTED_WARNINGS=1 bash examples/test_strict_compatibility_sweep.sh`
- `npm run docs:check`
- `npm run docs:build`
- `npm run docs:check:links`

## [v0.18.0] - 2026-04-25

Fuzz-system maturation follow-up to v0.17.0. Every deferred bug from the
v0.17.0 release is now resolved (#16 unchecked-uint256 divergence, #23–#25
ABI codec gaps, BLS12-381 runtime stub closed as #26, plus a new in-contract
`T[].length` divergence found and fixed as #28). The state-batch
non-atomic semantics ambiguity is resolved with a canonical
"skip-and-continue" contract documented and pinned. Six additional fixes,
two new cargo-fuzz targets, libfuzzer dictionaries (970 entries), and
substantial new proptest infrastructure land alongside. The headline test
delta is small (+47 tests) but the *effective* fuzz coverage is materially
higher: 43 of those new cases land in `fuzz_tests`, and the bounded-grammar
workaround in `optimizer_props` was removed (the test now runs against
the full `u32::MAX` literal/argument domain rather than `u16::MAX`). Seven
recent review waves found 0 new bugs in well-explored areas — the suite
has reached steady-state.

### Added

- **2 additional cargo-fuzz targets** (was 9, now 11):
  - `fuzz_target_method_tokens` — drives CALLT (0x37) dispatch under
    randomly-generated NEF method-token tables, exercising the
    `invoke_native_contract` path under attacker-controlled token
    metadata.
  - `fuzz_target_nef_manifest_mutation` — bit-flips, byte-writes,
    insertions, and deletions on valid NEF / manifest seeds (53 NEF
    seeds + 24,296 manifest seeds + embedded fallback). Reaches deeper
    parser code paths than pure-random fuzzing.
- **Libfuzzer dictionaries** (`fuzz/dict/*.dict`): 970 domain-specific
  tokens across all 11 cargo-fuzz targets. Each dictionary is target-aware
  (Solidity keywords for `structured_sol`, NEF magic / opcodes for
  `nef_*`, JSON tokens for `manifest_json` / `standard_json`, Yul
  builtins for `yul_assembly`, etc.). Speeds time-to-coverage on a cold
  corpus by an order of magnitude.
- **17 new proptest modules** (effective fuzz coverage materially higher
  even though test-count delta is +43). New modules:
  `abi_roundtrip_props` (21 round-trip tests across all elementary +
  dynamic ABI types), `constructor_lifecycle_props` (declaration-init,
  ctor-body init, ctor-with-args, deploy-runs-once, modifier-on-
  constructor, payable-constructor `msg.value`),
  `diagnostic_stability_props`, `erc1155_proxy_props`,
  `examples_smoke_props`, `in_contract_array_return_props`,
  `multi_source_compile_props`, `pathological_corpus_smoke`. Plus
  follow-on additions to existing modules across the recent agent-team
  waves.
- **End-to-end smoke tests**: 54 single-source + 13 multi-source example
  contracts now compile cleanly under proptest (42 of which are also
  runtime-invoked end-to-end). Multi-source standard-JSON resolver
  verified across simple / transitive / circular / library import
  shapes.
- **ABI codec round-trip framework** (`tests/fuzz_tests/abi_roundtrip_props.rs`):
  21 proptests pinning encode → decode symmetry across `int{8,16,32,64,128,256}`,
  `uint{8,16,32,64,128,256}`, `address`, `bool`, `bytes{1..32}`,
  `string`, `bytes`, `T[]` for static T (`uint256[]`, `address[]`),
  and tuples. Caught and pinned bugs #23 / #24 / #25 / #28.
- **BLS12-381 runtime fully implemented**: all 12 native handlers
  (`bls12381Serialize` / `Deserialize` / `Equal` / `Add` / `Mul` /
  `Pairing` / `G1Add` / `G1Mul` / `G1Neg` / `G2Add` / `G2Mul` / `G2Neg`)
  in `src/runtime/execution/execution_impl_part2_native/crypto.rs`,
  backed by `bls12_381 = "0.8"` (promoted from dev-dep to runtime dep).
  Length-dispatched G1/G2 (48-byte vs 96-byte compressed); subgroup
  checks via `from_compressed`'s built-in torsion verification; pairing
  serialised via `Fp12` Debug bytes (deterministic, equality-preserving).
  All 4 previously-gated `differential_bls12381_*` proptests now pass
  byte-for-byte against the reference crate.
- **ERC-1155 + Proxy lifecycle tests** (`tests/fuzz_tests/erc1155_proxy_props.rs`):
  multi-token mint/transfer/burn flows and proxy-pattern delegate
  routing, surfacing bug #28 in `balanceOfBatch`.
- **State-batch canonical "skip-and-continue" semantics**: the
  ambiguity flagged as a v0.17.0 known limitation is now resolved.
  `src/runtime/state/impl/batch.rs` non-atomic mode silently skips
  invalid changes and continues — pinned by the now-unignored
  `state_batch_non_atomic_partial_apply_spec` test in
  `tests/runtime_state_batch_tests.rs`.
- **26 corpus seed files** (was 11): added 15 pathological seeds covering
  deep parens, nested arrays, large literals, malformed UTF-8, and
  adversarial Yul/NEF shapes. Speeds first-coverage on cold corpora.
- **Storage-key boundary tests**: 93 LOC of new boundary-encoding tests
  in `src/storage_key/tests/encode_integer.rs` added via a
  mutation-testing audit.
- **GitHub-contracts pipeline scaffolding**: `scripts/github_contracts_pipeline.js`
  + example manifests for harvesting third-party contract corpora.

### Fixed

- **#16** — uint256 unchecked-arith narrow-i64 vs BigInt-folded divergence.
  New `emit_widen_to_u256_unsigned` / `emit_truncate_u256` helpers in
  `src/ir/expressions/dispatch/binary.rs` route unchecked uint256 ops
  through the runtime's BigInt path, sidestepping the prior naïve fix's
  signed-interpretation breakage. The `u16::MAX` grammar workaround was
  removed from `optimizer_props.rs`.
- **#21** — `StdLib.atoi("--42", 10)` returned 42 instead of 0; magnitude
  parser switched from `i128::from_str_radix` to `u128::from_str_radix`
  so any sign character in the body yields 0.
- **#22** — `StdLib.base64Decode("AB=C")` returned 3-byte garbage;
  decoder now rejects `=` outside the trailing-pad position of the
  final chunk (matches RFC 4648 + the `base64` crate STANDARD engine).
- **#23** — narrow signed-int `abi.encode` zero-padded instead of
  sign-extending (`int8(-1)` produced `00..00 ff..ff` instead of
  `ff..ff`); new `emit_abi_fixed_buffer_signed` helper branches on
  source sign and pre-fills the destination buffer with `0xFF` for
  negatives.
- **#24** — `abi.decode` pass-through for `string` / `bytes` (returned
  raw 96-byte head+length+pad payload); new
  `emit_abi_decode_dynamic_top_level` helper walks the EVM-canonical
  offset/length/data/pad chain.
- **#25** — `abi.decode` unusable for `T[]` dynamic arrays
  (`.length` faulted `SIZE: unsupported type`); same patch materialises
  a `NEWARRAY` of decoded scalars via per-slot SUBSTR + CONVERT.
- **#26** — BLS12-381 runtime returned `StackItem::Null` for all 12
  native methods (compiler+IR were wired but the runtime had a stubbed
  `_ => Null` fall-through). Functional gap now closed; see Added.
- **#28** — in-contract `this.method()` returns of `T[] memory`
  corrupted `.length` (read wire-byte-length instead of element count);
  new `try_lower_this_external_dynamic_assign` helper in
  `src/ir/statements/assignments/lower_assignment.rs` routes through
  the same dynamic-decode chain as #25. Pinned by
  `tests/fuzz_tests/in_contract_array_return_props.rs`.
- **Payable-constructor `msg.value` snapshot/restore** in
  `call_method_with_deploy_args`: ctor-side msg.value override is now
  snapshotted before the deploy and restored after, so a payable ctor
  observes the deploy-time value rather than leaking a subsequent
  call's override into ctor scope.
- **Storage-key boundary fixes** surfaced via mutation-testing audit
  (`src/storage_key/tests/encode_integer.rs` regressions).

### Test counts

| Suite | v0.17.0 | v0.18.0 | Δ |
|-------|--------:|--------:|--:|
| fuzz_tests (proptest) | 912 | **968** | +56 |
| runtime_state_batch_tests | 2 (1 ignored) | **3** | +1 (ambiguity resolved) |
| lib unit tests | 508 | **511** | +3 |
| conformance_tests | 40 | **40** | — |
| e2e_compilation_tests | 80 | **80** | — |
| **Total** | 1,542 | **1,602** | **+60** |
| cargo-fuzz targets | 9 | **11** | +2 |
| Fixed bugs | 22 | **28** | +6 (cumulative) |

The +60 figure understates the practical coverage gain: removing the
`u16::MAX` literal/arg bound from `optimizer_props` widens the
optimizer-differential search domain to the full `u32::MAX` range, and
each of the 11 cargo-fuzz targets now ships with a libfuzzer
dictionary that compresses time-to-coverage on a cold corpus.

### Known Limitations

None — all open items from v0.17.0 are resolved.

## [v0.17.0] - 2026-04-25

Fuzz-system overhaul release driven by 11 sequential review waves.
**912 proptest cases pass** (was 743), **11 cargo-fuzz targets** (was 6, all
crash-free), **22 bug fixes total** (was 14), plus **one latent allocation-DoS**
hardened proactively. Coverage: **68.42% region / 67.42% line** via
`cargo-llvm-cov`.

### Added

- **5 new cargo-fuzz targets** (was 6, now 11):
  - `fuzz_target_runtime_exec` — feeds arbitrary bytes as NeoVM bytecode
    against a bounded `NeoRuntime` (gas + memory capped). Surfaced bug #15
    (NEWARRAY OOM DoS) on the first run.
  - `fuzz_target_method_tokens` — drives CALLT (0x37) dispatch under
    randomly-generated NEF method-token tables, exercising the
    `invoke_native_contract` path under attacker-controlled token metadata.
  - `fuzz_target_nef_manifest_mutation` — bit-flips, byte-writes,
    insertions, and deletions on valid NEF / manifest seeds (53 NEF seeds,
    24,296 manifest seeds + embedded fallback). Reaches deeper parser code
    paths than pure-random fuzzing.
  - `fuzz_target_structured_sol` — `arbitrary`-driven Solidity AST grammar.
    This release extended the grammar with **inheritance** (multi-base
    linearisation, abstract / `virtual` / `override`), **`using <Lib> for
    <Type>`**, **try/catch**, **revert("string")**, and
    **require(cond, "string")**. Coverage rose from ~8,200 PCs at the
    initial baseline to **11,190 PCs** (+36%) after grammar extensions.
  - `fuzz_target_yul_assembly` — `arbitrary`-driven Yul AST inside
    Solidity `assembly { … }` blocks (let/assign/if/for/switch/break/
    continue + mload/sload/mstore/sstore/return/revert). Drives
    `src/ir/statements/assembly.rs` lowering through random opt-levels.
- **20+ new proptest modules** (912 cases vs 743 baseline):
  `arithmetic_helpers_props`, `compile_runtime_roundtrip`,
  `conditional_jumps`, `contract_upgrade_props`,
  `convergence_props`, `custom_error_envelope_props`,
  `determinism_props`, `devpack_props`, `differential`,
  `disasm_stability_props`, `modifier_rewrite_props`,
  `native_contract_props`, `native_resolver_props`,
  `openzeppelin_patterns_props`, `optimizer_props`,
  `reentrancy_props`, `stdlib_native_props`,
  `storage_iterator_stress`, `storage_state_machine`.
- **Differential testing**: hash differentials extended to 4096-byte inputs
  + boundary-byte patterns (`0x00*N`, `0xFF*N`, alternating); BLS12-381
  G1/G2 add+mul + pairing equality vs `bls12_381` reference crate (gated
  on runtime-implementation arrival); manifest event parameter type
  fidelity vs spec.
- **Coverage analysis**: `cargo-llvm-cov` integrated; coverage gaps
  documented (see Known Limitations / Dead Code in
  `FUZZ_STATUS_REPORT.md`).
- **State-machine tests**: storage iterator stress against a `BTreeMap`
  reference model; 50-call gas-stability sweep on the same `NeoRuntime`
  (no per-call linear-growth from leaks); recursive Fibonacci(10)
  exercising 177 self-external `System.Contract.Call` frames cleanly.

### Fixed

- **#15 — Host DoS via NEWARRAY/NEWARRAY_T/NEWSTRUCT/NEWBUFFER**: a 6-byte
  bytecode `02 ff 00 0c 17 c6` (PUSHINT32 + NEWSTRUCT) requested ~387M
  Null items via `Vec::with_capacity(count)` and OOM-aborted the host
  process before gas accounting could fire. Fixed by bounding `count *
  size_of::<StackItem>()` against `memory_limit` in
  `src/runtime/execution/collections/construction.rs` and `len` against
  `memory_limit` in `src/runtime/execution/execution_impl_part3_bytes.rs`
  (mirrors PUSHDATA4 guard). Regression pinned by `batch132*`.
- **#17 — Host DoS via Storage.Put**: `RuntimeConfig::storage_limit` had
  no consumer (`grep -rn storage_limit src/` returned the declaration and
  default only, zero readers). Storage.put / Local.Put inserted into
  `storage_overlay` with no size check. Fixed by adding `storage_limit`
  to `ExecutionContext`, propagating from config, and adding
  `enforce_storage_limit` (cumulative-bytes guard) on every Put.
- **#18 — Host DoS via Storage.Find**: `helpers/storage.rs::
  build_storage_entries` queried with `limit: None` and used
  `Vec::with_capacity(entries.len())`; an attacker who populated storage
  with many keys (10K writes fit in default gas_limit) could OOM the
  host with one `Find` syscall. Fixed by bounding `StorageQuery::limit`
  against `storage_limit / MIN_ENTRY_BYTES`, plus a post-merge cap to
  cover overlay-only matches.
- **#19 — Gas DoS via flat per-syscall pricing**: `Storage.Put` charged
  a flat 1000 gas regardless of value size; `CryptoLib.sha256/keccak256/
  ripemd160/sha1/murmur32` charged a flat 512 gas regardless of input
  length. Fixed by `syscall_extra_input_gas` (in
  `src/runtime/execution/instruction/syscall.rs`) — peek-and-charge
  before the handler with `STORAGE_PUT_PER_BYTE_GAS=100` and
  `HASH_PER_BYTE_GAS=50` on top of the flat base. CALLT path mirrored at
  `src/runtime/execution/instruction/flow/calls.rs`. Regression pinned by
  `batch133a–d`.
- **#20 — Gas DoS via CheckMultisig**: ran `O(N·M)` secp256k1 verifies
  (up to 4096 verifies for 64×64 input) for a single flat 1000-gas
  charge. Fixed with `CHECKMULTISIG_PER_VERIFY_GAS=1000` charged upfront
  as `pub_count * sig_count * per_verify` (saturating).
- **#21 — Correctness: `StdLib.atoi("--42", 10)` returned 42**:
  `i128::from_str_radix(body, radix)` accepts a leading `-` in `body`
  after the outer `neg` flag was already set, so a double-negative
  becomes a positive number. Fixed by switching magnitude parser to
  `u128::from_str_radix` so any sign character in `body` (`-`, `+`,
  anything else non-digit) yields 0.
- **#22 — Correctness: `StdLib.base64Decode("AB=C")` returned 3-byte
  garbage**: the decode loop accepted `=` at any position inside a 4-char
  chunk and treated it as a literal-zero sextet, ignoring RFC 4648's
  mid-chunk-pad-rejection rule. Fixed: `base64_decode` now rejects `=`
  outside the trailing-pad position of the FINAL chunk.
- **Preventive harden** for `read_input_slice` (calldata reader) — same
  bug-#15 shape (`vec![0u8; length]` with no memory_limit check). Not
  opcode-reachable today, but the methods are public; future opcode
  wiring would inherit the OOM. Now propagates an error if `length >
  memory_limit`.

### Known Limitations

- **Bug #16 — uint256 unchecked-arith narrow-i64 divergence (deferred)**:
  the `optimizer_four_level_differential_random_expr` proptest surfaced a
  real divergence between unoptimized and optimized lowerings of
  unchecked uint256 arithmetic. The naïve fix (force-widen unchecked
  uint256 BinaryOps to 32-byte ByteArray so the runtime's BigInt path
  runs) breaks tests that pass `uint256.max` as a 32-byte ByteArray
  because `coerce_item_to_bigint` treats it as **signed**. Resolving
  properly requires plumbing an unsigned-LE BigInt path through
  `cmp_needs_bigint_path`, `bigint_to_stack_item`, and the wide
  arithmetic helpers — out of scope for this release. Mitigation:
  `dexpr_strategy` literals + `a/b/c` test args bounded to `0..=u16::MAX`.
- **BLS12-381 runtime stub**: the compiler is fully wired (IR resolver
  whitelists 13 method names, devpack exposes `bls12381Add/Mul/Pairing`
  + EVM-precompile shims), but
  `src/runtime/execution/execution_impl_part2_native/crypto.rs::invoke_
  native_cryptolib` does not implement the BLS handlers — every
  `bls12381*` falls through to `StackItem::Null`. Contracts using BLS
  compile cleanly but silently return zero. 4 differential proptests are
  gated to auto-activate when the runtime implementation lands.
- **State batch non-atomic semantics ambiguity**:
  `src/runtime/state/impl/batch.rs:25` does `let _ =
  self.apply_change(change)` in non-atomic mode (silent error discard).
  Could be intentional ("skip invalid, keep going") or a bug ("stop
  after first invalid"). Pinned by an ignored test
  (`state_batch_non_atomic_partial_apply_spec`) so the choice can be
  decided in a future release.

### Dead Code (deletion candidates, not removed in this release)

`cargo-llvm-cov` analysis identified ~6,200 LOC across the codebase that
is **0% covered AND has zero importers** anywhere in `src/` or `tests/`:
- Orphaned standalone Yul frontend (~4,500 LOC):
  `src/{lexer,parser,codegen,semantic,optimizer}/`, `src/error.rs`,
  `src/types.rs`. The Yul-in-Solidity surface (`assembly { … }`) reaches
  its own active lowering at `src/ir/statements/assembly.rs`.
- VM bridge layer (~1,700 LOC): `src/runtime/bridge/`. `VMBridge` is
  constructed only inside its own subtree.
- Top-level orphans: `src/{security,warning,validation,testing}.rs`.

Removing these would push real coverage well above 90%. Listed for
awareness; pending an explicit cleanup decision.

### Test counts

| Suite | v0.16.0 | v0.17.0 | Δ |
|-------|--------:|--------:|--:|
| fuzz_tests (proptest) | 743 | **912** | +169 |
| runtime_state_batch_tests | — | **2** | +2 |
| lib unit tests | 508 | **508** | — |
| conformance_tests | 40 | **40** | — |
| e2e_compilation_tests | 80 | **80** | — |
| **Total** | 1,371 | **1,542** | **+171** |
| cargo-fuzz targets | 6 | **11** | +5 |
| Fixed bugs | 14 | **22** | +8 |

## [v0.16.0] - 2026-04-19

Major stability release driven by continuous fuzz-harness review (Tasks #94–#183).
Approximately 90 distinct fixes landed across compiler, IR lowering, runtime,
ABI encoding, arithmetic, modifiers, yul, try/catch, and storage. The compiler
now passes **409 fuzz harnesses** (0 failed, 1 ignored) covering the full
Solidity 0.8.x feature matrix plus Neo N3 integration.

### Added

- **Narrow Yul support (Task #99, #100, #183)**: inline `assembly { ... }`
  blocks now lower `mstore`/`mload`/`return` along with `let`/`:=` bindings,
  basic arithmetic, and reference semantics. `tstore`/`tload` implement
  EIP-1153 transient storage backed by a `NEWMAP` local (with `HasKey`
  guard for unset slots returning `0`). Yul identifiers now also resolve
  to outer Solidity function parameters via the new `StoreParameter` /
  `LoadParameter` IR variants (Task #183).
- **EIP-1153 transient storage in Yul (Task #100)**: `tstore(slot, value)` /
  `tload(slot)` lowers to per-function transient map ops.
- **New IR variants**: `Substr` (Task #95 bytes slicing), `NewMap` /
  `HasKey` (Task #100 yul transient storage), `StoreParameter` /
  `LoadParameter` (Task #156 parameter writes, Task #183 yul parameter
  binding).
- **`msg.value` host injection (Task #113)**: `NeoRuntime::override_value` /
  `override_sender_and_value` thread a per-call msg.value override through
  the runtime; a dedicated syscall slot in `execution/syscalls/runtime.rs`
  returns the active value. Neo N3 has no intrinsic `msg.value`, so this
  gives Solidity tests a faithful EVM-style calldata surface.
- **`msg.data` synthesis (Task #112)**: public-method dispatch now
  synthesises `selector || abi.encode(args)` and threads it to the
  executing frame so the `msg.data` builtin returns the EVM-canonical
  calldata payload. `bytesN(arg)` / `address(arg)` cast leaves in the
  synthesised path are left-aligned (Task #112 refinement) so they match
  the on-wire encoding.
- **Canonical `Panic(uint256)` envelope (Task #107, #108)**: ALL
  panic sites (div-by-zero, mod-by-zero, INT256_MIN/-1 divide, int
  overflow, unary negate overflow, array OOB, `pop()` empty, `abi.decode`
  short-buffer) now route through the shared `emit_panic` helper which
  emits the 4-byte selector `0x4e487b71` followed by the BE-32 code (e.g.
  `0x11`, `0x12`, `0x32`, `0x41`). Replaces the earlier ad-hoc
  `"Panic: 0xNN"` ByteString payloads so `catch Panic(uint256)` clauses
  can match against the canonical EVM shape.
- **Canonical `Error(string)` / `revert` envelope (Task #131)**: `require`
  with a string literal and `revert("reason")` now both emit the EVM
  `Error(string)` 4-byte selector + abi-encoded string, aligning
  `require` with `revert` payload shape.
- **Delegatecall hard-reject (Task #101)**: `target.delegatecall(data)` is
  now rejected at IR lowering with a precise diagnostic directing users
  toward proxy patterns or target-contract inheritance. Previously the
  call silently routed to `System.Contract.Call`.
- **Bytes slicing (Task #95)**: `b[start:end]` on `bytes` / `bytes memory`
  / `bytes calldata` values lowers via the new `Substr` IR instruction to
  a contiguous ByteString.
- **Write-to-parameter support (Task #156)**: Solidity permits assigning
  to function parameters directly. The new `StoreParameter` IR variant
  lowers via NeoVM `STARG0..6`, and tuple swaps of the form
  `(a, b) = (b, a)` on function parameters now work. Compound
  assignments (`a += 1;`) on parameters are also handled.
- **Narrow signed integer checked arithmetic (Task #154)**: Add/Sub/Mul on
  `int8`/`int16`/`int32`/`int64`/`int128` now emits a post-op range
  guard against `[-(2^(bits-1)), 2^(bits-1)-1]` and routes overflow
  through `Panic(0x11)`.
- **357-test fuzz suite**: 357 new fuzz harnesses land in
  `tests/fuzz_tests/batches_*.rs` (in addition to the baseline 52),
  bringing the total to **409 passed / 0 failed / 1 ignored** on the
  `fuzz_tests` binary.
- **StdLib native coverage (Task #51)**: `itoa`, `atoi`, `base64Encode`,
  `base64Decode` now implemented in `execution_impl_part2_native/stdlib.rs`.
- **Dynamic-array encoding (Task #121, #137)**: `T[]` returns and
  `abi.encode(T[])` / selector-side args emit the EVM-canonical
  `offset || length || BE-32 elements` layout.
- **Struct flattening for `abi.encode` (Task #124)**: whole-struct args
  are expanded into per-field stack items so the `AbiEncode` builtin
  classifies them as static types when appropriate; includes a new
  `try_flatten_struct_arg_for_abi_encode` pattern.
- **Custom-error struct envelopes (Task #181)**: `revert CustomErr(struct)`
  positional and named-args forms flatten struct args into the ABI
  tuple and render struct type strings via the canonical EVM
  `(T1,T2,...)` shape.
- **Sticky caller override (Task #176)**: `pending_caller_account` now
  re-arms from a sticky slot so caller overrides survive across
  self-external calls.
- **`virtual caller` script hash (Task #123)**: per-frame
  `msg.sender` override pushed by self-external dispatch, deterministic
  virtual script hash derivation, resolved in the `GetCallingScriptHash`
  syscall.
- **Self-method dispatch table (Task #70)**: manifest-derived
  `(method_name, offset, arg_count)` table installed on each execute,
  enabling `this.someFn()` without cross-contract syscall overhead.

### Changed

- **Event topic0 = keccak256(signature) (Task #39, pre-landed)**:
  `emit` lowering now produces the EVM-canonical log shape
  (`topics[0]` is the keccak256 of the event signature, with indexed
  args taking subsequent topic slots; non-indexed args go to `data`).
- **Try/catch envelope matching (Task #103, #86)**: each catch clause
  uses a shape-specific guard (`Error(string)`, `Panic(uint256)`,
  named custom error, wildcard). Raw stack-top bytes are preserved
  through `try_frames.rs` so the catch handler receives the EVM
  envelope verbatim.
- **Override sticky semantics (Task #105)**: `default_timestamp` pinned
  to `1_704_067_200` (2024-01-01T00:00:00Z). Pending metadata overrides
  (timestamp, index, sender) are now snapshotted BEFORE the user
  method observes them and drained after the call.
- **Require → Error(string) envelope (Task #131)**: `require(cond, "msg")`
  with a string literal now emits the same EVM `Error(string)` shape as
  `revert("msg")`.
- **Storage reference handling (Task #117)**: `resolve_storage_reference`
  widened to recognise Array- and Mapping-typed state variables as
  storage pointers when the base has no `field_path`. Regression guard
  preserves the per-field-slot layout for struct-array writes
  (Task #104).
- **`using` directive inlining (Task #91)**: library functions whose
  first parameter is `T storage` are inlined with caller-parameter
  hiding and a per-call inline-return redirect, preserving source
  semantics across the inline boundary.
- **Modifier epilogue semantics (Task #114)**: when at least one
  applied modifier has a body statement after `_`, every `return` in
  the function body now redirects through a synthetic modifier-wrap
  break label so the epilogue runs exactly once regardless of which
  path the function takes.
- **Interface expansion (Task #115)**: interface casts `I(expr)` in
  statements, interface-typed parameters, and interface references are
  expanded to the primary contracts that implement them, allowing
  cross-contract calls through interfaces.
- **Fallback dispatch (Task #126)**: a primary contract's `fallback()`
  now acts as a universal catch when the explicitly-named method
  isn't in the merged dispatch table. `Receive` is included alongside
  `Fallback` in the dispatch scan.
- **Tuple return flattening (Task #94)**: nested tuple return
  expressions are now flattened recursively. Parenthesised tuple
  return/parameter types accepted at semantic level.
- **`abi.decode` short-buffer guard (Task #84)**: emits
  `Panic(0x41)` instead of a raw throw when `buf.length <
  expected_static_bytes`.
- **BigInt comparison (Task #30 slice 1 Part C)**: wide-ByteArray
  comparisons route through BigInt so `uint256` values pushed as
  21+ byte payloads compare by magnitude, not by raw bytes.
- **Wide bitwise ops (Task #50)**: `~`, `|`, `&`, `^` on wide operands
  route through BigInt with a post-op 256-bit mask, so `~uint256(x)`
  produces `u256::MAX - x` rather than `!(x as u64)`. Narrow
  i64/u64 shift path truncation also fixed.
- **BigInt shift (Task #H4)**: wide-ByteArray left/right shifts route
  through BigInt. Reuses the Task #50 infrastructure and clamps shift
  amounts > 255 to an all-zero result per EIP-145 (Task #33).
- **Interface kind metadata (Task #115)**: collected per-interface at
  analysis time and threaded through lowering for virtual dispatch.
- **Self external-call routing (Task #83)**: `new B(); b.foo()` where
  `B` is a sibling-merged primary contract now routes through
  `self_method_offsets` via a 20-byte zero placeholder, bypassing the
  System.Contract.Call syscall for bandwidth-free internal dispatch.
- **Framework / NEP standards (devpack v1.1.0)**: `FrameworkBase.sol`,
  `NEP17.sol`, `NEP11.sol`, `NativeCalls.sol`, `Neo.sol`,
  `Runtime.sol`, `Storage.sol` refreshed for the new compiler
  semantics. New `Precompiles.sol` library (EVM precompile routing)
  and `PrecompileShowcase.sol` example.

### Fixed

- **MEMCPY leak in 8+ sites (Task #109, #66, #89, #76)**: `bytesN(..)` /
  `address(..)` cast args no longer leak the MEMCPY source pointer
  into downstream builtins. Fixed in `events.rs`, `abi.rs`,
  `member_access.rs`, `resolved.rs`, `return_revert.rs`,
  `builtins.rs`, and the packed-encoding path.
- **`bytes32 ↔ uint256` (Task #111)**: `uint256(bytes32)` is a
  bit-identity reinterpret — Solidity spec §4.7.3 — not a decode.
  Preserves magnitude across the cast.
- **`payable(x)` (Task #128)**: now a pure type-only cast (§4.3, §4.7.3),
  not a no-op wrap.
- **`string(bytes_value)` (Task #171)**: recognised as a semantic
  no-op on the value stack.
- **Nested tuple return (Task #94, #64)**: `return (a, (b, c))` flattens
  to the canonical tuple layout.
- **Bytes slice SUBSTR (Task #95)**: `b[i:j]` on dynamic `bytes` lowers
  correctly via the new `Substr` IR op.
- **Modifier epilogue (Task #114)**: runs once regardless of return path.
- **Struct array push/read (Task #104, #170)**: `P[] ps; ps.push(P(a,b))`
  and subsequent reads use the correct per-field slot layout;
  symmetric narrow-ByteArray × UnsignedInteger arithmetic arms added.
- **Delegatecall silent routing (Task #101)**: now hard-rejected with a
  diagnostic, no longer silently routes to System.Contract.Call.
- **9-arg `abi.encode` selector-side (Task #121 mirror)**: `T[]` in
  selector-side args emits the same 32-byte BE offset/length/elements
  shape as the return side.
- **Dynamic-array return JSON leak (Task #137)**: single-value
  dynamic-array returns (`return uint[]`) now emit the canonical EVM
  layout instead of leaking an internal JSON Array envelope.
- **Post-increment / post-decrement wrap (Task #30 slice 4)**:
  `lower_post_inc_dec` now routes through the checked-arithmetic
  path so `x++` at `uint256.max` emits `Panic(0x11)`.
- **Checked arithmetic — 6 ops (Task #30, #67, #154)**: all Add/Sub/Mul
  ops on `uint256`/`int256`/narrow signed types now route through
  BigInt with post-op range guards against the per-type domain.
- **`uint256` BigInt path (Task #32)**: Sub/Mul no longer wrap at 64-bit
  when operands are wide ByteArray.
- **`abi.encode*` / `abi.decode` shape (Task #44)**: EVM-canonical
  `pad32_be(arg_i)` encoding for static args, offset/length/payload for
  dynamic args. Round-trip decoder accepts the BE-packed payload.
- **INT256_MIN / -1 (Task #30 slice 4)**: runtime-side guard for the
  unrepresentable signed-division case routes to `Panic(0x11)`.
- **`emit` with 0 indexed args (Task #39)**: produces exactly 1 topic
  (the signature hash).
- **NatSpec permissions override**: `@custom:neo.manifest.permissions`
  comments correctly substitute wildcard manifests (pre-landed).
- **Modifier re-entry guard**: modifier-epilogue redirect does not
  interfere with nested try/catch break targets.
- **`NeoRuntime::call_function` (Task #19 — earlier session)**: pushes
  args to eval stack, skips `_deploy`, respects offset. All
  `call_method`-driven harnesses now deliver args correctly.
- **`ecrecover` (Task #20 — earlier session)**: returns
  Ethereum-spec address via `keccak256(pubkey[1..])[12..]`, not the
  Neo-native script-hash shape.
- **Narrow-operand bitwise (Task #118)**: left-aligned bytesN fuzzing
  harness no longer flagged as negative via the sign-extended interpretation.
- **`.length` on indirected storage dynamic arrays (Task #161)**:
  returns the live length from the canonical slot instead of a stale
  SIZE value.
- **`pop()` empty-array Panic (Task #98)**: now routes through the
  shared `emit_panic` helper with code `0x31`.
- **`.transfer(uint)` on payable (Task #162)**: lowering now works
  end-to-end instead of faulting on argument packing.
- **`new T[N]` memory allocation (Task #49)**: `T[N] memory a;` (no
  initializer) now allocates a real array with N default-initialized
  slots instead of leaving the local as a null reference.
- **Cross-contract `new B(); b.foo()` routing (Task #160)**: catch
  arms in try/catch around the sibling-merged external-call routing
  receive the revert envelope correctly.
- **`try X { ... } catch ...` parse-shape (Task #125)**: leading
  call expression in a try statement is correctly matched against
  the sibling-merge routing.

### Infrastructure

- **Fuzz test split**: the 24k-line `tests/fuzz_tests.rs` monolith was
  reorganised into 10 submodules under `tests/fuzz_tests/` for
  maintainability:
  - `baseline_tests.rs` — the original 52 harnesses.
  - `batches_18_30.rs`, `batches_31_45.rs`, `batches_46_64.rs`,
    `batches_66_80.rs` — per-batch harness groups.
  - `arithmetic_props.rs`, `compiler_props.rs`, `optimizer_props.rs`,
    `storage_props.rs` — proptest property tests.
  - `task107_catch_panic_tests.rs` — dedicated Panic(uint256)
    envelope coverage.
  - `common.rs` — shared helpers.
- **Regression corpus**: `.proptest-regressions` files under
  `tests/fuzz_tests/` and `tests/` checkpoint failing seeds for all
  property tests.
- **Devpack bump**: `@neo-devpack-solidity/contracts` → **1.1.0** to
  align with the new compiler semantics (new `Precompiles.sol`
  library, refreshed `Framework.sol`, `NEP17.sol`, `NEP11.sol`).

### Known Limitations

- **Task #182** — nested struct dynamic-array `.length` returns still
  serialize via the internal JSON Array shape instead of the
  EVM-canonical 32-byte BE length. Workaround: return the field as
  a separate `uint256` computed from `arr.length`. The corresponding
  harness is marked `#[ignore]`.

## [v0.15.0] - 2026-03-18

### Added

- **onNEP11Payment Support**: `msg.sender`, `msg.value`, and `msg.data` now correctly map to parameter indices in `onNEP11Payment` callbacks (msg.data uses param 3, after tokenId at param 2).
- **Test Coverage**: Added focused IR codegen tests for `msg.data` with selector prefix and `onNEP11Payment` parameter mapping.

### Changed

- **msg.data with Selector Prefix**: `msg.data` outside callbacks now produces `selector || abi.encode(current args)` instead of just `abi.encode(args)`, giving users a proper EVM-style calldata approximation.
- **block.coinbase**: Now maps to `address(0)` instead of `getNextBlockValidators()`, matching EVM's `address` return type.
- **block.sha3**: Fixed incorrect mapping from `GetRandom()` to `Ledger.currentHash` (the current block's hash).
- **encodeWithSelector Resolution**: Added `encodeWithSelector` to `builtin_library_supported_members` and `resolve_abi_member` for proper resolution.
- **Documentation Overhaul**: Comprehensive update across all documentation files to reflect actual compiler behavior:
  - `delegatecall` documented as warning (not blocked)
  - `msg.value` documented as warning + returns 0 (not error)
  - `parity-and-limitations.md` split Blocked vs Auto-Mapped features
  - All feature tables updated with correct mappings

### Fixed

- **Stale Comments**: Fixed comments in `runtime_values.rs` that referenced wrong warning codes or incorrect behavior.
- **block.parenthash Comment**: Fixed comment that incorrectly referenced `getBlock(currentIndex-1).prevHash`.

## [v0.14.0] - 2026-03-13

### Added

- **EVM Try/Catch Multi-return**: `try/catch` blocks now natively support EVM's multiple return syntax (`try returns(uint a, uint b)`) by seamlessly unwrapping the NeoVM `Array` return payload.
- **Documentation Parity**: Completely refactored the VitePress documentation architecture to identically mirror the official `soliditylang.org` sidebar, taxonomy, and feature coverage, fully tailored for Neo N3.

### Changed

- **Graceful EVM Call Options**: Extraneous call options (e.g., `contract.method{value: x}()` or `new Contract{value: x}()`) are now safely ignored, emitting a semantic warning instead of halting compilation.
- **Inline Assembly Fallback**: `assembly { ... }` blocks now compile gracefully into NeoVM no-ops with a warning, unblocking compilation of heavily optimized Ethereum libraries where the assembly isn't strictly required.
- **Unsupported Call Translation**: Unsupported low-level EVM calls (`delegatecall`, `staticcall`) are now lowered to returning a dummy boolean `false` with a semantic warning instead of a hard E3001 abort.
- **Obsolete EVM Globals**: `msg.data` now compiles to `selector || abi.encode(current args)` outside of the `onNEP17Payment` callback (param 2) and `onNEP11Payment` callback (param 3). `msg.sig` now compiles to the current function selector with a warning about internal-call semantics.

### Fixed

- **Infinite Loop Prevention**: Patched the Neo IR `CallFunction` dataflow analysis to accurately track return arities, preventing `neo-solc` from hanging infinitely on complex void-return functions (like those found in DAO Governance contracts).
- **NatSpec Overrides**: Fixed missing `load_manifest_permissions_override_from_natspec` linkages, ensuring `@custom:neo.manifest.permissions` comments correctly substitute wildcard manifests.
- **Runtime Exception Handlers**: Hardened the execution context bridging, replacing manual modulo bitwise checks with `.is_multiple_of()` to appease strict CI linting.

## [v0.13.1] - 2026-02-18

### Changed

- **Release workflow resilience**: release matrix now uses `fail-fast: false` so one
  target failure no longer cancels other platform builds.

### Fixed

- **ARM64 Linux release builds**: hardened aarch64 cross-compilation setup with explicit
  linker/toolchain environment (`CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER`,
  `CC/CXX/AR` target-specific vars, and cross pkg-config allowance).
- **Release pipeline reliability**: added missing ARM64 cross tool dependencies in CI
  (`g++-aarch64-linux-gnu`, `binutils-aarch64-linux-gnu`, `libc6-dev-arm64-cross`).

## [v0.13.0] - 2026-02-18

### Added

- **Transparent EVM-to-Neo auto-mapping**: 9 previously blocked EVM-specific
  Solidity features now compile with Neo N3 equivalents and compile-time warnings.
- **`super` keyword support**: `super.method()` resolves through flattened inheritance,
  preserving base overrides as `__super_{methodName}` during lowering.
- **User-defined value types (`type X is Y`)**: alias propagation across file/contract
  scopes and inheritance; `wrap`/`unwrap` lower as no-ops.
- **`type(T).name` expression**: compile-time string constants for contract and type names.
- **`require(condition, CustomError(...))` support**: Solidity 0.8.26+ form supported with
  diagnostic-preserving error signature text.
- **Devpack expansion**: added NEP-26 standard coverage, `NativeContracts` native address
  helpers, and reusable `NeoBytes` / `NeoMath` base libraries.

### Changed

- **`using` directive semantics hardened**: member-style library calls now require explicit
  `using ... for` scope, enforce receiver type targeting, and enforce named-function lists
  from `using {f,g} for T` declarations.
- **Frontend-to-IR metadata flow** extended for `using` directives, so library binding scope
  and target constraints are preserved through lowering.
- **Release process docs** updated to match actual repository workflow (validation + tag push).

### Fixed

- **Native contract lowering**: aligned `NativeContracts` and `NativeCalls` member-call paths;
  lowered native constants as address literals in IR/codegen path.
- **Diagnostics quality**: reduced duplicate constant warning noise in merged library contexts.
- **Toolchain compatibility**: replaced unstable `is_multiple_of` usage with stable modulo logic.

## [v0.12.0] - 2026-02-13

### Changed

- **Developer Tools**: 85% → 95% complete
  - Added debug tooling support (@neo-devpack-solidity/types/debugger)
  - Added network configurations for Neo TestNet/MainNet
  - Added artifact management
  - Added source map support

## [v0.11.0] - 2026-02-13

### Changed

- **Progress Update**: Updated all progress metrics to 95% complete
- **Status Badge**: Updated to "🟢 Production-Ready · 95% Complete · 620+ Tests"

### Updated

- **Core Compiler**: 90% → 95% (function overloading, public state variable getters)
- **Runtime Library**: 80% → 95% (iterator handles, per-syscall gas accounting, full opcode suite)
- **Testing**: 85% → 95% (620+ test coverage, end-to-end tests complete)
- **Developer Tools**: 75% → 85% (Hardhat, Foundry, ABI Router, Types, CLI Tools)
- **Documentation**: 85% → 95% (Solitity support matrix, Error reference, Architecture)

### Known Limitations

- Oracle integration (stub only - requires external oracle service)
- Fuzzing framework (planned)
- Differential testing (planned)
- IDE debugging tools (planned)

## [v0.10.0] - 2026-02-13

### Fixed

- **Code generation**: Fixed variable assignment to emit correct `STLOC` instructions
- **Loop control**: Implemented `break`/`continue` with proper loop context tracking
- **Variable handling**: Added variable index table for efficient local variable access
- **Semantic analysis**: Improved variable scope tracking with scope stack

### Added

- **CompilerConfig builder methods**: Added `include_abi()`, `include_source_map()`, `validate_only()`, `analyze_only()`
- **Optimization helpers**: Added `is_optimized()` and `optimization_passes()` methods
- **SemanticModel methods**: Added `public_functions()`, `get_function()`, `get_state_variables()`, `is_payable()`
- **Error codes**: Added `BreakOutsideLoop`, `ContinueOutsideLoop`, `InvalidJumpOffset`
- **Helper functions**: Added `emit_ldloc()` and `emit_stloc()` for proper NeoVM bytecode generation

### Refactored

- **Code generator**: Improved variable handling with proper index-based storage
- **Optimizer**: Better code structure with clearer separation of concerns
- **Error handling**: More specific error codes for better debugging

### Changed

- **Gas estimation**: Updated to use more accurate NeoVM cost values (crypto: 700000, storage: 1000000)
- **Devpack documentation**: Clarified `contractCallWithFlags` flags parameter status

## [v0.9.10] - 2026-02-11

### Added

- **Import support expansion**: wildcard namespace bindings now support
  static member calls, namespace-qualified contract/interface casts, and
  selector access forms such as `NS.IFoo.foo.selector`.
- Standard JSON regression coverage for alias/wildcard import behavior,
  including namespace cast and selector forms.
- Low-level call regression coverage for `abi.encodeCall(...)` inline,
  local-variable, and invalid-reference cases.

### Changed

- Low-level call parsing now accepts `abi.encodeCall(...)` payloads in the
  same lowering path as `encodeWithSignature/encodeWithSelector`, including
  simple wrapper forms like `bytes(...)` / `string(...)`.
- `try/catch` lowering now emits runtime type-guard dispatch (`ISTYPE`) for
  multi-clause catch handling, with clearer NeoVM-specific Panic diagnostics.
- `immutable` state variable enforcement tightened to constructor / `_deploy`
  initialization only.
- Feature matrices and README support notes updated to reflect current import,
  low-level call, and catch-clause behavior.

### Fixed

- **Low-level `abi.encodeCall` validation bug**: non-function member
  expressions (e.g. `s.x`) are no longer accepted as function references for
  dynamic low-level calls.
- Fixed-size `new T[N]` allocations now lower correctly for compile-time sizes.
- Nested tuple destructuring lowering reliability improved for mixed targets.

## [v0.9.9] - 2026-02-09

### Added

- **Native contract runtime support**: Policy, Oracle, RoleManagement, Ledger,
  Notary, and Treasury native contracts are now callable from the embedded
  runtime, with per-contract dispatch modules and gas hints.
- `notary.rs` and `treasury.rs` dispatch modules under
  `src/runtime/execution/execution_impl_part2_native/`.
- `NativeContractShowcase.sol` example demonstrating Policy, Ledger, and
  RoleManagement calls from Solidity.
- `OracleRelayStrictShowcase.sol` for strict Oracle request/response relay
  patterns with on-chain callback verification.
- `UpgradeLifecycleShowcase.sol` covering `ContractManagement.update` and
  `ContractManagement.destroy` lifecycle operations.
- `WitnessGuardShowcase.sol` demonstrating `Runtime.checkWitness` guard
  patterns and multi-signer authorization.
- Neo-Express smoke test scripts for the new showcase contracts
  (`test_neoxp_new_showcases_smoke.sh`, `test_neoxp_oracle_relay_smoke.sh`,
  `test_neoxp_upgrade_lifecycle_smoke.sh`, `test_neoxp_witness_guard_smoke.sh`).
- `test_strict_compatibility_sweep.sh` for batch strict-mode compilation
  validation across all showcase contracts.
- `runtime_native_contract_tests.rs` integration test suite for native contract
  dispatch coverage.
- Diagnostic infrastructure activation: structured JSON warnings and errors
  (`--json-warnings`, `--json-errors`) wired through the full pipeline.
- Import path relaxation: the `-I` flag now resolves transitive imports more
  flexibly, reducing false "file not found" errors in multi-directory layouts.

### Changed

- `ExecutionContext` and `ExecutionState` extended with native-contract routing
  tables and overlay storage hooks for Policy/Oracle/Ledger.
- CI workflow (`.github/workflows/ci.yml`) updated with a dedicated
  `neoxp-showcases` job that validates the new showcase contracts end-to-end.
- `bridge_impl_syscalls.rs` and `bridge_impl_core/initialize.rs` updated to
  register Notary and Treasury service endpoints.

### Fixed

- Oracle dispatch now correctly propagates callback contract hash instead of
  defaulting to the calling contract.
- RoleManagement `getDesignatedByRole` returns an empty array (instead of
  panicking) when no nodes are designated for the requested role.

## [v0.9.8] - 2026-02-08

### Added

- **ERC-to-NEP pattern detection**: the compiler recognizes ERC-20, ERC-721,
  ERC-1155, ERC-2612, and ERC-4626 interface patterns and maps them to their
  Neo equivalents (NEP-17, NEP-11) in the generated manifest.
- BN254 (alt_bn128) precompile stubs for pairing and scalar-mul operations.
- Comprehensive test suite expansion: `runtime_syscall_tests.rs` with syscall
  coverage for `Runtime.checkWitness`, `Runtime.getTime`,
  `Runtime.getInvocationCounter`, and `Runtime.getRandom`.
- `erc_nep_patterns.rs` validation module with pattern-matching heuristics for
  standard detection.
- `e2e_compilation_tests.rs` expanded to cover the new showcase contracts and
  native-call paths.
- Metadata test suite (`src/cli/tests/metadata/erc_nep_patterns.rs`) validating
  that manifests carry correct NEP standard annotations.

### Changed

- `semantic_model.rs` updated with ERC/NEP mapping tables used during manifest
  generation.
- `src/solidity/validate/contract/methods.rs` and `return_types.rs` tightened
  to reject incompatible return-type overrides in standard interfaces.

### Fixed

- Manifest `supportedstandards` field now correctly lists detected NEP
  standards instead of leaving the array empty when ERC interfaces are used.

## [v0.9.7] - 2026-02-07

### Added

- **NEP standard detection**: contracts implementing `NEP17` or `NEP11`
  interfaces from the devpack are automatically annotated in the manifest.
- Type inference improvements for `address`-to-`Hash160` and `uint256`-to-
  `Integer` conversions in cross-contract call arguments.
- `CompleteNEP11NFT.sol` and `CompleteNEP17Token.sol` devpack examples
  demonstrating full standard compliance.
- `MultiStandardToken.sol` example implementing both NEP-17 and NEP-11 on a
  single contract.
- `EventIndexedShowcase.sol` demonstrating indexed event parameters and their
  mapping to Neo notifications.
- `InterfaceShowcase.sol` covering Solidity interface inheritance and virtual
  dispatch.
- `TypeCastingShowcase.sol` for explicit and implicit type conversion patterns.

### Changed

- `devpack/standards/NEP11.sol` and `NEP17.sol` updated with complete method
  signatures matching the Neo N3 standard specification.
- `devpack/contracts/Framework.sol` extended with helper methods for standard
  detection at compile time.
- CLI standards module (`src/cli/cli_parts/cli_manifest/standards.rs`) rewritten
  to support automatic and manual standard annotations.

### Fixed

- Type inference no longer silently drops `bytes32` arguments when passed to
  native contract methods expecting `ByteString`.

## [v0.9.6] - 2026-02-06

### Added

- Six famous DeFi contract ports under `examples/famous/`:
  WGAS, FlashLoan, SimpleAMM, TokenVesting, SimpleLending, SimpleDAO.
- `Bank.sol` and `Vault.sol` examples demonstrating deposit/withdraw patterns
  with NEP-17 integration.
- `LowLevelCallShowcase.sol` demonstrating `address.call()` lowering to
  `System.Contract.Call`.
- Improved ERC-20 and ERC-721 example contracts with Neo-specific adaptation
  notes.

### Changed

- `examples/README.md` reorganized with categorized contract listings and
  compilation instructions.

## [v0.9.5] - 2026-02-05

### Added

- Enhanced devpack libraries: `Runtime.sol`, `Storage.sol`, `Neo.sol` updated
  with additional helper methods and NatSpec documentation.
- `NativeCalls.sol` extended with `gasTransfer`, `neoTransfer`, and
  `getContract` wrappers.
- `NEP17Rescue.sol` utility contract for recovering stuck NEP-17 tokens.
- `Syscalls.sol` updated with complete syscall constant definitions.
- `devpack/README.md` rewritten with usage examples and import instructions.

### Changed

- Devpack library method signatures aligned with Neo N3 reference
  implementation parameter names.

## [v0.9.4] - 2026-02-04

### Fixed

- **MEMCPY codegen bug**: memory-copy operations for dynamic `bytes` and
  `string` types now emit correct NeoVM `MEMCPY` sequences instead of
  truncating at 32-byte boundaries.
- **Void DROP bug**: functions returning `void` no longer emit a spurious `DROP`
  opcode that corrupted the evaluation stack when called as statements.
- **LogicalNot codegen bug**: the `!` (logical not) operator now correctly emits
  `NZ` + `NOT` instead of a bare `NOT`, which previously produced incorrect
  results for non-boolean integer operands.

### Changed

- `src/ir/expressions/calls/type_constructors.rs` and
  `src/ir/expressions/variable.rs` refactored to centralize type-width
  validation during IR lowering.

---

[Unreleased]: https://github.com/r3e-network/neo-devpack-solidity/compare/v0.28.0...HEAD
[v0.28.0]: https://github.com/r3e-network/neo-devpack-solidity/compare/v0.27.0...v0.28.0
[v0.27.0]: https://github.com/r3e-network/neo-devpack-solidity/compare/v0.26.0...v0.27.0
[v0.22.0]: https://github.com/r3e-network/neo-devpack-solidity/compare/v0.21.0...v0.22.0
[v0.19.0]: https://github.com/r3e-network/neo-devpack-solidity/compare/v0.18.1...v0.19.0
[v0.18.1]: https://github.com/r3e-network/neo-devpack-solidity/compare/v0.18.0...v0.18.1
[v0.18.0]: https://github.com/r3e-network/neo-devpack-solidity/compare/v0.17.0...v0.18.0
[v0.17.0]: https://github.com/r3e-network/neo-devpack-solidity/compare/v0.16.0...v0.17.0
[v0.16.0]: https://github.com/r3e-network/neo-devpack-solidity/compare/v0.15.0...v0.16.0
[v0.15.0]: https://github.com/r3e-network/neo-devpack-solidity/compare/v0.14.0...v0.15.0
[v0.14.0]: https://github.com/r3e-network/neo-devpack-solidity/compare/v0.13.1...v0.14.0
[v0.13.1]: https://github.com/r3e-network/neo-devpack-solidity/compare/v0.13.0...v0.13.1
[v0.13.0]: https://github.com/r3e-network/neo-devpack-solidity/compare/v0.12.0...v0.13.0
[v0.12.0]: https://github.com/r3e-network/neo-devpack-solidity/compare/v0.11.0...v0.12.0
[v0.11.0]: https://github.com/r3e-network/neo-devpack-solidity/compare/v0.10.0...v0.11.0
[v0.10.0]: https://github.com/r3e-network/neo-devpack-solidity/compare/v0.9.10...v0.10.0
[v0.9.10]: https://github.com/r3e-network/neo-devpack-solidity/compare/v0.9.9...v0.9.10
[v0.9.9]: https://github.com/r3e-network/neo-devpack-solidity/compare/v0.9.8...v0.9.9
[v0.9.8]: https://github.com/r3e-network/neo-devpack-solidity/compare/v0.9.7...v0.9.8
[v0.9.7]: https://github.com/r3e-network/neo-devpack-solidity/compare/v0.9.6...v0.9.7
[v0.9.6]: https://github.com/r3e-network/neo-devpack-solidity/compare/v0.9.5...v0.9.6
[v0.9.5]: https://github.com/r3e-network/neo-devpack-solidity/compare/v0.9.4...v0.9.5
[v0.9.4]: https://github.com/r3e-network/neo-devpack-solidity/releases/tag/v0.9.4
