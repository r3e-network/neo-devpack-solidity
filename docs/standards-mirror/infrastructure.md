---
title: Infrastructure & Patterns — ERC ↔ Neo Mirror
description: Proxies, registries, modular contracts, deterministic deployment — Ethereum infrastructure standards mirrored to Neo C#.
---

# Infrastructure & Patterns

Sixteen standards covering interface detection, registries, proxy upgrades,
deterministic deployment, minimal proxies, init-arg meta-proxies, blueprint
contract format, diamond storage, off-chain data retrieval, cross-chain messaging
gateways, modular smart accounts, and meta-transactions. Several of these are
straightforward ports; others are subsumed by Neo's manifest-driven contract
model or its native oracle service.

## Standards

| Standard | Neo Mapping | Status | Category |
| --- | --- | --- | --- |
| [ERC-165 — Interface Detection](/standards-mirror/infrastructure/erc-165) | Manifest supportedstandards | Final | Detection |
| [ERC-1167 — Minimal Proxy (Clones)](/standards-mirror/infrastructure/erc-1167) | ContractManagement.Deploy (parameterised) | Final | Proxy |
| [ERC-3448 — MetaProxy Standard](/standards-mirror/infrastructure/erc-3448) | ContractManagement.Deploy with constant slots | Final | Proxy |
| [ERC-5202 — Blueprint Contract Format](/standards-mirror/infrastructure/erc-5202) | NEF blob in Storage + ContractManagement.Deploy | Final | Deployment |
| [ERC-8042 — Diamond Storage](/standards-mirror/infrastructure/erc-8042) | Storage prefix per facet (idiomatic) | Final | Storage |
| [ERC-5269 — ERC Detection and Discovery](/standards-mirror/infrastructure/erc-5269) | Native manifest `supportedstandards` | Review | Detection |
| [ERC-6357 — Single-contract Multi-delegatecall](/standards-mirror/infrastructure/erc-6357) | Native multi-invoke transaction script | Last Call | Pattern |
| [ERC-7746 — Composable Security Middleware Hooks](/standards-mirror/infrastructure/erc-7746) | Pre/post hook chain on every external method | Last Call | Pattern |
| [ERC-1820 — Pseudo-introspection Registry](/standards-mirror/infrastructure/erc-1820) | Manifest + native registry | Final | Registry |
| [ERC-1967 — Standard Proxy Storage Slots](/standards-mirror/infrastructure/erc-1967) | NEP-22 ContractManagement.Update | Final | Upgrade |
| [ERC-2535 — Diamond Standard](/standards-mirror/infrastructure/erc-2535) | Modular dispatch (port) | Final | Modularity |
| [ERC-3668 — CCIP Read (Off-chain Data)](/standards-mirror/infrastructure/erc-3668) | Native Oracle service | Final | Off-chain Data |
| [ERC-7201 — Namespaced Storage Layout](/standards-mirror/infrastructure/erc-7201) | Storage prefixes (idiomatic) | Final | Storage |
| [ERC-7786 — Cross-Chain Messaging Gateway](/standards-mirror/infrastructure/erc-7786) | Bridge adapter pattern | Final | Cross-Chain |
| [ERC-1014 — Skinny CREATE2](/standards-mirror/infrastructure/erc-1014) | ContractManagement.Deploy | Final | Deployment |
| [ERC-2470 — Singleton Factory](/standards-mirror/infrastructure/erc-2470) | ContractManagement.Deploy (no factory needed) | Final | Deployment |
| [ERC-1056 — Lightweight Identity](/standards-mirror/infrastructure/erc-1056) | Neo C# port | Final | Identity |
| [ERC-7579 — Modular Smart Accounts](/standards-mirror/infrastructure/erc-7579) | Modular NEP-30 account (port) | Final | Smart Accounts |
| [ERC-2771 — Trusted Forwarder (Meta-Tx)](/standards-mirror/infrastructure/erc-2771) | Native witness scopes | Final | Meta-Tx |

## Related Pages

- [Standards Mirror Overview](/standards-mirror/)
- [Semantic Standards Mapping](/mapping/standards)
- [Standards and Contracts](/additional-material/neo-standards)
