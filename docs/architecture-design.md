# Neo DevPack Solidity — Architecture Design & Long-Term Evolution Plan

**Version**: 1.0 | **Date**: 2026-07-04 | **Status**: Proposed

---

## 1. Executive Summary

This document presents a comprehensive architecture design and long-term evolution plan for the neo-devpack-solidity compiler system. The system is a Solidity-to-NeoVM bytecode compiler with 503 Rust source files (~90.5K LOC), 965 tests, and 11 Neo N3 native contracts.

The current architecture suffers from two structural debts: a **god-module CLI** (151 files mixing bytecode, manifest, optimizer, and tests) and a **highly coupled runtime** (167 files with 7+ nesting levels and bidirectional dependencies between 4 subsystems). These debts slow development velocity and make safe refactoring risky.

We propose a **hexagonal target architecture** with 4 bounded contexts (Parser, IR, Codegen, Manifest) inside the compiler core, a separate Runtime bounded context communicating via a port interface, and trait-based extension points for Solidity versions, native contracts, and optimizer passes.

The migration uses the **strangler fig pattern** across 5 phases, each shipping independently with zero test regressions. The public API (`compile_contracts`, `disassemble_neovm_bytecode`) stays stable throughout.

---

## 2. Current State Analysis

### 2.1 Module Inventory

| Module | Files | LOC (approx) | Role |
|--------|-------|-------------|------|
| `cli` | 151 | ~27K | Compilation entry, bytecode, manifest, optimizer, tests |
| `runtime` | 167 | ~17K | NeoVM execution simulator (test-only, default-enabled) |
| `ir` | 111 | ~20K | Intermediate representation construction + lowering |
| `solidity` | 40 | ~7K | Inheritance, modifiers, sibling merge analysis |
| `frontend` | 7 | ~1.5K | Solidity parsing, pragma validation |
| `neo` | 7 | ~1.5K | NEF/manifest building, method tokens |
| `opcode` | 3 | ~0.5K | NeoVM opcode enum definitions |
| `type_system` | 3 | ~0.5K | Type system primitives |
| `storage_key` | 2 | ~0.3K | Storage key utilities |
| `interop` | 1 | ~0.2K | Interop definitions |
| `semantic_model` | 1 | ~0.3K | Semantic model |
| `utils` | 1 | ~0.2K | Shared utilities |
| **Total** | **503** | **~90.5K** | |

### 2.2 Compilation Pipeline

```
Source → frontend (parse) → solidity (analyse) → ir (lower) → ir (optimize)
      → cli/bytecode (codegen) → cli/manifest (NEF+manifest) → Output
```

The pipeline is one-directional (no back-edges), which is a strength. However, the last two stages live inside the `cli` module alongside test infrastructure and the standard-JSON input processor, creating a god module.

### 2.3 Architectural Debt Catalog

#### Debt 1: CLI God Module (Severity: HIGH)

**Problem**: The `cli` module (151 files) contains 6 distinct responsibilities:
- Compilation orchestration (`cli_compile/`)
- Bytecode assembly (`cli/bytecode/`)
- Manifest generation (`cli/cli_parts/cli_manifest/`)
- IR optimization (`cli/ir_optimize/`)
- Standard JSON processing (`cli/standard_json/`)
- Integration tests (`cli/tests/` — 16+ test directories)

**Impact**: Any change to bytecode generation risks touching test infrastructure. Import paths are long and brittle. New developers can't find where compilation logic lives.

#### Debt 2: Runtime Deep Coupling (Severity: HIGH)

**Problem**: The runtime module (167 files, 25 subdirectories) has 4 subsystems with bidirectional dependencies:
- `bridge` ↔ `execution` — bridge delegates to execution, execution calls back to bridge for EVM compatibility
- `execution` ↔ `state` — execution mutates state, state provides execution context
- `state` ↔ `storage` — state persists to storage, storage notifies state of changes

Nesting depth reaches 7+ levels (e.g., `runtime/execution/execution_impl_part2_native/contract_management.rs`).

**Impact**: Cannot modify one subsystem without understanding all four. The `execution_impl_part2_native` naming convention suggests the module was already split but the split is along implementation-part boundaries rather than domain boundaries.

