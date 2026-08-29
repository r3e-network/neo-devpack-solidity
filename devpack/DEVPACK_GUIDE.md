# Neo N3 Devpack for Solidity - Complete Guide

**A comprehensive development framework for building Neo N3 smart contracts in Solidity**

## 🎯 Overview

The Neo N3 Devpack provides Solidity-facing interfaces for Neo N3 features supported by the
`neo-devpack-solidity` compiler (syscalls, native contract calls, and NEP standard examples).

> Important: `neo-devpack-solidity` treats `Runtime`, `Storage`, `Syscalls`, `Neo`, `NativeCalls` and `abi`
> as **compiler intrinsics** (built-in libraries). Calls to supported members are lowered directly
> to Neo N3 syscalls / native contract calls. The Solidity bodies of these libraries are not
> compiled, and unsupported members will fail compilation with an error that lists the supported
> intrinsics.

## 🚀 Quick Start

### Installation

```bash
# Install via npm
npm install --save-dev @neo-devpack-solidity/contracts hardhat@^2.28.6 @neo-devpack-solidity/hardhat-solc-neo @neo-devpack-solidity/hardhat-neo-deployer

# Or clone and build
git clone https://github.com/r3e-network/neo-devpack-solidity.git
cd neo-devpack-solidity/devpack
npm install
```

The current Hardhat plugins target Hardhat 2.28.x. Hardhat 3 uses a different plugin/runtime model
and needs a dedicated migration before it can be supported.

### Basic Usage

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@neo-devpack-solidity/contracts/standards/NEP17.sol";

contract MyToken is NEP17 {
    constructor() NEP17("My Token", "MTK", 18, 1000000, 10000000) {
        // Your token is now ready with full Neo N3 integration!
    }
}
```

## 🏗️ Deploying Contracts With Constructor Arguments

Neo N3 deployment invokes `_deploy(data, update)`. `neo-devpack-solidity` injects `_deploy` automatically.

When your Solidity contract constructor has parameters, pass them through `_deploy.data` as an
array of arguments:

- Neo-Express / CLI tooling: pass a JSON array string (for example `[7]`).
- SDKs that support StackItems directly may pass an Array value.
- Contract-to-contract deploy flows may pass StdLib.serialize(...) bytes (for example `abi.encode(...)`).

The injected deploy prologue attempts to parse `data` via `StdLib.jsonDeserialize`, then falls back
to `StdLib.deserialize` (binary serialization), and finally uses `data` as-is when native calls
throw. The resulting Array is then fed into the constructor.

This means the generated manifest will include permission entries for `StdLib.jsonDeserialize` and
`StdLib.deserialize`.

## 📚 Core Components

### 🏗️ FrameworkBase.sol

Base framework providing Neo N3 integration with minimally-permissioned manifests.

```solidity
import "@neo-devpack-solidity/contracts/contracts/FrameworkBase.sol";

contract MyContract is FrameworkBase {
    event MyEvent(string data);

    function myFunction() public withWitness {
        // Function requires valid witness (signature)

        // Access Neo blockchain data
        (uint256 blockIndex, bytes32 hash, uint256 timestamp,) = getCurrentBlock();

        // Interact with native contracts
        uint256 gasBalance = getBalance();

        // Emit Neo-compatible events
        emit MyEvent("data");
    }
}
```

### ⚠️ Framework.sol (Dynamic Calls)

`Framework.sol` extends `FrameworkBase.sol` and exposes `callContract(...)`, which performs fully
dynamic contract calls (dynamic target + method). This requires full wildcard permissions in the
Neo N3 manifest (`{"contract":"*","methods":"*"}`) and should be used only when you explicitly
need that surface.

To enforce manifest strictness in CI/production builds, compile with:

- `--deny-wildcard-permissions` (reject `{"contract":"*","methods":"*"}`)
- `--deny-wildcard-contracts` (reject any `{"contract":"*", ...}`)
- `--deny-wildcard-methods` (reject any `{..., "methods":"*"}`)

If you must keep dynamic calls but still want a deployable non-wildcard manifest, provide an explicit
allowlist and replace wildcard entries:

- `--manifest-permissions permissions.json --manifest-permissions-mode replace-wildcards`

Example `permissions.json`:

```json
[
  {
    "contract": "0x0102030405060708090a0b0c0d0e0f1011121314",
    "methods": ["ping"]
  }
]
```

### 🔧 Syscalls.sol

Neo N3 syscall (and syscall-like) integration used by `neo-devpack-solidity`:

```solidity
import "@neo-devpack-solidity/contracts/contracts/Syscalls.sol";

