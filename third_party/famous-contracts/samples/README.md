# Famous-contract compile samples

Self-contained, real-world Solidity contracts from well-known Ethereum
projects, vendored **solely as compile-conformance samples** for the
Solidity→NeoVM compiler (`neo-solc`). Each file compiles **hermetically**
(no include paths / no external dependencies — any small imports were
inlined) and is exercised by `tests/famous_samples_compile.rs`, which
asserts every sample still compiles.

These are third-party works included under their own licenses (see the
`SPDX-License-Identifier` header preserved in each file); they are not part
of the neo-devpack-solidity licensed sources. They span Solidity
`0.5.x`–`0.8.x` and cover DeFi, NFT, GameFi, zero-knowledge, and
infrastructure/DAO patterns, plus `patterns/` — minimal reproductions of
real-contract shapes that pinned specific `neo-solc` fixes.

**33 samples** across 6 categories.

## DeFi — AMMs, lending, stablecoins, wrapped-native, permit tokens

| Contract | License | Pragma | Lines |
| --- | --- | --- | --- |
| `ConstantProductAMM.sol` | MIT | `=0.5.16` | 213 |
| `Dai.sol` | AGPL-3.0-or-later | `^0.6.12` | 150 |
| `ERC20Permit.sol` | AGPL-3.0-only | `>=0.8.0` | 230 |
| `UniswapV2ERC20.sol` | GPL-3.0-or-later | `=0.5.16` | 136 |
| `WETH9.sol` | GPL-3.0 | `>=0.4.22 <0.6` | 84 |
| `WhitePaperInterestRateModel.sol` | BSD-3-Clause | `^0.8.10` | 101 |

## GameFi — staking, farms, game items, vesting

| Contract | License | Pragma | Lines |
| --- | --- | --- | --- |
| `GameItem.sol` | MIT | `^0.8.20` | 453 |
| `MasterChef.sol` | MIT | `0.6.12` | 874 |
| `StakingRewards.sol` | MIT | `^0.8.0` | 455 |
| `TokenVesting.sol` | MIT | `^0.5.0` | 390 |

## Infrastructure & DAO — multicall, proxies, multisig, timelock, governance, access control

| Contract | License | Pragma | Lines |
| --- | --- | --- | --- |
| `AccessControl.sol` | MIT | `^0.8.0` | 161 |
| `Create2Factory.sol` | MIT | `^0.8.0` | 46 |
| `ERC1967Slots.sol` | MIT | `^0.8.0` | 71 |
| `MultiSigWallet.sol` | LGPL-3.0-only | `^0.8.0` | 127 |
| `Multicall3.sol` | MIT | `0.8.12` | 217 |
| `Ownable.sol` | MIT | `^0.8.0` | 103 |
| `ReentrancyGuard.sol` | MIT | `^0.8.0` | 77 |
| `SimpleGovernor.sol` | MIT | `^0.8.0` | 163 |
| `TimelockController.sol` | MIT | `^0.8.0` | 124 |

## NFT — ERC-721/1155 collections & marketplaces

| Contract | License | Pragma | Lines |
| --- | --- | --- | --- |
| `BoredApeYachtClub.sol` | MIT | `^0.8.0` | 317 |
| `ERC721A.sol` | MIT | `^0.8.4` | 988 |
| `OnChainNFT.sol` | MIT | `^0.8.0` | 299 |
| `OpenZeppelinERC721.sol` | MIT | `^0.8.0` | 906 |
| `SolmateERC1155.sol` | AGPL-3.0-only | `>=0.8.0` | 302 |
| `SolmateERC721.sol` | AGPL-3.0-only | `>=0.8.0` | 286 |

## Patterns — minimal real-contract shapes that pinned neo-solc fixes

| Contract | License | Pragma | Lines |
| --- | --- | --- | --- |
| `AaveLibraryEvents.sol` | MIT | `^0.8.10` | 28 |
| `CompoundAbstractOracle.sol` | MIT | `^0.8.20` | 32 |
| `FiatTokenInheritedEvents.sol` | MIT | `^0.8.20` | 35 |

## Zero-knowledge — SNARK/PLONK verifiers, Merkle trees, hashers

| Contract | License | Pragma | Lines |
| --- | --- | --- | --- |
| `ETHTornado.sol` | MIT | `^0.7.0` | 355 |
| `MerkleTreeWithHistory.sol` | MIT | `^0.7.0` | 152 |
| `PlonkVerifier.sol` | GPL-3.0 | `>=0.7.0 <0.9.0` | 714 |
| `PoseidonT3.sol` | MIT | `>=0.7.0` | 392 |
| `SemaphoreVerifier.sol` | MIT | `>=0.8.23 <0.9.0` | 669 |

