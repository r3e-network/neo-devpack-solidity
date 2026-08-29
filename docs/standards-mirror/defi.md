---
title: DeFi Building Blocks — ERC ↔ Neo Mirror
description: Vaults, flash loans, governance — Ethereum DeFi standards mirrored to Neo C# implementations.
---

# DeFi Building Blocks

Eleven DeFi standards covering yield-bearing vaults (including a native-asset
variant and async vaults with cancel), flash loans, governance voting, protocol
clocks, and tokenised bonds. Each one is a clean port to Neo C#.

## Standards

| Standard | Neo Mapping | Status | Category |
| --- | --- | --- | --- |
| [ERC-4626 — Tokenized Vault](/standards-mirror/defi/erc-4626) | NEP-17 vault pattern | Final | Vaults |
| [ERC-3156 — Flash Loans](/standards-mirror/defi/erc-3156) | Neo C# port | Final | Flash Loans |
| [ERC-7535 — Native Asset Vault](/standards-mirror/defi/erc-7535) | NEP-17 vault with NEO/GAS as asset | Final | Vaults |
| [ERC-7540 — Asynchronous Vaults](/standards-mirror/defi/erc-7540) | Neo C# port | Final | Vaults |
| [ERC-7944 — Async Cancellation for ERC-7540](/standards-mirror/defi/erc-7944) | Cancel-by-id flow on the request queue | Final | Vaults |
| [ERC-3475 — Abstract Storage Bonds](/standards-mirror/defi/erc-3475) | Neo C# port — class/nonce keyed bond storage | Final | Bonds |
| [ERC-7092 — Financial Bonds](/standards-mirror/defi/erc-7092) | Neo C# port — issuer-facing bond surface | Final | Bonds |
| [ERC-7575 — Multi-Asset Vault](/standards-mirror/defi/erc-7575) | Neo C# port | Final | Vaults |
| [ERC-5805 — Voting With Delegation](/standards-mirror/defi/erc-5805) | Neo C# port | Final | Governance |
| [ERC-6372 — Contract Clock](/standards-mirror/defi/erc-6372) | Neo C# port (uses Ledger.CurrentIndex) | Final | Governance |
| [ERC-7818 — Expirable ERC-20](/standards-mirror/defi/erc-7818) | Neo C# port | Final | Token Lifecycle |

## Related Pages

- [Standards Mirror Overview](/standards-mirror/)
- [Semantic Standards Mapping](/mapping/standards)
- [Standards and Contracts](/additional-material/neo-standards)
