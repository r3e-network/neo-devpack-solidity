---
title: "Standards and Contracts: Compiler Diagnostics for Standards"
description: "Compiler Diagnostics for Standards from Standards and Contracts."
---

# Compiler Diagnostics for Standards

[Back to Standards and Contracts](/additional-material/neo-standards)

## Overview

The compiler emits actionable diagnostics when it detects Ethereum patterns that need migration. These help catch common mistakes during ERC → NEP porting:

| Code   | Pattern Detected                               | Suggestion                                       |
| ------ | ---------------------------------------------- | ------------------------------------------------ |
| `W101` | `transfer(to, amount)` — 2-param ERC-20        | Add `from` and `data` params for NEP-17          |
| `W102` | `transfer(from, to, amount)` — 3-param partial | Add `data` parameter for NEP-17                  |
| `W103` | `approve`/`allowance`/`transferFrom` present   | Not in NEP-17 spec; keep as extensions or remove |
| `W104` | `transferFrom(from, to, tokenId)` — ERC-721    | Replace with `transfer(to, tokenId, data)`       |
| `W105` | `receive()` / `fallback()` present             | Replace with `onNEP17Payment` callback           |
| `W106` | `supportsInterface(bytes4)` present            | Remove — manifest handles interface detection    |
| `W107` | ERC-1155 multi-token pattern                   | Split into separate NEP-17 and NEP-11 contracts  |
| `W108` | ERC-2612 permit (7-param)                      | Use `Runtime.checkWitness()` instead             |
