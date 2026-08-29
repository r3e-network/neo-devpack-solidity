# Standards Mirror — Deferred Deployment Queue

> **Queue empty for the checked-in deployment snapshot.** The recurring weekly
> agent has been retired; future changes should use the normal standards mirror
> deploy workflow and update `results.json` / `RESULTS.md`.
>
> The mirror catalog now exposes **129 ERC/EIP mapping pages**. The TestNet matrix
> covers **47 deployable standards**. The eighty-two catalog entries without a
> live contract pair fall into three groups:
>
> **Protocol-irrelevant** (3): EIP-1559 (fee-market auction), EIP-4844 (blob
> transactions), EIP-3074 (superseded by EIP-7702 and covered by witness scopes).
>
> **Documented Neo C# port without checked-in fixture** (1): ERC-6909.
>
> **Subsumed by a native Neo mechanism** (78): ERC-1363 (NEP-17 callback),
> ERC-3009 (witness scopes), ERC-6551 (registry-pattern TBA), ERC-1167
> (ContractManagement.Deploy), ERC-4361 (SIWE — off-chain server convention),
> ERC-3668 (native Oracle service), ERC-7528 (well-known native contract hashes),
> ERC-7656 (registry-pattern TBA generalisation), ERC-3448 (parameterised
> ContractManagement.Deploy), ERC-7535 (collapses to ERC-4626 + native asset),
> ERC-4907 (NEP-11 storage extension), ERC-6093 (vocabulary convention),
> ERC-7786 (bridge-adapter, per-transport deploy), ERC-5313 (view convention,
> satisfied by every contract with `getOwner`), ERC-5564 (announcer + wallet-SDK
> dependency), ERC-6066 (NEP-11 per-tokenId verify extension), ERC-3643
> (multi-contract framework, per-deployment shape), ERC-5202 (Neo's deploy
> primitive already separates blob storage from execution), ERC-7944
> (extension of existing ERC-7540 deployment), ERC-8042 (storage convention
> for existing ERC-2535 deployment), ERC-5679 (selector convention),
> ERC-2135 (NEP-11 storage extension), ERC-7160 (NEP-11 storage extension),
> ERC-3475 (per-issuance configuration), ERC-7092 (per-bond configuration),
> ERC-6982 (NEP-11 storage extension), ERC-7677 (off-chain JSON-RPC convention),
> ERC-7144 (per-deployment validator shape), ERC-7943 (capability-flag
> umbrella for RWA, per-deployment shape), ERC-5006 (NEP-11 divisible
> + record storage), ERC-2767 (interface convention satisfied by any
> governance-owned contract), ERC-7758 (witness scopes — same as ERC-3009),
> ERC-5169 (one-method addition to any NEP-11 / NEP-17 contract),
> ERC-5375 (NEP-11 metadata extension), ERC-5023 (NEP-11 multi-holder
> extension), ERC-7066 (NEP-11 lock-state extension), ERC-7432 (NEP-11
> role storage extension), ERC-6105 (NEP-11 listing/swap extension),
> ERC-5615 (NEP-11 divisible supply extension), ERC-5773 (NEP-11
> multi-asset extension), ERC-6059 (NEP-11 parent-child extension),
> ERC-4519 (NEP-11 + device-pubkey extension), ERC-5570 (NEP-11
> receipt-metadata extension), ERC-6150 (NEP-11 hierarchical
> extension), ERC-6220 (NEP-11 + catalog extension), ERC-5380
> (NEP-11 entitlement extension), ERC-5489 (NEP-11 hyperlink
> extension), ERC-6672 (NEP-11 multi-redemption extension), ERC-7634
> (NEP-11 transfer-counter extension), ERC-7007 (NEP-11 AIGC-attestation
> extension), ERC-5725 (NEP-11 + NEP-17 vesting extension), ERC-6454
> (NEP-11 IsTransferable view), ERC-7715 (witness-scope-based AA
> permission module), ERC-5507 (NEP-11 refund extension), ERC-5528
> (NEP-17 refund extension), ERC-5521 (NEP-11 citation-graph extension),
> ERC-5646 (NEP-11 state-fingerprint view), ERC-5585 (NEP-11 rights
> authorization extension), ERC-6239 (NEP-11 RDF-claims extension),
> ERC-7053 (event-emission convention), ERC-7405 (per-account module
> dispatch), ERC-7585 (off-chain library convention), ERC-6381
> (standalone emote repository singleton), ERC-7857 (NEP-11
> encrypted-metadata extension), ERC-7231 (NEP-11 identity-Merkle
> extension), ERC-7531 (staking-contract view), ERC-5216 (NEP-11
> divisible allowance extension), ERC-5008 (NEP-11 nonce extension),
> ERC-4910 (NEP-11/NEP-24 royalty escrow extension), ERC-5732
> (standalone commit-reveal contract), ERC-4400 (NEP-11 consumer-slot
> extension), ERC-5750 (NEP-style trailing-data convention, satisfied
> by every NEP-17/NEP-11 contract), ERC-5269 (manifest
> `supportedstandards` already provides this natively), ERC-5453
> (witness scopes already cover per-call signature auth), ERC-5496
> (NEP-11 multi-privilege storage extension), ERC-6357 (Neo's
> transaction script natively supports multi-invoke), ERC-1046
> (NEP-17 view convention), ERC-7746 (per-deployment hook chain).
>
> A single fixture deploy would not exercise these standards meaningfully — they
> are catalog explainers, not checked-in TestNet pairs.

