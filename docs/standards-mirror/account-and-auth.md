---
title: Account & Authentication — ERC ↔ Neo Mirror
description: Ethereum signature, ownership, and account-abstraction standards mirrored to Neo's native witness model and idiomatic C# patterns.
---

# Account & Authentication

Twenty-four standards covering ownership (full / light / governance-mediated), off-chain
signatures (legacy + modern + endorsement), smart-contract signature verification,
NFT-aware signature validation, stealth-address privacy, account abstraction,
EOA-as-contract proposals, NFT-bound accounts, generalised contract-linked services,
paymaster handshakes, modular wallet plugins, scoped permission grants, and SIWE
(single-chain + generalised). Most of these are elaborate workarounds for Ethereum's
"EOA can only do ECDSA" limitation; Neo's witness model handles the same use cases at
the protocol level.

## Standards

| Standard | Neo Mapping | Status | Category |
| --- | --- | --- | --- |
| [ERC-173 — Contract Ownership](/standards-mirror/account-and-auth/erc-173) | Owner pattern + NEP-22 | Final | Ownership |
| [ERC-5313 — Light Contract Ownership](/standards-mirror/account-and-auth/erc-5313) | `getOwner()` view convention | Final | Ownership |
| [ERC-2767 — Contract Ownership Governance](/standards-mirror/account-and-auth/erc-2767) | Governance contract as owner + manifest-permission gate | Final | Ownership |
| [ERC-1271 — Smart Contract Signatures](/standards-mirror/account-and-auth/erc-1271) | Native witness model | Final | Signatures |
| [ERC-2612 — Permit (Gasless Approval)](/standards-mirror/account-and-auth/erc-2612) | Native witness scopes | Final | Signatures |
| [ERC-3009 — Transfer With Authorization](/standards-mirror/account-and-auth/erc-3009) | Native witness scopes | Final (USDC) / Stagnant (EIP) | Signatures |
| [ERC-7758 — Transfer With Authorization (modern)](/standards-mirror/account-and-auth/erc-7758) | Native witness scopes (same as [ERC-3009](/standards-mirror/account-and-auth/erc-3009)) | Review | Signatures |
| [ERC-4337 — Account Abstraction](/standards-mirror/account-and-auth/erc-4337) | Native NEP-30 verify | Final | Smart Accounts |
| [ERC-7677 — Paymaster Web Service Capability](/standards-mirror/account-and-auth/erc-7677) | Sponsor relayer + signed gas-budget pattern | Review | Smart Accounts |
| [ERC-7715 — Permission Grants for Smart Accounts](/standards-mirror/account-and-auth/erc-7715) | Witness scopes + per-grant allowlist contract | Draft | Smart Accounts |
| [ERC-7405 — Modular Wallet Interface](/standards-mirror/account-and-auth/erc-7405) | NEP-30 verify + module dispatch (extends [ERC-7579](/standards-mirror/infrastructure/erc-7579)) | Draft | Smart Accounts |
| [ERC-7585 — Permitted Authentication Scheme](/standards-mirror/account-and-auth/erc-7585) | Native witness over scheme-tagged message | Draft | Authentication |
| [ERC-5453 — Endorsement (Permit for Any Functions)](/standards-mirror/account-and-auth/erc-5453) | Native witness scopes per call | Last Call | Signatures |
| [ERC-4361 — Sign-In with Ethereum (SIWE)](/standards-mirror/account-and-auth/erc-4361) | Native witness over a domain-bound message | Final | Authentication |
| [ERC-5564 — Stealth Addresses](/standards-mirror/account-and-auth/erc-5564) | secp256r1 ECDH + announcer contract | Final | Privacy |
| [ERC-6066 — Signature Validation Method for NFTs](/standards-mirror/account-and-auth/erc-6066) | NEP-11 + NEP-30 verify, scoped per tokenId | Final | Signatures |
| [EIP-712 — Typed Structured Data Signing](/standards-mirror/account-and-auth/eip-712) | Native witness model | Final | Signatures |
| [EIP-191 — Signed Data Prefix](/standards-mirror/account-and-auth/eip-191) | Native witness model | Final | Signatures |
| [ERC-5267 — EIP-712 Domain Retrieval](/standards-mirror/account-and-auth/erc-5267) | Native | Final | Signatures |
| [ERC-6492 — Signatures for Pre-deployed Contracts](/standards-mirror/account-and-auth/erc-6492) | Native (no counterfactual deploy) | Final | Signatures |
| [ERC-6551 — Token Bound Accounts](/standards-mirror/account-and-auth/erc-6551) | Registry + per-NFT contract account | Review | Smart Accounts |
| [ERC-7656 — Generalized Contract-Linked Services](/standards-mirror/account-and-auth/erc-7656) | Registry pattern (mode-flagged) | Final | Service Discovery |
| [EIP-7702 — Set Code for EOAs](/standards-mirror/account-and-auth/eip-7702) | Native (every account is a contract) | Final (Pectra) | Smart Accounts |
| [EIP-3074 — AUTH and AUTHCALL](/standards-mirror/account-and-auth/eip-3074) | Native (witness scopes) | Stagnant (superseded by 7702) | Smart Accounts |

## Related Pages

- [Standards Mirror Overview](/standards-mirror/)
- [Semantic Standards Mapping](/mapping/standards)
- [Standards and Contracts](/additional-material/neo-standards)