#### Debt 3: Monolithic Files (Severity: MEDIUM)

**Problem**: 13 files exceed 800 lines:

| File | Lines | Type |
|------|-------|------|
| `ir/ir_statements/assembly.rs` | 1440 | Monolithic match chain |
| `runtime/.../stdlib.rs` | 1372 | Monolithic match chain |
| `solidity/solidity_analyse.rs` | 1207 | Monolithic pipeline |
| `ir/.../lower_assignment.rs` | 1140 | Monolithic lowering |
| `ir/.../low_level.rs` | 1103 | Monolithic dispatch |
| `frontend/frontend_parse.rs` | 1037 | Monolithic parser |
| `ir/.../abi_encode.rs` | 1017 | Monomorphic encoder |
| `ir/.../abi_decode.rs` | 979 | Monomorphic decoder |
| `ir/ir_context/builtins/resolve.rs` | 944 | Monolithic resolver |
| `ir/ir_expressions/arrays.rs` | 918 | Monolithic handler |

**Impact**: Files too large to review effectively. Merge conflicts are frequent. Changes in one branch conflict with changes in another.

#### Debt 4: No Extension Points (Severity: MEDIUM)

**Problem**: Adding a new Solidity version requires changes across `frontend` (parser version gate), `solidity` (analysis rules), `ir` (lowering), and `devpack` (Solidity libraries). There's no trait or plugin mechanism to isolate version-specific behavior.

Similarly, adding a new NeoVM native contract requires changes in `runtime/execution/execution_impl_part2_native/` (handler implementation), `devpack/contracts/native/` (Solidity wrapper), `devpack/contracts/NativeContracts.sol` (hash constant), and `runtime/spec/` (specification table).

**Impact**: Feature additions are spread across the codebase, making them error-prone and hard to review.

---

## 3. Target Architecture Design

### 3.1 Design Principles

1. **One-way dependency flow** — Parser → IR → Codegen → Manifest. No back-edges. Each context depends only on the previous one and the shared kernel.
2. **Bounded contexts** — Each major compilation stage is a self-contained context with clear boundaries. Contexts communicate through well-defined data structures, not direct function calls into internals.
3. **Port/Adapter for Runtime** — The runtime is a separate bounded context that communicates with the compiler core through a port (trait). This allows the runtime to be compiled out entirely for production builds.
4. **Trait-based extension** — Solidity version support, native contract handlers, and optimizer passes are defined as traits. Implementations are registered, not hard-coded.
5. **Shared kernel immutability** — The shared kernel (opcodes, types, spec tables) contains only immutable primitives. No business logic. Any context can depend on it, but it depends on nothing.

### 3.2 Bounded Contexts

#### Context 1: Parser (frontend + solidity)

**Responsibility**: Parse Solidity source into a contract metadata tree.

**Current state**: `frontend` (7 files) + `solidity` (40 files) = 47 files.

**Target state**: Merge into a single `parser` context with clear sub-modules:
- `parser/lexer` — token stream (from `frontend`)
- `parser/ast` — AST construction (from `frontend`)
- `parser/analysis` — inheritance, modifiers, sibling merge (from `solidity`)
- `parser/version` — Solidity version validation and feature gating (new)

**Extension point**: `SolidityVersion` trait — each Solidity version implements version-specific parsing rules, pragma validation, and feature flags.

**Dependencies**: Shared kernel only.

#### Context 2: IR (ir)

**Responsibility**: Lower contract metadata into an intermediate representation with basic blocks and instructions.

**Current state**: 111 files across 8 sub-modules.

**Target state**: Keep the existing sub-module structure but flatten the deeper nesting:
- `ir/lower` — expression and statement lowering
- `ir/types` — IR type system
- `ir/context` — builtin resolution
- `ir/module` — IR module construction

**Extension point**: `IrLoweringPass` trait — pluggable lowering rules for new Solidity constructs.

**Dependencies**: Parser (consumes `ContractMetadata`), Shared kernel.

#### Context 3: Codegen (extracted from cli)

