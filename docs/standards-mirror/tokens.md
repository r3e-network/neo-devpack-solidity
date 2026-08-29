---
title: "Standards Mirror Token Standards"
description: Every Ethereum token standard mirrored to its Neo N3 implementation — fungibles, NFTs, multi-token, semi-fungible, soulbound, royalty.
---

# Standards Mirror Token Standards

Sixty-five Ethereum token standards, each shown with the Solidity reference and an idiomatic
Neo C# implementation. Where a NEP exists (NEP-17, NEP-11, NEP-24), the C# tab uses
that. Where no NEP exists, the C# tab shows a clean Neo port — proof that the standard
is implementable on Neo today.

## Standards

| Standard | Neo Mapping | Status | Category |
| --- | --- | --- | --- |
| [ERC-20 — Fungible Token](/standards-mirror/tokens/erc-20) | NEP-17 | Final | Fungible |
| [ERC-721 — Non-Fungible Token](/standards-mirror/tokens/erc-721) | NEP-11 | Final | NFT |
| [ERC-777 — Token w/ Hooks](/standards-mirror/tokens/erc-777) | NEP-17 + NEP-27 callback | Final | Fungible |
| [ERC-1155 — Multi-Token](/standards-mirror/tokens/erc-1155) | NEP-11 (divisible) + NEP-17 | Final | Multi-Token |
| [ERC-1363 — Payable Token](/standards-mirror/tokens/erc-1363) | NEP-17 (`onNEP17Payment`) | Final | Fungible Extension |
| [ERC-2981 — NFT Royalty Standard](/standards-mirror/tokens/erc-2981) | NEP-24 | Final | NFT Extension |
| [ERC-6093 — Custom Errors for Common Tokens](/standards-mirror/tokens/erc-6093) | Named exception convention | Final | Convention |
| [ERC-7528 — Native Asset Address](/standards-mirror/tokens/erc-7528) | Native NEO / GAS contract hashes | Final | Convention |
| [ERC-3525 — Semi-Fungible Token](/standards-mirror/tokens/erc-3525) | Neo C# port | Final | Hybrid |
| [ERC-2309 — Consecutive NFT Mints](/standards-mirror/tokens/erc-2309) | Neo C# port | Final | NFT Extension |
| [ERC-4906 — NFT Metadata Update](/standards-mirror/tokens/erc-4906) | Neo C# port | Final | NFT Extension |
| [ERC-4494 — Permit for ERC-721](/standards-mirror/tokens/erc-4494) | Native witness scopes | Final | NFT Extension |
| [ERC-5192 — Soulbound NFTs](/standards-mirror/tokens/erc-5192) | Neo C# port | Final | NFT Extension |
| [ERC-5484 — Consensual Soulbound](/standards-mirror/tokens/erc-5484) | Neo C# port | Final | NFT Extension |
| [ERC-6909 — Minimal Multi-Token](/standards-mirror/tokens/erc-6909) | Neo C# port | Final | Multi-Token |
| [ERC-5114 — Soulbound Badge](/standards-mirror/tokens/erc-5114) | Neo C# port | Final | NFT Extension |
| [ERC-6147 — NFT Guard](/standards-mirror/tokens/erc-6147) | Neo C# port | Final | NFT Extension |
| [ERC-4907 — Rental NFT (User Role)](/standards-mirror/tokens/erc-4907) | NEP-11 + per-token user/expires | Final | NFT Extension |
| [ERC-3643 — T-REX Regulated Token](/standards-mirror/tokens/erc-3643) | NEP-17 + identity registry + compliance modules | Final | Compliance |
| [ERC-5679 — Token Minting and Burning](/standards-mirror/tokens/erc-5679) | NEP-17 / NEP-11 mint/burn helper convention | Final | Convention |
| [ERC-2135 — Consumable Interface](/standards-mirror/tokens/erc-2135) | NEP-11 + per-token consumed flag | Final | NFT Extension |
| [ERC-7160 — ERC-721 Multi-Metadata Extension](/standards-mirror/tokens/erc-7160) | NEP-11 + URI list + active-index storage | Final | NFT Extension |
| [ERC-6982 — Default Lockable Tokens](/standards-mirror/tokens/erc-6982) | NEP-11 + lock-state storage | Final | NFT Extension |
| [ERC-7144 — ERC-20 with Transaction Validation Step](/standards-mirror/tokens/erc-7144) | NEP-17 transfer override + validator contract | Review | Fungible Extension |
| [ERC-7943 — Universal Real World Asset Interface](/standards-mirror/tokens/erc-7943) | NEP-17/NEP-11 + capability flags + compliance hook | Last Call | RWA |
| [ERC-5006 — Rental NFT, NFT User Extension](/standards-mirror/tokens/erc-5006) | NEP-11 (divisible) + per-record user/expires | Final | NFT Extension |
| [ERC-5169 — Client Script URI for Token Contracts](/standards-mirror/tokens/erc-5169) | NEP-11 / NEP-17 + scriptURI() view | Final | Token Metadata |
| [ERC-5375 — NFT Author Information and Consent](/standards-mirror/tokens/erc-5375) | NEP-11 + author/consent metadata fields | Final | NFT Extension |
| [ERC-5023 — Shareable Non-Fungible Token](/standards-mirror/tokens/erc-5023) | NEP-11 with multi-holder share extension | Final | NFT Extension |
| [ERC-7066 — Lockable Extension for ERC-721](/standards-mirror/tokens/erc-7066) | NEP-11 + per-token approval-based lock | Final | NFT Extension |
| [ERC-7432 — Non-Fungible Token Roles](/standards-mirror/tokens/erc-7432) | NEP-11 + (tokenId, role) → grant storage with TTL | Final | NFT Extension |
| [ERC-6105 — No Intermediary NFT Trading Protocol](/standards-mirror/tokens/erc-6105) | NEP-11 with built-in listing + atomic-swap surface | Final | NFT Extension |
| [ERC-5615 — ERC-1155 Supply Extension](/standards-mirror/tokens/erc-5615) | NEP-11 (divisible) + per-tokenId supply tracking | Final | Multi-Token Extension |
| [ERC-5773 — Context-Dependent Multi-Asset Tokens](/standards-mirror/tokens/erc-5773) | NEP-11 + per-token asset list + priority | Final | NFT Extension |
| [ERC-6059 — Parent-Governed Nestable NFTs](/standards-mirror/tokens/erc-6059) | NEP-11 + parent-child storage + accept flow | Final | NFT Extension |
| [ERC-4519 — NFTs Tied to Physical Assets](/standards-mirror/tokens/erc-4519) | NEP-11 + per-token device pubkey + state machine | Final | RWA / NFT Extension |
| [ERC-5570 — Digital Receipt NFT](/standards-mirror/tokens/erc-5570) | NEP-11 + structured receipt metadata schema | Final | NFT Extension |
| [ERC-6150 — Hierarchical NFTs](/standards-mirror/tokens/erc-6150) | NEP-11 + parent/children pointer storage | Final | NFT Extension |
| [ERC-6220 — Composable NFTs Equippable Parts](/standards-mirror/tokens/erc-6220) | NEP-11 + catalog + equip-slot storage | Final | NFT Extension |
| [ERC-5380 — ERC-721 Entitlement Extension](/standards-mirror/tokens/erc-5380) | NEP-11 + per-(tokenId, action, delegate) storage | Final | NFT Extension |
| [ERC-5489 — NFT Hyperlink Extension](/standards-mirror/tokens/erc-5489) | NEP-11 + per-(tokenId, slot) URL storage | Final | NFT Extension |
| [ERC-6672 — Multi-Redeemable NFTs](/standards-mirror/tokens/erc-6672) | NEP-11 + per-(tokenId, operator, redemptionId) state | Final | NFT Extension |
| [ERC-7634 — Limited Transfer Count NFT](/standards-mirror/tokens/erc-7634) | NEP-11 + per-token transfer counter | Final | NFT Extension |
| [ERC-7007 — Verifiable AI-Generated Content Token](/standards-mirror/tokens/erc-7007) | NEP-11 + (prompt, output, proof) attestation | Final | NFT Extension / AI |
| [ERC-5725 — Transferable Vesting NFT](/standards-mirror/tokens/erc-5725) | NEP-11 + per-token vesting schedule + claim | Final | NFT Extension / DeFi |
| [ERC-6454 — Minimal Transferable NFT detection](/standards-mirror/tokens/erc-6454) | NEP-11 + IsTransferable view | Final | NFT Extension |
| [ERC-5507 — Refundable Tokens](/standards-mirror/tokens/erc-5507) | NEP-11 + per-token refund window + escrow | Final | NFT Extension |
| [ERC-5528 — Refundable Fungible Token](/standards-mirror/tokens/erc-5528) | NEP-17 + per-buyer refund window + escrow | Final | Fungible Extension |
| [ERC-5521 — Referable NFT](/standards-mirror/tokens/erc-5521) | NEP-11 + per-token referring/referred edges | Final | NFT Extension |
| [ERC-5646 — Token State Fingerprint](/standards-mirror/tokens/erc-5646) | NEP-11 + per-token state-hash view | Final | NFT Extension |
| [ERC-5585 — ERC-721 NFT Authorization](/standards-mirror/tokens/erc-5585) | NEP-11 + per-(tokenId, rights, user) storage | Final | NFT Extension |
| [ERC-6239 — Semantic Soulbound Tokens](/standards-mirror/tokens/erc-6239) | NEP-11 soulbound + RDF triple metadata | Final | NFT Extension |
| [ERC-7053 — Interoperable Digital Media Indexing](/standards-mirror/tokens/erc-7053) | NEP-11 + media-indexing event emission | Final | NFT Extension |
| [ERC-6381 — Public NFT Emote Repository](/standards-mirror/tokens/erc-6381) | Standalone repository: (collection, tokenId, emoji, user) | Final | NFT Extension / Social |
| [ERC-7857 — AI Agents NFT with Private Metadata](/standards-mirror/tokens/erc-7857) | NEP-11 + encrypted-metadata reference + re-encryption | Final | NFT Extension / AI |
| [ERC-7231 — Identity-aggregated NFT](/standards-mirror/tokens/erc-7231) | NEP-11 + per-token identity-binding storage | Final | NFT Extension / Identity |
| [ERC-7531 — Staked ERC-721 Ownership Recognition](/standards-mirror/tokens/erc-7531) | Staking contract OriginalOwnerOf view | Review | NFT Extension |
| [ERC-5216 — ERC-1155 Allowance Extension](/standards-mirror/tokens/erc-5216) | NEP-11 (divisible) + per-(owner, spender, tokenId) allowance | Last Call | Multi-Token Extension |
| [ERC-5008 — ERC-721 Nonce Extension](/standards-mirror/tokens/erc-5008) | NEP-11 + per-token nonce auto-incremented on transfer | Last Call | NFT Extension |
| [ERC-4910 — Royalty Bearing NFTs](/standards-mirror/tokens/erc-4910) | NEP-11 + NEP-24 + escrow + claim flow | Final | NFT Extension |
| [ERC-5732 — Commit Interface](/standards-mirror/tokens/erc-5732) | Standalone commit-reveal contract | Final | Pattern |
| [ERC-4400 — EIP-721 Consumable Extension](/standards-mirror/tokens/erc-4400) | NEP-11 + per-token consumer slot | Final | NFT Extension |
| [ERC-5750 — General Extensibility for Method Behaviors](/standards-mirror/tokens/erc-5750) | NEP-style trailing-data parameter convention | Final | Pattern |
| [ERC-5496 — Multi-privilege Management NFT Extension](/standards-mirror/tokens/erc-5496) | NEP-11 + per-(tokenId, privilegeId, holder) storage | Last Call | NFT Extension |
| [ERC-1046 — tokenURI Interoperability](/standards-mirror/tokens/erc-1046) | NEP-17 + tokenURI() view returning metadata pointer | Final | Fungible Extension |

## Related Pages

- [Standards Mirror Overview](/standards-mirror/)
- [Semantic Standards Mapping](/mapping/standards)
- [Standards and Contracts](/additional-material/neo-standards)
