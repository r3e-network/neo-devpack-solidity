# Famous EVM Contracts Runtime-Verified on Neo N3 TestNet

- Generated at (UTC): `2026-02-15T11:02:38.153Z`
- Snapshot scope: historical TestNet runtime output; rerun the TestNet runtime script before treating these results as current release evidence.
- RPC endpoint: `http://seed1t5.neo.org:20332`
- Network magic: `894710606`
- Deployer address: `NTmHjwiadq4g3VHpJ5FQigQcD4fF5m8TyX`
- Total contracts: `6`
- Pass: `6`
- Fail: `0`
- Assertions passed: `12/12`
- GAS before: `3308892238902`
- GAS after: `3308899474196`

| # | Project | Contract | Result | Assertions | Contract Hash | Deploy Tx | Source |
|---:|---|---|---|---:|---|---|---|
| 1 | OpenZeppelin | ERC20 | ✅ pass | 2/2 | 0xf38b0b0ded81ed23a1675c251a697cd04286fe45 | ALREADY_DEPLOYED | `third_party/famous-contracts/sources/@openzeppelin/contracts/token/ERC20/ERC20.sol` |
| 2 | OpenZeppelin | ERC721 | ✅ pass | 2/2 | 0x8381348d65e3edfa30be05b111525bb3026eb7bf | ALREADY_DEPLOYED | `third_party/famous-contracts/sources/@openzeppelin/contracts/token/ERC721/ERC721.sol` |
| 3 | Aave V3 | WETH9 | ✅ pass | 2/2 | 0xd0d4d626c833e820c411e5fa60ff5c3101929788 | ALREADY_DEPLOYED | `third_party/famous-contracts/sources/@aave/core-v3/contracts/dependencies/weth/WETH9.sol` |
| 4 | Uniswap V2 Core | UniswapV2Pair | ✅ pass | 2/2 | 0x36eb6ef88ecf41e687007f840e558b1aecd81e95 | ALREADY_DEPLOYED | `third_party/famous-contracts/sources/@uniswap/v2-core/contracts/UniswapV2Pair.sol` |
| 5 | Uniswap V2 Core | UniswapV2ERC20 | ✅ pass | 2/2 | 0x19fcd30e4473d167203d6f054b53faec302d6079 | ALREADY_DEPLOYED | `third_party/famous-contracts/sources/@uniswap/v2-core/contracts/UniswapV2ERC20.sol` |
| 6 | Chainlink | MockV3Aggregator | ✅ pass | 2/2 | 0x7c198cd20c57da4bdc7a5fcf6a00682a977715a2 | ALREADY_DEPLOYED | `third_party/famous-contracts/sources/@chainlink/contracts/src/v0.8/shared/mocks/MockV3Aggregator.sol` |

## Assertion Details

### OpenZeppelin / ERC20

- Status: `pass`
- Contract hash: `0xf38b0b0ded81ed23a1675c251a697cd04286fe45`
- Deploy tx: `ALREADY_DEPLOYED`
- Assertions:
  - ✅ write `approve` tx=0x208d87d55951fbf1150a776a585d592dc028d3907f4b821a92f1e43d84f904fb actual={"vmstate":"HALT"}
  - ✅ read `allowance` expected={"type":"Integer","value":"123"} actual={"type":"Integer","value":"123"}

### OpenZeppelin / ERC721

- Status: `pass`
- Contract hash: `0x8381348d65e3edfa30be05b111525bb3026eb7bf`
- Deploy tx: `ALREADY_DEPLOYED`
- Assertions:
  - ✅ write `setApprovalForAll` tx=0xbd38024203535984b56c283894802e7443674de12239e20d1c17d23b486a0ed9 actual={"vmstate":"HALT"}
  - ✅ read `isApprovedForAll` expected={"type":"Boolean","value":true} actual={"type":"Boolean","value":true}

### Aave V3 / WETH9

- Status: `pass`
- Contract hash: `0xd0d4d626c833e820c411e5fa60ff5c3101929788`
- Deploy tx: `ALREADY_DEPLOYED`
- Assertions:
  - ✅ write `approve` tx=0xc585d307f05e996e70e99f891f910e85a90ca5abdbbe404ffb44eb4bfdb330d3 actual={"vmstate":"HALT"}
  - ✅ read `allowance` expected={"type":"Integer","value":"77"} actual={"type":"Integer","value":"77"}

### Uniswap V2 Core / UniswapV2Pair

- Status: `pass`
- Contract hash: `0x36eb6ef88ecf41e687007f840e558b1aecd81e95`
- Deploy tx: `ALREADY_DEPLOYED`
- Assertions:
  - ✅ write `approve` tx=0x548627f5862e80af059ac26ed56e70eabc6113ef1b83676f4b7112ec537c1934 actual={"vmstate":"HALT"}
  - ✅ read `allowance` expected={"type":"Integer","value":"321"} actual={"type":"Integer","value":"321"}

### Uniswap V2 Core / UniswapV2ERC20

- Status: `pass`
- Contract hash: `0x19fcd30e4473d167203d6f054b53faec302d6079`
- Deploy tx: `ALREADY_DEPLOYED`
- Assertions:
  - ✅ write `approve` tx=0x7369017268e331ad452a550aa029dfef3827611f6359b6b2267af1fcf00d1e53 actual={"vmstate":"HALT"}
  - ✅ read `allowance` expected={"type":"Integer","value":"456"} actual={"type":"Integer","value":"456"}

### Chainlink / MockV3Aggregator

- Status: `pass`
- Contract hash: `0x7c198cd20c57da4bdc7a5fcf6a00682a977715a2`
- Deploy tx: `ALREADY_DEPLOYED`
- Assertions:
  - ✅ write `updateAnswer` tx=0xe0da3f758e6f0d59d078ba6133564167f1f6f1c55273bbb6d4fcbd5270183ac1 actual={"vmstate":"HALT"}
  - ✅ read `latestAnswer` expected={"type":"Integer","value":"1771153313341"} actual={"type":"Integer","value":"1771153313341"}
