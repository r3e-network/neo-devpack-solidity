---
title: "Troubleshooting: Compiler Warning Reference"
description: "Compiler Warning Reference from Troubleshooting."
---

# Compiler Warning Reference

[Back to Troubleshooting](/advisory-content/troubleshooting)

## Auto-Mapping Warnings

These warnings indicate EVM globals that are auto-mapped to approximate Neo equivalents:

| Warning            | Pattern                                    | Action                                               |
| ------------------ | ------------------------------------------ | ---------------------------------------------------- |
| `block.coinbase`   | Auto-mapped to `address(0)`                | Review — always zero on Neo (no block miner in dBFT) |
| `block.difficulty` | Auto-mapped to `Runtime.getRandom()`       | Review — different randomness model                  |
| `block.gaslimit`   | Auto-mapped to `Policy.getExecFeeFactor()` | Review — different gas accounting                    |
| `block.basefee`    | Auto-mapped to `Policy.getFeePerByte()`    | Review — different fee model                         |
| Ether literals     | Parsed but warned                          | Convert to Neo GAS units (10^8 decimals)             |

## Standards Migration Warnings

| Code | Pattern                              | Fix                                             |
| ---- | ------------------------------------ | ----------------------------------------------- |
| W101 | 2-param `transfer(to, amount)`       | Add `from` and `data` params for NEP-17         |
| W102 | 3-param `transfer(from, to, amount)` | Add `data` parameter for NEP-17                 |
| W103 | `approve`/`allowance`/`transferFrom` | Remove or keep as optional extensions           |
| W104 | `transferFrom(from, to, tokenId)`    | Replace with `transfer(to, tokenId, data)`      |
| W105 | `receive()` / `fallback()`           | Replace with `onNEP17Payment` callback          |
| W106 | `supportsInterface(bytes4)`          | Remove — manifest handles detection             |
| W107 | ERC-1155 multi-token pattern         | Split into separate NEP-17 and NEP-11 contracts |
| W108 | ERC-2612 permit pattern              | Use `Runtime.checkWitness()` instead            |

---
