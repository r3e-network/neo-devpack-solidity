---
title: "Devpack Overview: Libraries"
description: "Libraries from Devpack Overview."
---

# Libraries

[Back to Devpack Overview](/additional-material/neo-devpack)

## Neo.sol

High-level blockchain integration library. Wraps `Syscalls` and `NativeCalls` into a convenient API:

| Category        | Functions                                                                      |
| --------------- | ------------------------------------------------------------------------------ |
| Block info      | `getCurrentBlock()`, `getBlockByIndex()`, `getBlockHeight()`, `getBlockTime()` |
| Transactions    | `getTransaction()`, `getTransactionHeight()`, `transactionExists()`            |
| Account/balance | `getNeoBalance()`, `getGasBalance()`, `transferNeo()`, `transferGas()`         |
| Cryptographic   | `verifyWithWitness()`, `verifySignature()`, `sha256Hash()`, `ripemd160Hash()`, `getRandom()` |
| Contract mgmt   | `callContract()`, `deployContract()`                                           |
| Network/policy  | `getNetworkMagic()`, `getGasPrice()`, `getStoragePrice()`                      |
| Governance      | `isCommittee()`, `getCommittee()`, `getValidators()`, `isValidator()`          |

```solidity
using Neo for *;

uint256 height = Neo.getBlockHeight();
uint256 gasBalance = Neo.getGasBalance(msg.sender);
```

## Storage.sol

Storage operations built on top of `Syscalls` storage syscalls:

| Category   | Functions                                              |
| ---------- | ------------------------------------------------------ |
| Context    | `getContext()`, `getReadOnlyContext()`, `asReadOnly()` |
| Basic CRUD | `put()`, `get()`, `remove()`                           |
| Iterators  | `find()` — prefix scan via `System.Storage.Find`       |
| Metadata   | `putContractMetadata()`                                |

```solidity
using Storage for *;

// Basic operations
Storage.put("owner", abi.encode(msg.sender));
bytes memory data = Storage.get("owner");
bool hasKey = data.length > 0;
```

::: info Storage Contexts Are Contract-Private
Neo N3 storage contexts are always private to the owning contract — other
contracts cannot read or write your storage. A former `putLocal()`/`getLocal()`
"local storage" API was removed: it lowered to `System.Storage.Local.*`
syscalls that do not exist on Neo N3 (calls would FAULT on a real node), and
the privacy it promised is already provided by regular `put()`/`get()`.
Higher-level helpers (batch operations, prefix counting/clearing, typed
accessors, TTL wrappers) were also removed; implement them in contract code
using `put`/`get`/`remove`/`find`.
:::

## Runtime.sol

Runtime services and utilities (currently supported as compiler intrinsics):

| Category         | Functions                                                                                              |
| ---------------- | ------------------------------------------------------------------------------------------------------ |
| Events           | `notify()`, `notifyIndexed()`                                                                          |
| Witness          | `checkWitness()`, `requireWitness()`, `checkAnyWitness()`, `checkAllWitnesses()`, `checkMultiSigWitness()` |
| Runtime context  | `gasLeft()`, `getTime()`, `getTrigger()`, `getInvocationCounter()`, `getCurrentSigners()`            |
| Contract context | `getCallFlags()`, `getScriptContainer()`, `loadScript()`, `getExecutingScriptHash()`, `getCallingScriptHash()`, `getEntryScriptHash()` |
| Platform         | `getNetwork()`, `getPlatform()`, `getAddressVersion()`, `getRandom()`                                 |
| Logging          | `log()`, `burnGas()`, `initializeServices()`                                                           |

```solidity
using Runtime for *;

// Witness verification
bool hasWitness = Runtime.checkWitness(msg.sender);
require(hasWitness, "invalid witness");

// Gas management
uint256 remaining = Runtime.gasLeft();
Runtime.burnGas(1000000); // burn 0.01 GAS
```

::: warning Runtime Intrinsic Coverage
`Runtime`, `Storage`, and `Neo` are compiler intrinsics: only the members listed above are lowered by neo-devpack-solidity, and their Solidity bodies are never compiled. Calling any other member fails compilation with a diagnostic listing the supported intrinsics. Former callback-oriented helpers (e.g. `optimizeGasUsage`, `tryWithFallback`) were removed because NeoVM does not support first-class internal function callbacks — use inline logic (`gasLeft` guards and `try/catch`) instead.
:::
