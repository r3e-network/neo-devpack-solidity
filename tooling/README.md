# Neo DevPack for Solidity Comprehensive Development Tooling

A complete toolchain under active development for Neo DevPack for Solidity. Some packages remain experimental, but the core workspace now has build/test/lint/typecheck coverage.

> ⚠️ **Current Status**  
> - `@neo-devpack-solidity/hardhat-solc-neo`: compile/clean/verify tasks work; advanced Hardhat integration is still evolving.  
> - `@neo-devpack-solidity/hardhat-neo-deployer`: builds/signs/sends real Neo N3 deploy transactions (NEF + manifest); still experimental.  
> - The Hardhat plugins currently target Hardhat 2.28.x. Hardhat 3 uses a different plugin/runtime model and needs a dedicated migration before it can be supported.
> - `@neo-devpack-solidity/neo-foundry` (`neo-forge`, `neo-cast`, `neo-anvil`): `init` is real, and `neo-forge test` delegates to the native `neo-test` runner (see `docs/TESTING.md`); build/deploy flows remain scaffolding.  
> - `@neo-devpack-solidity/abi-router`, `@neo-devpack-solidity/cli-tools`: usable for ABI/CLI composition, but still not full end-to-end deployment frameworks.
> - `@neo-devpack-solidity/templates`, `@neo-devpack-solidity/integration-tests`: now wired into the workspace and covered by the tooling test/lint/typecheck pipeline.

## 🏗️ Architecture Overview

```
Neo DevPack for Solidity Tooling Ecosystem
├── Hardhat Integration
│   ├── @neo-devpack-solidity/hardhat-solc-neo      # Compilation plugin
│   └── @neo-devpack-solidity/hardhat-neo-deployer  # Deployment plugin
├── Foundry Integration  
│   └── @neo-devpack-solidity/neo-foundry           # Foundry-style scaffold + init flow
├── Core Libraries
│   ├── @neo-devpack-solidity/types                 # Shared type definitions
│   ├── @neo-devpack-solidity/abi-router            # ABI compatibility layer
│   └── @neo-devpack-solidity/cli-tools             # Command-line tools
└── Developer Experience
    ├── @neo-devpack-solidity/templates             # Project scaffolding templates
    ├── @neo-devpack-solidity/integration-tests     # Cross-package smoke coverage
    ├── Network configurations              # Neo network presets
    ├── Artifact management                 # Build output handling
    └── Debugging support                   # Development debugging
```

## 🚀 Quick Start

### 1. Hardhat Setup (compile + deploy)

```bash
npm install --save-dev hardhat@^2.28.6
npm install --save-dev @neo-devpack-solidity/hardhat-solc-neo
npm install --save-dev @neo-devpack-solidity/hardhat-neo-deployer

# hardhat.config.ts
import "@neo-devpack-solidity/hardhat-solc-neo";
import "@neo-devpack-solidity/hardhat-neo-deployer";

export default {
  neoSolc: {
    solidity: {
      version: "0.8.34",
      settings: {
        optimizer: { enabled: true, runs: 200 },
        neo: { callt: true }
      }
    }
  },
  neoNetworks: {
    testnet: {
      rpcUrls: ["https://testnet1.neo.coz.io:443"],
      magic: 894710606,
      accounts: [process.env.NEO_WIF || ""] // WIF or private key (hex)
    }
  }
};

# Compile contracts
npx hardhat neo-compile

# Deploy contracts
npx hardhat neo-deploy --contract MyContract --network testnet
```

### 2. Foundry Setup

```bash
npm install -g @neo-devpack-solidity/neo-foundry

# Initialize project
neo-forge init my-neo-project
cd my-neo-project

# neo-foundry.toml configuration
[profile.default]
src = "src"
test = "test" 
script = "script"
out = "out"

# Build and test
# `init` is implemented; `test` delegates to the native `neo-test` runner
# (build it with `cargo build --release --bin neo-test`); `build` remains scaffold-only today.
neo-forge build
neo-forge test
```

### 3. CLI Tools

