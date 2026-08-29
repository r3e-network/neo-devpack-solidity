---
title: "Testing Contracts: Code Coverage"
description: "Code Coverage from Testing Contracts."
---

# Code Coverage

[Back to Testing Contracts](/basics/testing-contracts)

## Overview

Generate coverage reports using `cargo-tarpaulin`:

```bash
# Install tarpaulin (one-time)
cargo install cargo-tarpaulin

# Generate HTML coverage report
make coverage
# Report is at coverage/tarpaulin-report.html

# Generate LCOV for CI
make coverage-ci
# Report is at coverage/lcov.info

# Check minimum coverage threshold (70%)
make check-coverage
```