**Responsibility**: Convert IR modules into NeoVM bytecode.

**Current state**: Lives inside `cli/bytecode/` (40+ files) and `cli/cli_parts/cli_compile/compile.rs`.

**Target state**: Extract into a standalone `codegen` context:
- `codegen/assembly` — bytecode assembly (from `cli/bytecode/`)
- `codegen/dispatch` — function selector and dispatch (from `cli/bytecode/bytecode_builtins/`)
- `codegen/storage` — storage layout (from `cli/bytecode/bytecode_helpers/storage/`)
- `codegen/disasm` — disassembler (from `cli/bytecode/bytecode_disasm/`)

**Dependencies**: IR (consumes `ir::Module`), Shared kernel.

#### Context 4: Manifest (extracted from cli)

**Responsibility**: Generate NEF files and contract manifests.

**Current state**: Lives inside `cli/cli_parts/cli_manifest/` and `src/neo/`.

**Target state**: Merge into a single `manifest` context:
- `manifest/nef` — NEF file building (from `src/neo/`)
- `manifest/permissions` — manifest permission management (from `cli/cli_parts/cli_manifest/permissions/`)
- `manifest/metadata` — contract metadata serialization (from `cli/cli_parts/cli_compile/`)

**Dependencies**: Codegen (consumes bytecode output), Shared kernel.

#### Context 5: Optimizer (extracted from cli)

**Responsibility**: Apply optimization passes to IR and bytecode.

**Current state**: Lives inside `cli/ir_optimize/` (small, ~5 files).

**Target state**: Extract into a standalone `optimizer` context:
- `optimizer/passes` — IR optimization passes (constant folding, dead code elimination)
- `optimizer/peephole` — bytecode peephole optimization

**Extension point**: `OptimizerPass` trait — each optimization is a pluggable pass that can be enabled/disabled via the optimizer level.

**Dependencies**: IR (consumes and produces `ir::Module`), Shared kernel.

#### Context 6: Runtime (separate bounded context)

**Responsibility**: Simulate NeoVM execution for testing and fuzzing.

**Current state**: 167 files with 4 bidirectionally-coupled subsystems.

**Target state**: Decouple into 4 independent components communicating through a port:
- `vm-core` — instruction execution engine (no dependencies on state/storage)
- `vm-state` — execution state (depends on vm-core types only, not implementation)
- `vm-storage` — storage simulation (depends on shared kernel only)
- `vm-bridge` — EVM compatibility layer (depends on vm-core + vm-state)

**Port interface**: `RuntimePort` trait — the compiler core defines this trait; the runtime implements it. The compiler never calls runtime internals directly.

**Dependencies**: Shared kernel, implements `RuntimePort` trait.

### 3.3 Shared Kernel

The shared kernel contains immutable primitives shared across all contexts:

| Component | Current Location | Content |
|-----------|-----------------|---------|
| `opcode` | `src/opcode/` (3 files) | NeoVM opcode enum + helpers |
| `type_system` | `src/type_system/` (3 files) | Type system primitives |
| `spec` | `src/runtime/spec/` | Opcode/syscall/native lookup tables |
| `storage_key` | `src/storage_key/` (2 files) | Storage key utilities |
| `utils` | `src/utils/` (1 file) | Shared utility functions |

**Rule**: The shared kernel depends on nothing. It contains no business logic, no I/O, no state mutation. Any context may depend on it.

### 3.4 Extension Points

#### SolidityVersion Trait

```rust
pub trait SolidityVersion: Send + Sync {
    fn version_string(&self) -> &str;
    fn supported_features(&self) -> SolidityFeatures;
    fn validate_pragma(&self, pragma: &str) -> Result<(), PragmaError>;
    fn parse_rules(&self) -> &ParseRules;
}
```

**Current**: Version-specific behavior is hard-coded in `frontend/frontend_parse.rs` (1037 lines).
**Target**: Each Solidity version (0.8.19, 0.8.20, ..., 0.8.28) implements the trait. The parser dispatches to the appropriate implementation based on the pragma directive.

#### NativeContractHandler Trait

