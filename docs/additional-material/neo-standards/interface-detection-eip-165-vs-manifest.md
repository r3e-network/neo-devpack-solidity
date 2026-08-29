---
title: "Standards and Contracts: Interface Detection: EIP-165 vs Manifest"
description: "Interface Detection: EIP-165 vs Manifest from Standards and Contracts."
---

# Interface Detection: EIP-165 vs Manifest

[Back to Standards and Contracts](/additional-material/neo-standards)

## Overview

Ethereum and Neo take fundamentally different approaches to interface detection:

|                  | EVM (EIP-165)                                 | Neo (Manifest)                     |
| ---------------- | --------------------------------------------- | ---------------------------------- |
| **Mechanism**    | Runtime query via `supportsInterface(bytes4)` | Static manifest JSON               |
| **Query method** | `staticcall` to contract                      | `ContractManagement.getContract()` |
| **Gas cost**     | Per-query gas cost                            | Free (manifest is metadata)        |
| **Maintenance**  | Manual `supportsInterface()` implementation   | Compiler auto-populates            |

No `supportsInterface()` function is needed on Neo. The compiler automatically populates the `supportedstandards` array based on method signature analysis.

| EIP-165 Selector | ERC Standard | Neo Manifest Entry |
| ---------------- | ------------ | ------------------ |
| `0x80ac58cd`     | ERC-721      | `"NEP-11"`         |
| `0x36372b07`     | ERC-20       | `"NEP-17"`         |
| `0x2a55205a`     | ERC-2981     | `"NEP-24"`         |

::: warning supportsInterface() Is Unnecessary
If your contract includes `supportsInterface(bytes4)`, the compiler emits warning `W106`:

```
warning[W106]: function 'supportsInterface' (EIP-165) is unnecessary on Neo N3.
  Neo uses the manifest 'supportedstandards' array for interface detection,
  which the compiler populates automatically.
  = suggestion: Remove — Neo N3 uses manifest-based interface discovery
```

:::
