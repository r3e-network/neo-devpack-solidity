---
title: "Syscalls: Devpack Wrapper Reference"
description: "Devpack Wrapper Reference from Syscalls."
---

# Devpack Wrapper Reference

[Back to Syscalls](/internals/syscalls)

The devpack provides syscall access at two abstraction levels.

## Low-Level: `Syscalls.sol`

Located at `devpack/contracts/Syscalls.sol`. Provides typed wrappers for the Neo N3 syscall surface, plus convenience functions for native contract calls.

Key features:

- Typed wrappers for the registered syscall surface exposed as `internal` Solidity functions
- Data structures: `Block`, `Transaction`, `Signer`, `StorageContext`, `Iterator`, `Notification`
- Constants: trigger types, witness scopes, witness conditions, named curve hashes
- Native contract script hash constants for the runtime-supported native contracts: NEO, GAS, ContractManagement, Policy, Oracle, RoleManagement, Notary, Treasury, Ledger, CryptoLib, and StdLib

## High-Level: Ergonomic Libraries

| Library   | File                            | Built On                                   |
| --------- | ------------------------------- | ------------------------------------------ |
| `Storage` | `devpack/libraries/Storage.sol` | `System.Storage.*` syscalls                |
| `Runtime` | `devpack/libraries/Runtime.sol` | `System.Runtime.*` syscalls                |
| `Neo`     | `devpack/libraries/Neo.sol`     | Multiple syscall categories + native calls |

These libraries are compiler intrinsics: only the members the compiler can lower faithfully are exposed. The `Storage` library wraps context handling around `put`/`get`/`remove`/`find` (plus `putContractMetadata`); higher-level patterns such as batch operations, typed accessors, or prefix utilities should be implemented in contract code on top of these primitives.

```solidity
// Low-level: direct syscall
Syscalls.StorageContext memory ctx = Syscalls.getStorageContext();
Syscalls.storagePut(ctx, "key", abi.encode(42));

// High-level: ergonomic wrapper (context handled for you)
Storage.put("key", abi.encode(42));
bytes memory value = Storage.get("key");
```
