# Neo DevPack Solidity — Project Conventions

## Architecture
- Solidity-to-NeoVM compiler: Parsing → IR → Optimization → Bytecode → NEF + Manifest
- 503 Rust source files + 115 test files (~90.5K LOC)
- 965 tests across 54 targets, ~87% famous-contract compilation coverage
- 11 Neo N3 native contracts supported, all syscall surfaces verified
- 196/196 opcodes implemented, 35/35 syscalls handled, 101/107 native methods (6 intentional stubs)
- `foundry-solang-parser` v0.3.9 for Solidity parsing (latest available, published May 2025)

## Module Layout (post-refactoring)
- `src/frontend/` — Solidity parsing, pragma validation, contract IR extraction
- `src/solidity/` — Solidity analysis (inheritance, modifiers, sibling merge)
- `src/ir/` — Intermediate representation construction, lowering, expressions, statements
- `src/codegen/` — NeoVM bytecode generation (extracted from cli in v0.29.0)
- `src/optimizer/` — IR optimization passes (extracted from cli in v0.29.0)
- `src/manifest/` — Manifest generation, permissions inference, standards detection (extracted from cli in v0.29.1)
- `src/runtime/` — Embedded NeoVM runtime simulator (167 files, deep nesting, high coupling)
- `src/cli/` — Command-line interface, standard JSON, tests (106 files, down from 151)
- `src/neo/` — NEF/manifest building and parsing utilities
- `devpack/contracts/` — Solidity library contracts: Framework, Syscalls, NativeCalls

## Key Conventions
- No `#[path = "..."]` anti-pattern — all modules use standard Rust layout (0 remaining)
- Maximum directory depth target: ≤4 levels (was 10, currently at 5 in some runtime paths)
- Files >800 lines: 10 remaining. Monolithic pipelines (assembly, solidity_analyse) resist mechanical splitting
- Files >800 lines: 13 remaining. Monolithic pipelines (assembly, solidity_analyse) resist mechanical splitting
- Test files follow Rust conventions: `tests/mod.rs` for integration tests, `#[cfg(test)] mod tests;` for unit tests
- Solidity contracts use `^0.8.19` pragma (minimum) or `>=0.8.19 <0.8.28` for compiler-supported range
- Native contract hashes defined in both Rust (`NATIVE_CONTRACTS` array) and Solidity (`NativeContracts.sol`)

## File Splitting Best Practices
- ✅ Files with clear domain boundaries + independent pub functions: safe to mechanically split
- ❌ Single monolithic function (match/if-let chain): do NOT mechanically split — use surgical refactoring
- ❌ Deeply nested modules (>3 levels): import complexity can cause cascading visibility issues
- `opcode.rs` and `member_nativecalls.rs` are good examples of successful splits

## Testing
- `cargo check` must pass clean (0 warnings, 0 errors) before any commit
- `cargo test --no-run` to verify all test binaries compile
- Famous contract compilation test: `cargo test -p neo-devpack-solidity` (55 targets)
- Fuzz tests in `tests/fuzz_tests/` — now using standard `mod.rs` layout (no #[path])

## v1.0 Remaining Roadmap
- Phase 1: COMPLETE (v0.29.0 + v0.29.1) — CLI decomposition done (codegen + optimizer + manifest extracted)
- Phase 2 (v0.30.x): Runtime isolation — decouple 4 subsystems via port interface (MEDIUM RISK)
- Phase 3 (v0.31.x): Extension points — trait plugins (LOW RISK, parallel with Phase 2)
- Phase 4 (v0.32.x): File refactoring — break 13 files >800 lines (MEDIUM RISK)
- Phase 5 (v1.0, optional): Crate split — workspace (HIGH RISK, only if needed)
- Solidity 0.8.29+ feature additions (function types, fixed/ufixed) — feature work

## Version History
- **v0.30.1** (2026-07-04): Code quality fixes from team capability improvement plan
  - Eliminated 7 `eprintln!` in manifest/build.rs → warnings collector pattern
  - Fixed 3 risky `unwrap()` in runtime (empty-bytes guard, documented invariants)
  - Added `# Safety` comment to undocumented `unsafe` block
  - Team capability improvement plan, code review checklist, CONTRIBUTING.md created
- **v0.30.0** (2026-07-04): Architecture Phase 4 — file refactoring
  - Split 3 largest files: assembly.rs (1440→5 files), frontend_parse.rs (1037→5 files), lower_assignment.rs (1140→2 files)
  - Files >800 lines reduced from 13 to 10
  - Remaining 10 are monolithic match chains requiring surgical refactoring
- **v0.29.1** (2026-07-04): Architecture Phase 1 complete — manifest extraction
  - Extracted `src/manifest/` (11 files) from `src/cli/cli_parts/cli_manifest/`
  - Broke bidirectional dependency (moved `solidity_to_manifest_type` to manifest)
  - Broke CompileError ownership (created `ManifestError` type)
  - CLI reduced from 117 to 106 files. Phase 1 total: 45 files extracted
- **v0.29.0** (2026-07-04): Architecture Phase 1 — CLI decomposition
  - Extracted `src/codegen/` (29 files) from `src/cli/bytecode/`
  - Extracted `src/optimizer/` (5 files) from `src/cli/ir_optimize/`
  - CLI reduced from 151 to 117 files
  - Manifest extraction deferred (2 blocking couplings identified)
  - Architecture design docs: `docs/architecture-design.md`, `docs/adr/adr-001-to-006.md`
- **v0.28.1** (2026-07-03): Audit-driven patch — fixed all P2/P3 issues from v0.28.0 audit
  - P2-1: Nested EQUAL type-strictness (recursive Array/Map comparison)
  - P2-2: NativeTypes.ContractState field types corrected
  - P2-3: 20 missing runtime handlers implemented (NEO 5, ContractMgmt 4, StdLib 10, CryptoLib 1)
  - P2-4/P2-5: Dead code removed
  - P3-1 through P3-5: Clippy, imports, using directives, pragma, 14 missing Solidity wrappers
  - New deps: bs58 (base58), ed25519-dalek (Ed25519 verification)
- **v0.28.0** (2026-07-03): Precision & correctness release (ByteString/Buffer distinction, gas scaling, streaming iterator, Oracle enrichment)