```rust
pub trait NativeContractHandler: Send + Sync {
    fn contract_hash(&self) -> &str;
    fn handle_method(&self, method: &str, args: &[StackItem]) -> Result<StackItem, RuntimeError>;
    fn supported_methods(&self) -> &[&str];
}
```

**Current**: Native contract handlers are hard-coded in `runtime/execution/execution_impl_part2_native/` (11 files, one per contract).
**Target**: Each native contract implements the trait. Handlers are registered in a registry. The runtime dispatches to the registered handler.

#### OptimizerPass Trait

```rust
pub trait OptimizerPass: Send + Sync {
    fn name(&self) -> &str;
    fn min_level(&self) -> u8;
    fn run(&self, module: &mut ir::Module) -> Vec<OptimizationWarning>;
}
```

**Current**: Optimization passes are hard-coded in `cli/ir_optimize/`.
**Target**: Each pass implements the trait. Passes are registered and applied in order based on the optimizer level.

---

## 4. Evolution Roadmap

### 4.1 Strategy: Strangler Fig Pattern

We use the strangler fig pattern: new structure grows alongside the old, gradually replacing it. At no point is the system broken. Each phase is a complete, shippable release.

### 4.2 Phase 1: CLI Decomposition (v0.29.x) — LOW RISK

**Goal**: Extract codegen, optimizer, and manifest from the CLI god module into first-class modules.

**Steps**:
1. Create `src/codegen/` — move `cli/bytecode/` contents (excluding tests) into it
2. Create `src/optimizer/` — move `cli/ir_optimize/` contents into it
3. Create `src/manifest/` — move `cli/cli_parts/cli_manifest/` and `src/neo/` contents into it
4. Update `cli/` to import from the new modules — `cli` becomes a thin orchestration layer
5. Move `cli/tests/` to `tests/` at the crate root (standard Rust convention)

**Deliverables**:
- `cli` module reduced from 151 to ~40 files (orchestration + standard JSON only)
- 3 new top-level modules: `codegen`, `optimizer`, `manifest`
- All 965 tests pass unchanged
- Public API unchanged

**Risk mitigation**: This is mechanical file moving with import path updates. No logic changes. `cargo check` + `cargo test` after each sub-step.

**Estimated effort**: 2-3 days

### 4.3 Phase 2: Runtime Isolation (v0.30.x) — MEDIUM RISK

**Goal**: Decouple the 4 runtime subsystems and introduce a port interface.

**Steps**:
1. Define `RuntimePort` trait in `src/runtime/port.rs` — the contract between compiler and runtime
2. Break bidirectional `bridge ↔ execution` dependency:
   - Extract shared types into `src/runtime/types/` (already exists)
   - Bridge calls execution through a trait, not directly
3. Break `execution ↔ state` dependency:
   - Execution receives a `StateRef` trait, not a concrete `ExecutionState`
   - State implements `StateRef`
4. Break `state ↔ storage` dependency:
   - State receives a `StorageBackend` trait
   - Storage implements `StorageBackend`
5. Flatten directory structure: merge `execution_impl_part2_native/` into `execution/native/` (domain-based naming)
6. Reduce nesting from 7+ to ≤4 levels

**Deliverables**:
- Runtime reduced from 167 to ~80-100 files
- 4 independent components with trait-based communication
- Directory depth ≤4 levels
- `RuntimePort` trait defined and implemented
- All 965 tests pass

**Risk mitigation**: The VM bridging coupling is the highest-risk area. Approach incrementally — break one bidirectional dependency at a time, run full test suite after each. If a subsystem can't be cleanly separated, leave a thin adapter rather than forcing a break.

**Estimated effort**: 5-7 days

### 4.4 Phase 3: Extension Points (v0.31.x) — LOW RISK

**Goal**: Introduce trait-based plugins for Solidity versions, native contracts, and optimizer passes.

**Steps**:
1. Define `SolidityVersion` trait — extract version-specific logic from `frontend` and `solidity`
2. Implement trait for 0.8.19 (minimum supported) and 0.8.28 (maximum supported)
3. Define `NativeContractHandler` trait — extract handler logic from `runtime/execution/execution_impl_part2_native/`
4. Implement trait for all 11 native contracts
5. Create a `NativeContractRegistry` — handlers are registered, not hard-coded
6. Define `OptimizerPass` trait — extract pass logic from `optimizer/`
7. Implement trait for existing passes (constant folding, dead code elimination, etc.)

