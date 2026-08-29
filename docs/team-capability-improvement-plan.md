# Team Capability Improvement Plan

**Version**: 1.0 | **Date**: 2026-07-04 | **Author**: Senior Developer

---

## 1. Current State Assessment

### 1.1 Code Quality Audit Results

| Dimension | Score | Status | Key Finding |
|-----------|-------|--------|-------------|
| Build hygiene | 8/10 | Good | 0 clippy warnings, 0 compiler warnings |
| Test coverage | 7/10 | Good | 965 tests, ~87% compilation coverage |
| Error handling | 7/10 | Fair | 6 `eprintln!` in production, 30+ `unwrap()` in non-test code |
| Architecture | 9/10 | Excellent | Hexagonal design, clear bounded contexts |
| Module structure | 9/10 | Excellent | CLI reduced 151→106 files, 3 modules extracted |
| Doc coverage | 5/10 | Poor | Only 41% of non-test files have doc comments |
| Unsafe code | 8/10 | Good | Only 2 `unsafe` blocks (runtime storage_ops.rs) |
| File size | 6/10 | Fair | 10 files still >800 lines |

### 1.2 Specific Issues Found

#### Issue 1: Production `eprintln!` Usage (6 files)

**Problem**: 6 production files use `eprintln!` for diagnostics instead of structured logging.

**Files affected**:
- `src/manifest/build.rs` — manifest override warnings
- `src/cli/cli_parts/cli_analyze.rs` — upgrade analysis output
- `src/cli/cli_parts/cli_run/standard_json.rs` — JSON processing errors
- `src/cli/cli_parts/cli_run/single_file.rs` — compilation messages
- `src/codegen/bytecode_core.rs` — method token warnings
- `src/neo_test_main.rs` — test runner output

**Impact**: Diagnostics can't be filtered, redirected, or structured. Makes tooling integration harder.

#### Issue 2: `unwrap()`/`expect()` in Production Code (30+ instances)

**Problem**: 30+ instances of `unwrap()` or `expect()` in non-test source code.

**Distribution**:
- `src/ir/ir_context/builtins/resolve.rs` — 10 instances
- `src/solidity/upgrade.rs` — 10 instances
- `src/cli/cli_parts/cli_compile/permissions.rs` — 7 instances
- `src/ir/ir_context/ctx_locals_scopes.rs` — 2 instances
- `src/optimizer/constant_folding.rs` — 2 instances (with `#[allow]`)
- `src/ir/ir_expressions/power.rs` — 1 instance
- `src/neo/contract_hash.rs` — 1 instance
- `src/runtime/execution/...` — 3 instances
- `src/manifest/build.rs` — 1 instance

**Impact**: Any `unwrap()` is a potential panic in production. Compiler should never crash on user input.

#### Issue 3: `unsafe` Blocks in Runtime (2 instances)

**Problem**: 2 `unsafe { ptr.as_mut() }` blocks in `src/runtime/execution/helpers/storage_ops.rs`.

**Context**: Used for borrowing a `*mut StorageHost` raw pointer in the streaming iterator. This is a legitimate use case for FFI, but should be audited for soundness.

**Impact**: Low risk (test-only runtime), but sets a bad precedent for the team.

#### Issue 4: Documentation Coverage (41%)

**Problem**: Only 176 of 429 non-test source files (41%) have any doc comments.

**Most affected areas**:
- `src/ir/` — 111 files, many undocumented lowering functions
- `src/runtime/` — 167 files, mostly undocumented execution internals
- `src/codegen/` — 29 files, bytecode generation undocumented

**Impact**: New team members can't understand the codebase without reading implementation. Makes onboarding expensive.

---

## 2. Improvement Plan — 4 Focus Areas

### Focus Area 1: Rust Error Handling (Week 1-2, Priority: HIGH)

