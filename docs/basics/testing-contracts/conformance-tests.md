---
title: "Testing Contracts: Conformance Tests"
description: "Conformance Tests from Testing Contracts."
---

# Conformance Tests

[Back to Testing Contracts](/basics/testing-contracts)

## Overview

Conformance tests compare compiler output against reference Neo N3 implementations:

```bash
cargo test --test conformance_tests
```

The suite currently includes 40 test vectors and enforces a minimum 95.0% pass rate. Each vector specifies:

- Input Solidity source
- Expected NeoVM opcodes or behavior
- Expected manifest structure

Failing conformance tests indicate behavioral divergence from the Neo N3 reference. These are tracked in `docs/NEO_VM_PARITY_TODO.md`.

::: tip Conformance Gate
`cargo test --test conformance_tests` fails when the pass rate drops below the checked-in threshold. Check `docs/NEO_VM_PARITY_TODO.md` before relying on behavior that is still listed as a parity gap.
:::
