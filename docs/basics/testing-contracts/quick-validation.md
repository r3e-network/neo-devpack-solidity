---
title: "Testing Contracts: Quick Validation"
description: "Quick Validation from Testing Contracts."
---

# Quick Validation

[Back to Testing Contracts](/basics/testing-contracts)

## Overview

To test a contract you are writing, run its Foundry-style test file with `neo-test` — it compiles with `neo-solc` and executes `test*` / `testFail*` / `setUp()` on the in-tree NeoVM, with per-test isolation, decoded reverts, `console.log`, gas, and cheatcodes:

```bash
neo-test test/Counter.t.sol    # or `neo-test` to scan ./test for *.t.sol
neo-forge test                 # Neo Foundry delegates to the same runner
```

See [Writing Your Own Tests](/basics/testing-contracts/writing-your-own-tests) and the [full `neo-test` guide](/TESTING).

For fast feedback while working on the compiler itself:

```bash
# Run all Rust tests (unit + integration + E2E + conformance)
make test
# or equivalently:
cargo test --workspace
```

For a quick compilation sanity check (verifies NEF magic bytes and manifest structure):

```bash
bash examples/test_compilation.sh
```
