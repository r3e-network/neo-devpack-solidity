---
title: "Syscalls: Storage Syscalls"
description: "Storage Syscalls from Syscalls."
---

# Storage Syscalls

[Back to Syscalls](/internals/syscalls)

## Storage API

Storage syscalls provide persistent key-value state for contracts. Each contract has its own isolated storage context.

| Syscall Name                        | Gas Cost | Description                       | Devpack Wrapper                        |
| ----------------------------------- | -------: | --------------------------------- | -------------------------------------- |
| `System.Storage.GetContext`         |        1 | Get current storage context       | `Syscalls.getStorageContext()`         |
| `System.Storage.GetReadOnlyContext` |        1 | Get read-only context             | `Syscalls.getReadOnlyStorageContext()` |
| `System.Storage.AsReadOnly`         |        1 | Convert context to read-only      | `Syscalls.storageAsReadOnly(ctx)`      |
| `System.Storage.Get`                |      100 | Read value by key                 | `Syscalls.storageGet(ctx, key)`        |
| `System.Storage.Put`                |    1,000 | Write key-value pair              | `Syscalls.storagePut(ctx, key, val)`   |
| `System.Storage.Delete`             |      100 | Delete key                        | `Syscalls.storageDelete(ctx, key)`     |
| `System.Storage.Find`               |      100 | Find by prefix (returns iterator) | `Syscalls.storageFind(ctx, prefix)`    |

::: warning No "Local" Storage Family
Neo N3 registers only the seven `System.Storage.*` syscalls above. A previous
devpack revision exposed `Syscalls.storage{Get,Put,Delete,Find}Local` wrappers
mapped to fictional `System.Storage.Local.*` syscalls; those names are absent
from Neo N3's interop table, so calls faulted on real nodes. The wrappers have
been removed, and the bundled runtime now treats these syscall names like any
other unknown syscall (FAULT), matching real-node behavior. Neo N3 storage contexts are already private to the owning
contract, so the context-based wrappers provide the same isolation.
:::

## Example Usage

```solidity
import "devpack/contracts/Syscalls.sol";

contract TokenVault {
    // The compiler lowers state variables to Storage.Get/Put automatically.
    // For manual control, use the Syscalls library directly:

    function manualStorageExample(bytes memory key, bytes memory value) internal {
        // Get the current contract's storage context
        Syscalls.StorageContext memory ctx = Syscalls.getStorageContext();

        // Write a key-value pair (costs 1,000 GAS units)
        Syscalls.storagePut(ctx, key, value);

        // Read it back (costs 100 GAS units)
        bytes memory stored = Syscalls.storageGet(ctx, key);

        // Delete when no longer needed (costs 100 GAS units)
        Syscalls.storageDelete(ctx, key);
    }
}
```

## Cost Guidance

::: tip Storage Cost Awareness
`System.Storage.Put` costs 1,000 GAS units — 10x more expensive than `Get` or `Delete` (100 each). Minimize writes by batching updates in contract logic and avoiding redundant puts; each individual put incurs the full syscall cost.
:::