contract MySyscallContract {
    using Syscalls for *;

    function useBlockchain() public view {
        // Runtime / blockchain information
        uint256 currentHeight = Syscalls.getCurrentIndex();
        uint256 timeMs = Syscalls.getTime();

        // Crypto and witness checks
        bytes32 hash = Syscalls.sha256("data");
        bool verified = Syscalls.checkWitness(msg.sender);

        currentHeight;
        timeMs;
        hash;
        verified;
    }
}
```

### 🏛️ NativeCalls.sol

Direct integration with Neo native contracts:

```solidity
import "@neo-devpack-solidity/contracts/contracts/NativeCalls.sol";

contract MyNativeContract {
    using NativeCalls for *;

    function useNativeContracts() public {
        // NEO token operations
        uint256 neoBalance = NativeCalls.neoBalanceOf(msg.sender);
        bool success = NativeCalls.neoTransfer(msg.sender, address(this), 1, "");

        // GAS token operations
        uint256 gasBalance = NativeCalls.gasBalanceOf(msg.sender);

        // Contract management
        address newContract = NativeCalls.deployContract(nefData, manifestData);

        // Policy queries
        uint256 storagePrice = NativeCalls.getStoragePrice();

        // Oracle requests
        NativeCalls.requestOracleData("https://api.example.com", "$.price", "callback", "", 10000000);
    }
}
```

## 📋 NEP Standards

### 🪙 NEP-17 Fungible Tokens

```solidity
import "@neo-devpack-solidity/contracts/standards/NEP17.sol";

contract MyNEP17Token is NEP17 {
    constructor() NEP17("My Token", "MTK", 18, 1000000, 0) {
        // Token with 18 decimals, 1M initial supply, no max supply
    }

    event CustomMint(address indexed to, uint256 amount);

    function customMint(address to, uint256 amount) public onlyMinter {
        mint(to, amount);

        // Emit custom event
        emit CustomMint(to, amount);
    }
}
```

### 🎨 NEP-11 Non-Fungible Tokens

```solidity
import "@neo-devpack-solidity/contracts/standards/NEP11.sol";

contract MyNEP11NFT is NEP11 {
    constructor() NEP11("My NFT", "MNFT", 0, "https://api.mynft.com/", 10000, false) {
        // Indivisible NFT with 10k max supply
    }

    function mintNFT(address to, bytes memory metadata) public onlyMinter returns (bytes32) {
        bytes32 tokenId = bytes32(_currentTokenId++);
        mint(to, tokenId, metadata);
        return tokenId;
    }
}
```

### 💸 NEP-24 Royalties (NFT)

```solidity
import "@neo-devpack-solidity/contracts/standards/NEP24.sol";

contract MyRoyaltyMixin is NEP24Royalty {
    constructor(address recipient, uint96 bps) {
        // Example: 500 = 5%
        _setDefaultRoyalty(recipient, bps);
    }
}
```

## 🛠️ Advanced Libraries

### 🌐 Neo.sol - Blockchain Utilities

```solidity
import "@neo-devpack-solidity/contracts/libraries/Neo.sol";
import "@neo-devpack-solidity/contracts/libraries/Runtime.sol";

