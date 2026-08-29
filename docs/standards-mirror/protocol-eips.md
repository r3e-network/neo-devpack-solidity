---
title: Protocol-Level EIPs — ERC ↔ Neo Mirror
description: Ethereum protocol EIPs (fee market, opcodes, transaction format) and how Neo handles the equivalent concern natively.
---

# Protocol-Level EIPs

These EIPs change Ethereum's protocol itself — fee market, opcodes, transaction
formats — rather than introducing application-layer standards. Most are no-ops on
Neo because Neo's protocol already addresses the underlying concern. The third
tab here typically shows the **Neo equivalent mechanism** rather than user-level
contract code.

## Standards

| Standard | Neo Mapping | Status | Category |
| --- | --- | --- | --- |
| [EIP-1559 — Fee Market Reform](/standards-mirror/protocol-eips/eip-1559) | Native polynomial GAS pricing | Final | Fees |
| [EIP-2718 — Typed Transaction Envelope](/standards-mirror/protocol-eips/eip-2718) | Native single transaction type | Final | Transactions |
| [EIP-2930 — Access Lists](/standards-mirror/protocol-eips/eip-2930) | Witness scopes (CustomContracts) | Final | Transactions |
| [EIP-3198 — BASEFEE Opcode](/standards-mirror/protocol-eips/eip-3198) | Policy.GetFeePerByte | Final | Fees |
| [EIP-3855 — PUSH0 Opcode](/standards-mirror/protocol-eips/eip-3855) | Native PUSH0 in NeoVM | Final | Opcodes |
| [EIP-3860 — Initcode Size Limit](/standards-mirror/protocol-eips/eip-3860) | NEF format limits | Final | Deployment |
| [EIP-4844 — Proto-Danksharding (Blobs)](/standards-mirror/protocol-eips/eip-4844) | Native sharding via state channels + Oracle | Final | Scaling |
| [EIP-1153 — Transient Storage](/standards-mirror/protocol-eips/eip-1153) | Native (Storage with transaction-scoped lifetime) | Final | Storage |
| [EIP-6780 — SELFDESTRUCT Restriction](/standards-mirror/protocol-eips/eip-6780) | ContractManagement.Destroy (explicit) | Final | Lifecycle |
| [EIP-2098 — Compact Signatures](/standards-mirror/protocol-eips/eip-2098) | Native (Neo signatures already compact) | Final | Signatures |

## Related Pages

- [Standards Mirror Overview](/standards-mirror/)
- [Semantic Standards Mapping](/mapping/standards)
- [Standards and Contracts](/additional-material/neo-standards)
