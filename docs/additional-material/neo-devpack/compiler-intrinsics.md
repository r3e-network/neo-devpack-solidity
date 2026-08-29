---
title: "Devpack Overview: Compiler Intrinsics"
description: "Compiler Intrinsics from Devpack Overview."
---

# Compiler Intrinsics

[Back to Devpack Overview](/additional-material/neo-devpack)

## Overview

The following devpack surfaces are recognized as compiler intrinsics by `neo-solc`:

| Intrinsic Surface                      | Lowered To                                |
| -------------------------------------- | ----------------------------------------- |
| `Runtime.checkWitness()`               | `System.Runtime.CheckWitness`             |
| `Runtime.gasLeft()`                    | `System.Runtime.GasLeft`                  |
| `Runtime.notify()`                     | `System.Runtime.Notify`                   |
| `Storage.put()` / `get()` / `remove()` | `System.Storage.Put` / `Get` / `Delete`   |
| `Storage.find()`                       | `System.Storage.Find`                     |
| `Syscalls.sha256()`                    | `CryptoLib.sha256` (native call)          |
| `Syscalls.verifyWithECDsa()`           | `CryptoLib.verifyWithECDsa` (native call) |
| `Syscalls.neoKeccak256()`              | `CryptoLib.keccak256` (native call)       |
| `Syscalls.contractCall()`              | `System.Contract.Call`                    |
| `NativeCalls.gasTransfer()`            | `System.Contract.Call` → GAS `transfer`   |
| `abi.encode()` / `abi.decode()`        | NeoVM stack manipulation                  |
| Solidity `keccak256()`                 | `CryptoLib.keccak256` (native call)       |

When you call a member that the compiler does not recognize as an intrinsic, compilation fails with a diagnostic:

```
error: unsupported intrinsic 'Runtime.unsupportedMethod'
  --> MyContract.sol:12:5
   |
12 |     Runtime.unsupportedMethod();
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^
   = note: available Runtime intrinsics: checkWitness, gasLeft, notify, log, ...
```