contract MyContract {
    using Neo for *;
    using Runtime for *;

    function advancedOperations() public {
        // Account management (native contracts)
        uint256 neoBalance = Neo.getNeoBalance(msg.sender);
        uint256 gasBalance = Neo.getGasBalance(msg.sender);

        // Governance helpers (committee/validators are derived from public keys)
        bool isCommittee = Neo.isCommittee(msg.sender);
        bytes[] memory committee = Neo.getCommittee(); // ECPoint public keys (33 bytes each)

        // Contract interaction
        bytes memory result = Neo.callContract(targetContract, "method", params);

        // Security (Neo witness check)
        bool verified = Runtime.checkWitness(msg.sender);
        uint256 random = Neo.getRandom();

        neoBalance;
        gasBalance;
        isCommittee;
        committee;
        result;
        verified;
        random;
    }
}
```

### 💾 Storage.sol - Advanced Storage

```solidity
import "@neo-devpack-solidity/contracts/libraries/Storage.sol";
import "@neo-devpack-solidity/contracts/contracts/Syscalls.sol";

contract MyStorageContract {
    using Storage for *;

    function advancedStorage() public {
        // Basic operations (lowered to System.Storage.*)
        Storage.put("key", abi.encode(uint256(123)));
        uint256 value = abi.decode(Storage.get("key"), (uint256));
        Storage.remove("key");

        // Iteration (Storage.find returns an iterator over [key, value] pairs)
        Syscalls.Iterator memory it = Storage.find("prefix");
        while (it.next()) {
            bytes memory k = it.currentKey;
            bytes memory v = it.value();
            k;
            v;
        }

        value;
    }
}
```

### ⚡ Runtime.sol - Runtime Services

```solidity
import "@neo-devpack-solidity/contracts/libraries/Runtime.sol";

contract MyRuntimeContract {
    using Runtime for *;

    event MyEvent(string data);

    function runtimeOperations() public {
        // Event emission (recommended)
        emit MyEvent("data");
        // Advanced: Runtime.notify("MyEvent", abi.encode("data")) (requires matching event declaration)

        // Authorization (Neo witness check)
        bool ok = Runtime.checkWitness(msg.sender);

        // Gas / time
        uint256 gasRemaining = Runtime.gasLeft();
        uint256 timeMs = Runtime.getTime();

        ok;
        gasRemaining;
        timeMs;
    }
}
```

## 🎨 Complete Examples

### 💰 Advanced NEP-17 Token

See [`examples/CompleteNEP17Token.sol`](./examples/CompleteNEP17Token.sol) for:

- ✅ Full NEP-17 compliance
- ✅ Staking system with rewards
- ✅ Oracle price integration
- ✅ Governance system
- ✅ Multi-signature operations
- ✅ Emergency controls
- ✅ Gas optimization

### 🖼️ Advanced NEP-11 NFT

See [`examples/CompleteNEP11NFT.sol`](./examples/CompleteNEP11NFT.sol) for:

- ✅ NEP-11 method/event surface (note: manifest types deviate from the
  canonical NEP-11 spec — `bytes32` token IDs emit as `Hash256`, `tokensOf`
  returns an `Array` instead of an iterator, `properties` returns `ByteArray`
  instead of `Map`; see `standards/STANDARDS_MAPPING.md` → "Manifest Type
  Deviations")
- ✅ Royalty system
- ✅ Marketplace integration
- ✅ Oracle metadata updates
- ✅ Curation system
- ✅ Fractionalization
- ✅ Bundle creation

## 🔧 Integration Examples

### Basic Token Implementation

```solidity
pragma solidity ^0.8.19;

import "@neo-devpack-solidity/contracts/standards/NEP17.sol";

contract SimpleToken is NEP17 {
    constructor() NEP17("Simple Token", "SIMPLE", 8, 1000000, 0) {
        // 1M tokens with 8 decimals, no max supply
    }

    // Token is ready to use with all Neo N3 features!
}
```

### Basic NFT Implementation

```solidity
pragma solidity ^0.8.19;

import "@neo-devpack-solidity/contracts/standards/NEP11.sol";

contract SimpleNFT is NEP11 {
    constructor() NEP11("Simple NFT", "SNFT", 0, "https://api.simple.nft/", 1000, false) {
        // Collection with 1000 max supply, indivisible
    }

    function mintNFT(address to, string memory metadata) public onlyMinter returns (bytes32) {
        bytes32 tokenId = generateTokenId(msg.sender, block.timestamp);
        mint(to, tokenId, bytes(metadata));
        return tokenId;
    }
}
```

### Oracle Integration (Native Oracle)

```solidity
pragma solidity ^0.8.19;

