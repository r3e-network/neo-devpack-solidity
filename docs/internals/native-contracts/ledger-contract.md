---
title: "Native Contracts: Ledger Contract"
description: "Ledger Contract from Native Contracts."
---

# Ledger Contract

[Back to Native Contracts](/internals/native-contracts)

Provides read-only access to blockchain data — blocks, transactions, and their metadata.

## Methods

| Method                    | Signature                                  | Return        | Safe | Embedded Runtime Behavior |
| ------------------------- | ------------------------------------------ | ------------- | :--: | ------------------------- |
| `currentIndex`            | `currentIndex()`                           | `uint256`     |  ✅  | Current block height. Maps from `block.number`. |
| `currentHash`             | `currentHash()`                            | `bytes32`     |  ✅  | Hash of the current block. |
| `getBlock`                | `getBlock(uint256)`                        | `Block`       |  ✅  | Returns a cached or synthetic block when the index is at or below the current height; returns `null` for future heights. |
| `getBlock`                | `getBlock(bytes32)`                        | `Block`       |  ✅  | Returns a matching cached/synthetic block when available; detailed chain lookup is not modeled. |
| `getTransaction`          | `getTransaction(bytes32)`                  | `Transaction` |  ✅  | Returns a transaction only when it exists in the embedded runtime cache; otherwise `null`. |
| `getTransactionHeight`    | `getTransactionHeight(bytes32)`            | `int256`      |  ✅  | Returns current height for cached transactions, otherwise `-1`. |
| `getTransactionFromBlock` | `getTransactionFromBlock(uint256,uint256)` | `Transaction` |  ✅  | Currently stubbed; per-block transaction lookup is not modeled and this returns `null`. |
| `getTransactionSigners`   | `getTransactionSigners(bytes32)`           | `Signer[]`    |  ✅  | Returns an empty array; transaction signers are not modeled. |
| `getTransactionVMState`   | `getTransactionVMState(bytes32)`           | `uint8`       |  ✅  | Returns the default HALT state (`1`); detailed per-transaction VM state is not modeled. |

::: info
The compiler auto-maps `block.number` to `Ledger.currentIndex()` and `blockhash(n)` to `Ledger.getBlock(n).hash`. See [Units and Globally Available Variables](/language-description/units-and-global-variables) for the full list of auto-mapped block context values.
:::

---
