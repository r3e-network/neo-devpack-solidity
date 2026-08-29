---
title: "Runtime Specification: Syscall Implementation"
description: "Syscall Implementation from Runtime Specification."
---

# Syscall Implementation

[Back to Runtime Specification](/internals/runtime-specification)

Syscalls are the bridge between NeoVM bytecode and the Neo N3 platform services. They are invoked via the `SYSCALL` opcode followed by a 4-byte interop ID.

## Syscall ID Computation

Syscall IDs are the first 4 bytes of the SHA-256 hash of the syscall name string:

```
ID = SHA256("System.Storage.Get")[0..4]
```

This is computed at compile time and embedded in the bytecode.

## Storage Syscalls

| Syscall                             | Description                                           |
| ----------------------------------- | ----------------------------------------------------- |
| `System.Storage.GetContext`         | Get the current contract's read-write storage context |
| `System.Storage.GetReadOnlyContext` | Get a read-only storage context                       |
| `System.Storage.AsReadOnly`         | Convert a context to read-only                        |
| `System.Storage.Get`                | Read a value by key from storage                      |
| `System.Storage.Put`                | Write a key-value pair to storage                     |
| `System.Storage.Delete`             | Delete a key from storage                             |
| `System.Storage.Find`               | Find entries by key prefix, returns iterator          |

## Iterator Syscalls

| Syscall                   | Description                                     |
| ------------------------- | ----------------------------------------------- |
| `System.Iterator.Next`    | Advance iterator to next entry; returns boolean |
| `System.Iterator.Value`   | Get current iterator entry value                |

Iterator handles are created by `Storage.Find` and are validated by `ISTYPE` (type code `0x80`) while alive.

::: info
The embedded runtime currently registers `Iterator.Next` and `Iterator.Value`. It does not expose an `Iterator.Dispose` syscall.
:::

## Runtime Syscalls

| Syscall                                 | Description                                                     |
| --------------------------------------- | --------------------------------------------------------------- |
| `System.Runtime.GetTrigger`             | Get execution trigger type                                      |
| `System.Runtime.Platform`               | Get platform identifier string                                  |
| `System.Runtime.GetNetwork`             | Get network magic number                                        |
| `System.Runtime.GetAddressVersion`      | Get address version byte                                        |
| `System.Runtime.GetTime`                | Get current block timestamp                                     |
| `System.Runtime.GetScriptContainer`     | Get the transaction that triggered execution                    |
| `System.Runtime.GetExecutingScriptHash` | Get hash of currently executing contract                        |
| `System.Runtime.GetCallingScriptHash`   | Get hash of the calling contract                                |
| `System.Runtime.GetEntryScriptHash`     | Get hash of the entry-point contract                            |
| `System.Runtime.LoadScript`             | Registered for Neo N3 parity, but intentionally unsupported in the embedded runtime; use `System.Contract.Call` for inter-contract calls |
| `System.Runtime.CheckWitness`           | Verify that the specified account has witnessed the transaction |
| `System.Runtime.GetInvocationCounter`   | Get number of times current contract was called                 |
| `System.Runtime.GetRandom`              | Get a pseudo-random number                                      |
| `System.Runtime.Log`                    | Emit a log message                                              |
| `System.Runtime.Notify`                 | Emit a notification event (name + state array)                  |
| `System.Runtime.GetNotifications`       | Embedded runtime returns `[]`; notification lookup state is not exposed |
| `System.Runtime.GasLeft`                | Get remaining gas                                               |
| `System.Runtime.BurnGas`                | Burn specified amount of gas                                    |
| `System.Runtime.CurrentSigners`         | Embedded runtime returns `[]`; transaction signer set is not modeled |
| `System.Runtime.GetMsgValue`            | Read the runtime-side `msg.value` override; returns 0 when no host override is set |

Neo-side ABI serialization fallbacks use the `StdLib` native contract (`serialize` / `deserialize`) rather than `System.Runtime.Serialize` or `System.Runtime.Deserialize`.

## Crypto Syscalls

| Syscall                       | Description                           |
| ----------------------------- | ------------------------------------- |
| `System.Crypto.CheckSig`      | Verify a single signature (secp256k1) |
| `System.Crypto.CheckMultisig` | Verify multiple signatures            |

Additional crypto operations are available through the CryptoLib native contract: SHA256, RIPEMD160, Keccak256, Murmur32, Hash160, Hash256.

## Contract Syscalls

| Syscall                                 | Description                                         |
| --------------------------------------- | --------------------------------------------------- |
| `System.Contract.Call`                  | Call another contract by hash                       |
| `System.Contract.GetCallFlags`          | Get current call flags                              |
| `System.Contract.CreateStandardAccount` | Create standard account script hash from public key |
| `System.Contract.CreateMultisigAccount` | Create multisig account script hash                 |