import "@neo-devpack-solidity/contracts/contracts/OracleService.sol";

contract PriceConsumer is IOracleServiceReceiver {
    OracleService private _oracle;
    mapping(string => uint256) public prices;

    constructor(address oracleService) {
        _oracle = OracleService(oracleService);
    }

    function updatePrice(string calldata symbol) external returns (uint256 requestId) {
        string memory url = string(abi.encodePacked("https://example.com/prices/", symbol));
        requestId = _oracle.request(url, "", abi.encode(symbol), 20_000_000);
    }

    function onOracleResponse(
        uint256,
        uint256 code,
        bytes calldata result,
        bytes calldata userData
    ) external override {
        require(msg.sender == address(_oracle), "unauthorized oracle response");
        if (code == 0) {
            string memory symbol = abi.decode(userData, (string));
            uint256 price = abi.decode(result, (uint256));
            prices[symbol] = price;
        }
    }
}
```

## 🔒 Security Best Practices

### Witness Verification

```solidity
contract SecureContract is Framework {
    function secureFunction() public withWitness {
        // Function automatically verifies witness
        // Only callable with valid signature
    }

    function manualWitnessCheck() public {
        require(Runtime.checkWitness(msg.sender), "Invalid witness");
        // Manual witness verification
    }

    function multiSigOperation(uint256 m, bytes[] memory publicKeys) public {
        // Neo witness checks work with standard and multisig account script hashes.
        address multisig = Syscalls.createMultisigAccount(m, publicKeys);
        require(Runtime.checkWitness(multisig), "Insufficient signatures");
    }
}
```

### Gas Management

```solidity
contract GasOptimizedContract is Framework {
    function expensiveOperation() public withGasLimit(50000000) { // Require ~0.5 GAS
        // Gas-intensive operation
    }

    function conditionalOperation() public {
        // Guard expensive work with a gas check.
        if (Runtime.gasLeft() < 10000000) {
            return;
        }

        // Execute only if enough gas.
    }
}
```

### Storage Security

```solidity
contract SecureStorageContract is Framework {
    using Storage for *;

    function secureStorage() public {
        // Secure storage with checksum (implement in-contract using Storage.put/get + keccak256).
        bytes memory value = abi.encode(secretValue);
        bytes32 checksum = keccak256(value);
        Storage.put("sensitive_data", abi.encode(value, checksum));

        bytes memory encoded = Storage.get("sensitive_data");
        if (encoded.length > 0) {
            (bytes memory inner, bytes32 storedChecksum) =
                abi.decode(encoded, (bytes, bytes32));
            require(keccak256(inner) == storedChecksum, "corrupt");
        }

        // Access-controlled storage
        require(Runtime.checkWitness(owner()), "unauthorized");
        Storage.put("admin_data", abi.encode(adminValue));

        // Expiring storage (example: expires after N blocks)
        Storage.put("temp_data", abi.encode(tempValue, block.number + 1000));
    }
}
```

## 📊 Performance Optimization

### Batch Operations

```solidity
contract OptimizedContract is NEP17 {
    function batchTransfers() public {
        // Use built-in batch transfer
        address[] memory recipients = [addr1, addr2, addr3];
        uint256[] memory amounts = [100, 200, 300];
        batchTransfer(recipients, amounts, new bytes[](3));
    }

    function batchStorage() public {
        // Batch storage operations (simple loop)
        bytes[] memory keys = [key1, key2, key3];
        bytes[] memory values = [val1, val2, val3];
        for (uint256 i = 0; i < keys.length; i++) {
            Storage.put(keys[i], values[i]);
        }
    }
}
```

### Gas Optimization

```solidity
contract GasOptimized is Framework {
    function optimizedLoop() public {
        // Neo N3 exposes remaining gas via Runtime.GasLeft().
        // Use it to stop early if needed.
        for (uint256 i = 0; i < 1000; i++) {
            if (Runtime.gasLeft() < 5000) break;
            Storage.put(abi.encode("item", i), abi.encode(i * 2));
        }
    }
}
```

## 🎯 Deployment Guide

### Hardhat Configuration

Use Hardhat 2.28.x with the Neo plugins shown below. Hardhat 3 is intentionally outside the current
peer dependency range because its plugin API and network configuration model are incompatible with
these packages today.

```javascript
// hardhat.config.js
require("@neo-devpack-solidity/hardhat-solc-neo");
require("@neo-devpack-solidity/hardhat-neo-deployer");