#### Learning Objectives
- Understand why `unwrap()`/`expect()` in production is an anti-pattern
- Master `Result<T, E>` propagation and the `?` operator
- Learn structured error types with `thiserror`
- Practice replacing `eprintln!` with `tracing` macros

#### Action Items

**1.1 Replace all `eprintln!` with `tracing` macros**

```rust
// BAD — current pattern in manifest/build.rs
eprintln!("warning: ignoring @custom:{tag} because its value is not valid JSON: {err}");

// GOOD — structured logging with tracing
use tracing::warn;
warn!(tag = %tag, error = %err, "ignoring @custom tag: invalid JSON");
```

**1.2 Audit and fix all `unwrap()`/`expect()` in production code**

```rust
// BAD — current pattern in ir_context/builtins/resolve.rs
let neo_type = param.neo_type.as_ref().unwrap();

// GOOD — explicit error handling
let neo_type = param.neo_type.as_ref()
    .ok_or_else(|| ResolveError::MissingNeoType(param.name.clone()))?;
```

**1.3 Introduce `thiserror` for all error types**

```rust
// Current: ad-hoc string errors
return Err(format!("contract '{}' not found", name));

// Target: structured error types
#[derive(Debug, thiserror::Error)]
pub enum ResolveError {
    #[error("contract '{0}' not found")]
    ContractNotFound(String),
    #[error("missing neo type for parameter '{0}'")]
    MissingNeoType(String),
}
```

**1.4 Code review checklist**
- [ ] No `unwrap()` or `expect()` in non-test code (use `?` or `ok_or_else`)
- [ ] No `eprintln!` or `println!` in non-test code (use `tracing`)
- [ ] All error types implement `std::error::Error` (via `thiserror`)
- [ ] Error messages include context (what failed and why)

#### Expected Outcome
- 0 `unwrap()` in production code
- 0 `eprintln!` in production code
- All error types use `thiserror`
- Team understands Rust error handling idioms

---

### Focus Area 2: Testing Depth (Week 3-4, Priority: HIGH)

#### Learning Objectives
- Understand the difference between integration tests and unit tests
- Learn property-based testing with `proptest`
- Master benchmarking with `criterion`
- Practice test-driven refactoring

#### Action Items

**2.1 Add unit tests for pure functions**

Currently, 965 tests are mostly integration-level (compile famous contracts and verify output). We need unit tests for pure logic.

```rust
// Target: unit tests in src/optimizer/constant_folding.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fold_constant_add() {
        let mut block = ir::BasicBlock {
            instructions: vec![
                ir::Instruction::PushLiteral(ir::LiteralValue::Integer(1.into())),
                ir::Instruction::PushLiteral(ir::LiteralValue::Integer(2.into())),
                ir::Instruction::BinaryOp(ir::BinaryOperator::Add),
            ],
        };
        fold_constant_binary_ops(&mut block);
        assert_eq!(block.instructions.len(), 1);
        match &block.instructions[0] {
            ir::Instruction::PushLiteral(ir::LiteralValue::Integer(v)) => {
                assert_eq!(*v, BigInt::from(3));
            }
            _ => panic!("expected folded literal"),
        }
    }
}
```

**2.2 Property-based tests for IR transforms**

```rust
// Target: proptest in tests/optimizer_props.rs
proptest! {
    #[test]
    fn constant_fold_add_is_associative(a in -1000i64..1000, b in -1000i64..1000) {
        let result = evaluate_binary_literal(
            &ir::LiteralValue::Integer(a.into()),
            &ir::LiteralValue::Integer(b.into()),
            ir::BinaryOperator::Add,
        );
        prop_assert!(result.is_some());
    }
}
```

**2.3 Benchmark suite with criterion**

```rust
// Target: benches/compile.rs
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_compile(c: &mut Criterion) {
    c.bench_function("compile_simple_contract", |b| {
        b.iter(|| compile_contracts(SIMPLE_CONTRACT, false, 2))
    });
}
```