**Deliverables**:
- 3 traits defined and implemented
- Version-specific and contract-specific behavior isolated to trait implementations
- Adding a new Solidity version or native contract touches only the trait implementation + registration
- All 965 tests pass

**Risk mitigation**: This is additive — traits wrap existing logic. No existing code is removed until the trait implementation is verified to produce identical behavior.

**Estimated effort**: 3-4 days

### 4.5 Phase 4: Monolithic File Refactoring (v0.32.x) — MEDIUM RISK

**Goal**: Break down the 13 files exceeding 800 lines.

**Strategy**: Each file requires surgical refactoring, not mechanical splitting:

| File Type | Strategy |
|-----------|----------|
| Monolithic match chain (assembly.rs, stdlib.rs) | Split by domain — each match arm becomes a function in a domain-specific file |
| Monolithic pipeline (solidity_analyse.rs, frontend_parse.rs) | Extract pipeline stages into separate functions, then files |
| Monomorphic handler (abi_encode.rs, abi_decode.rs) | Split by type — each type's encoding/decoding in its own file |
| Monolithic dispatch (low_level.rs) | Split by dispatch category |

**Deliverables**:
- No file exceeds 800 lines
- Each split file has a clear single responsibility
- All 965 tests pass

**Risk mitigation**: Refactor one file at a time. After each split, verify with `cargo test`. The assembly.rs match chain is the highest risk — its arms have implicit dependencies (shared locals, fallthrough behavior).

**Estimated effort**: 4-5 days

### 4.6 Phase 5: Crate Split (v1.0) — HIGH RISK (OPTIONAL)

**Goal**: Split the single crate into a Cargo workspace with separate crates.

**Proposed workspace layout**:
```
neo-devpack-solidity/
├── neo-vm-core/          # Shared kernel (opcode, types, spec)
├── neo-devpack-parser/   # Parser bounded context
├── neo-devpack-ir/       # IR bounded context
├── neo-devpack-codegen/  # Codegen bounded context
├── neo-devpack-optimizer/# Optimizer bounded context
├── neo-devpack-manifest/ # Manifest bounded context
├── neo-devpack-runtime/  # Runtime bounded context (optional feature)
├── neo-devpack-cli/      # CLI binary (depends on all above)
└── neo-devpack-test/     # Integration tests
```

**When to do this**:
- Compilation time exceeds 60 seconds for incremental builds
- Team grows beyond 3 active developers
- External tools need to depend on individual compiler stages (e.g., a language server depending on the parser only)

**When NOT to do this**:
- Current compilation time is acceptable (~15-20s incremental)
- Small team (1-2 developers)
- No external consumers need stage-level dependencies

**Risk mitigation**: This phase is optional. If the team is small and compilation is fast, skip it. The internal module boundaries established in Phases 1-4 provide the same logical separation without the workspace overhead.

**Estimated effort**: 3-5 days (if undertaken)

---

## 5. Architecture Decision Records

### ADR-001: Hexagonal Architecture for Compiler Core

**Status**: Proposed

**Context**: The current architecture has 11 top-level modules with unclear boundaries. The CLI module (151 files) contains 6 distinct responsibilities. Developers can't easily determine where compilation logic lives versus test infrastructure versus optimization.

**Decision**: Adopt a hexagonal architecture for the compiler core with 4 bounded contexts (Parser, IR, Codegen, Manifest) plus a separate Optimizer context. Each context has a clear single responsibility and communicates with the next through well-defined data structures.

**Options considered**:
1. **Hexagonal (chosen)** — Clear boundaries, one-way flow, testable contexts
2. **Layered** — Simpler but doesn't address the god-module problem (layers can still be thick)
3. **Microservices** — Overkill for a compiler; no network boundary needed
4. **Status quo** — Unacceptable; debt continues to compound

