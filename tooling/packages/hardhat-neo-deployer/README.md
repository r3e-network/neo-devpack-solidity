# @neo-devpack-solidity/hardhat-neo-deployer

Deploy Neo N3 contracts (NEF + manifest) from Hardhat.

This plugin is designed to work with build artifacts produced by `@neo-devpack-solidity/hardhat-solc-neo` (it reads the embedded `contract.neo.nef.image` + `contract.neo.manifest` fields).

## Installation

```bash
npm install --save-dev hardhat@^2.28.6 @neo-devpack-solidity/hardhat-solc-neo @neo-devpack-solidity/hardhat-neo-deployer
```

This plugin currently supports Hardhat 2.28.x. Hardhat 3 requires a separate plugin/runtime
migration before this package can support it.

Enable the plugins:

```ts
import "@neo-devpack-solidity/hardhat-solc-neo";
import "@neo-devpack-solidity/hardhat-neo-deployer";

export default {
  neoSolc: {
    solidity: {
      settings: {
        neo: {
          // optional compiler flags forwarded to neo-solc
          callt: true
        }
      }
    }
  },
  neoNetworks: {
    testnet: {
      rpcUrls: ["https://testnet1.neo.coz.io:443"],
      magic: 894710606,
      addressVersion: 0x35,
      nativeTokens: {
        gas: { name: "GasToken", symbol: "GAS", hash: "0xd2a4cff31913016155e38e474a2c06d08be276cf", decimals: 8 },
        neo: { name: "NeoToken", symbol: "NEO", hash: "0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5", decimals: 0 }
      },
      accounts: [
        // WIF or private key (hex)
        process.env.NEO_WIF!
      ]
    }
  }
};
```

## Usage

Compile (writes Neo build artifacts via `neo-solc --standard-json`):

```bash
npx hardhat neo-compile
```

Deploy:

```bash
npx hardhat neo-deploy --contract MyContract --network testnet
```

Constructor args are passed as a JSON array:

```bash
npx hardhat neo-deploy --contract MyContract --args '[\"Nep17Token\", \"N...sender...\"]' --network testnet
```

Interact (read + write):

```ts
const deployed = await hre.run("neo-deploy", { contract: "MyContract", network: "testnet" });
const contract = await hre.neoDeploy.deployer.getContract("MyContract", deployed.address);

// Read-only simulation (invokescript)
const result = await contract.methods.balanceOf.call("N...address...");

// On-chain invocation (sendrawtransaction)
const tx = await contract.methods.transfer.invoke("N...to...", "100000000", ""); // args depend on ABI
const receipt = await tx.wait();
```

## Notes

- The deploy transaction calls `ContractManagement.deploy(nef, manifest, data)` and passes constructor arguments via `_deploy(data, update)` as an array.
- RPC payload encoding differs between node implementations. This plugin auto-detects `neo-go` and sends base64 payloads where required (e.g., `sendrawtransaction`, `calculatenetworkfee`, `invokescript`).
- You need GAS to pay system + network fees; unfunded accounts will fail with RPC mempool validation errors.
