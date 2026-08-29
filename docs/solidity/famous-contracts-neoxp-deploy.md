# Famous EVM Contracts Deployed on Neo N3 (Neo Express)

- Generated at (UTC): `2026-02-15T09:14:27.438Z`
- Snapshot scope: historical Neo Express deployment output; rerun `npm run deploy:famous-contracts:neoxp` before treating these results as current release evidence.
- neoxp: `3.9.1.3+e36e7bad7e`
- Total cases: `13`
- Pass: `13`
- Fail: `0`

This matrix deploys upstream contracts (EVM ecosystem) onto Neo N3 via `neo-solc` + `neoxp`.
For selected ERC-style contracts, `supportedstandards` is cleared in manifest to bypass strict NEP schema checks while keeping source code unchanged.

| # | Project | Contract | Result | Probe | Contract Hash | Source | Note |
|---:|---|---|---|---|---|---|---|
| 1 | OpenZeppelin | ERC20 | ✅ pass | HALT | 0xe116d4f26c4e4af4ed3b6cbf9c9b1505973687cf | `third_party/famous-contracts/sources/@openzeppelin/contracts/token/ERC20/ERC20.sol` | Clears supportedstandards for EVM ERC-20 vs NEP-17 strict schema mismatch. |
| 2 | OpenZeppelin | ERC721 | ✅ pass | HALT | 0x931e486a418c1da855dab93f8790750b6a96a161 | `third_party/famous-contracts/sources/@openzeppelin/contracts/token/ERC721/ERC721.sol` | Clears supportedstandards for EVM ERC-721 vs NEP-11 strict schema mismatch. |
| 3 | OpenZeppelin | AccessControl | ✅ pass | HALT | 0x9792e50cb0ba36252e1e90a2dc9b506e4a26a807 | `third_party/famous-contracts/sources/@openzeppelin/contracts/access/AccessControl.sol` | Original upstream contract deployed as-is. |
| 4 | Aave V3 | WETH9 | ✅ pass | HALT | 0x70a37cd49461eeeceee53ba9609ef15bc589b473 | `third_party/famous-contracts/sources/@aave/core-v3/contracts/dependencies/weth/WETH9.sol` | Clears supportedstandards to bypass strict NEP method-shape checks. |
| 5 | Aave V3 | PoolConfigurator | ✅ pass | SKIPPED | 0x4d05f64ee940f5d02bb412d0400e9d16c728ec96 | `third_party/famous-contracts/sources/@aave/core-v3/contracts/protocol/pool/PoolConfigurator.sol` | Original upstream contract deployed; runtime integration calls require further protocol wiring. |
| 6 | Safe | Safe | ✅ pass | HALT | 0xf04634c55fbc3fd5421b3c25c971f57ccc431a37 | `third_party/famous-contracts/sources/@safe-global/safe-contracts/contracts/Safe.sol` | Original upstream multisig core contract deployed. |
| 7 | Safe | SafeL2 | ✅ pass | HALT | 0x40e6300e17d0f393ea84ff59313b15a0c6f1ba9d | `third_party/famous-contracts/sources/@safe-global/safe-contracts/contracts/SafeL2.sol` | Original upstream L2-flavored Safe contract deployed. |
| 8 | Safe | SafeProxyFactory | ✅ pass | HALT | 0x1eb9b3930d49389f6669454fa042b262cb7fbc96 | `third_party/famous-contracts/sources/@safe-global/safe-contracts/contracts/proxies/SafeProxyFactory.sol` | Original upstream proxy factory deployed. |
| 9 | Safe | MultiSend | ✅ pass | SKIPPED | 0x6c7cd6f7e8128df4b11bb5ed7834ab401e5f7a35 | `third_party/famous-contracts/sources/@safe-global/safe-contracts/contracts/libraries/MultiSend.sol` | Original upstream library contract deployed (no zero-arg probe method). |
| 10 | Uniswap V2 Core | UniswapV2Pair | ✅ pass | HALT | 0x9bf49d45d85d3ba71a67a201e01c8e07ca185523 | `third_party/famous-contracts/sources/@uniswap/v2-core/contracts/UniswapV2Pair.sol` | Clears supportedstandards due ERC-style strict schema mismatch. |
| 11 | Uniswap V2 Core | UniswapV2ERC20 | ✅ pass | HALT | 0x0bd1fc83a16b718bf0ed559c668e4db6c231e565 | `third_party/famous-contracts/sources/@uniswap/v2-core/contracts/UniswapV2ERC20.sol` | Clears supportedstandards due ERC-style strict schema mismatch. |
| 12 | Chainlink | MockV3Aggregator | ✅ pass | HALT | 0xab83be98437967a5605af09bae8286fb6373b387 | `third_party/famous-contracts/sources/@chainlink/contracts/src/v0.8/shared/mocks/MockV3Aggregator.sol` | Original upstream oracle mock deployed. |
| 13 | Chainlink | OwnerIsCreator | ✅ pass | HALT | 0x516a69c1618dfa5c4b86e5c6c2d2446284a498f8 | `third_party/famous-contracts/sources/@chainlink/contracts/src/v0.8/shared/access/OwnerIsCreator.sol` | Original upstream access-control primitive deployed. |

## Notes

- Deployment target in this report is local Neo N3 (`neoxp`) for deterministic CI-style validation.
- Runtime/business integration tests for protocol-level flows (AMM swaps, Safe execution pipelines, Aave pool wiring) still require additional fixture contracts and state setup.
