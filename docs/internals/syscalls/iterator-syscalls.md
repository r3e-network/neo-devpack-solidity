---
title: "Syscalls: Iterator Syscalls"
description: "Iterator Syscalls from Syscalls."
---

# Iterator Syscalls

[Back to Syscalls](/internals/syscalls)

## Iterator API

Iterator syscalls traverse results returned by `System.Storage.Find`.

| Syscall Name            | Gas Cost | Description                | Devpack Wrapper                |
| ----------------------- | -------: | -------------------------- | ------------------------------ |
| `System.Iterator.Next`  |        1 | Advance iterator           | `Syscalls.iteratorNext(iter)`  |
| `System.Iterator.Value` |        1 | Get current iterator value | `Syscalls.iteratorValue(iter)` |

Iterators are the only way to enumerate storage keys by prefix on Neo N3. They are returned by `System.Storage.Find` and consumed with the `Next`/`Value` pair.

## Example Usage

```solidity
import "devpack/contracts/Syscalls.sol";

contract Registry {
    bytes constant PREFIX_USER = "user:";

    function listUsers() external view returns (bytes[] memory) {
        // Find all storage keys starting with "user:" (100 GAS units)
        Syscalls.StorageContext memory ctx = Syscalls.getReadOnlyStorageContext();
        Syscalls.Iterator memory iter = Syscalls.storageFind(ctx, PREFIX_USER);

        // Iterate through results (1 GAS unit per Next + 1 per Value)
        bytes[] memory temp = new bytes[](100);
        uint256 count = 0;

        while (Syscalls.iteratorNext(iter) && count < 100) {
            temp[count] = Syscalls.iteratorValue(iter);
            count++;
        }

        // Trim to actual size
        bytes[] memory result = new bytes[](count);
        for (uint256 i = 0; i < count; i++) {
            result[i] = temp[i];
        }
        return result;
    }
}
```

## Execution Limits

::: warning
Iterators cannot be passed across contract boundaries or stored persistently. They are valid only within the execution context that created them. Attempting to use an iterator from a different invocation will fail.
:::
