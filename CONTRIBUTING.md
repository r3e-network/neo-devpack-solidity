# Contributing to Neo DevPack Solidity

**Version**: 1.0 | **Date**: 2026-07-04

---

## Getting Started

### Prerequisites
- Rust 1.88+ (install via `rustup`)
- Foundry (for Solidity testing)
- Neo N3 node (for integration testing)

### Setup
```bash
git clone <repo-url>
cd neo-devpack-solidity
cargo build
cargo test
```

### Development Workflow
1. Create a branch: `git checkout -b feature/your-feature`
2. Make changes following the code review checklist
3. Run quality gates: `cargo check && cargo clippy && cargo test`
4. Create a PR with a clear description
5. Address review feedback
6. Squash and merge

---

## Code Standards

### Build Quality Gates (Must Pass)

```bash
# All three must pass clean before committing
cargo check    # 0 errors, 0 warnings
cargo clippy   # 0 warnings
cargo test     # all tests pass
```

### Error Handling Rules

**No `unwrap()` or `expect()` in production code.**

```rust
// BAD
let value = option.unwrap();
let result = function().expect("should work");

// GOOD
let value = option.ok_or_else(|| MyError::MissingValue)?;
let result = function().map_err(|e| MyError::Wrapped(e))?;
```

**No `eprintln!()` or `println!()` in production code.**

```rust
// BAD
eprintln!("warning: invalid input: {}", err);

// GOOD
tracing::warn!(error = %err, "invalid input");
```

**All error types use `thiserror`.**

```rust
#[derive(Debug, thiserror::Error)]
pub enum CompileError {
    #[error("contract '{0}' not found")]
    ContractNotFound(String),
    #[error("parse error: {0}")]
    Parse(#[from] ParseError),
}
```

### File Size Limits

- No file > 800 lines
- No function > 100 lines
- No match statement > 10 arms
- Module depth ≤ 4 levels from `src/`

### Documentation Requirements

- All `pub fn` must have `///` doc comments
- All `pub struct` and `pub enum` must have doc comments
- New modules must have `//!` module-level docs
- Doc comments explain "why", not just "what"

```rust
/// Build a Neo N3 contract manifest from compiled metadata.
///
/// Generates ABI methods, events, and permissions sections.
/// Custom overrides from NatSpec tags are applied after generation.
///
/// # Errors
/// Returns `ManifestError` if declared standards validation fails.
pub fn build_manifest(...) -> Result<Value, ManifestError> {
```

---

## Testing Standards

### Test Hierarchy

1. **Unit tests** — `#[cfg(test)] mod tests` in source files
   - Test individual functions in isolation
   - Required for all pure functions

2. **Integration tests** — `tests/` directory
   - Test compilation of real Solidity contracts
   - Test runtime execution of compiled bytecode

3. **Property-based tests** — `tests/fuzz_tests/`
   - Test invariants with `proptest`
   - Required for IR transforms

4. **Benchmarks** — `benches/`
   - Track compilation time
   - Required for performance-critical changes

### Test Naming

```rust
// GOOD — describes behavior
#[test]
fn constant_fold_add_with_two_integers() { }

#[test]
fn manifest_includes_transfer_event_for_nep17() { }

// BAD — describes implementation
#[test]
fn test1() { }

#[test]
fn test_function() { }
```

### Test Structure (Arrange-Act-Assert)

```rust
#[test]
fn optimizer_removes_dead_code_after_return() {
    // Arrange
    let mut block = ir::BasicBlock {
        instructions: vec![
            ir::Instruction::Return,
            ir::Instruction::PushLiteral(ir::LiteralValue::Integer(1.into())),
        ],
    };

    // Act
    prune_after_terminator(&mut block, &HashSet::new());

    // Assert
    assert_eq!(block.instructions.len(), 1);
    assert!(matches!(block.instructions[0], ir::Instruction::Return));
}
```

---

## Git Conventions

### Branch Naming
- `feature/description` — new features
- `fix/description` — bug fixes
- `refactor/description` — code refactoring
- `docs/description` — documentation only

### Commit Messages
```
type: short description

Optional body explaining why (not what).
```

Types: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`

```
feat: add ManifestError type to decouple manifest from CLI

The manifest module was returning CompileError, creating a circular
dependency. This introduces a manifest-specific error type and maps
it at the CLI boundary.
```

### PR Description Template
```
## What
Brief description of the change.

## Why
Why this change is needed (context, problem solved).

## Testing
- cargo check: 0 errors
- cargo clippy: 0 warnings
- cargo test: N tests pass
- Manual: (if applicable)

## Breaking Changes
- (list any breaking changes, or "None")
```

---

## Architecture Decisions

### When to Write an ADR
- Adding a new top-level module
- Changing the public API
- Introducing a new dependency
- Changing the compilation pipeline
- Modifying the error handling strategy

### ADR Template
```markdown
# ADR-XXX: [Decision Title]

## Status
Proposed | Accepted | Deprecated | Superseded

## Context
What problem motivates this decision?

## Decision
What is the change?

## Consequences
What becomes easier or harder?
```

---

## Module Organization

### Current Module Layout (v0.30.0)
```
src/
├── frontend/      # Solidity parsing
├── solidity/      # Solidity analysis
├── ir/            # Intermediate representation
├── codegen/       # NeoVM bytecode generation
├── optimizer/     # IR optimization passes
├── manifest/      # Manifest generation
├── runtime/       # NeoVM execution simulator
├── cli/           # CLI orchestration + tests
├── neo/           # NEF building/parsing
├── opcode/        # NeoVM opcode definitions
├── type_system/   # Type system primitives
└── ...
```

### Dependency Rules
1. **One-way flow**: Parser → IR → Codegen → Manifest (no back-edges)
2. **Shared kernel**: opcode, type_system — immutable primitives only
3. **No circular deps**: If module A imports B, B must not import A
4. **CLI is orchestration**: CLI calls other modules, other modules don't call CLI

---

## Getting Help

- **Code review**: All PRs reviewed by senior developer
- **Pair programming**: 2 sessions/week available
- **Tech talks**: Weekly 30-min knowledge sharing
- **Questions**: Ask in PR comments or team chat
