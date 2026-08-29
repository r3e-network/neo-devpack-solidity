---
title: "Native Contracts: Notary Contract"
description: "Notary Contract from Native Contracts."
---

# Notary Contract

[Back to Native Contracts](/internals/native-contracts)

The Notary native contract models Neo N3 notary deposits and notary-assisted transaction helpers. The embedded runtime implements a deterministic in-memory subset for compiler/runtime tests; it does not model the full network notary service.

## Methods

| Method                         | Devpack wrapper                         | Return    | Safe | Embedded runtime behavior                                      |
| ------------------------------ | --------------------------------------- | --------- | :--: | -------------------------------------------------------------- |
| `verify`                       | `NativeCalls.notaryVerify(signature)`   | `bool`    |  ✅  | Returns `true`.                                                |
| `balanceOf`                    | `NativeCalls.notaryBalanceOf(account)`  | `uint256` |  ✅  | Reads the embedded deposit amount for the account, or zero.    |
| `expirationOf`                 | `NativeCalls.notaryExpirationOf(account)` | `uint256` | ✅ | Reads the embedded deposit expiration height, or zero.         |
| `lockDepositUntil`             | `NativeCalls.notaryLockDepositUntil(account,till)` | `bool` | ❌ | Extends an existing embedded deposit when `till` is greater.   |
| `withdraw`                     | `NativeCalls.notaryWithdraw(from,to)`   | `bool`    |  ❌  | Removes an embedded deposit for `from`; `to` is not modeled.   |
| `getMaxNotValidBeforeDelta`    | `NativeCalls.notaryGetMaxNotValidBeforeDelta()` | `uint256` | ✅ | Returns the embedded max-not-valid-before delta value.         |
| `setMaxNotValidBeforeDelta`    | `NativeCalls.notarySetMaxNotValidBeforeDelta(value)` | `void` | ❌ | Updates the embedded max-not-valid-before delta value.         |
| `onNEP17Payment`               | `NativeCalls.notaryOnNEP17Payment(from,amount,data)` | `void` | ❌ | Records an embedded deposit and optional expiration value.     |

## Embedded Runtime Limits

Authorization, real transaction-notary coordination, and network service behavior are not modeled. Use Neo-Express or TestNet for flows that depend on production notary consensus semantics.

---