module.exports = {
  solidity: {
    version: "0.8.19",
    settings: {
      optimizer: { enabled: true, runs: 200 },
      neo: {
        devpack: true,
        syscalls: "all",
        nativeContracts: "all",
        nepStandards: ["NEP-17", "NEP-11", "NEP-24", "NEP-22", "NEP-26", "NEP-27", "NEP-29", "NEP-30", "NEP-31"],
      },
    },
  },
  networks: {
    neo_testnet: {
      url: "http://seed1t5.neo.org:20332",
      accounts: ["your-private-key"],
    },
  },
};
```

### Deployment Workflow

Use the Neo-native Hardhat tasks exposed by `@neo-devpack-solidity/hardhat-solc-neo` and
`@neo-devpack-solidity/hardhat-neo-deployer`. They operate on Neo build artifacts and constructor arguments
encoded as a JSON array.

```bash
# Compile to Neo artifacts
npx hardhat neo-compile

# Deploy a compiled contract to Neo TestNet
npx hardhat neo-deploy \
  --network neo_testnet \
  --contract CompleteNEP17Token \
  --args '["My Token","MTK",18,1000000,10000000]'

# Verify on-chain NEF + manifest against the local build artifact
npx hardhat neo-verify \
  --network neo_testnet \
  --contract CompleteNEP17Token \
  --address <contract-address> \
  --constructor-args '["My Token","MTK",18,1000000,10000000]'
```

### Compilation

```bash
# Compile with Neo devpack
npx hardhat neo-compile

# Deploy to TestNet
npx hardhat neo-deploy --network neo_testnet --contract CompleteNEP17Token --args '["My Token","MTK",18,1000000,10000000]'

# Verify against deployed Neo bytecode + manifest
npx hardhat neo-verify --network neo_testnet --contract CompleteNEP17Token --address <contract-address> --constructor-args '["My Token","MTK",18,1000000,10000000]'
```

## 🧪 Testing

### Artifact-Level Integration Tests

```javascript
const { expect } = require("chai");
const fs = require("fs");
const path = require("path");
const hre = require("hardhat");

