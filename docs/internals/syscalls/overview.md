---
title: "Syscalls Overview"
description: "Overview from Syscalls."
---

# Syscalls Overview

[Back to Syscalls](/internals/syscalls)

## Overview

Syscalls are invoked via the `SYSCALL` opcode followed by a 4-byte identifier. The identifier is the first 4 bytes of `SHA256(syscall_name_string)`. For example, `System.Storage.Get` has the ID derived from `SHA256("System.Storage.Get")`.

The compiler maps Solidity patterns to the appropriate syscalls during code generation. State variable reads become `System.Storage.Get`, event emissions become `System.Runtime.Notify`, and cross-contract calls become `System.Contract.Call`.

The devpack provides two levels of access:

- **Direct wrappers** — `Syscalls.sol` exposes the supported syscall surface as typed Solidity functions.
- **Ergonomic libraries** — `Storage.sol`, `Runtime.sol`, and `Neo.sol` provide higher-level APIs built on top of the raw syscalls.

```solidity
import "devpack/contracts/Syscalls.sol";

// Direct syscall access
address caller = Syscalls.getCallingScriptHash();

// Or use the ergonomic wrapper
import "devpack/libraries/Runtime.sol";
bool authorized = Runtime.checkWitness(caller);
```