## Snapshot log

- **Baseline deploys** (the original mirror set, predates this log): ERC-20,
  ERC-721, ERC-173, ERC-1056, ERC-1155, ERC-1271, ERC-1820, ERC-1967,
  ERC-2535, ERC-2771, ERC-2981, ERC-3156, ERC-3525, ERC-4626, ERC-5192,
  ERC-5805, ERC-6372, ERC-7818.
- Earlier deploy batches added ERC-777, ERC-5267, ERC-5114, ERC-5484,
  ERC-6147, ERC-2470, ERC-2309, ERC-4906, ERC-165, ERC-7201, EIP-1153,
  EIP-3198, ERC-1014, EIP-191, ERC-2612, ERC-4494, ERC-7540, ERC-7575,
  ERC-7579, ERC-4337, ERC-6492, EIP-712, and EIP-2098.
- The current snapshot also includes live demos for EIP-2718, EIP-2930,
  EIP-3855, EIP-3860, EIP-6780, and EIP-7702.
- Total: **47 live pairs** in `results.json` (snapshot
  `2026-04-29T02:47:17.306Z`); 18 baseline + 23 earlier batches + 6 most-recent
  protocol-EIPs = 47.

## Compile guardrails (kept for future contract authors)

These came out of the deploy effort that produced PRs #5–#11. Future Neo DevPack for Solidity
contracts must respect them or they fault on real Neo nodes:

- **`uint256(uintN var)` casts in constructor fault at deploy** (neo-solc 0.18,
  still open). Use literal pre-computed values.
- **`emit Event(...)` whose `keccak256(signature)` starts with byte `0xDD`** would
  fault before PR #6's compiler fix. The fix shipped, but stick to non-deploy
  paths for emits as a safety margin.
- **`target.call(opaque_bytes)` is unsupported by neo-solc** — it requires
  statically-known method names. Use named-method invocation patterns instead.
- **`msg.sender` at constructor time on Neo is the ManagementContract**, not the
  deploying user. Use a `claimOwner()` / `claimIssuer()` post-deploy pattern in
  Solidity. The deploy runner must invoke the claim in the same deployment flow;
  manual deploys that leave a claimable owner unset allow the first post-deploy
  caller to seize that role. PR #6's `msg.sender` fix routes to
  `Transaction.Sender` for the ContractManagement caller, but constructor-time
  identity is still risky.
- **Neo C# `(ByteString)str` for string→ByteString**, never
  `Encoding.UTF8.GetBytes` (nccs rejects).
- **Use `Runtime.Transaction.Sender` at deploy time** in C#, not
  `Runtime.CallingScriptHash`.
- **Add `[Safe]` to every read-only method** so the manifest correctly advertises
  read/write surface.