describe("Neo artifact generation", function () {
  it("writes Neo build-info and manifest outputs", async function () {
    await hre.run("neo-compile", { force: true, quiet: true });

    const buildInfoDir = path.join(__dirname, "..", "artifacts", "neo-build-info");
    const buildInfoFiles = fs.readdirSync(buildInfoDir).filter((file) => file.endsWith(".json"));

    expect(buildInfoFiles.length).to.be.greaterThan(0);

    const artifact = JSON.parse(
      fs.readFileSync(
        path.join(__dirname, "..", "artifacts", "contracts", "Framework.sol", "Framework.json"),
        "utf8"
      )
    );

    expect(artifact.contract.neo.manifest.name).to.equal("Framework");
    expect(artifact.contract.neo.manifest.abi.methods).to.not.be.empty;
  });
});
```

### Deployment Verification

After a successful `neo-deploy`, use `neo-verify` to compare the deployed contract's NEF script and
manifest with the local build artifact. This is the supported verification path for Neo deployments;
the generic EVM `hardhat verify` flow is not used here.

## 📖 API Reference

### Framework Methods

| Method              | Description              | Gas Cost  |
| ------------------- | ------------------------ | --------- |
| `getCurrentBlock()` | Get current block info   | Low       |
| `getBalance()`      | Get contract GAS balance | Low       |
| `transferGas()`     | Transfer GAS tokens      | Medium    |
| `callContract()`    | Call another contract    | High      |
| `deployContract()`  | Deploy new contract      | Very High |

### Storage Methods

| Method            | Description            | Gas Cost |
| ----------------- | ---------------------- | -------- |
| `put(key, value)` | Store value            | Medium   |
| `get(key)`        | Retrieve value         | Low      |
| `remove(key)`     | Delete value           | Low      |
| `find(prefix)`    | Find keys with prefix  | High     |

### Runtime Methods

| Method           | Description       | Gas Cost |
| ---------------- | ----------------- | -------- |
| `notify()`       | Emit event        | Low      |
| `checkWitness()` | Verify signature  | Medium   |
| `gasLeft()`      | Get remaining gas | Low      |
| `log()`          | Write to logs     | Low      |

## 🚨 Error Handling

### Common Errors

```solidity
contract ErrorHandling is Framework {
    function handleErrors() public {
        try this.riskyOperation() {
            // Success
        } catch Error(string memory reason) {
            Runtime.log(string(abi.encodePacked("Error: ", reason)));
        } catch {
            Runtime.log("Unknown error occurred");
        }
    }

    function riskyOperation() external {
        require(Runtime.gasLeft() > 1000000, "Insufficient gas");
        require(Runtime.checkWitness(msg.sender), "Invalid witness");

        // Operation that might fail
    }
}
```

## 📈 Best Practices

### 1. Always Use Witness Verification

```solidity
function secureFunction() public withWitness {
    // Secure by default
}
```

### 2. Optimize Gas Usage

```solidity
function batchOperation() public {
    // Prefer explicit batching + early exit when gas is low.
    for (uint256 i = 0; i < items.length; i++) {
        if (Runtime.gasLeft() < 5000) break;
        // ... process items[i]
    }
}
```

### 3. Use Typed Storage

```solidity
// Store typed values using abi.encode / abi.decode (NeoVM serialization),
// or prefer Solidity state variables/mappings when possible.
Storage.put("balance", abi.encode(amount));
```

### 4. Handle Oracle Responses

```solidity
function oracleCallback(string calldata url, bytes calldata userData, uint256 code, bytes calldata result) external {
    url; // optional: correlate callback source URL
    if (code == 0) {
        // Handle success
    } else {
        // Handle error
        Runtime.log("Oracle request failed");
    }
}
```

### 5. Implement Emergency Controls

```solidity
event EmergencyStop(address caller, uint256 timestamp);

function emergencyStop() public onlyOwner withWitness {
    // Emergency pause functionality
    _pause();
    emit EmergencyStop(msg.sender, block.timestamp);
}
```

## 🎓 Learning Resources

### Step-by-Step Tutorials

1. **[Basic Token](./examples/CompleteNEP17Token.sol)** - Create your first NEP-17 token
2. **[NFT Collection](./examples/CompleteNEP11NFT.sol)** - Build complete NFT marketplace
3. **[Oracle Integration](./contracts/OracleService.sol)** - Use external data sources
4. **[DeFi Protocol](./examples/VaultPattern.sol)** - Build advanced DeFi applications

### Code Examples

- **[Token Examples](./examples/)** - Various token implementations
- **[NFT Examples](./examples/)** - Different NFT use cases
- **[Oracle Examples](./examples/)** - Oracle integration patterns
- **[DeFi Examples](./examples/)** - DeFi protocol examples

## 💬 Support

- **📖 Documentation**: Complete API reference and guides
- **🐛 Issues**: [GitHub Issues](https://github.com/r3e-network/neo-devpack-solidity/issues)
- **💬 Discord**: Community support and discussions
- **📧 Email**: jimmy@r3e.network for technical support

## 🤝 Contributing

1. Fork the repository
2. Create feature branch
3. Add tests for new features
4. Submit pull request
5. Follow our [contributing guidelines](../CONTRIBUTING.md)

---

**Built with ❤️ by R3E Network**

_Bringing Ethereum's developer ecosystem to Neo blockchain with full N3 integration_
