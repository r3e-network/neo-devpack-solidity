---
title: "Runtime Specification: Storage Emulation"
description: "Storage Emulation from Runtime Specification."
---

# Storage Emulation

[Back to Runtime Specification](/internals/runtime-specification)

The embedded runtime provides a full in-memory storage emulation layer.

## Storage Context Model

Each contract has its own isolated storage context. Storage contexts can be:

- **Read-write** -- obtained via `System.Storage.GetContext`
- **Read-only** -- obtained via `System.Storage.GetReadOnlyContext` or `AsReadOnly`

Write operations (`Put`, `Delete`) on a read-only context will fail.

## Get / Put / Delete Semantics

- **Get**: Returns the byte array value for a given key, or null if the key does not exist.
- **Put**: Writes a key-value pair. Both key and value are byte arrays. Overwrites existing values.
- **Delete**: Removes a key-value pair. No error if the key does not exist.

## Find and Iterator Operations

`Storage.Find` takes a key prefix and returns an iterator handle over all matching entries:

1. The runtime materializes all entries matching the prefix into an ordered list
2. An iterator token (byte handle) is created and pushed onto the stack
3. `Iterator.Next` advances the cursor and returns `true` if an entry is available
4. `Iterator.Value` returns the current entry as a key-value struct
5. Iterator handles are scoped to the execution context; the embedded runtime does not expose an `Iterator.Dispose` syscall

::: warning
The current implementation materializes all matching entries at `Find` time. This works correctly but is less efficient than streaming for large datasets. See [Parity and Limitations](/internals/parity-and-limitations) for details.
:::
