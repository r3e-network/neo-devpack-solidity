# Neo DevPack for Solidity Testing Guide

## Overview

This document describes the actual testing infrastructure available in the Neo DevPack for Solidity project.

## Quick Start

### Running Tests

```bash
# Run all Rust tests (unit + integration)
cargo test --workspace

# Run tests with output
cargo test -- --nocapture

# Run specific test file
cargo test runtime_flow_tests

# Run release build tests
cargo test --release
```

### Testing Solidity Contracts with `neo-test`

`neo-test` is a Foundry-style test runner for Solidity contracts that executes
on the in-tree NeoVM (no node required). It supports `test*`/`testFail*`
functions, `setUp()`, per-test isolation, cheatcodes (`vm.prank`, `vm.warp`,
`vm.roll`, `vm.deal`, `vm.expectRevert`, ...), decoded reverts/Panic codes,
`console.log`, and gas reporting:

```bash
# Build the runner
cargo build --release --bin neo-test

# Run Solidity tests (scans ./test, then ., for *.t.sol when no path is given)
./target/release/neo-test test/Counter.t.sol --gas
```

See [`docs/TESTING.md`](docs/TESTING.md) for the full guide (assertions,
cheatcodes, `neo-std/Test.sol`, and `neo-forge test` integration).

### Running Neo-Express Smoke Tests

The project includes comprehensive deployment tests using Neo-Express:

```bash
# Basic deployment test
make test-deploy-smoke

# CALLT optimization test
make test-deploy-callt-smoke

# Constructor argument test
make test-deploy-constructor-smoke

# Contract update test
make test-deploy-update-smoke

# Manifest permissions test
make test-deploy-permissions-smoke

# ABI encoding test
make test-deploy-encoding-smoke

# All smoke tests
make test-deploy-smoke-full
```

## Test Structure

### Unit Tests (`src/` and `tests/`)

Runtime primitive tests located in `tests/`:

- `runtime_account_tests.rs` - Account operations
- `runtime_array_tests.rs` - Array manipulation
- `runtime_assert_tests.rs` - Assertion handling
- `runtime_buffer_tests.rs` - Buffer operations
- `runtime_contract_management_tests.rs` - Contract management
- `runtime_crypto_hash_tests.rs` - Cryptographic operations
- `runtime_edge_tests.rs` - Edge cases
- `runtime_exception_tests.rs` - Exception handling
- `runtime_flow_tests.rs` - Control flow
- `runtime_gas_tests.rs` - Gas estimation
- `runtime_logic_tests.rs` - Logical operations
- `runtime_return_tests.rs` - Return values
- `runtime_size_tests.rs` - Size operations
- `runtime_storage_iterator_tests.rs` - Storage iterators
- `runtime_syscall_tests.rs` - Syscall interface
- `runtime_value_map_tests.rs` - Map operations

### Compiler Tests (`src/cli/tests/`)

Compiler integration tests for:

- Selector generation
- Low-level calls
- Control flow
- State mutability
- Storage layout
- Native calls
- And more

### Neo-Express Smoke Tests (`examples/`)

Real-world deployment tests using Neo-Express:

- `test_neoxp_deploy.sh` - Basic deployment
- `test_neoxp_callt_smoke.sh` - CALLT optimization
- `test_neoxp_constructor_smoke.sh` - Constructor arguments
- `test_neoxp_update_smoke.sh` - Contract updates
- `test_neoxp_permissions_smoke.sh` - Manifest permissions
- `test_neoxp_encoding_smoke.sh` - ABI encoding
- `test_neoxp_abortmsg_smoke.sh` - Error messages
- `test_neoxp_lowlevel_call_smoke.sh` - Low-level calls
- `test_neoxp_view_readonly_call_smoke.sh` - View functions
- `test_neoxp_compound_assignment_smoke.sh` - Compound operators
- `test_neoxp_struct_array_element_smoke.sh` - Struct arrays
- `test_neoxp_nested_struct_storage_smoke.sh` - Nested structs
- `test_neoxp_delete_smoke.sh` - Delete operations

## Building the Compiler

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Install locally
cargo install --path .
```

## Compilation Examples

```bash
# Compile a single contract
./target/release/neo-solc examples/SimpleStorage.sol -O2 -o build/SimpleStorage

# Compile all examples
mkdir -p build/examples
for f in examples/*.sol; do
  ./target/release/neo-solc "$f" -I devpack -O2 -o "build/examples/$(basename "$f" .sol)"
done

# With CALLT optimization
./target/release/neo-solc examples/ERC20Token.sol --callt -O3 -o build/ERC20Token
```

## Known Gaps

See `docs/NEO_VM_PARITY_TODO.md` for a comprehensive list of runtime parity gaps and planned improvements.

## Contributing Tests

When adding new tests:

1. Unit tests go in appropriate `tests/*.rs` files
2. Compiler integration tests go in `src/cli/tests/`
3. Add smoke tests to `examples/` for end-to-end validation
4. Follow Rust testing conventions
5. Document test purpose with comments

## CI/CD

Tests run automatically on:

- Pull requests
- Pushes to main branch
- All branches in CI matrix (Linux, macOS, Windows)

See `.github/workflows/` for CI configuration.
