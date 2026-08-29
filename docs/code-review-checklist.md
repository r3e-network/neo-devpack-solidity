# Code Review Checklist

**Project**: Neo DevPack Solidity | **Version**: 1.0 | **Date**: 2026-07-04

---

## Mandatory Checks (Must Pass Before Merge)

### Build & Tests
- [ ] `cargo check` — 0 errors, 0 warnings
- [ ] `cargo clippy` — 0 warnings
- [ ] `cargo test` — all tests pass (965+ tests)
- [ ] `cargo test --no-run` — all test binaries compile

### Error Handling
- [ ] No `unwrap()` or `expect()` in non-test code
- [ ] No `panic!()`, `unimplemented!()`, or `todo!()` in non-test code
- [ ] No `eprintln!()` or `println!()` in non-test code (use `tracing`)
- [ ] All error types implement `std::error::Error` (via `thiserror`)
- [ ] Error messages include context (what failed and why)
- [ ] `Result<T, E>` used instead of `Option<T>` when failure is meaningful

### Safety
- [ ] No `unsafe` blocks without a `// SAFETY:` comment explaining soundness
- [ ] No raw pointer dereferences without justification
- [ ] No `as` casts that could truncate silently

### Code Organization
- [ ] No function > 100 lines (extract helpers if needed)
- [ ] No match statement > 10 arms (extract sub-dispatchers)
- [ ] No file > 800 lines (split into submodules)
- [ ] Cyclomatic complexity < 15 per function
- [ ] Module depth ≤ 4 levels from `src/`

### Documentation
- [ ] All new `pub fn` have `///` doc comments
- [ ] All new `pub struct` and `pub enum` have doc comments
- [ ] Module-level `//!` docs for new modules
- [ ] Doc comments explain "why", not just "what"

### Testing
- [ ] Unit tests for new pure functions (`#[cfg(test)] mod tests`)
- [ ] Integration tests for new compilation features
- [ ] Test names describe the behavior being tested
- [ ] No `#[ignore]` tests without a tracking issue

---

## Recommended Checks (Should Pass)

### Architecture
- [ ] No circular module dependencies (`mod A` depends on `mod B` depends on `mod A`)
- [ ] Public API changes documented in CHANGELOG.md
- [ ] Architectural changes documented as ADR
- [ ] No new `use super::*` glob imports in leaf modules (use explicit imports)

### Performance
- [ ] No `clone()` in hot paths (use borrows)
- [ ] No `Vec::new()` in loops without `with_capacity`
- [ ] No `String` concatenation in loops (use `format!` or `push_str`)
- [ ] Benchmark added for performance-critical changes

### Rust Idioms
- [ ] Use `?` operator instead of `match` for error propagation
- [ ] Use `if let` instead of `match` when only one arm matters
- [ ] Use `matches!()` macro instead of single-arm match
- [ ] Use `impl Trait` instead of `Box<dyn Trait>` where possible
- [ ] Use `&[T]` instead of `&Vec<T>` in function signatures
- [ ] Use `&str` instead of `&String` in function signatures

### Git Hygiene
- [ ] Commit messages follow conventional format
- [ ] One logical change per commit
- [ ] No `// WIP` or `// TODO` without tracking issue
- [ ] Branch name describes the change

---

## Review Comment Guidelines

### For Reviewers

**Good review comments**:
- "Consider using `?` here instead of `match` — it's more idiomatic and reduces nesting."
- "This function is 120 lines. Could the validation logic be extracted into a helper?"
- "What happens if `metadata.methods` is empty? Add a test for that case."
- "This `unwrap()` will panic if the parser returns an empty AST. Use `ok_or_else` instead."

**Bad review comments**:
- "Change this." (no explanation)
- "This is wrong." (no suggestion)
- "LGTM" (no actual review)
- Nits about style that clippy already catches

### For Authors

**Good PR descriptions**:
```
## What
Add `ManifestError` type to decouple manifest module from CLI.

## Why
The manifest module was returning `CompileError`, creating a circular
dependency. This extracts a manifest-specific error type and maps it
at the CLI boundary.

## Testing
- `cargo check` — 0 errors
- `cargo test` — 965 tests pass
- Manual: verified manifest generation still produces identical output
```

**Bad PR descriptions**:
- "Fixed stuff"
- "Updated code"
- (empty)
