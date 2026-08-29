---
title: "Solidity Feature Support: I. ERC to NEP Protocol Mapping"
description: "I. ERC to NEP Protocol Mapping from Solidity Feature Support."
---

# I. ERC to NEP Protocol Mapping

[Back to Solidity Feature Support](/solidity/feature-support)

## Overview

| Ethereum Standard            | Neo Standard                  | Status | Notes                                                                          |
| ---------------------------- | ----------------------------- | :----: | ------------------------------------------------------------------------------ |
| ERC-20 (Fungible Token)      | NEP-17                        |   ✅   | Auto-detected. `transfer(to,amount)` warns to use 4-param NEP-17 form.         |
| ERC-721 (NFT)                | NEP-11                        |   ✅   | Auto-detected. `transferFrom` warns to use NEP-11 `transfer(to,tokenId,data)`. |
| ERC-20 `approve`/`allowance` | N/A                           |   ⚠️   | Warning: not part of NEP-17 spec. Neo uses `Runtime.checkWitness()`.           |
| ERC-165 `supportsInterface`  | Manifest `supportedstandards` |   ⚠️   | Warning: unnecessary on Neo. Manifest-based discovery.                         |
| ERC-4626 (Tokenized Vault)   | NEP-17                        |   ⚠️   | Vault logic compiles. ERC-20 interactions must use NEP-17 equivalents.         |
| ERC-2981 (Royalty)           | NEP-24                        |   ✅   | Auto-detected. Multiple royalty recipients supported.                          |
| `receive()` / `fallback()`   | `onNEP17Payment()`            |   ⚠️   | Diagnostic suggests callback pattern.                                          |

For detailed standard migration guides, see the [Standards Mapping](/additional-material/neo-standards).

---
