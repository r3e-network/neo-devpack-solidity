---
title: "Testing Contracts: E2E Compilation Tests"
description: "E2E Compilation Tests from Testing Contracts."
---

# E2E Compilation Tests

[Back to Testing Contracts](/basics/testing-contracts)

## Overview

The E2E test suite compiles every example contract through the full pipeline and validates the output:

```bash
cargo test --test e2e_compilation_tests
```

This workspace currently has 80 E2E test functions. The exact count should be treated as source-controlled test inventory rather than an API promise; check it with:

```bash
rg -n "^\s*#\[test\]" tests/e2e_compilation_tests.rs | wc -l
```

The suite:

1. Parse each Solidity source file
2. Run it through the full compilation pipeline
3. Verify the NEF is valid (magic bytes, checksum)
4. Verify the manifest is valid JSON with correct ABI structure
5. Check that method signatures match the Solidity source

To run with parallel threads for faster execution:

```bash
cargo test --test e2e_compilation_tests -- --test-threads=4
```