**Consequences**:
- **Easier**: Understanding where to make changes, testing individual contexts, onboarding new developers
- **Harder**: More modules to navigate initially, slightly more boilerplate for context boundaries
- **Trade-off**: More structure upfront in exchange for long-term maintainability

### ADR-002: Runtime as Separate Bounded Context with Port Interface

**Status**: Proposed

**Context**: The runtime module (167 files, 17K LOC) is test-only infrastructure that's compiled by default. It has bidirectional coupling between 4 subsystems. The compiler core doesn't need the runtime for production builds — only tests and fuzzing do.

**Decision**: Make the runtime a separate bounded context that communicates with the compiler core through a `RuntimePort` trait. The runtime is gated behind the `runtime` feature flag (already partially implemented). The compiler core defines the port; the runtime implements it.

**Options considered**:
1. **Port/adapter (chosen)** — Clean separation, runtime can be compiled out, testable in isolation
2. **Direct calls with feature flag** — Current approach; coupling remains even when compiled out
3. **Separate crate** — Strongest isolation but adds workspace overhead; defer to Phase 5
4. **Delete the runtime** — Too risky; 965 tests depend on it for verification

**Consequences**:
- **Easier**: Production builds without runtime overhead, testing compiler and runtime independently, evolving runtime without compiler changes
- **Harder**: One level of indirection for runtime calls, port trait must be maintained
- **Trade-off**: Slight runtime overhead (trait dispatch) in exchange for compile-time isolation

### ADR-003: Trait-Based Extension Points

**Status**: Proposed

**Context**: Adding new Solidity version support or native contract handlers requires changes across 4+ modules. There's no mechanism to isolate version-specific or contract-specific behavior.

**Decision**: Introduce three trait-based extension points:
- `SolidityVersion` — pluggable Solidity version support
- `NativeContractHandler` — pluggable native contract implementations
- `OptimizerPass` — pluggable optimization passes

Each extension is a trait implementation registered in a registry. The compiler dispatches to the registered implementation.

**Options considered**:
1. **Traits + registry (chosen)** — Compile-time safety, runtime flexibility, idiomatic Rust
2. **Plugin DLLs** — Overkill for a compiler; no dynamic loading needed
3. **Feature flags** — Already used for optional dependencies; doesn't scale to 11+ native contracts
4. **Hard-coded dispatch** — Current approach; doesn't scale

**Consequences**:
- **Easier**: Adding new versions/contracts/passes touches only the implementation + registration
- **Harder**: One level of indirection for dispatch, trait definitions must be maintained
- **Trade-off**: Slight dispatch overhead in exchange for extensibility

### ADR-004: Strangler Fig Migration Strategy

**Status**: Proposed

**Context**: The system has 965 passing tests and ~87% famous-contract compilation coverage. A big-bang rewrite would risk losing this coverage and introducing subtle semantic regressions. The team is small (1-2 developers).

**Decision**: Use the strangler fig pattern — new architecture grows alongside the old, gradually replacing it across 5 phases. Each phase ships independently with zero test regressions.

**Options considered**:
1. **Strangler fig (chosen)** — Preserves tests, incremental, low risk
2. **Big-bang rewrite** — Faster to target shape but high risk of regression
3. **Branch and merge** — Long-lived branch diverges; merge hell at the end
4. **Status quo** — Debt compounds; velocity decreases over time

**Consequences**:
- **Easier**: Safe migration, continuous delivery, no "big bang" risk
- **Harder**: Temporary coexistence of old and new patterns, some duplication during migration
- **Trade-off**: Longer migration time in exchange for zero-downtime refactoring

### ADR-005: Shared Kernel Immutability Rule

**Status**: Proposed

**Context**: The shared kernel (opcodes, types, spec tables) is referenced by all contexts. If the kernel contains business logic or mutable state, every context becomes coupled to that logic.

**Decision**: The shared kernel contains only immutable primitives: enum definitions, constant tables, type definitions, and pure functions. No I/O, no state mutation, no business logic. The kernel depends on nothing.

**Consequences**:
- **Easier**: Any context can depend on the kernel without coupling concerns
- **Harder**: Some utility functions that could live in the kernel must live in individual contexts instead
- **Trade-off**: Minor code duplication risk in exchange for clean dependency graph

