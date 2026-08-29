---
title: ERC / EIP ↔ Neo Standards Mirror
description: Every meaningful Ethereum standard, mirrored to its Neo N3 equivalent — with side-by-side Solidity and Neo C# implementations.
---

# ERC / EIP ↔ Neo Standards Mirror

Most application-level Ethereum standards have a Neo N3 counterpart — sometimes a
one-to-one NEP, sometimes a stronger native primitive that makes the EIP
unnecessary, and sometimes a migration pattern with known semantic differences.
This module catalogs the checked-in mirror set: pick a category below, then flip
between the **ERC/EIP detail**, the **Solidity implementation**, and the
equivalent **Neo C# implementation**.

Use the mirror as a migration map, then validate production behavior against the
deployment results and the Solidity support matrix.

## Categories

<div class="mirror-cat-grid">

  <a class="mirror-cat" href="./tokens">
    <div class="cat-eyebrow">Token Standards</div>
    <div class="cat-title">Fungibles, NFTs, multi-token, RWA, multi-priv, fungible-metadata</div>
    <div class="cat-desc">
      ERC-20, 721, 777, 1046, 1155, 1363, 2135, 2981, 3525, 3643, 4400, 4519, 4906, 4907, 4494, 4910, 5006, 5008, 5023, 5169, 5192, 5216, 5375, 5380, 5484, 5489, 5496, 5507, 5521, 5528, 5570, 5585, 5615, 5646, 5679, 5725, 5732, 5750, 5773, 6059, 6093, 6105, 6150, 6220, 6239, 6381, 6454, 6672, 6909, 6982, 2309, 5114, 6147, 7007, 7053, 7066, 7144, 7160, 7231, 7432, 7528, 7531, 7634, 7857, 7943
    </div>
    <div class="cat-pill-row">
      <span class="cat-pill cat-pill-direct">65 standards</span>
      <span class="cat-pill cat-pill-neo">NEP-11 / NEP-17 / NEP-24</span>
    </div>
  </a>

  <a class="mirror-cat" href="./account-and-auth">
    <div class="cat-eyebrow">Account & Authentication</div>
    <div class="cat-title">Ownership, signatures, smart accounts, modular wallets, SIWE, paymasters, endorsements</div>
    <div class="cat-desc">
      ERC-173, 1271, 2612, 2767, 3009, 4337, 4361, 5267, 5313, 5453, 5564, 6066, 6492, 6551, 7405, 7585, 7656, 7677, 7715, 7758, EIP-712, EIP-191, EIP-7702, EIP-3074
    </div>
    <div class="cat-pill-row">
      <span class="cat-pill cat-pill-direct">24 standards</span>
      <span class="cat-pill cat-pill-native">Native witness scopes</span>
    </div>
  </a>

  <a class="mirror-cat" href="./infrastructure">
    <div class="cat-eyebrow">Infrastructure & Patterns</div>
    <div class="cat-title">Proxies, blueprints, diamond storage, registries, multicall, security middleware</div>
    <div class="cat-desc">
      ERC-165, 1014, 1056, 1167, 1820, 1967, 2470, 2535, 2771, 3448, 3668, 5202, 5269, 6357, 7201, 7579, 7746, 7786, 8042
    </div>
    <div class="cat-pill-row">
      <span class="cat-pill cat-pill-direct">19 standards</span>
      <span class="cat-pill cat-pill-neo">NEP-22 + Oracle native</span>
    </div>
  </a>

  <a class="mirror-cat" href="./defi">
    <div class="cat-eyebrow">DeFi Building Blocks</div>
    <div class="cat-title">Vaults (sync, async, cancellable), flash loans, governance, bonds</div>
    <div class="cat-desc">
      ERC-3156, 3475, 4626, 5805, 6372, 7092, 7535, 7540, 7575, 7818, 7944
    </div>
    <div class="cat-pill-row">
      <span class="cat-pill cat-pill-direct">11 standards</span>
      <span class="cat-pill cat-pill-pattern">Composition patterns</span>
    </div>
  </a>

  <a class="mirror-cat" href="./protocol-eips">
    <div class="cat-eyebrow">Protocol-Level EIPs</div>
    <div class="cat-title">Things Ethereum had to fix at the protocol</div>
    <div class="cat-desc">
      EIP-1559, 2718, 2930, 3198, 3855, 3860, 4844, 1153, 6780, 2098
    </div>
    <div class="cat-pill-row">
      <span class="cat-pill cat-pill-direct">10 EIPs</span>
      <span class="cat-pill cat-pill-native">Most are no-ops on Neo</span>
    </div>
  </a>

