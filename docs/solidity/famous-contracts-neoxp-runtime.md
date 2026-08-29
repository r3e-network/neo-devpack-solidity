# Famous EVM Contracts Runtime-Verified on Neo N3 (Type-3)

- Generated at (UTC): `2026-02-15T09:34:32.300Z`
- Snapshot scope: historical Neo Express runtime output; rerun `npm run verify:famous-contracts:neoxp-runtime` before treating these results as current release evidence.
- neoxp: `3.9.1.3+e36e7bad7e`
- Total contracts: `6`
- Pass: `6`
- Fail: `0`
- Assertions passed: `13/13`

Type-3 criteria in this report:
- Deploy transaction reaches `HALT`.
- At least one state-changing invocation reaches `HALT`.
- Post-state readback equals expected value.

| # | Project | Contract | Result | Assertions | Contract Hash | Source | Note |
|---:|---|---|---|---:|---|---|---|
| 1 | OpenZeppelin | ERC20 | ✅ pass | 2/2 | 0x85ab1b3d2099b82651bb53209feeec11b7ccafc8 | `third_party/famous-contracts/sources/@openzeppelin/contracts/token/ERC20/ERC20.sol` | Original upstream ERC20; clears supportedstandards due ERC-20 vs NEP-17 shape mismatch. |
| 2 | OpenZeppelin | ERC721 | ✅ pass | 2/2 | 0x119452bd4506acd5c3968311e7920127662d8aac | `third_party/famous-contracts/sources/@openzeppelin/contracts/token/ERC721/ERC721.sol` | Original upstream ERC721; clears supportedstandards due ERC-721 vs NEP-11 shape mismatch. |
| 3 | Aave V3 | WETH9 | ✅ pass | 2/2 | 0x3b52f88af02a715e1dd29030806b31fe531a79ad | `third_party/famous-contracts/sources/@aave/core-v3/contracts/dependencies/weth/WETH9.sol` | Original upstream WETH9; clears supportedstandards due ERC-style schema mismatch. |
| 4 | Uniswap V2 Core | UniswapV2Pair | ✅ pass | 2/2 | 0x32cc8797d0eb49ce7447daa6f9ee34847b59eb26 | `third_party/famous-contracts/sources/@uniswap/v2-core/contracts/UniswapV2Pair.sol` | Original upstream pair contract; approve/allowance path runtime-verified. |
| 5 | Uniswap V2 Core | UniswapV2ERC20 | ✅ pass | 2/2 | 0x34d817e33b081a437cca01478c20e4078e04ed35 | `third_party/famous-contracts/sources/@uniswap/v2-core/contracts/UniswapV2ERC20.sol` | Original upstream UniswapV2ERC20; approve/allowance runtime-verified. |
| 6 | Chainlink | MockV3Aggregator | ✅ pass | 3/3 | 0xeb44cf01c2b3caf3f961edce38e1a67378efd076 | `third_party/famous-contracts/sources/@chainlink/contracts/src/v0.8/shared/mocks/MockV3Aggregator.sol` | Original upstream Chainlink mock oracle with end-to-end update/read verification. |

## Assertion Details

### OpenZeppelin / ERC20

- Status: `pass`
- Contract hash: `0x85ab1b3d2099b82651bb53209feeec11b7ccafc8`
- Deploy tx: `0xcf8addae9b85996b18b2dc8b5c36c38f79011b80dc9b682cc9b8e980c4c5a5e5`
- Assertions:
  - ✅ write `approve` tx=0xf3cf98bda239d7e69b46f3be1fdb8c3c3b9c4871879f695a4b3b971eca5f2b8f actual={"vmstate":"HALT"}
  - ✅ read `allowance` expected={"type":"Integer","value":"123"} actual={"type":"Integer","value":"123"}

### OpenZeppelin / ERC721

- Status: `pass`
- Contract hash: `0x119452bd4506acd5c3968311e7920127662d8aac`
- Deploy tx: `0xa19cafe023d5a02ad5859e02d0b51665a5bbd0097a0ec815585ec41b1cb6429f`
- Assertions:
  - ✅ write `setApprovalForAll` tx=0xa2b8f0587b6eac5b2decd67d0770fecb90e9eae3d0f3f14f98ac82867737e229 actual={"vmstate":"HALT"}
  - ✅ read `isApprovedForAll` expected={"type":"Boolean","value":true} actual={"type":"Boolean","value":true}

### Aave V3 / WETH9

- Status: `pass`
- Contract hash: `0x3b52f88af02a715e1dd29030806b31fe531a79ad`
- Deploy tx: `0xdb6011bbea604ba29fb5be2be78662c5b2983d2ba97a4d11c73563106562a6d8`
- Assertions:
  - ✅ write `approve` tx=0xf2beb28ed65c83c241caef4ec47b41bdde37c99f07c1b4b3a021eddf415ba2df actual={"vmstate":"HALT"}
  - ✅ read `allowance` expected={"type":"Integer","value":"77"} actual={"type":"Integer","value":"77"}

### Uniswap V2 Core / UniswapV2Pair

- Status: `pass`
- Contract hash: `0x32cc8797d0eb49ce7447daa6f9ee34847b59eb26`
- Deploy tx: `0x67143c9b06897c8f73c557f65c0df79750295e1940bce34b02fa9cede2f8fba7`
- Assertions:
  - ✅ write `approve` tx=0x442eee3c3826a5a86f34580ad96d342171545d22e48111cb830c32d233e2a4b2 actual={"vmstate":"HALT"}
  - ✅ read `allowance` expected={"type":"Integer","value":"321"} actual={"type":"Integer","value":"321"}

### Uniswap V2 Core / UniswapV2ERC20

- Status: `pass`
- Contract hash: `0x34d817e33b081a437cca01478c20e4078e04ed35`
- Deploy tx: `0x4ae5e4cea66c0e90c3f13b2b3a2e54d4b5af3bdec60a9245902306dc26e615b1`
- Assertions:
  - ✅ write `approve` tx=0x5795f508e82155f48df3256d8c4b941577ae36e9cb31a269e7b53fa69e361db6 actual={"vmstate":"HALT"}
  - ✅ read `allowance` expected={"type":"Integer","value":"456"} actual={"type":"Integer","value":"456"}

### Chainlink / MockV3Aggregator

- Status: `pass`
- Contract hash: `0xeb44cf01c2b3caf3f961edce38e1a67378efd076`
- Deploy tx: `0x9318aa8cf7260554017d4b2536593f90afb721ff097d31ac17808fe0bc421307`
- Assertions:
  - ✅ read `latestAnswer` expected={"type":"Integer","value":"123456789"} actual={"type":"Integer","value":"123456789"}
  - ✅ write `updateAnswer` tx=0xd6ca169f594c5bc8cc102d2f9d2d84ae08e6a43f889237ff1cfe8e3bd27cd584 actual={"vmstate":"HALT"}
  - ✅ read `latestAnswer` expected={"type":"Integer","value":"987654321"} actual={"type":"Integer","value":"987654321"}

## Notes

- Source contracts are upstream originals from the referenced package paths.
- For ERC-style contracts, `supportedstandards` is cleared in manifest to avoid strict NEP schema shape rejection while preserving Solidity source logic.
- This report intentionally excludes deploy-only/probe-only cases and keeps only runtime-verified positive flows.