**2.4 Mutation testing**
- Use `cargo-mutants` to verify test effectiveness
- Target: mutation score >80%

#### Expected Outcome
- 1000+ tests (up from 965)
- Unit test coverage for all pure functions
- Property-based tests for IR transforms
- Benchmark suite tracking compilation time
- Mutation score >80%

---

### Focus Area 3: Documentation (Week 5-6, Priority: MEDIUM)

#### Learning Objectives
- Write effective rustdoc comments
- Document "why" not "what"
- Create module-level documentation
- Establish documentation standards

#### Action Items

**3.1 Document all `pub fn` (target: 80% coverage)**

```rust
// BAD — no documentation
pub(crate) fn build_manifest(
    metadata: &ContractMetadata,
    ir_module: &ir::Module,
    bytecode: &[u8],
    tokens: &[MethodToken],
) -> Result<Value, ManifestError> {

// GOOD — documented
/// Build a Neo N3 contract manifest from compiled metadata.
///
/// Generates the ABI methods, events, permissions, and supported standards
/// sections of the manifest. Manifest custom overrides from NatSpec tags
/// (`@custom:neo.manifest.*`) are applied after initial generation.
///
/// # Arguments
/// * `metadata` — Contract metadata (methods, events, state variables)
/// * `ir_module` — IR module for permission inference
/// * `bytecode` — Compiled bytecode for native call scanning
/// * `tokens` — Method tokens for permission validation
///
/// # Errors
/// Returns `ManifestError` if declared standards validation fails.
pub(crate) fn build_manifest(
    metadata: &ContractMetadata,
    ir_module: &ir::Module,
    bytecode: &[u8],
    tokens: &[MethodToken],
) -> Result<Value, ManifestError> {
```

**3.2 Module-level documentation**

Every `mod.rs` should start with:
```rust
//! Module name — one-line description
//!
//! Detailed explanation of the module's responsibility, architecture,
//! and key design decisions. Include references to related modules.
```

**3.3 Create CONTRIBUTING.md**

Document:
- Code review checklist
- Branch naming conventions
- Commit message format
- Testing requirements
- Documentation standards

**3.4 Architecture Decision Records**
- All major decisions documented as ADRs
- Template provided in `docs/adr/`
- Review ADRs in team meetings

#### Expected Outcome
- 80% doc coverage (up from 41%)
- All `pub fn` documented
- CONTRIBUTING.md created
- ADRs maintained for all major changes

---

### Focus Area 4: Code Craft (Week 7-8, Priority: MEDIUM)

#### Learning Objectives
- Surgical refactoring of monolithic functions
- Trait-based design for extensibility
- Unsafe code elimination
- Complexity budgeting

#### Action Items

**4.1 Split remaining 10 files >800 lines**

Each file requires surgical refactoring:

| File | Lines | Strategy |
|------|-------|----------|
| `stdlib.rs` | 1372 | Split `invoke_native_stdlib` into sub-dispatchers |
| `solidity_analyse.rs` | 1207 | Extract 5 pipeline stages into helper functions |
| `low_level.rs` | 1103 | Split by call type (staticcall, delegatecall, etc.) |
| `abi_encode.rs` | 1017 | Split by type category (uint, bytes, array, struct) |
| `abi_decode.rs` | 979 | Mirror abi_encode split |
| `resolve.rs` | 944 | Split by resolution category |
| `arrays.rs` | 918 | Split by array operation |
| `member_calls.rs` | 912 | Split by member type |
| `binary_u256_softarith.rs` | 897 | Split by operation |
| `return_lower.rs` | 866 | Split by return type |

**4.2 Extract trait extension points**

```rust
// Target: src/extension/solidity_version.rs
pub trait SolidityVersion: Send + Sync {
    fn version_string(&self) -> &str;
    fn supported_features(&self) -> SolidityFeatures;
    fn validate_pragma(&self, pragma: &str) -> Result<(), PragmaError>;
}
```

