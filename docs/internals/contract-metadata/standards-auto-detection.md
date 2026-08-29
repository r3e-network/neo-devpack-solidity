---
title: "Manifest Standards Auto-Detection"
description: "Standards Auto-Detection from Manifest Specification."
---

# Manifest Standards Auto-Detection

[Back to Manifest Specification](/internals/contract-metadata)

The compiler analyzes your contract's public method signatures and events to automatically populate the `supportedstandards` array. Detection is case-insensitive.

## NEP-17 Detection (Fungible Token)

| Requirement      | Detail                                                              |
| ---------------- | ------------------------------------------------------------------- |
| Required methods | `symbol`, `decimals`, `totalSupply`, `balanceOf`, `transfer`        |
| Exclusion        | `ownerOf` must NOT be present (indicates NFT)                       |
| Transfer event   | `Transfer` event with 3 parameters                                  |
| Hint             | `transfer` should have 4 parameters: `from`, `to`, `amount`, `data` |
| Near-miss        | Warning emitted if 3+ of 5 required methods are present             |

## NEP-11 Detection (Non-Fungible Token)

| Requirement        | Detail                                                       |
| ------------------ | ------------------------------------------------------------ |
| Core methods       | `balanceOf` AND `ownerOf`                                    |
| Transfer mechanism | At least one of: `transfer`, `transferFrom`, `tokensOf`      |
| Transfer event     | `Transfer` event with 4 parameters                           |
| Hint               | `transfer` should have 3 parameters: `to`, `tokenId`, `data` |
| Near-miss          | Warning if `ownerOf` present but no transfer mechanism       |

## NEP-24 Detection (Token Discovery / Royalty)

| Requirement | Detail                                     |
| ----------- | ------------------------------------------ |
| Trigger     | `tokenUri` OR `royaltyInfo` method present |

## NEP-26 Detection (Contract Upgrade)

| Requirement      | Detail                                            |
| ---------------- | ------------------------------------------------- |
| Required methods | Both `update` AND `destroy`                       |
| Near-miss        | Info diagnostic if only one of the two is present |

::: tip
The compiler emits diagnostics when your contract almost matches a standard. Check compiler output for `[warning][NEP-17]` or `[info][NEP-26]` messages to catch missing methods before deployment.
:::
