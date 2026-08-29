---
title: Standards Mirror — Coverage Audit & Gap Report
description: Cross-reference of STANDARDS_MAPPING.md against the mirror catalog, plus a prioritized list of meaningful Ethereum standards not yet mirrored.
---

# Standards Mirror — Coverage Audit & Gap Report

This page answers the two recurring questions about the mirror:

1. **Audit** — does every standard mentioned in
   [`devpack/standards/STANDARDS_MAPPING.md`](https://github.com/r3e-network/neo-devpack-solidity/blob/main/devpack/standards/STANDARDS_MAPPING.md)
   have a corresponding mirror page?
2. **Gap report** — which Ethereum-side standards are notable enough to be
   worth mirroring but have not been added yet?

For the live deploy state of every mirrored standard, see the
[Coverage Matrix](./coverage-matrix).

::: tip Code samples are illustrative
Each mirror page's **Neo C#** tab shows a representative implementation
sketch, not a copy-paste-ready production contract. The samples target the
shapes exposed by [`Neo.SmartContract.Framework`](https://github.com/neo-project/neo-devpack-dotnet/tree/master/src/Neo.SmartContract.Framework)
(`Nep17Token`, `Nep11Token<TokenState>`, etc.) and follow these
conventions:

- **Mint / Burn:** the canonical NEP-17 base exposes
  `protected static void Mint(UInt160 account, BigInteger amount)` and
  `protected static void Burn(UInt160 account, BigInteger amount)` —
  two-arg only. NEP-11 base exposes
  `protected static void Mint(ByteString tokenId, TokenState token)`. The
  samples wrap these from public methods that perform per-extension setup.
- **Custom hooks:** the base classes do **not** expose `OnMint` / `OnBurn`
  / `OnBeforeTransfer` virtual hooks. Samples that show such overrides are
  illustrative — in production, perform the custom logic in your own
  public mint / burn / transfer wrappers and then call the base
  helpers, or reimplement `Transfer` with `public new static`.
- **Storage casts:** `(UInt160)Storage.Get(...)` and `(string)Storage.Get(...)`
  rely on the implicit `ByteString` ↔ scalar conversions provided by the
  framework — these compile and run correctly when the stored value
  matches the expected length / shape.
- **Account derivation from a public key:** use
  `Contract.CreateStandardAccount(pubKey)` — never cast
  `pubKey.EncodePoint(true)` to `UInt160` (33-byte compressed encoding ≠
  20-byte script hash).
- **NEP-11 "divisible" base:** several mirror pages (ERC-1155, ERC-5006,
  ERC-5216, ERC-5615, ERC-6909) describe the Neo mapping as
  "NEP-11 (divisible)". The framework only ships
  `Nep11Token<TokenState>` (non-divisible — every tokenId has a single
  owner). Divisible-style multi-balance NEP-11 (per
  [NEP-11 spec](https://github.com/neo-project/proposals/blob/master/nep-11.mediawiki))
  must be implemented manually with a `(owner, tokenId) → balance`
  storage map and custom `transfer(from, to, amount, tokenId, data)` /
  `balanceOf(owner, tokenId)` methods. The samples that inherit
  `Nep11Token<T>` for divisible semantics are showing the storage layout
  and method shape, not a turnkey base class.
- **`CallFlags`:** prefer `CallFlags.ReadOnly` for view-only
  cross-contract calls (`ownerOf`, `balanceOf`, `properties`,
  `isAuthorised`, etc.); use `CallFlags.All` only when the callee may
  mutate state or fire notifications you depend on.
- **`[Safe]` on view methods:** every read-only public method should
  carry `[Safe]` so the manifest advertises it as state-non-mutating
  (wallets and explorers rely on this for "read vs write" UI gating).
  The catalog applies it consistently to expression-bodied view methods
  (`=> Storage.Get(...)`); multi-line view bodies follow the same rule
  but are easy to miss when adapting samples — add `[Safe]` on the
  declaration line above any method that doesn't write storage or fire
  notifications.
- **Event names:** every `public static event Action<...>` declaration
  is paired with a `[DisplayName("CanonicalName")]` so the manifest
  emits the spec-canonical name (`Transfer`, `Approval`, etc.) even
  when the C# field uses an `OnXxx` alias to avoid clashing with the
  base class's notification field. Pages inheriting `Nep17Token` /
  `Nep11Token<T>` never redeclare `Transfer` — they reuse the base
  class's event field.
- **Storage key building:** samples build prefixed keys manually with
  `new byte[] { Prefix_X }.Concat(suffix)` rather than using the
  `StorageMap` helper. Both produce identical on-chain layouts, but
  the manual form makes the prefix byte explicit in every method
  (helpful for porters who want to predict storage keys before reading
  the helper's source). Production code may prefer
  `new StorageMap(Storage.CurrentContext, Prefix_X)` for ergonomics —
  the migration is mechanical and doesn't change semantics.

Verify against the canonical sources before deploying:
[Nep17Token.cs](https://github.com/neo-project/neo-devpack-dotnet/blob/master/src/Neo.SmartContract.Framework/Nep17Token.cs),
[Nep11Token.cs](https://github.com/neo-project/neo-devpack-dotnet/blob/master/src/Neo.SmartContract.Framework/Nep11Token.cs).
:::

## 1. STANDARDS_MAPPING.md Audit

The devpack mapping doc references the following ERCs / EIPs in its quick
reference table. Each row below shows whether a dedicated mirror page exists.

| Mapping Doc Row | Mirror Page |
| --- | --- |
| ERC-20 ↔ NEP-17 | ✅ [tokens/erc-20](./tokens/erc-20) |
| ERC-721 ↔ NEP-11 | ✅ [tokens/erc-721](./tokens/erc-721) |
| ERC-2981 ↔ NEP-24 | ✅ [tokens/erc-2981](./tokens/erc-2981) |
| ERC-1155 — Multi-Token | ✅ [tokens/erc-1155](./tokens/erc-1155) |
| EIP-165 ↔ Manifest | ✅ [infrastructure/erc-165](./infrastructure/erc-165) |
| EIP-712 ↔ Witness model | ✅ [account-and-auth/eip-712](./account-and-auth/eip-712) |
| EIP-2612 (Permit) | ✅ [account-and-auth/erc-2612](./account-and-auth/erc-2612) |
| EIP-1967 (Proxy) | ✅ [infrastructure/erc-1967](./infrastructure/erc-1967) |
| ERC-721 Receiver ↔ NEP-26 | ✅ Documented inline on [tokens/erc-721](./tokens/erc-721); no dedicated page (it's a callback contract, not an ERC) |
| ERC-677 / ERC-1363 hooks ↔ NEP-27 | ✅ [tokens/erc-1363](./tokens/erc-1363) |

**Audit result:** every ERC/EIP row in the quick-reference table now has a
mirror page. The Neo-side callback NEPs (NEP-22, NEP-26, NEP-27, NEP-29,
NEP-30, NEP-31) are described inline in the relevant ERC pages and in
[`additional-material/neo-standards/`](/additional-material/neo-standards),
which is the right place for the Neo-native side of the mirror.

### Suggested follow-ups

- **Standalone NEP pages.** `additional-material/neo-standards/` ships
  [NEP-11](/additional-material/neo-standards/nep-11-non-fungible-tokens),
  [NEP-17](/additional-material/neo-standards/nep-17-fungible-tokens), and
  [NEP-24](/additional-material/neo-standards/nep-24-royalty-standard) but no
  dedicated docs for NEP-22 (update), NEP-26 / NEP-27 (payment callbacks),
  NEP-29 (deploy lifecycle), NEP-30 (verify), or NEP-31 (destroy). These are
  referenced by many ERC mirror pages and warrant first-class explainers.
- **Mapping doc consolidation.** `STANDARDS_MAPPING.md` lives under
  `devpack/standards/` and predates the mirror's category split. Most of its
  prose is now duplicated in the per-ERC mirror pages. Either (a) convert it
  to a one-page index that links into the mirror, or (b) keep it as the
  canonical machine-readable mapping table and remove the long-form prose.

## 2. Gap Report — Notable Final ERCs Not Mirrored

The Ethereum EIPs index lists ~140 ERCs at status **Final**. The mirror
covers the standards with the highest application-level relevance for Neo
migration — token primitives, accounts, and infrastructure. The list below
is the curated set of meaningful gaps, grouped by priority.

### Recently closed (now mirrored)

Successive priority passes have closed the application-relevant gaps from
the [Ethereum EIPs index](https://eips.ethereum.org/erc) — 129 mirror pages
across the five categories above. The table below is the audit trail of the
highest-impact additions, grouped by reason:

| Standard | Mirror Page | Why it mattered |
| --- | --- | --- |
| ERC-1167 | [infrastructure/erc-1167](./infrastructure/erc-1167) | Single most-used proxy pattern; underpins every Ethereum factory |
| ERC-3448 | [infrastructure/erc-3448](./infrastructure/erc-3448) | Init-supporting MetaProxy variant of ERC-1167 |
| ERC-3643 | [tokens/erc-3643](./tokens/erc-3643) | Compliance framework for tokenised securities (T-REX) |
| ERC-3668 | [infrastructure/erc-3668](./infrastructure/erc-3668) | Off-chain data via CCIP Read; mirrors to native Oracle |
| ERC-4361 | [account-and-auth/erc-4361](./account-and-auth/erc-4361) | De-facto web3 sign-in (SIWE) |
| ERC-4907 | [tokens/erc-4907](./tokens/erc-4907) | Rental NFT user-role extension; gaming + scholarship use cases |
| ERC-5202 | [infrastructure/erc-5202](./infrastructure/erc-5202) | Blueprint contract format; completes the "deploy from data" family |
| ERC-5313 | [account-and-auth/erc-5313](./account-and-auth/erc-5313) | Light Contract Ownership; view-only convention |
| ERC-5564 | [account-and-auth/erc-5564](./account-and-auth/erc-5564) | Stealth Addresses; privacy primitive on secp256r1 |
| ERC-6066 | [account-and-auth/erc-6066](./account-and-auth/erc-6066) | NFT-aware signature validation (ERC-1271 per tokenId) |
| ERC-6093 | [tokens/erc-6093](./tokens/erc-6093) | Standardised error vocabulary for NEP-17 / NEP-11 ports |
| ERC-7528 | [tokens/erc-7528](./tokens/erc-7528) | Native asset address convention; relevant for any DEX or vault |
| ERC-7535 | [defi/erc-7535](./defi/erc-7535) | Native asset version of ERC-4626 vaults |
| ERC-7656 | [account-and-auth/erc-7656](./account-and-auth/erc-7656) | Generalisation of ERC-6551 to any contract |
| ERC-7786 | [infrastructure/erc-7786](./infrastructure/erc-7786) | Cross-chain messaging gateway interface |
| ERC-7944 | [defi/erc-7944](./defi/erc-7944) | Async cancellation for ERC-7540 vaults |
| ERC-8042 | [infrastructure/erc-8042](./infrastructure/erc-8042) | Diamond storage convention for ERC-2535 |
| ERC-5679 | [tokens/erc-5679](./tokens/erc-5679) | Standardised mint/burn surface across token types |
| ERC-2135 | [tokens/erc-2135](./tokens/erc-2135) | Consumable interface (tickets, permits, vouchers) |
| ERC-7160 | [tokens/erc-7160](./tokens/erc-7160) | ERC-721 multi-metadata extension |
| ERC-3475 | [defi/erc-3475](./defi/erc-3475) | Abstract storage bonds (multi-class, multi-nonce) |
| ERC-7092 | [defi/erc-7092](./defi/erc-7092) | Financial bonds (issuer-facing surface) |
| ERC-6982 | [tokens/erc-6982](./tokens/erc-6982) | Default lockable tokens (lighter than 5192/6147) |
| ERC-7677 | [account-and-auth/erc-7677](./account-and-auth/erc-7677) | Paymaster web service capability (AA handshake) |
| ERC-7144 | [tokens/erc-7144](./tokens/erc-7144) | ERC-20 with transaction validation step (lightweight T-REX) |
| ERC-7943 | [tokens/erc-7943](./tokens/erc-7943) | Universal Real World Asset Interface (uRWA) |
| ERC-5006 | [tokens/erc-5006](./tokens/erc-5006) | Rental NFT user extension for ERC-1155 multi-tokens |
| ERC-2767 | [account-and-auth/erc-2767](./account-and-auth/erc-2767) | Contract Ownership Governance |
| ERC-7758 | [account-and-auth/erc-7758](./account-and-auth/erc-7758) | Modern Transfer-With-Authorization |
| ERC-5169 | [tokens/erc-5169](./tokens/erc-5169) | Client Script URI for token contracts |
| ERC-5375 | [tokens/erc-5375](./tokens/erc-5375) | NFT Author Information and Consent |
| ERC-5023 | [tokens/erc-5023](./tokens/erc-5023) | Shareable Non-Fungible Token (multi-holder) |
| ERC-7066 | [tokens/erc-7066](./tokens/erc-7066) | Approval-based NFT Lock Extension |
| ERC-7432 | [tokens/erc-7432](./tokens/erc-7432) | NFT Roles (time-bounded grants) |
| ERC-6105 | [tokens/erc-6105](./tokens/erc-6105) | No-Intermediary NFT Trading Protocol |
| ERC-5615 | [tokens/erc-5615](./tokens/erc-5615) | ERC-1155 Supply Extension |
| ERC-5773 | [tokens/erc-5773](./tokens/erc-5773) | Context-Dependent Multi-Asset Tokens |
| ERC-6059 | [tokens/erc-6059](./tokens/erc-6059) | Parent-Governed Nestable NFTs |
| ERC-4519 | [tokens/erc-4519](./tokens/erc-4519) | NFTs Tied to Physical Assets |
| ERC-5570 | [tokens/erc-5570](./tokens/erc-5570) | Digital Receipt NFT |
| ERC-6150 | [tokens/erc-6150](./tokens/erc-6150) | Hierarchical NFTs |
| ERC-6220 | [tokens/erc-6220](./tokens/erc-6220) | Composable NFTs Equippable Parts |
| ERC-5380 | [tokens/erc-5380](./tokens/erc-5380) | ERC-721 Entitlement Extension |
| ERC-5489 | [tokens/erc-5489](./tokens/erc-5489) | NFT Hyperlink Extension |
| ERC-6672 | [tokens/erc-6672](./tokens/erc-6672) | Multi-Redeemable NFTs |
| ERC-7634 | [tokens/erc-7634](./tokens/erc-7634) | Limited Transfer Count NFT |
| ERC-7007 | [tokens/erc-7007](./tokens/erc-7007) | Verifiable AI-Generated Content Token |
| ERC-5725 | [tokens/erc-5725](./tokens/erc-5725) | Transferable Vesting NFT |
| ERC-6454 | [tokens/erc-6454](./tokens/erc-6454) | Minimal Transferable NFT detection |
| ERC-7715 | [account-and-auth/erc-7715](./account-and-auth/erc-7715) | Permission Grants for Smart Accounts (AA) |
| ERC-5507 | [tokens/erc-5507](./tokens/erc-5507) | Refundable Tokens |
| ERC-5528 | [tokens/erc-5528](./tokens/erc-5528) | Refundable Fungible Token |
| ERC-5521 | [tokens/erc-5521](./tokens/erc-5521) | Referable NFT |
| ERC-5646 | [tokens/erc-5646](./tokens/erc-5646) | Token State Fingerprint |
| ERC-5585 | [tokens/erc-5585](./tokens/erc-5585) | ERC-721 NFT Authorization |

### High priority (next-up)

Eleven batches closed. Remaining high-priority covers physical-asset
binding, semantic soulbound, NFT discoverability, modular wallet, and
SIWE generalisation:

| Standard | Title | Status | Why it matters |
| --- | --- | --- | --- |
| **ERC-6239** | Semantic Soulbound Tokens | Final | RDF-tagged soulbound credentials; pairs with ERC-5192/5484 for richer credential metadata. |
| **ERC-7053** | Interoperable Digital Media Indexing | Final | Cross-platform media discovery; closes the indexer-side story for ERC-5169 / ERC-5489. |
| **ERC-7405** | Modular Wallet Interface | Draft | Pairs with ERC-7579 / ERC-7677 / ERC-7715 to complete the AA stack. |
| **ERC-7585** | Permitted Authentication Scheme | Draft | Generalised ERC-4361 (SIWE) for any chain; formalises the cross-chain auth story. |
| **ERC-6381** | Public NFT Emote Repository | Final | Reaction system for NFTs; thin but commonly-requested social primitive. |

### Medium priority

| Standard | Title | Status | Notes |
| --- | --- | --- | --- |
| ERC-7857 | AI Agents NFT with Private Metadata | Final | Pairs with ERC-7007 for AI-agent NFTs. |
| ERC-7231 | Identity-aggregated NFT | Final | Identity-bound NFTs aggregating multiple credentials. |
| ERC-7531 | Staked ERC-721 Ownership Recognition | Review | Acknowledge staked NFTs as ownership equivalents for snapshots. |
| ERC-6982 | Already mirrored ✓ | — | — |
| ERC-6093 | Already mirrored ✓ | — | — |
| ERC-7160 | Already mirrored ✓ | — | — |

### Watch list (Review / Last Call)

These are not yet Final but are widely deployed or actively shipped:

| Standard | Title | Status | Notes |
| --- | --- | --- | --- |
| **ERC-7758** | Transfer With Authorization (modern) | Review | Successor to ERC-3009 (which we mirror). Add when promoted to Final or when adoption shifts. |
| ERC-7715 | Permission grants for accounts | Draft | Account-abstraction permission model. Track for ERC-4337 / ERC-7579 integration. |
| ERC-7943 | Universal Real World Asset Interface | Last Call | RWA standardisation. Track and add when Final. |
| ERC-7786 | Cross-Chain Messaging Gateway | Final | Cross-chain interop. Worth mirroring once the bridge story for Neo is more concrete. |

### Intentionally out of scope

Some ERCs are not worth mirroring because they describe Ethereum-specific
infrastructure with no meaningful Neo equivalent:

- **EIP-2333 / 2334 / 2335** (BLS12-381 key derivation) — Beacon-chain specific.
- **EIP-4844** (Blob transactions) — Already explained in `protocol-eips`.
- **ERC-820 / ERC-1820** — We mirror 1820 (the live one); 820 is the
  superseded predecessor.
- Most NFT-extension drafts under `4400-7900` that are single-use community
  proposals without traction — track in the watch list, add only on demand.

## How to Add a New Mirror Page

1. Pick the right category directory under
   `docs/standards-mirror/{tokens,defi,infrastructure,protocol-eips,account-and-auth}/`.
2. Copy an existing mirror page as a template (recommended: `tokens/erc-20.md`
   for a deployed example, `tokens/erc-6909.md` for a catalog-only example).
3. Fill in `<StandardEntry>` props (`id`, `title`, `eip`, `status`,
   `neoMapping`, `category`, `parityLabel`, `parityClass`).
4. Add the row to the category index page (e.g. `tokens.md`) and bump the
   "X standards" count in the prose.
5. Update the main [Standards Mirror Overview](./) — bump category counts
   and the live-on-TestNet headline if you also deploy a pair.
6. Add to the [Coverage Matrix](./coverage-matrix) and (if catalog-only)
   to the explanatory list in `deployments/DEFERRED.md`.
7. Run `npm run docs:check` to validate links.

## Related

- [Standards Mirror Overview](./)
- [Coverage Matrix](./coverage-matrix)
- [Latest TestNet Results](./deployments/RESULTS)
- [Deferred Deployment Queue](./deployments/DEFERRED)
- [`devpack/standards/STANDARDS_MAPPING.md`](https://github.com/r3e-network/neo-devpack-solidity/blob/main/devpack/standards/STANDARDS_MAPPING.md)