```bash
npm install -g @neo-devpack-solidity/cli-tools

# Compile contracts directly
solc-neo compile contracts/*.sol --optimize --gas-model hybrid

# Analyze contracts
solc-neo analyze contracts/*.sol --gas-report --size-report

# Verify on-chain NEF/manifest matches your local compilation output
solc-neo verify-contract --address N123... --source Token.sol --contract Token --network testnet
```

## 📦 Package Ecosystem

### Core Packages

#### `@neo-devpack-solidity/types`
Shared TypeScript interfaces and type definitions for all tooling packages.

**Key Types:**
- `NeoDevpackSolidityConfig` - Compiler configuration
- `NeoNetworkConfig` - Network definitions  
- `BuildArtifact` - Compilation artifacts
- `ContractDeployment` - Deployment results
- `NeoRpcProvider` - RPC interface

#### `@neo-devpack-solidity/abi-router` 
ABI-compatible interface layer that bridges Ethereum tooling to Neo contracts. It currently supports static ABIs and best-effort event scanning; transaction signing + deployments are handled by `@neo-devpack-solidity/hardhat-neo-deployer` or native Neo tooling.

**Current capabilities / caveats:**
- ✅ Ethereum-style contract interaction (call/send/estimate).
- ✅ `AbiRouter.deployContract({ nef, manifest }, abi)` when you pass in the compiler artifacts manually.
- ✅ Basic event filtering (linear scan via RPC application logs).
- ⚠️ No automatic artifact registry or dynamic ABI decoding yet.
- ⚠️ Large block ranges can be slow because filtering is sequential.

```typescript
import { readFileSync } from 'fs';
import { AbiRouter } from '@neo-devpack-solidity/abi-router';

const router = new AbiRouter(neoRpcProvider);
const artifacts = { nef: readFileSync('Token.nef', 'hex'), manifest: require('./Token.manifest.json') };

// Deploy a NEF/manifest pair compiled by neo-solc
const deployed = await router.deployContract(artifacts, abi);

// Wrap an existing Neo contract with Ethereum-style methods
const contract = router.createContract(deployed.address, abi, signer);
await contract.transfer(recipient, amount);
      const balance = await contract.balanceOf(account);
```

#### `@neo-devpack-solidity/templates`
Project scaffolding helpers for Neo DevPack for Solidity. The package now ships as a real workspace package and can generate current Hardhat-based starter projects that use `neo-compile`, `neo-deploy`, and `neo-verify` instead of stale EVM-only scripts.

#### `@neo-devpack-solidity/integration-tests`
Workspace-level smoke coverage for the package boundary between templates, CLI tools, Neo Foundry, and the ABI router. These tests intentionally validate the behavior the packages support today rather than future placeholder workflows.

### Hardhat Integration

#### `@neo-devpack-solidity/hardhat-solc-neo`
Hardhat plugin for compiling Solidity to NeoVM bytecode. The runtime extension only exposes `hre.neoSolc.compiler` and `hre.neoSolc.artifacts`; other helpers were removed until a real Neo RPC workflow exists.

**Tasks (currently supported):**
- `neo-compile` - Compile contracts
- `neo-clean` - Clean build artifacts  
- `neo-verify` - Verify on-chain NEF/manifest matches local build artifact (updates deployment metadata)

Pair this with `@neo-devpack-solidity/hardhat-neo-deployer` if you want `neo-deploy` tasks inside Hardhat.

**Configuration:**
```typescript
neoSolc: {
  solidity: {
    version: "0.8.34",
    settings: {
      optimizer: { enabled: true, runs: 200 },
      neo: { callt: true }
    }
  }
}
```

#### `@neo-devpack-solidity/hardhat-neo-deployer`
Experimental Hardhat plugin for deploying Neo N3 contracts. It builds/signs/sends `ContractManagement.deploy` transactions using the NEF + manifest embedded in build artifacts.

**Status**
- `neo-deploy` / `neo-deploy-batch`: submit real deployment transactions.
- `neo-deploy-estimate`: uses `invokescript` + `calculatenetworkfee` to estimate fees.
- `neo-accounts` helpers: derive accounts from WIF/private key; no encrypted keystore yet.

