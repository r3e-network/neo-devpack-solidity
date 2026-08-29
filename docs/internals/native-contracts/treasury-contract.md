---
title: "Native Contracts: Treasury Contract"
description: "Treasury Contract from Native Contracts."
---

# Treasury Contract

[Back to Native Contracts](/internals/native-contracts)

The Treasury native contract records treasury-related payment callbacks. The embedded runtime implements the callback surface needed by compiler/runtime tests and stores received NEP-17/NEP-11 data in memory.

## Methods

| Method           | Devpack wrapper                         | Return | Safe | Embedded runtime behavior                                  |
| ---------------- | --------------------------------------- | ------ | :--: | ---------------------------------------------------------- |
| `verify`         | `NativeCalls.treasuryVerify()`          | `bool` |  ✅  | Returns `true`.                                            |
| `onNEP17Payment` | `NativeCalls.treasuryOnNEP17Payment(from,amount,data)` | `void` | ❌ | Adds the received amount to an embedded per-account balance. |
| `onNEP11Payment` | `NativeCalls.treasuryOnNEP11Payment(from,amount,tokenId,data)` | `void` | ❌ | Records the received token ID in embedded per-account state. |

## Embedded Runtime Limits

The embedded runtime does not model treasury governance, authorization, or live-chain asset custody. Use Neo-Express or TestNet for production treasury behavior.

---