</div>

## Live on TestNet

The frontend catalog currently exposes **129 ERC/EIP mapping pages** across the five
categories above. **Forty-seven standards** in this mirror have Solidity and Neo C# contract pairs
deployed on Neo N3 TestNet (network magic `894710606`). The same invocation matrix
runs against the Solidity (compiled with `neo-solc`) and the Neo C# (compiled with
`nccs`) versions, recording pass/fail assertion results for both implementations.

::: warning Snapshot status
The checked-in TestNet snapshot is a validation matrix, not an all-green parity certification. The latest checked-in result records 147 / 183 assertions passing; failures and repro details are listed in `deployments/RESULTS.md`.
:::

| Standard | Solidity | Neo C# |
|---|---|---|
| **ERC-20** ↔ NEP-17 | [d76434af…f96](https://dora.coz.io/contract/neo3/testnet/0xd76434af829dc4c936c12648aa77932fa94c0f96) | [1f3a9b41…b43a](https://dora.coz.io/contract/neo3/testnet/0x1f3a9b414de1c60434543dd8a05ac5e08b75b43a) |
| **ERC-165** InterfaceDetector | [2b5db552…1e6](https://dora.coz.io/contract/neo3/testnet/0x2b5db552d1c23a43f51a0ea50765e4a1a7a051e6) | [a400b6cb…f49](https://dora.coz.io/contract/neo3/testnet/0xa400b6cb20159fb3140798401c41edcb06e00f49) |
| **ERC-721** ↔ NEP-11 | [48b5f8f5…7aa](https://dora.coz.io/contract/neo3/testnet/0x48b5f8f579810b402fed660844145fed406f77aa) | [15c664d5…baf](https://dora.coz.io/contract/neo3/testnet/0x15c664d51340a102490dbf5dec5647f541775baf) |
| **ERC-777** Hooked | [d0f1fb49…7d](https://dora.coz.io/contract/neo3/testnet/0xd0f1fb49a76b1e6aaf63cf2e2e132607950e5e7d) | [0d64d453…849](https://dora.coz.io/contract/neo3/testnet/0x0d64d453a705033c2698de7a4de9e5fd934b2849) |
| **ERC-1014** DeterministicFactory | [c267a2ea…b0e](https://dora.coz.io/contract/neo3/testnet/0xc267a2eaa32edae5ac95d484a69e77653fe10b0e) | [462113ca…4f8](https://dora.coz.io/contract/neo3/testnet/0x462113ca40c8a41597165ccbeada2e70e57764f8) |
| **ERC-1155** Multi-Token | [f1d7867c…317](https://dora.coz.io/contract/neo3/testnet/0xf1d7867c140a016333b69d3e1795b0ee224d0317) | [ef019e6f…6bd](https://dora.coz.io/contract/neo3/testnet/0xef019e6feb75fd331149cb7c9c3ddfcaa8ba86bd) |
| **ERC-2309** BatchMint | [20262b3b…900](https://dora.coz.io/contract/neo3/testnet/0x20262b3b96d92a0db7bfdc4782903fb3d997f900) | [2e157ce2…918](https://dora.coz.io/contract/neo3/testnet/0x2e157ce2532dee6084f53c1a848975bd960be918) |
| **ERC-2470** SingletonFactory | [625c19cb…012](https://dora.coz.io/contract/neo3/testnet/0x625c19cbd8d0b5cf03bd9996b67a824c14448012) | [602d11ec…df5](https://dora.coz.io/contract/neo3/testnet/0x602d11eca4ebba2799b076fdbba251d1d9eaedf5) |
| **ERC-2981** ↔ NEP-24 | [ade57dfd…234](https://dora.coz.io/contract/neo3/testnet/0xade57dfd9ad85fff8dca3845cf22206346468234) | [bf3fe7eb…5e1](https://dora.coz.io/contract/neo3/testnet/0xbf3fe7eb875750c81c2915d53123c380685a65e1) |
| **ERC-3525** Bond | [d0fd56da…2c6](https://dora.coz.io/contract/neo3/testnet/0xd0fd56dad510d54ca7877bab2c578d63b82a52c6) | [fcfde62a…6c](https://dora.coz.io/contract/neo3/testnet/0xfcfde62a4764cbcd9b35615084e0075c4bddba6c) |
| **ERC-4906** DynamicMetadataNFT | [86f4d37e…793](https://dora.coz.io/contract/neo3/testnet/0x86f4d37e2471fdddf6738bb977de99646102b793) | [3429da47…061](https://dora.coz.io/contract/neo3/testnet/0x3429da478e520ac009dc64520c8c3ccd00158061) |
| **ERC-5114** Achievement | [91e34b16…41f](https://dora.coz.io/contract/neo3/testnet/0x91e34b16c373f845024013c3bd585ac9739b741f) | [d9d32f5f…039](https://dora.coz.io/contract/neo3/testnet/0xd9d32f5f8d2d0cd5196cd94b49e3d11ac46d7039) |
| **ERC-5192** Soulbound | [1b75ecb9…034](https://dora.coz.io/contract/neo3/testnet/0x1b75ecb9e926203e66283e3f875ba5097f3c3034) | [7081fcf3…2b](https://dora.coz.io/contract/neo3/testnet/0x7081fcf36db56a716b416ef553829ed23c07da2b) |
| **ERC-5267** DomainExposer | [1dd8a392…b6c](https://dora.coz.io/contract/neo3/testnet/0x1dd8a39225d515a4621c5214f336c78f4b19bb6c) | [dcfa0661…877](https://dora.coz.io/contract/neo3/testnet/0xdcfa06612bfa8614e4d197bc8206b68320cd9877) |
| **ERC-5484** ConsensualSBT | [8a9e1835…951](https://dora.coz.io/contract/neo3/testnet/0x8a9e1835270c95ddf5250ee84a1d4714552cb951) | [02317b71…ac](https://dora.coz.io/contract/neo3/testnet/0x02317b7192e3d91ba1739ae2a9f5fdcd44bf2dac) |
| **ERC-6147** GuardedNFT | [af32605e…64f](https://dora.coz.io/contract/neo3/testnet/0xaf32605e284ccf3e5e281af082f72605c506064f) | [274c031d…9df](https://dora.coz.io/contract/neo3/testnet/0x274c031d361e30a518d30035d527eb95efac19df) |
| **ERC-7201** NamespacedStorage | [bb2553c7…f32](https://dora.coz.io/contract/neo3/testnet/0xbb2553c79f3a740113bf22fbadb6828a9bdbdf32) | [0932ad78…05d](https://dora.coz.io/contract/neo3/testnet/0x0932ad78b3d71c7af06468604f1d00ef89c3205d) |
| **ERC-7818** Expirable | [fcaaf98f…64d](https://dora.coz.io/contract/neo3/testnet/0xfcaaf98f8c4693b326f883d52db9d9e4a8c6564d) | [cb1b0441…6e0](https://dora.coz.io/contract/neo3/testnet/0xcb1b0441c5b02a2f7de348951c6bf8e2a6ae56e0) |
| **ERC-173** Ownable | [19977aea…be4](https://dora.coz.io/contract/neo3/testnet/0x19977aea6f158de3844f3261988b17381156bbe4) | [ce89aec2…459](https://dora.coz.io/contract/neo3/testnet/0xce89aec2e79b121ec264231be49cd96111824459) |
| **ERC-1271** MultiSig | [88eec008…de7](https://dora.coz.io/contract/neo3/testnet/0x88eec008aaeb09d10ce68f93f6d98efbe92b9de7) | [88079ecd…682](https://dora.coz.io/contract/neo3/testnet/0x88079ecdd4af98cf932c25c80c0bb218a8cfb682) |
| **ERC-1820** Registry | [02704624…ee4](https://dora.coz.io/contract/neo3/testnet/0x02704624615747bdcc7994a6be347be42ad52ee4) | [8f36ff27…59d](https://dora.coz.io/contract/neo3/testnet/0x8f36ff27ef6564209956c05a4b886c0c99cec59d) |
| **ERC-1056** DID | [dd6d4a48…f50](https://dora.coz.io/contract/neo3/testnet/0xdd6d4a4806445d04982afc68866c9dc92ef41f50) | [d13806f6…b30](https://dora.coz.io/contract/neo3/testnet/0xd13806f6c06854ad3d8b731aebee40f8b74c1b30) |
| **ERC-1967** Upgradeable | [48f6d58a…245](https://dora.coz.io/contract/neo3/testnet/0x48f6d58aa74ad1d507cb2eb07242e033bfdbd245) | [096f01e4…976](https://dora.coz.io/contract/neo3/testnet/0x096f01e40f7cf9cea4304195cc2ab6bb481be976) |
| **ERC-2535** Diamond | [26b6f333…527](https://dora.coz.io/contract/neo3/testnet/0x26b6f333b18bffd00702348b1cec5b55cf79f527) | [1b3c602c…dcf](https://dora.coz.io/contract/neo3/testnet/0x1b3c602c1a208238f981125e2ad3045734c5bdcf) |
| **ERC-2612** PermitToken | [edd521fd…aa0](https://dora.coz.io/contract/neo3/testnet/0xedd521fdaa7422b7465673fa5df6551590c16aa0) | [b451279f…70c](https://dora.coz.io/contract/neo3/testnet/0xb451279fd8ab0e735e50edd6c6ca7e60eb90b70c) |
| **ERC-2771** Forwarder | [6653a8da…ed9](https://dora.coz.io/contract/neo3/testnet/0x6653a8da9bac7b622987670d97bf740c2c124ed9) | [1463ad54…0a](https://dora.coz.io/contract/neo3/testnet/0x1463ad54cf6a8fc7c0ffe3740ad1cf04a6280c0a) |
| **ERC-3156** FlashLender | [b7d5cd14…7ec](https://dora.coz.io/contract/neo3/testnet/0xb7d5cd146852006f8bc5d8c1621852c9117d37ec) | [a82c8142…84b](https://dora.coz.io/contract/neo3/testnet/0xa82c8142c02ec0cf748bbaa57819f9c61440984b) |
| **ERC-4337** SmartAccount | [aa11f9ef…a56](https://dora.coz.io/contract/neo3/testnet/0xaa11f9ef784be1ccad7b0f7b23c32508c5705a56) | [e26da230…09f](https://dora.coz.io/contract/neo3/testnet/0xe26da2300e0073c98f2292becda67170cbc6209f) |
| **ERC-4494** PermitNFT | [e683fa29…b66](https://dora.coz.io/contract/neo3/testnet/0xe683fa29d01521d99abfe15cb7e4b1f69ed47b66) | [c7056410…257](https://dora.coz.io/contract/neo3/testnet/0xc705641088ed11e564c946bf8a9a1569a9cc9257) |
| **ERC-4626** Vault | [faf678fd…0f](https://dora.coz.io/contract/neo3/testnet/0xfaf678fdb2053a279cf79f14a3623f3f1f9f810f) | [0e515ad2…0c5](https://dora.coz.io/contract/neo3/testnet/0x0e515ad2e892180273ab017a4883084e647740c5) |
| **ERC-5805** Voting | [b87fa58c…110](https://dora.coz.io/contract/neo3/testnet/0xb87fa58c80deef8dc910a0ca3a2cc186035f3110) | [1d33818b…692](https://dora.coz.io/contract/neo3/testnet/0x1d33818b3d053d291424848ed1ac7ebaa3243692) |
| **ERC-6372** Clock | [e3c55758…42b](https://dora.coz.io/contract/neo3/testnet/0xe3c55758861ba8034c9f3d223ed93cf5df77442b) | [eb454a6b…335](https://dora.coz.io/contract/neo3/testnet/0xeb454a6b6e102b2700fc1d3b18d58b861ed6c335) |
| **ERC-6492** PreDeploySig | [95170df7…9d9](https://dora.coz.io/contract/neo3/testnet/0x95170df70a3c425fdaad9795f77a68430ac659d9) | [7dd2cd08…fc9](https://dora.coz.io/contract/neo3/testnet/0x7dd2cd08072ab383cf7152bf5ed734d7d240bfc9) |
| **ERC-7540** AsyncVault | [d8838ba1…b41](https://dora.coz.io/contract/neo3/testnet/0xd8838ba126e5a77727c673215782aec59f465b41) | [c2137b33…26e](https://dora.coz.io/contract/neo3/testnet/0xc2137b33423fdda8ce5a240e7feec8db2c4b626e) |
| **ERC-7575** MultiAssetVault | [985a293c…6b1](https://dora.coz.io/contract/neo3/testnet/0x985a293c07f1b024f37a4ffc1d7fdf14edfe46b1) | [d85564c8…ffc](https://dora.coz.io/contract/neo3/testnet/0xd85564c8072776e0c5252797f5ba9dfb7e401ffc) |
| **ERC-7579** ModularAccount | [5e6edfc0…528](https://dora.coz.io/contract/neo3/testnet/0x5e6edfc08e536f6d8891af968a52f7d56c11a528) | [cbd2e64f…f86](https://dora.coz.io/contract/neo3/testnet/0xcbd2e64f3ef5d5c9069fadf9c7d72ffcb8664f86) |
| **EIP-191** PersonalSign | [64d3b4d2…eb1](https://dora.coz.io/contract/neo3/testnet/0x64d3b4d2e0ce6b26cf0dedad9a5c2d0bf96ddeb1) | [d06071f8…14e](https://dora.coz.io/contract/neo3/testnet/0xd06071f84b917cde1d16c23110f501b9dc3e914e) |
| **EIP-712** TypedData | [8e501310…fdd](https://dora.coz.io/contract/neo3/testnet/0x8e501310318f17d20674c639fc49b5e6100f5fdd) | [38e8f069…c9a](https://dora.coz.io/contract/neo3/testnet/0x38e8f069271cbbbdfd2032629e733a528cd57c9a) |
| **EIP-1153** TransientGuard | [7e4e4812…a66](https://dora.coz.io/contract/neo3/testnet/0x7e4e48124ed93c56eb4715965bb5b91fd0eb1a66) | [e67e6815…7bf](https://dora.coz.io/contract/neo3/testnet/0xe67e6815ad4d151bf87667af4e9aa9cbc3eaa7bf) |
| **EIP-2098** CompactSig | [d63ea34d…1d9](https://dora.coz.io/contract/neo3/testnet/0xd63ea34d63f0628c4cd413f58aa1c2623b1121d9) | [b2701b6d…eee](https://dora.coz.io/contract/neo3/testnet/0xb2701b6d89a734b5a865a1ec6c247466391c4eee) |
| **EIP-2718** TypedTx | [4e2300b8…bf6](https://dora.coz.io/contract/neo3/testnet/0x4e2300b8426b26eb8bbf57398a53ad810e313bf6) | [b600afb3…cb8](https://dora.coz.io/contract/neo3/testnet/0xb600afb3c034ff11a3e25a26fa03b58e263d9cb8) |
| **EIP-2930** AccessList | [02a174b4…2c3](https://dora.coz.io/contract/neo3/testnet/0x02a174b4080c57d8ceb2c66b223799a79d09a2c3) | [3570e80f…6ad](https://dora.coz.io/contract/neo3/testnet/0x3570e80f52a9329b604d5d18fb94de3133b0a6ad) |
| **EIP-3198** FeeAware | [8ed358ae…0a0](https://dora.coz.io/contract/neo3/testnet/0x8ed358aea7789a0d2c60a42c692c469bf1da60a0) | [32b98cb2…630](https://dora.coz.io/contract/neo3/testnet/0x32b98cb268f39b8ced382e7fe6d160833ab4f630) |
| **EIP-3855** Push0 | [52b86044…e4d](https://dora.coz.io/contract/neo3/testnet/0x52b860448a5d1ff537160a0bee8c83cdfe72fe4d) | [6704d604…9f6](https://dora.coz.io/contract/neo3/testnet/0x6704d604997959ce4e098bd96ecbacef358e9ff6) |
| **EIP-3860** InitcodeSize | [5d95f5db…728](https://dora.coz.io/contract/neo3/testnet/0x5d95f5db9f06ee751778208c36c408a49968d728) | [b58a104c…337](https://dora.coz.io/contract/neo3/testnet/0xb58a104cecaaa87e7c44915464f09b60e4768337) |
| **EIP-6780** SelfDestruct | [67a59c17…930](https://dora.coz.io/contract/neo3/testnet/0x67a59c179448a3769d5559a691a4c118c7de9930) | [8acf52e9…f8eb](https://dora.coz.io/contract/neo3/testnet/0x8acf52e9a1f696965480cd40046c9c3de020f8eb) |
| **EIP-7702** SetCode | [b156b370…751](https://dora.coz.io/contract/neo3/testnet/0xb156b370b4ad897caf955e38fffa637195438751) | [158d0d7f…bfd](https://dora.coz.io/contract/neo3/testnet/0x158d0d7fdf17d71b6e733997cd74b68f0d1c3bfd) |

**Checked-in TestNet snapshot: 147 / 183 assertions pass** across 47 deployed
pairs. The full pass/fail matrix is in
[`results.json`](https://github.com/r3e-network/neo-devpack-solidity/blob/main/docs/standards-mirror/deployments/results.json)
and
[`RESULTS.md`](https://github.com/r3e-network/neo-devpack-solidity/blob/main/docs/standards-mirror/deployments/RESULTS.md).
The deploy runner now exits non-zero when any compile, deploy, liveness, or
assertion check fails.

### What about the other catalog entries?

The checked-in catalog includes deployable demos for most entries. A small set remains outside the live TestNet pair matrix because the Ethereum mechanism is either protocol-specific, superseded by another EIP, or not yet part of the deployment snapshot.

Eighty-two catalog entries are not in the live TestNet pair matrix:

- **EIP-1559** (fee-market base-fee auction) — Neo doesn't auction fees.
- **EIP-4844** (blob transactions) — Neo doesn't have blobs.
- **EIP-3074** (AUTH/AUTHCALL) — superseded by EIP-7702 and covered by Neo witness scopes.
- **ERC-6909** (minimal multi-token) — documented as a direct Neo C# port but not part of the checked-in deployment snapshot.
- **ERC-1363** (payable token) — built into NEP-17's `onNEP17Payment` callback; no standalone deploy needed.
- **ERC-3009** (transfer with authorization) — subsumed by Neo's witness scopes; no separate contract required.
- **ERC-6551** (token bound accounts) — registry-pattern catalog entry; deploy targets a per-collection setup rather than a single fixture.
- **ERC-1167** (minimal proxy clones) — registry-pattern catalog entry; cloning is a deploy-time operation rather than a single fixture.
- **ERC-4361** (Sign-In with Ethereum) — primarily an off-chain server convention; on-chain verifier is application-specific.
- **ERC-3668** (CCIP Read) — subsumed by Neo's native Oracle service; no on-chain mirror contract needed.
- **ERC-7528** (native asset address) — Neo native assets (NEO, GAS) already have well-known contract hashes; nothing to deploy.
- **ERC-7656** (generalised contract-linked services) — registry-pattern catalog entry, like ERC-6551.
- **ERC-3448** (MetaProxy Standard) — registry-pattern catalog entry; same deploy-time shape as ERC-1167.
- **ERC-7535** (native asset vault) — collapses to ERC-4626 with `asset = NEO/GAS`; covered by the existing ERC-4626 deployment.
- **ERC-4907** (rental NFT) — NEP-11 storage extension; no separate fixture needed beyond a port-side example.
- **ERC-6093** (custom token errors) — vocabulary convention; no contract to deploy.
- **ERC-7786** (cross-chain messaging gateway) — bridge-adapter pattern; per-bridge deploy depends on the chosen transport.
- **ERC-5313** (light contract ownership) — view-only convention; satisfied by every existing contract that exposes `getOwner`.
- **ERC-5564** (stealth addresses) — announcer contract is small; deploy depends on a wallet-SDK rollout to be useful.
- **ERC-6066** (NFT signature validation) — NEP-11 storage extension scoped per tokenId; no separate fixture beyond an example NFT.
- **ERC-3643** (T-REX regulated token) — multi-contract framework (token + identity registry + compliance modules); per-deployment shape rather than a single fixture.
- **ERC-5202** (blueprint contract format) — Neo's deploy primitive already separates blob storage from execution; no magic-prefix container needed.
- **ERC-7944** (async cancellation for ERC-7540) — extension of ERC-7540's request queue; covered by the existing ERC-7540 deployment plus the additional cancel flow.
- **ERC-8042** (diamond storage) — storage convention for ERC-2535; covered by the existing ERC-2535 deployment plus storage-prefix discipline.
- **ERC-5679** (token mint/burn) — selector convention; satisfied by every NEP-17 / NEP-11 contract that exposes a `Mint` wrapper.
- **ERC-2135** (consumable interface) — NEP-11 storage extension; no separate fixture needed.
- **ERC-7160** (multi-metadata NFT) — NEP-11 storage extension; no separate fixture needed.
- **ERC-3475** (abstract storage bonds) — port-side primitive; deploy is per-issuance configuration.
- **ERC-7092** (financial bonds) — port-side primitive; deploy is per-bond configuration.
- **ERC-6982** (default lockable tokens) — NEP-11 storage extension; covered by per-collection deploy.
- **ERC-7677** (paymaster web service) — off-chain JSON-RPC convention; on-chain footprint is optional escrow contract.
- **ERC-7144** (ERC-20 with transaction validation) — NEP-17 transfer override + per-policy validator; per-deployment shape.
- **ERC-7943** (Universal RWA Interface) — capability-flag + compliance umbrella; per-deployment shape rather than a single fixture.
- **ERC-5006** (Rental NFT user extension for ERC-1155) — NEP-11 divisible + record storage; per-collection shape.
- **ERC-2767** (contract ownership governance) — interface convention; satisfied whenever a governance contract owns the governed contract.
- **ERC-7758** (modern transfer-with-authorization) — same Neo answer as ERC-3009; covered by witness scopes.
- **ERC-5169** (client script URI) — token-metadata extension; one method addition to any NEP-11 / NEP-17 contract.
- **ERC-5375** (NFT author and consent) — NEP-11 metadata extension; per-collection configuration shape.
- **ERC-5023** (shareable NFT) — NEP-11 multi-holder extension; per-collection shape.
- **ERC-7066** (lockable extension) — NEP-11 storage extension; covered by per-collection deploy.
- **ERC-7432** (NFT roles) — NEP-11 storage extension; per-collection role conventions.
- **ERC-6105** (no-intermediary trading) — NEP-11 listing/swap extension; per-collection shape.
- **ERC-5615** (ERC-1155 supply extension) — NEP-11 divisible storage extension; covered by per-collection deploy.
- **ERC-5773** (multi-asset tokens) — NEP-11 storage extension; per-collection shape.
- **ERC-6059** (nestable NFTs) — NEP-11 parent-child storage extension; per-collection shape.
- **ERC-4519** (physical-asset NFTs) — NEP-11 + device-pubkey extension; per-device deploy depending on physical batch.
- **ERC-5570** (digital receipt NFT) — NEP-11 metadata extension; per-merchant deploy shape.
- **ERC-6150** (hierarchical NFTs) — NEP-11 storage extension; per-collection shape.
- **ERC-6220** (composable equippable parts) — NEP-11 + catalog extension; per-collection shape.
- **ERC-5380** (entitlement extension) — NEP-11 storage extension; per-collection shape.
- **ERC-5489** (NFT hyperlink) — NEP-11 storage extension; per-collection shape.
- **ERC-6672** (multi-redeemable NFTs) — NEP-11 storage extension; per-collection / per-operator shape.
- **ERC-7634** (limited transfer count) — NEP-11 storage extension; per-collection shape.
- **ERC-7007** (AI-generated content) — NEP-11 attestation extension; per-collection / per-model shape.
- **ERC-5725** (vesting NFT) — NEP-11 + NEP-17 underlying-token mechanics; per-deployment configuration.
- **ERC-6454** (transferable detection) — NEP-11 view extension; satisfied by every contract that exposes IsTransferable.
- **ERC-7715** (AA permission grants) — Witness-scope-based authorisation; per-account permission module deploy.
- **ERC-5507** (refundable NFTs) — per-token escrow + refund window; per-collection deploy.
- **ERC-5528** (refundable ERC-20) — per-buyer escrow + refund window; per-deployment shape.
- **ERC-5521** (referable NFT) — citation-graph extension; per-collection shape.
- **ERC-5646** (state fingerprint) — view extension; satisfied by every contract that hashes its mutable state.
- **ERC-5585** (NFT authorization rights) — per-(tokenId, rights) storage extension; per-collection shape.
- **ERC-6239** (semantic SBT) — NEP-11 + RDF claims storage; per-collection shape.
- **ERC-7053** (digital media indexing) — event-emission convention; no per-fixture deploy needed.
- **ERC-7405** (modular wallet) — per-account module dispatch; per-wallet deploy.
- **ERC-7585** (cross-chain SIWE) — off-chain library convention; on-chain verifier is application-specific.
- **ERC-6381** (NFT emote repository) — standalone singleton; one global deploy serves all NEP-11s.
- **ERC-7857** (AI agent NFT) — NEP-11 + encrypted-metadata extension; per-collection / per-service deploy.
- **ERC-7231** (identity-aggregated NFT) — NEP-11 + Merkle-root extension; per-collection shape.
- **ERC-7531** (staked NFT recognition) — staking-contract view; per-staking-contract deploy.
- **ERC-5216** (ERC-1155 allowance) — NEP-11 divisible storage extension; per-collection shape.
- **ERC-5008** (NFT nonce) — NEP-11 storage extension; per-collection shape.
- **ERC-4910** (royalty-bearing NFTs) — NEP-11 + NEP-24 escrow extension; per-collection shape.
- **ERC-5732** (commit interface) — standalone commit-reveal contract; one global deploy serves any consumer.
- **ERC-4400** (NFT consumable, lite) — NEP-11 storage extension; per-collection shape.
- **ERC-5750** (method extensibility) — convention satisfied by every NEP-17 / NEP-11 (already four-parameter shape).
- **ERC-5269** (ERC detection) — manifest `supportedstandards` already provides this natively.
- **ERC-5453** (endorsement / generalised permit) — native witness scopes already provide per-call signature auth.
- **ERC-5496** (multi-privilege NFT) — NEP-11 storage extension; per-collection shape.
- **ERC-6357** (multi-delegatecall) — Neo's transaction script natively supports multi-invoke without contract changes.
- **ERC-1046** (fungible tokenURI) — NEP-17 view convention; per-deployment metadata pointer.
- **ERC-7746** (security middleware hooks) — pre/post hook chain pattern; per-deployment shape.

The other protocol entries in the table have live demos exposing their Neo counterparts, such as transaction version, witness scopes, PUSH0, NEF size, ContractManagement.Destroy, and NEP-30 verify.

Source pairs, deploy script, full results JSON, and instructions to reproduce live
under
[`docs/standards-mirror/deployments/`](https://github.com/r3e-network/neo-devpack-solidity/tree/main/docs/standards-mirror/deployments).

## How to Read This

Each entry shows three views in a tabbed nav bar:

| Tab | What it shows |
| --- | --- |
| **ERC / EIP Detail** | The standard's purpose, interface, motivation, and the corresponding Neo mechanism. |
| **Solidity Implementation** | A reference Solidity implementation as deployed on Ethereum. |
| **Neo C# Implementation** | The idiomatic Neo equivalent — either a direct NEP, a composition pattern, or an explanation of why the EIP is a no-op on Neo because the protocol already covers it. |

## How To Read The Pills

Each entry carries a parity pill that summarizes the mirror relationship:

| Pill | Meaning |
| --- | --- |
| **NEP-XX** (green) | Direct NEP equivalent. Same interface shape, sometimes simpler. |
| **Native** (blue) | Subsumed by Neo's protocol primitives. No NEP or contract needed. |
| **Pattern** (orange) | Composition pattern in the Neo devpack — no dedicated NEP, but a clear recipe. |

## Module Internals

Each category page registers its entries with a shared `<StandardsMirror>` Vue component
that drives the master/detail UI. All content is plain Markdown — VitePress's bundled
Shiki highlights every Solidity and C# block, and the search index covers everything.

To add another standard, append a `<StandardEntry>` block to the appropriate category
page. The page automatically picks it up.

For deeper reference on the Neo standards behind the mirrors, see
[Standards and Contracts](/additional-material/neo-standards) and
[`devpack/standards/STANDARDS_MAPPING.md`](https://github.com/r3e-network/neo-devpack-solidity/blob/main/devpack/standards/STANDARDS_MAPPING.md).
