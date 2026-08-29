---
title: "Testing Contracts: Strict Compatibility Compilation Sweep"
description: "Strict Compatibility Compilation Sweep from Testing Contracts."
---

# Strict Compatibility Compilation Sweep

[Back to Testing Contracts](/basics/testing-contracts)

## Overview

This sweep compiles all devpack libraries and example contracts with strict settings to catch regressions:

```bash
make test-compile-strict
# or:
bash examples/test_strict_compatibility_sweep.sh
```

The sweep:

1. Compiles every `.sol` file in `devpack/` and `examples/new/`
2. Uses strict compiler flags
3. Reports any unexpected warnings or errors
4. Fails if `STRICT_SWEEP_FAIL_ON_UNEXPECTED_WARNINGS=1` is set (used in CI)