### Foundry Integration

#### `@neo-devpack-solidity/neo-foundry`
Foundry-style tooling for Neo. `neo-forge init` creates a working project layout and config, and `neo-forge test` delegates to the native `neo-test` runner (Foundry-style `test*`/`testFail*`/`setUp`, cheatcodes, decoded reverts — see `docs/TESTING.md`); build/transaction flows remain scaffold-only.

**Tools:**
- `neo-forge` - Project/test CLI (`init` + `test` real; `build` prints stub messages today)
- `neo-cast` - Contract interaction tool (WIP)
- `neo-anvil` - Local Neo blockchain stub

**Commands:**
```bash
# Build system (WIP)
neo-forge build --watch

# Testing (delegates to the native `neo-test` runner)
neo-forge test --gas-report
neo-forge clean

# Contract interaction (WIP)
neo-cast call 0x123... balanceOf 0xabc...
neo-cast send 0x123... transfer 0xdef... 100

# Local blockchain (WIP)
neo-anvil --port 40332 --accounts 10
```

## 🌐 Network Configuration

### Predefined Networks

```typescript
// Built-in network configurations
const networks = {
  mainnet: {
    name: "Neo MainNet",
    rpcUrls: ["https://mainnet1.neo.coz.io:443"],
    magic: 860833102,
    nativeTokens: { gas: "0xd2a4...", neo: "0xef40..." }
  },
  testnet: {
    name: "Neo TestNet", 
    rpcUrls: ["https://testnet1.neo.coz.io:443"],
    magic: 894710606,
    testnet: true
  },
  private: {
    name: "Neo Private",
    rpcUrls: ["http://localhost:40332"],
    magic: 12345,
    testnet: true
  }
};
```

### Custom Network Setup

```typescript
// hardhat.config.ts
neoNetworks: {
  "custom-network": {
    name: "Custom Neo Network",
    rpcUrls: ["https://rpc.custom-neo.com"],
    magic: 123456,
    addressVersion: 0x35,
    accounts: ["0x..."] // Private keys or mnemonic
  }
}
```

## 🔧 Development Workflow

### 1. Project Structure

```
my-neo-project/
├── contracts/           # Solidity source files
│   ├── Token.sol
│   └── interfaces/
├── test/               # Test files
│   └── Token.test.ts
├── scripts/            # Deployment scripts
│   └── deploy.ts
├── artifacts/          # Build artifacts
│   └── contracts/
├── deployments/        # Deployment records
│   ├── testnet/
│   └── mainnet/
├── hardhat.config.ts   # Hardhat configuration
└── neo-foundry.toml   # Neo Foundry configuration
```

### 2. Contract Development

```solidity
// contracts/Token.sol
pragma solidity ^0.8.34;

contract MyToken {
    mapping(address => uint256) public balances;
    uint256 public totalSupply;

    constructor(uint256 initialSupply) {
        totalSupply = initialSupply;
        balances[msg.sender] = initialSupply;
    }

    function transfer(address from, address to, uint256 amount, bytes memory data) external returns (bool) {
        data;
        require(from == msg.sender, "from must be caller");
        require(balances[from] >= amount, "insufficient balance");
        balances[from] -= amount;
        balances[to] += amount;
        return true;
    }
}
```

### 3. Testing

```typescript
// test/Token.test.ts
import { expect } from "chai";
import hre from "hardhat";

describe("MyToken build", function () {
  it("emits Neo build artifacts", async function () {
    await hre.run("neo-compile", { force: true, quiet: true });

    const artifact = await hre.neoSolc.artifacts.getBuildArtifact("MyToken");

    expect(artifact).to.not.equal(null);
    expect(artifact?.contract.neo.manifest.name).to.equal("MyToken");
    expect(artifact?.contract.neo.manifest.abi.methods.length).to.be.greaterThan(0);
  });
});
```

### 4. Deployment

