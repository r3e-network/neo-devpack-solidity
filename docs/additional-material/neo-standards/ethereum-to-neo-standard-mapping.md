---
title: "Standards and Contracts: Ethereum to Neo Standard Mapping"
description: "Ethereum to Neo Standard Mapping from Standards and Contracts."
---

# Ethereum to Neo Standard Mapping

[Back to Standards and Contracts](/additional-material/neo-standards)

## Overview

| Ethereum          | Neo                                  | Key Differences |
| ----------------- | ------------------------------------ | --------------- |
| ERC-20            | NEP-17                               | 4-parameter `transfer(from, to, amount, data)`, witness authorization, and `onNEP17Payment` for token receipts |
| ERC-721           | NEP-11                               | 3-parameter `transfer(to, tokenId, data)`, `tokensOf(owner)`, witness authorization, and ByteString-compatible token IDs |
| ERC-2981          | NEP-24                               | Multiple royalty recipients and an explicit `royaltyToken`; returned royalty values are computed amounts, not percentages |
| ERC-1155          | No single NEP                        | Split into NEP-17, NEP-11, or manual NEP-11-divisible-style storage depending on token semantics |
| EIP-165           | Manifest `supportedstandards`        | Interface detection is manifest-based; `supportsInterface()` is unnecessary on Neo |
| ERC-2612 / EIP-712 permit | Witness-scoped transactions | `Runtime.checkWitness()` usually replaces token-level permit flows; signed-message ports still need nonce/deadline replay protection |
| EIP-1967 (Proxy)  | NEP-22 / NEP-29 / NEP-31             | Native in-place update, deploy/update callback, and optional destroy instead of proxy storage slots |
| ERC-721 receiver  | NEP-26                               | `onNEP11Payment(from, amount, tokenId, data)` receiver callback |
| ERC-677 / ERC-1363 hooks | NEP-27                         | `onNEP17Payment(from, amount, data)` receiver callback |

## How To Use This Table

The mapping table is a migration guide, not a substitute for a standards audit:

- Check the canonical NEP method/event shape before declaring
  `supportedstandards`.
- Treat compiler auto-detection as a helpful heuristic; it can diagnose
  ERC-shaped contracts that are not yet fully NEP-compliant.
- Validate deployed examples against the live snapshot in the
  [Standards Mirror Coverage Matrix](/standards-mirror/coverage-matrix).

For the detailed version, see [Standards Mapping](/mapping/standards) and the
[ERC / EIP to Neo Mirror](/standards-mirror/).