---

## 6. Quality Attribute Analysis

### 6.1 Scalability

**Current**: Single crate, single compilation unit. Incremental builds ~15-20s.

**Target**: Internal module boundaries enable parallel compilation if Phase 5 (crate split) is undertaken. Without Phase 5, compilation time stays similar but logical separation improves.

**Risk**: If the team grows or compilation time increases, Phase 5 can be triggered. Without Phase 5, the system still scales to ~2-3 developers.

### 6.2 Maintainability

**Current**: 13 files >800 lines. CLI god module. Runtime deep coupling.

**Target**: No file >800 lines. 6 bounded contexts with clear boundaries. Extension points isolate version/contract-specific logic.

**Measurement**: Track file count per module, max file size, and coupling metrics (imports per module) over time.

### 6.3 Testability

**Current**: 965 tests, ~87% compilation coverage. Tests are mostly integration-level (compile famous contracts and verify output).

**Target**: Each bounded context can be tested independently. Extension point traits enable mock implementations for unit testing.

**Risk**: During Phase 2 (runtime isolation), the runtime tests must be preserved. The port trait must not change the test surface.

### 6.4 Observability

**Current**: `verbose` flag prints diagnostic messages. No structured logging.

**Target**: Each bounded context emits structured diagnostic events. The CLI orchestrator collects and formats them. This enables:
- Per-stage timing metrics
- IR dump at each optimization pass
- Bytecode size tracking
- Warning/error structured reporting

**Trade-off**: Structured logging adds a small dependency (e.g., `tracing`) but enables much better debugging and profiling.

---

## 7. Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| Phase 2 runtime decoupling breaks VM semantics | Medium | High | Break one dependency at a time; full test suite after each; keep adapter if clean break isn't possible |
| Phase 4 file splitting introduces subtle bugs | Medium | Medium | One file at a time; diff compiled bytecode before/after each split |
| Extension point traits add dispatch overhead | Low | Low | Traits are `Send + Sync`; use static dispatch (`impl Trait`) where possible |
| Migration stalls mid-phase | Medium | Medium | Each phase is shippable independently; no phase depends on the next being complete |
| Phase 5 crate split breaks external consumers | Low | High | Public API stays stable; crate split is internal; `neo-devpack-solidity` remains the entry point |

---

## 8. Success Metrics

| Metric | Current | Target | Phase |
|--------|---------|--------|-------|
| Max file size | 1440 lines | <800 lines | Phase 4 |
| CLI module file count | 151 | ~40 | Phase 1 |
| Runtime module file count | 167 | ~80-100 | Phase 2 |
| Max directory nesting | 7+ levels | ≤4 levels | Phase 2 |
| Bidirectional dependencies | 3 pairs | 0 | Phase 2 |
| Extension point traits | 0 | 3 | Phase 3 |
| Test count | 965 | ≥965 | All phases |
| Public API stability | Stable | Stable | All phases |
| Incremental build time | ~15-20s | ≤20s | All phases |

---

## 9. Appendix: Module Migration Map

| Current Location | Target Location | Phase |
|-----------------|----------------|-------|
| `src/cli/bytecode/` | `src/codegen/` | 1 |
| `src/cli/ir_optimize/` | `src/optimizer/` | 1 |
| `src/cli/cli_parts/cli_manifest/` | `src/manifest/` | 1 |
| `src/neo/` | `src/manifest/` | 1 |
| `src/cli/tests/` | `tests/` | 1 |
| `src/frontend/` + `src/solidity/` | `src/parser/` | 3 (merge) |
| `src/runtime/bridge/` | `src/runtime/vm_bridge/` | 2 |
| `src/runtime/execution/` | `src/runtime/vm_core/` | 2 |
| `src/runtime/state/` | `src/runtime/vm_state/` | 2 |
| `src/runtime/storage/` | `src/runtime/vm_storage/` | 2 |
| `src/opcode/` + `src/type_system/` + `src/runtime/spec/` | `src/kernel/` | 5 (optional) |

---

*This document should be reviewed and updated at the completion of each phase.*
