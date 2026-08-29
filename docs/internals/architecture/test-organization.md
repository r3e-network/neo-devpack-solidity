---
title: "Architecture: Test Organization"
description: "Test Organization from Architecture."
---

# Test Organization

[Back to Architecture](/internals/architecture)

## Overview

```
tests/
├── runtime_*.rs              # Runtime unit tests (opcode/syscall behavior)
├── e2e_compilation_tests.rs  # End-to-end: Solidity source -> NEF output
└── conformance_tests.rs      # Conformance against Neo N3 spec
```