```bash
# Compile Neo build artifacts
npx hardhat neo-compile

# Deploy using the Neo deployer plugin
npx hardhat neo-deploy --contract MyToken --args '[1000000]' --network testnet

# Verify deployed NEF + manifest against the local build artifact
npx hardhat neo-verify --contract MyToken --address <contract-address> --constructor-args '[1000000]' --network testnet
```

## 🛠️ Advanced Features

### Artifact Management

The Hardhat Neo compiler plugin exposes artifact helpers through `hre.neoSolc.artifacts`:

```typescript
import hre from "hardhat";

const buildArtifact = await hre.neoSolc.artifacts.getBuildArtifact("MyToken");
const deployment = await hre.neoSolc.artifacts.getDeploymentArtifact("MyToken", "testnet");
const allBuildArtifacts = await hre.neoSolc.artifacts.getAllBuildArtifacts();
const stats = await hre.neoSolc.artifacts.getStatistics();
```

### Debugging Support

There is no published `@neo-devpack-solidity/debugger` package yet. For debugging today:

```bash
# Emit NeoVM assembly for inspection
neo-solc contracts/Token.sol -f assembly -o build/Token.asm

# Run against Neo-Express and inspect invocation logs
neoxp contract invoke <contract-hash> totalSupply
```

### Gas Optimization

Analyze and optimize gas usage:

```bash
# Generate gas reports
neo-forge test --gas-report

# Analyze optimization opportunities
solc-neo analyze contracts/*.sol --gas-report --size-report

# Output format options
solc-neo analyze --output table   # Console table
solc-neo analyze --output json    # JSON format  
solc-neo analyze --output csv     # CSV export
```

## 📚 API Reference

### Compiler Configuration

```typescript
interface NeoDevpackSolidityConfig {
  version?: string;
  optimizer?: {
    enabled: boolean;
    runs: number;
  };
  neo?: {
    gasCostModel?: 'ethereum' | 'neo' | 'hybrid';
    storageOptimization?: boolean;
    eventOptimization?: boolean;
  };
}
```

### Network Configuration

```typescript
interface NeoNetworkConfig {
  name: string;
  rpcUrls: string[];
  magic: number;
  addressVersion: number;
  nativeTokens: {
    gas: NeoToken;
    neo: NeoToken;
  };
}
```

### Contract Interface

```typescript
interface ContractWrapper {
  address: string;
  interface: Interface;
  
  // Read-only calls
  call(method: string, args: any[]): Promise<any>;
  
  // State-changing transactions  
  send(method: string, args: any[]): Promise<TransactionResponse>;
  
  // Gas estimation
  estimateGas(method: string, args: any[]): Promise<bigint>;
  
  // Event handling
  on(event: string, listener: Function): this;
  queryFilter(event: string, filter?: any): Promise<any[]>;
}
```

## 🔍 Troubleshooting

### Common Issues

**Compiler Not Found**
```bash
# Install compiler
npm install -g @neo-devpack-solidity/cli-tools
solc-neo install latest
```

**Network Connection Issues**
```typescript
// Check network configuration
neoNetworks: {
  testnet: {
    rpcUrls: ["https://testnet1.neo.coz.io:443"], // Verify URL
    magic: 894710606, // Correct magic number
    timeout: 30000    // Increase timeout
  }
}
```

**Gas Estimation Failures**
```typescript
// Increase gas limits
const tx = await contract.method({
  gasLimit: "50000000", // 0.5 GAS
  gasPrice: "1000"
});
```

### Debug Mode

Enable debug logging:

```bash
DEBUG=neo-devpack-solidity:* npx hardhat neo-compile
DEBUG=neo-foundry:* neo-forge build
```

## 🤝 Contributing

Contributions welcome! Please see [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
git clone https://github.com/r3e-network/neo-devpack-solidity
cd neo-devpack-solidity/tooling
npm install
npm run build
```

### Testing

```bash
npm test                 # Run all tests
npm run test:watch       # Watch mode
npm run test:coverage    # Coverage report
```

## 📄 License

MIT License - see [LICENSE](../LICENSE) for details.

---

**Built for the Neo ecosystem** 🚀