**4.3 Remove `unsafe` blocks**

```rust
// Current: unsafe raw pointer in storage_ops.rs
let storage = unsafe { ptr.as_mut() };

// Target: safe abstraction using Rc<RefCell<>> or Arc<Mutex<>>
let storage = self.storage_host.borrow_mut();
```

**4.4 Complexity budget**
- Cyclomatic complexity < 15 per function
- No function > 100 lines
- No match statement > 10 arms (extract sub-dispatchers)

#### Expected Outcome
- 0 files >800 lines
- 0 `unsafe` blocks
- Trait extension points defined
- Complexity budget enforced in CI

---

## 3. Code Review Checklist

### Mandatory Checks (must pass before merge)

- [ ] `cargo check` — 0 errors, 0 warnings
- [ ] `cargo clippy` — 0 warnings
- [ ] `cargo test` — all tests pass
- [ ] No `unwrap()` or `expect()` in non-test code
- [ ] No `eprintln!` or `println!` in non-test code
- [ ] No `unsafe` blocks without SAFETY comment
- [ ] All new `pub fn` have doc comments
- [ ] New functions < 100 lines (or justified)
- [ ] Error types implement `thiserror::Error`

### Recommended Checks (should pass)

- [ ] Unit tests for new pure functions
- [ ] Property-based tests for new transforms
- [ ] Module-level docs for new modules
- [ ] ADR for architectural changes
- [ ] Benchmark added for performance-critical code

---

## 4. Training Materials

### Week 1-2: Rust Error Handling
- **Reading**: [Rust Book Ch 9](https://doc.rust-lang.org/book/ch09-00-error-handling.html), `thiserror` docs
- **Exercise**: Replace all `unwrap()` in `resolve.rs` with proper error handling
- **Review**: Team code review session on error handling patterns

### Week 3-4: Testing Depth
- **Reading**: [Rust Testing Chapter](https://doc.rust-lang.org/book/ch11-00-testing.html), `proptest` docs
- **Exercise**: Write property-based tests for `constant_folding.rs`
- **Review**: Test effectiveness analysis with `cargo-mutants`

### Week 5-6: Documentation
- **Reading**: [rustdoc Book](https://doc.rust-lang.org/rustdoc/), internal `CONTRIBUTING.md`
- **Exercise**: Document all `pub fn` in `src/manifest/`
- **Review**: Documentation review session

### Week 7-8: Code Craft
- **Reading**: [Refactoring by Martin Fowler](https://refactoring.com/), internal ADRs
- **Exercise**: Surgically split `solidity_analyse.rs` into 5 stage functions
- **Review**: Refactoring technique workshop

---

## 5. Success Metrics

| Metric | Current | Target | Deadline |
|--------|---------|--------|----------|
| `unwrap()` in prod | 30+ | 0 | Week 2 |
| `eprintln!` in prod | 6 | 0 | Week 2 |
| Test count | 965 | 1000+ | Week 4 |
| Doc coverage | 41% | 80% | Week 6 |
| Files >800 lines | 10 | 0 | Week 8 |
| `unsafe` blocks | 2 | 0 | Week 8 |
| Clippy warnings | 0 | 0 (enforced in CI) | Week 1 |
| Mutation score | unknown | >80% | Week 4 |

---

## 6. Mentoring Approach

### Pair Programming Sessions
- 2 sessions per week, 2 hours each
- Focus on current focus area
- Senior developer reviews and guides

### Code Review Culture
- All PRs require review by senior developer
- Review comments focus on learning, not just correctness
- "Why" explanations required for all suggestions

### Knowledge Sharing
- Weekly 30-min tech talk by team member
- Topics rotate through focus areas
- Recordings shared for async learning

### Retrospective
- End of each 2-week focus area: team retrospective
- What worked, what didn't, what to adjust
- Update this plan based on feedback
