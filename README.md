# Neo DevPack for Solidity

<p align="center">
  <img src="docs/assets/neo-devpack-solidity-banner.png" alt="Neo DevPack for Solidity Banner" width="100%">
</p>

[![Build Status](https://github.com/r3e-network/neo-devpack-solidity/workflows/CI/badge.svg)](https://github.com/r3e-network/neo-devpack-solidity/actions)
[![Neo-Express Showcases Workflow](https://github.com/r3e-network/neo-devpack-solidity/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/r3e-network/neo-devpack-solidity/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.88+-blue.svg)](https://rustup.rs)
[![Neo Version](https://img.shields.io/badge/neo-N3%203.0+-green.svg)](https://neo.org)

**Fast, standards-oriented Solidity-to-NeoVM compiler for Neo N3.**

> **Status:** Production-focused · Actively fuzzed · Neo N3 focused

## 🎯 At a Glance

- **Solidity → NeoVM**: Compile Solidity 0.5.x–0.8.x (0.8.x primary) to Neo N3 (`.nef` + `.manifest.json`).
- **Primary Implementation**: Rust-based compiler (production-focused) with archived Go reference implementation.
- **EVM semantics**: ABI-compatible selectors and metadata; NEP standard detection (NEP-11/17/24).
- **Real-world proven**: ~87% end-to-end compilation across ~600 famous contracts (full dependency trees), plus `neo-test`, a Foundry-style Solidity test runner.
- **Optimized output**: Multi-level optimizer, Neo-specific lowering, manifest generation.
- **Tooling friendly**: CLI first, with active Hardhat/Foundry-adjacent workspace packages for compilation, deployment, scaffolding, and cross-package smoke coverage.
- **Quality-focused**: Unit/integration/runtime tests, clear diagnostics.

## 🚀 Quick Start

### Installation

```bash
# Install from source
git clone https://github.com/r3e-network/neo-devpack-solidity.git
cd neo-devpack-solidity
cargo install --path .

# Or download pre-built binaries
curl -L https://github.com/r3e-network/neo-devpack-solidity/releases/latest/download/neo-solc-linux-x64 -o neo-solc
chmod +x neo-solc
```

### Basic Usage

```bash
# Compile Solidity to Neo N3 contract (generates .nef + .manifest.json)
neo-solc contract.sol -o contract

# With optimization
neo-solc contract.sol -O3 -o contract

# Emit CALLT + method tokens (more efficient native contract calls)
neo-solc contract.sol --callt -O3 -o contract

# Generate only specific formats
neo-solc contract.sol -f nef -o contract.nef
neo-solc contract.sol -f manifest -o contract.manifest.json
neo-solc contract.sol -f assembly -o contract.asm
```

### Batch Compilation (Examples)

The repository ships a handful of non-trivial Solidity contracts under `examples/`
(ERC-20/721, UniswapV2Pair, governance, multisig). To compile them all into Neo
artifacts:

```bash
mkdir -p build/examples
for f in examples/*.sol; do
  target/release/neo-solc "$f" -I devpack -O2 -o "build/examples/$(basename "$f" .sol)"
done
```

For a quick end-to-end sanity check (NEF magic + manifest structure), run:

```bash
bash examples/test_compilation.sh
```

For a local deploy + invoke smoke test (fresh Neo-Express chain), run:

```bash
make test-deploy-smoke
# or:
bash examples/test_neoxp_deploy.sh
```

For a deploy smoke test that validates parameterised Solidity constructors
(constructor args passed via `_deploy(data, update)`), run:

```bash
make test-deploy-constructor-smoke
# or:
bash examples/test_neoxp_constructor_smoke.sh
```

For an additional deploy test that validates manifest permissions for native
contracts (StdLib/CryptoLib) via mapping storage, run:

```bash
make test-deploy-permissions-smoke
# or:
bash examples/test_neoxp_permissions_smoke.sh
```

To run all Neo-Express smoke tests:

```bash
make test-deploy-smoke-full
```

### Deploy Famous Upstream EVM Contracts on Neo N3

To demonstrate compiler capability on widely used upstream contracts (OpenZeppelin, Aave, Safe, Uniswap, Chainlink), run:

```bash
npm run deploy:famous-contracts:neoxp
```

This command compiles + deploys a curated set of upstream contracts to a fresh local Neo N3 chain (`neoxp`), then generates:

- `docs/data/famous-contracts-neoxp-deploy-results.json`
- `docs/solidity/famous-contracts-neoxp-deploy.md`

It auto-installs Neo Express `3.9.1` into `build/dotnet-tools/` when missing.
(Neo.Express tracks its own release cadence independent of the Neo N3 node
version — the compiler/runtime target Neo N3 **v3.10.0**, but the latest
Neo.Express toolchain is 3.9.1.)

For strict **type-3** verification (deploy + state-changing invoke + post-state assertion), run:

```bash
npm run verify:famous-contracts:neoxp-runtime
```

This generates:

- `docs/data/famous-contracts-neoxp-runtime-results.json`
- `docs/solidity/famous-contracts-neoxp-runtime.md`

Use this runtime report when you need executable correctness proof, not deploy-only coverage.

For the strict-safe new showcase suite specifically (wired in CI as `neoxp-showcases`):

```bash
make test-deploy-new-showcases-smoke
# or
bash examples/test_neoxp_new_showcases_smoke.sh
```

### Famous-Contract Compilation Coverage

`neo-solc` compiles the real-world Solidity ecosystem — validated against
~600 famous contracts (with full dependency trees) at **~87% end-to-end
compilation**. It handles the hardest material in the ecosystem: snarkjs /
PLONK zero-knowledge verifiers, MakerDAO's assembly `Vat`, Gnosis Safe, and
the full Uniswap V4 + Permit2 stack.

| Category | Representative contracts | Samples |
| --- | --- | --- |
| **Zero-knowledge** | Groth16 / PLONK / Fflonk verifiers, Tornado, Semaphore, Poseidon | 5 |
| **DeFi** (DEX · lending · stablecoin) | WETH9, Uniswap V2/V4, Permit2, Aave v3, Compound, MakerDAO `dss`/DAI, Liquity | 6 |
| **NFT & marketplaces** | ERC721A, Seaport core, BoredApeYachtClub, solmate 721/1155, OZ ERC721 | 6 |
| **GameFi & staking** | Synthetix StakingRewards, MasterChef, game items, vesting | 4 |
| **Infrastructure / DAO** | Gnosis Safe, OZ proxy/UUPS/beacon, Multicall3, Create2, Timelock, Governor | 9 |
| **Fix-pattern reproductions** | library events, abstract-typed fields, inherited events | 3 |

Aggregate ~87% (per-corpus: ZK **100%**, solmate **100%**, solady **99%**,
Uniswap-v3 **93%**, OpenZeppelin **89%**, Aave v3 **85%**).

A curated, **dependency-free** subset of **33 self-contained** real contracts
(Solidity 0.5.x–0.8.x) lives in
[`third_party/famous-contracts/samples/`](third_party/famous-contracts/samples/)
— see its [README](third_party/famous-contracts/samples/README.md) for the
full per-contract table (license, pragma, size). Every sample is asserted to
compile hermetically (no include paths) by:

```bash
cargo test --release --test famous_samples_compile
```

### Production Readiness Gate

Run one command to validate formatting, lint, release build, full tests,
strict-compatibility compile sweeps, and full Neo-Express deploy smokes:

```bash
make production-gate
```

### CI Coverage (Neo-Express Showcases)

The CI workflow (`.github/workflows/ci.yml`) includes a dedicated `neoxp-showcases` job that:

- installs Rust + .NET 8 + `jq` on Ubuntu
- runs `examples/test_neoxp_new_showcases_smoke.sh`
- validates `UpgradeLifecycleShowcase`, `WitnessGuardShowcase`, and `OracleRelayStrictShowcase` end-to-end

This keeps local and CI smoke coverage aligned for the new strict-safe showcase contracts.

For an on-chain check that `abi.encode` / `abi.decode` preserve argument order
(StdLib.serialize/deserialize round-trip), run:

```bash
make test-deploy-encoding-smoke
# or:
bash examples/test_neoxp_encoding_smoke.sh
```

### Runtime Semantics & Metadata

- **Execution overrides**: `ExecutionOverrides` lets you inject deterministic
  block height, timestamp, and calling script hash for a single invocation.
  Use `NeoRuntime::execute_with_overrides` and inspect `ExecutionMetadata` on
  `ExecutionResult`.
- **Iterator handles**: `Storage.Find` returns real iterator tokens; `Iterator.Next`
  and `Iterator.Value` operate on handles and respect overlay storage changes.
- **Syscall gas hints**: The embedded runtime uses per-syscall gas hints
  (storage/crypto/runtime/oracle/contract) to better mirror Neo N3 pricing.
- **Contract registry**: A lightweight in-memory ContractManagement surface
  supports `Deploy`, `Update`, and `GetContract`, tracking NEF/manifest bytes and
  update counters for native contract calls.

For a detailed runtime surface (opcodes, syscalls, native contracts, iterator
semantics, gas hints), see `docs/RUNTIME_SPEC.md`.

### Example Contract

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.34;

contract SimpleToken {
    mapping(address => uint256) public balances;
    uint256 public totalSupply;

    event Transfer(address indexed from, address indexed to, uint256 value);

    constructor(uint256 _totalSupply) {
        totalSupply = _totalSupply;
        balances[msg.sender] = _totalSupply;
    }

    function transfer(address from, address to, uint256 amount, bytes memory data) public returns (bool) {
        data;
        require(from == msg.sender, "from must be caller");
        require(balances[from] >= amount, "Insufficient balance");
        balances[from] -= amount;
        balances[to] += amount;
        emit Transfer(from, to, amount);
        return true;
    }
}
```

**Compilation & Deployment:**

```bash
# 1. Compile to Neo N3 contract files
neo-solc SimpleToken.sol -O2 -o SimpleToken
# This generates: SimpleToken.nef + SimpleToken.manifest.json

# 2. Deploy to Neo TestNet
# If your Solidity constructor has parameters, pass constructor args through
# `_deploy(data, update)`. Neo-Express / CLI tooling: pass a JSON array string
# (e.g. `[1000000]`); SDKs that support StackItems may pass an Array directly.
# For contract-to-contract deploy flows, `abi.encode(...)` (StdLib.serialize bytes) is also supported.
#
# For local Neo-Express deploys, see:
#   bash examples/test_neoxp_constructor_smoke.sh
neo-cli contract deploy SimpleToken.nef SimpleToken.manifest.json

# 3. Verify deployment
neo-cli contract invoke <contract-hash> totalSupply
```

## 🧩 **Solidity Feature Support on NeoVM**

> **146 Solidity features audited** — ✅ 114 fully supported (78%) · ⚠️ 29 partial (20%) · ❌ 2 unsupported (1%) · 🚫 1 intentionally blocked (1%)

The maintained per-feature source of truth is
[`docs/SOLIDITY_SUPPORT_MATRIX.md`](./docs/SOLIDITY_SUPPORT_MATRIX.md), with
rendered category pages under
[`docs/solidity/feature-support/`](./docs/solidity/feature-support/). The root
[`FEATURE_MATRIX.md`](./FEATURE_MATRIX.md) is only a stable redirect for older
links.

Important NeoVM differences:

| Area | Current behavior |
| --- | --- |
| Contract deployment | `new Contract(...)` is source-compatible but does not deploy a child contract; it inlines/simulates constructor-like logic and returns a zero-address placeholder. Use `ContractManagement.deploy(nef, manifest, data)` for real deployment. |
| Delegate execution | `address.delegatecall(...)` and `callcode(...)` are blocked at compile time because Neo N3 has no caller-storage execution context. |
| Value transfer | Ether-style attached value is not available; use NEP-17 callbacks and explicit GAS/NEP transfers. |
| ABI payloads | Neo contract calls use method names and `StdLib.serialize`-style payloads, not byte-identical EVM calldata. |

---

## 📚 **Complete Documentation**

### **🏗️ Architecture Overview**

The Neo DevPack for Solidity consists of several integrated components:

<p align="center">
  <img src="docs/assets/compiler-architecture.png" alt="Compiler Architecture" width="80%">
</p>

```mermaid
graph TB
    A[Solidity Source] --> B[solang-parser AST]
    B --> C[Neo DevPack for Solidity]
    C --> D[Frontend: parse + contract metadata]
    C --> E[Solidity Analysis: inheritance, modifiers, validation]
    C --> F[IR Lowering: expressions, statements, builtins]
    C --> G[Optimizer: constant folding, peephole, dead code]
    C --> H[Bytecode Emission: NeoVM ops + method tokens]
    H --> I[NeoVM Bytecode .nef]
    H --> J[Neo Manifest .manifest.json]
    H --> K[ABI JSON]

    L[Embedded Runtime Simulator] --> M[Storage Manager]
    L --> N[Syscall Handlers]
    L --> O[ABI Codec]
    L --> P[Crypto + BLS12-381]
    L --> Q[Event / Notify System]

    R[Developer Tools] --> S[Hardhat Plugin]
    R --> T[Foundry Adapter]
    R --> U[CLI Tools]
    R --> V[Debug Tools]
```

> [!NOTE]
> **Neo-Sol Runtime (C#) is standalone and experimental.** The `src/Neo.Sol.Runtime` C#
> library shown above is a separate EVM-emulation experiment: it is **not used by the
> `neo-solc` compiler**, is **not shipped in releases**, and uses an EVM keccak-style storage
> layout that is intentionally different from compiler-emitted contracts (which use
> `SHA256(variable_name)` base slots plus `keccak256(serialize(key) || slot)` for mapping
> elements). The two storage layouts are not interoperable.

### **🔧 Installation & Setup**

#### **System Requirements**

- **Rust**: 1.88 or higher
- **Node.js**: 20.19+ or 22.12+ (for tooling and documentation builds)
- **.NET SDK**: 10.0 or higher (for Neo-Express smoke tests and optional C# runtime)
- **Neo CLI**: 3.0+ (for deployment)
- **Memory**: 4GB RAM minimum, 8GB recommended
- **Disk Space**: 2GB for full installation

#### **Build from Source**

```bash
# Clone repository
git clone https://github.com/r3e-network/neo-devpack-solidity.git
cd neo-devpack-solidity

# Build compiler
cargo build --release

# (Optional) Build C# runtime library (requires .NET SDK)
dotnet build src/Neo.Sol.Runtime/Neo.Sol.Runtime.csproj --configuration Release

# (Optional) Build tooling packages
npm --prefix tooling install
npm --prefix tooling run build

# Run comprehensive tests
make test-all
```

#### **Development Setup**

```bash
# Install development dependencies (Rust + tooling)
make install-deps

# Build tooling packages
make tooling-build

# Run all test suites
make test-all
```

### CLI Reference

#### Basic Commands

```bash
# Compile with default settings (generates .nef + .manifest.json)
neo-solc contract.sol

# Specify output file prefix
neo-solc contract.sol -o MyContract

# Set optimization level (0-3)
neo-solc contract.sol -O3

# Generate specific formats
neo-solc contract.sol -f nef          # Only .nef file
neo-solc contract.sol -f manifest     # Only .manifest.json
neo-solc contract.sol -f complete     # Both files (default)
neo-solc contract.sol -f assembly     # NeoVM disassembly (.asm)

# Resolve Solidity imports (repeatable)
neo-solc contracts/Token.sol -I contracts -I lib -o build/Token
```

#### Advanced Options

```bash
# Emit CALLT + method tokens for native calls
neo-solc contract.sol --callt -O3 -o contract

# JSON output with all information
neo-solc contract.sol -f json -o contract.json

# Verbose output for debugging
neo-solc contract.sol -v

# Override NEF source field and emit JSON warnings
neo-solc contract.sol --nef-source https://example.com/src.sol --json-warnings

# Predict the deployed contract hash (Neo derives it from sender + NEF checksum + manifest name)
neo-solc contract.sol --deployer 0x0123456789abcdef0123456789abcdef01234567

# Only emit outputs for specific contracts (repeatable; useful when imports include extra contracts)
neo-solc contract.sol --contract MyContract -o build/MyContract

# Fail compilation if full wildcard manifest permissions are required
neo-solc contract.sol --deny-wildcard-permissions

# Stricter: fail compilation if any wildcard contract permissions are required
neo-solc contract.sol --deny-wildcard-contracts

# Stricter: fail compilation if any wildcard method permissions are required
neo-solc contract.sol --deny-wildcard-methods

# Provide an explicit allowlist to replace wildcard permissions (useful for dynamic calls)
neo-solc contract.sol --manifest-permissions permissions.json --manifest-permissions-mode replace-wildcards \
  --deny-wildcard-contracts --deny-wildcard-methods

# Emit structured errors to stderr as JSON
neo-solc contract.sol --json-errors

Structured diagnostics (stderr):
- Warnings (JSON): `COMPILER_WARNING`, `NEF_SOURCE_TRUNCATED`, `MANIFEST_FULL_WILDCARD`, `MANIFEST_WILDCARD_CONTRACT`, `MANIFEST_WILDCARD_METHODS`, validation codes (e.g., `DUPLICATE_SIGNATURE`, `INVALID_STORAGE_PARAM`)
- Errors (JSON): `VALIDATION_ERROR`, `IR_GENERATION_ERROR`, `MANIFEST_GENERATION_ERROR`, `GENERIC_ERROR`, `IO_ERROR`
```

> **Structured diagnostics:**
>
> - `--json-warnings` emits warnings as JSON lines on stderr (codes: `COMPILER_WARNING`, `NEF_SOURCE_TRUNCATED`).
> - `--json-errors` emits errors as JSON lines on stderr (codes: `VALIDATION_ERROR`, `IR_GENERATION_ERROR`, `GENERIC_ERROR`, `IO_ERROR`).  
>   These flags do not alter file outputs; they only change how diagnostics are printed.

#### Manifest Field Overrides (NatSpec)

Use NatSpec custom tags on a contract to override selected Neo manifest fields at compile time:

```solidity
/**
 * @custom:neo.manifest.groups [{"pubkey":"03...","signature":"AQID"}]
 * @custom:neo.manifest.features {}
 * @custom:neo.manifest.supportedstandards ["NEP-17","NEP-26"]
 * @custom:neo.manifest.trusts ["0x1111111111111111111111111111111111111111"]
 * @custom:neo.manifest.extra.Repository "https://github.com/acme/project"
 * @custom:neo.manifest.extra.Build {"commit":"abc123","pipeline":"ci"}
 */
contract MyContract { }
```

Supported tag prefixes: `@custom:neo.manifest.*` and `@custom:manifest.*`.
Supported fields:

- `name` (string)
- `groups` (JSON array)
- `features` (JSON object)
- `supportedstandards` (JSON array)
- `trusts` (JSON array or `"*"`)
- `extra.<Key>` (any JSON value, or plain string)

For Neo N3 compatibility, `features` must remain an empty object (`{}`); populated
feature keys are ignored because Neo rejects them at deploy time.

#### Batch Operations

```bash
# Compile multiple files
neo-solc src/*.sol -o build/

# Compile specific contract
neo-solc contracts/Token.sol -o build/Token

# Batch compilation with optimization
neo-solc contracts/*.sol -O3 -o build/
```

### Integration Guide

#### Hardhat Integration

Hardhat integration is primarily useful for **compilation + artifact management**.
Use Hardhat 2.28.x with the current Neo plugins; Hardhat 3 needs a separate
plugin/runtime migration before it is supported.

```ts
// hardhat.config.ts
import "@neo-devpack-solidity/hardhat-solc-neo";

export default {
  neoSolc: {
    solidity: {
      version: "0.8.34",
      settings: {
        optimizer: { enabled: true, runs: 200 },
        neo: {
          // neo-solc flags forwarded by the plugin:
          callt: true,
          denyWildcardContracts: true,
          denyWildcardMethods: true,
          // If your contract uses intentional dynamic calls, provide an allowlist:
          // manifestPermissions: "./permissions.json",
          // manifestPermissionsMode: "replace-wildcards",
        },
      },
    },
  },
};
```

```bash
# Compile contracts (standard-json via neo-solc)
npx hardhat neo-compile
```

```bash
# Deploy to Neo (requires a funded account in neoNetworks.<network>.accounts)
npx hardhat neo-deploy --contract MyContract --network testnet
```

#### Foundry Integration

```bash
# Install Neo Foundry
npm install -g @neo-devpack-solidity/neo-foundry

# Initialize project
neo-forge init my-project
cd my-project

# Build contracts
neo-forge build

# Run tests
neo-forge test

`neo-forge init` is implemented and writes a starter project layout. `neo-forge test` delegates to the native `neo-test` runner (build it with `cargo build --release --bin neo-test`; see [docs/TESTING.md](./docs/TESTING.md)). Build/deploy flows remain scaffold-level today; use `neo-solc` + Neo tooling (`neoxp` / `neo-cli`) for compilation and real deployment.
```

#### Direct Integration

There is no stable published JavaScript runtime package for the compiler itself in this repository.
For programmatic workflows today, prefer:

- the Rust `neo-solc` binary directly
- `@neo-devpack-solidity/cli-tools` for Node-based wrapper commands
- `@neo-devpack-solidity/hardhat-solc-neo` and `@neo-devpack-solidity/hardhat-neo-deployer` for Hardhat integration

### Testing Framework

#### Unit Testing

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test lexer_tests

# Run with output
cargo test -- --nocapture

# Run release-mode tests when investigating performance-sensitive behavior
cargo test --release
```

#### Integration Testing

```bash
# Full compilation pipeline tests
cargo test integration_tests

# Real contract examples in the integration module
cargo test test_erc20_like_contract
cargo test test_complex_control_flow

# Aggregate project test lane
make test-all
```

#### Property-Based Testing

```bash
# Fuzzing/property tests for robustness
cargo test --test fuzz_tests

# Deep property run
PROPTEST_CASES=200 cargo test --test fuzz_tests

# Differential testing (EVM vs NeoVM)
cargo test --test fuzz_tests differential
```

### **🎯 API Reference**

#### **Compiler API**

```rust
use neo_devpack_solidity::cli::compile_contracts;

let source = std::fs::read_to_string("contract.sol").expect("read source");
let artifacts = compile_contracts(&source, false, 3).expect("compile");

for artifact in artifacts {
    println!("Contract: {}", artifact.metadata.name);
    println!("Bytecode size: {}", artifact.bytecode.len());
    println!(
        "Manifest methods: {}",
        artifact.manifest["abi"]["methods"]
            .as_array()
            .map_or(0, |methods| methods.len())
    );
}
```

#### **Runtime API**

```csharp
using System.Numerics;
using Neo.Sol.Runtime;

var runtime = Evm.CreateRuntime();

// Memory operations
runtime.Memory.Store(0x40, new BigInteger(123));
var data = runtime.Memory.Load(0x40);
var value = runtime.Memory.LoadBigInteger(0x40);

// Storage operations
runtime.Storage.Store(BigInteger.Zero, value);
var retrieved = runtime.Storage.LoadBigInteger(BigInteger.Zero);

// Cryptographic operations
var hash = runtime.Keccak256(data);
var publicKey = runtime.EcRecover(hash, signature, recoveryId);
```

#### **ABI Encoder API**

```csharp
using Neo.Sol.Runtime.ABI;

// Encode function call
var selector = AbiEncoder.CalculateFunctionSelector("transfer(address,uint256)");
var encoded = AbiEncoder.EncodeCall("transfer(address,uint256)", recipient, amount);

// Decode function result
var success = AbiEncoder.DecodeBool(returnData);

// Encode events
runtime.Events.EmitEvent("Transfer(address,address,uint256)", new object[] { from, to }, amount);
```

### **🚀 Optimization Guide**

#### **Optimization Levels**

| Level | Description             | Use Case               | Compilation Time | Performance Gain |
| ----- | ----------------------- | ---------------------- | ---------------- | ---------------- |
| `-O0` | No optimization         | Development, debugging | Fastest          | None             |
| `-O1` | Basic optimization      | Testing, CI/CD         | Fast             | 10-20%           |
| `-O2` | Standard optimization   | Production builds      | Moderate         | 30-50%           |
| `-O3` | Aggressive optimization | Critical performance   | Slow             | 50-80%           |

#### **Performance Tips**

```solidity
// ✅ Good: Use unchecked only when wraparound is intentional or overflow is proven impossible
unchecked {
    for (uint256 i = 0; i < length; ++i) {
        total += values[i];
    }
}

// ✅ Good: Pack structs efficiently
struct PackedData {
    uint128 amount;      // 16 bytes
    uint64 timestamp;    // 8 bytes
    uint32 blockNumber;  // 4 bytes
    uint32 nonce;        // 4 bytes
}                        // Total: 32 bytes (1 storage slot)

// ✅ Good: Use mapping for O(1) lookups
mapping(address => uint256) balances;

// ❌ Avoid: Linear searches in arrays
address[] holders; // Expensive to search
```

#### **Gas Optimization**

```bash
# Compare optimization levels
neo-solc contract.sol -O0 -o contract-O0
neo-solc contract.sol -O3 -o contract-O3
# Compare the generated .nef file sizes
ls -la contract-O0.nef contract-O3.nef

# Inspect generated NeoVM assembly
neo-solc contract.sol -f assembly -o contract.asm
```

### **🔒 Security Best Practices**

#### **Automated Security Analysis**

```bash
# Analysis mode emits an EVM-to-Neo upgrade/readiness JSON report instead of artifacts.
# It is supported for single-file input; Standard JSON mode rejects it.
neo-solc contract.sol --analyze -o analysis.json

# Compile production artifacts with strict manifest permissions after review.
neo-solc contract.sol --deny-wildcard-permissions -O3 -o contract

# Stricter builds (recommended for production):
# - reject wildcard contract permissions (contract='*')
# - reject wildcard method permissions (methods='*')
neo-solc contract.sol --deny-wildcard-contracts --deny-wildcard-methods -O3 -o contract
```

#### **Common Security Patterns**

```solidity
// ✅ Reentrancy protection
bool private locked;
modifier noReentrancy() {
    require(!locked, "Reentrant call");
    locked = true;
    _;
    locked = false;
}

// ✅ Safe arithmetic (Solidity 0.8+)
function safeAdd(uint256 a, uint256 b) public pure returns (uint256) {
    return a + b; // Built-in overflow protection
}

// ✅ Input validation
function transfer(address to, uint256 amount) public {
    require(to != address(0), "Invalid recipient");
    require(amount > 0, "Invalid amount");
    require(balances[msg.sender] >= amount, "Insufficient balance");
    // ... rest of function
}
```

### **🐛 Debugging Guide**

#### **Debug Information**

```bash
# Compile with verbose output (prints a high-level IR summary)
neo-solc contract.sol -v

# View manifest information
cat contract.manifest.json | jq '.'

# View NeoVM assembly (disassembly)
neo-solc contract.sol -f assembly -o contract.asm
cat contract.asm
```

#### **Common Issues & Solutions**

| Error                      | Cause                                | Solution                        |
| -------------------------- | ------------------------------------ | ------------------------------- |
| `Stack too deep`           | Too many local variables             | Restructure code, use structs   |
| `Gas limit exceeded`       | Infinite loop or expensive operation | Add gas checks, optimize code   |
| `Invalid jump destination` | Corrupted bytecode                   | Check compiler version, rebuild |
| `Revert without reason`    | Failed require without message       | Add descriptive error messages  |

#### **Interactive Debugging**

neo-solc does not currently emit source-map/debug-info artifacts as standalone CLI output formats (`nef`, `manifest`, `json`, `assembly`, `complete` only). Source-map/debug internals exist in the compiler pipeline for downstream debugger-oriented tooling; interactive on-chain debugger support remains planned. Use Neo N3 tooling (neo-cli / neo-express / RPC tracing) for on-chain debugging.

### **📊 Performance Benchmarks**

#### **Compilation Performance**

| Contract Size | Lines of Code | Compilation Time (O2) | Memory Usage |
| ------------- | ------------- | --------------------- | ------------ |
| Simple Token  | 100           | 50ms                  | 15MB         |
| ERC721 NFT    | 500           | 200ms                 | 45MB         |
| DeFi Protocol | 2000          | 800ms                 | 120MB        |
| Large DAO     | 5000          | 2000ms                | 250MB        |

#### **Runtime Performance**

| Operation    | Neo-Sol Runtime | Native NeoVM | Overhead |
| ------------ | --------------- | ------------ | -------- |
| Arithmetic   | 1.2μs           | 1.0μs        | 20%      |
| Memory Load  | 2.1μs           | 1.8μs        | 17%      |
| Storage Load | 12.3μs          | 10.5μs       | 17%      |
| Keccak256    | 45.2μs          | N/A          | N/A      |
| EcRecover    | 156.8μs         | N/A          | N/A      |

### **🤝 Contributing**

#### **Development Workflow**

```bash
# 1. Fork and clone
git clone https://github.com/yourusername/neo-devpack-solidity.git

# 2. Create feature branch
git checkout -b feature/my-new-feature

# 3. Install dependencies
make install-deps

# 4. Make changes and test
make test-all

# 5. Format and lint
make format
make lint

# 6. Commit and push
git commit -m "Add new feature"
git push origin feature/my-new-feature

# 7. Create pull request
```

#### **Code Standards**

- **Rust**: Follow [Rust style guidelines](https://doc.rust-lang.org/1.0.0/style/)
- **C#**: Follow [Microsoft C# conventions](https://docs.microsoft.com/en-us/dotnet/csharp/programming-guide/inside-a-program/coding-conventions)
- **TypeScript**: Follow [Airbnb TypeScript Style Guide](https://github.com/airbnb/javascript/tree/master/packages/eslint-config-airbnb-typescript)
- **Tests**: 100% test coverage for new features
- **Documentation**: Update docs for all public APIs

#### **Release Process**

```bash
# 1. Update version numbers (Cargo + npm packages + docs)
#    Edit Cargo.toml / package.json / devpack/package.json / docs

# 2. Update changelog
#    Edit CHANGELOG.md: promote Unreleased -> new version section

# 3. Run release-readiness validation
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace --all-features

# 4. Commit and push
git add .
git commit -m "release: vX.Y.Z"
git push origin main

# 5. Tag and publish release pipeline
git tag -a vX.Y.Z -m "release vX.Y.Z"
git push origin vX.Y.Z
```

## 📋 **Project Status**

### **Implementation Language**

- **Primary**: Rust (src/) - Production-ready compiler and runtime
- **Archived**: Go implementation (archive/go_implementation/) - Reference implementation, no longer maintained

### **Current Progress**

#### **Core Compiler**

- ✅ Solidity frontend (solang-based parser) with semantic validation
- ✅ Multi-level optimizer (4 levels: 0-3)
- ✅ NeoVM code generator
- ✅ Solidity-style public state variable getters
- ✅ Error handling and reporting
- ✅ CLI interface with file, Standard JSON, manifest-policy, diagnostics, and analysis modes
- ✅ Neo N3 native formats (.nef and .manifest.json)
- ✅ Broad Solidity 0.5.x–0.8.x support (0.8.x primary); see `docs/SOLIDITY_SUPPORT_MATRIX.md` for current feature status
- ✅ Variable handling with proper index-based storage
- ✅ Loop control (break/continue) with context tracking
- ✅ Function overloading support with Neo ABI name mangling for same-arity overloads

#### **Runtime Library**

- ✅ EVM-compatible memory manager
- ✅ Storage manager with Solidity layout compatibility
- ✅ ABI encoder/decoder for basic types
- ✅ Cryptographic library (keccak256, ecrecover, sha256)
- ✅ Event system with Runtime.Notify integration
- ✅ Context objects (msg, tx, block) with Neo mapping
- ✅ External call manager (`call`/`staticcall` mappings; `delegatecall` is rejected)
- ✅ Exception handling (try/catch with runtime guards)
- ✅ Iterator handles for `Iterator.Next` and `Iterator.Value`
- ✅ Per-syscall gas accounting with approximate Neo N3 costs
- ✅ Broad documented opcode subset; unsupported opcodes are rejected explicitly
- ✅ Oracle native-contract routing with deterministic request IDs and local price state; live callbacks require Neo-Express/TestNet validation

#### **Testing**

- ✅ Unit tests for runtime primitives and compiler helpers
- ✅ Integration tests for compiler pipeline behavior
- ✅ E2E compilation tests for all examples (80 tests)
- ✅ Conformance test vectors (40 vectors, minimum 95.0% pass gate)
- ✅ Neo-Express deployment smoke tests
- ✅ Cross-platform CI/CD (Linux, macOS, Windows)
- ✅ End-to-end contract execution tests
- ✅ Fuzzing framework (property-based testing)
- ✅ Reference-crate differential tests for supported crypto, arithmetic, and disassembly paths
- 🔄 Broader EVM-vs-NeoVM differential testing (planned)

#### **Developer Tools**

- ✅ CLI tools (neo-solc) - fully functional
- ✅ Hardhat integration (@neo-devpack-solidity/hardhat-solc-neo)
- ✅ Hardhat deployer (@neo-devpack-solidity/hardhat-neo-deployer)
- ✅ Foundry adapter (@neo-devpack-solidity/neo-foundry)
- ✅ ABI router (@neo-devpack-solidity/abi-router)
- ✅ Shared types (@neo-devpack-solidity/types)
- ✅ CLI tools package (@neo-devpack-solidity/cli-tools)
- ✅ Debug/source-map type support for downstream tooling
- ✅ Network configurations for Neo TestNet/MainNet
- ✅ Artifact management
- 🔄 Standalone CLI source-map/debug artifact emission and interactive debugger integration (planned)

#### **Documentation**

- ✅ Comprehensive README with examples
- ✅ Architecture documentation (docs/ARCHITECTURE.md)
- ✅ Runtime specification (docs/RUNTIME_SPEC.md)
- ✅ NeoVM parity TODO list (docs/NEO_VM_PARITY_TODO.md)
- ✅ Solidity support matrix (docs/SOLIDITY_SUPPORT_MATRIX.md)
- ✅ Error reference (docs/ERROR_REFERENCE.md)
- ✅ Security best practices
- 🔄 Video tutorials and workshops (planned)

### **📈 Metrics & Statistics**

- **📊 Total Lines of Code**: ~50,000 (Rust implementation)
- **🧪 Test Coverage**: Layered Rust, fuzz, E2E, conformance, and Neo-Express validation
- **⚡ Performance**: Optimized code generation with multi-level optimization
- **🔒 Security**: Basic security analysis; external audit recommended for production
- **📚 Documentation**: Comprehensive guides and reference documentation
- **🛠️ Compatibility**: Solidity 0.5.x–0.8.x on NeoVM 3.0+; see `docs/SOLIDITY_SUPPORT_MATRIX.md` for the current feature audit

### **🎯 Production Readiness**

| Component           | Status              | Test Coverage                 | Documentation | Notes                       |
| ------------------- | ------------------- | ----------------------------- | ------------- | --------------------------- |
| **Compiler Core**   | 🟢 Production-focused | Unit + integration suites     | Complete      | Validate target contracts with the production gate and Neo-Express/TestNet |
| **Runtime Library** | 🟢 Production-focused | Runtime and property suites   | Complete      | Deterministic embedded runtime; validate final behavior on Neo-Express/TestNet |
| **Developer Tools** | 🟢 Stable           | Smoke Tests                   | Good          | CLI fully functional        |
| **Testing Suite**   | 🟢 Comprehensive    | Rust + fuzz + E2E + conformance | Good        | 40-vector conformance gate requires at least 95.0% |
| **Documentation**   | 🟢 Good             | Docs structure check          | Good          | Comprehensive guides        |

### **⚠️ Known Limitations**

The compiler is intended for production-oriented use, but please note:

| Area                     | Status  | Notes                                                                           |
| ------------------------ | ------- | ------------------------------------------------------------------------------- |
| **Oracle Integration**   | Partial | Embedded runtime records requests and price state, but does not contact oracle nodes or deliver live callbacks |
| **Fuzzing Framework**    | ✅ Done | Property-based tests plus 11 cargo-fuzz targets                                 |
| **Differential Testing** | Partial | Reference-crate differential tests exist; broader EVM-vs-NeoVM differential testing remains planned |
| **IDE Debugging**        | Planned | Interactive debugging tools not yet implemented                                 |

Note on intrinsic devpack libraries (`Runtime`, `Storage`, `Syscalls`, `NativeCalls`, `Neo`, `abi`):
they are compiler intrinsics. Their Solidity source may include overloaded/internal helper signatures
for tooling ergonomics; the compiler lowers supported members directly to Neo syscalls/native calls.

**Recommendation:** For MainNet deployment, thoroughly test your contracts on Neo N3 TestNet first.

### **🚀 Roadmap**

#### **Phase 1: Core Stability (Q1 2024)** ✅

- ✅ Complete compiler implementation
- ✅ Runtime library with EVM compatibility
- ✅ Basic tooling and CLI interface
- ✅ Comprehensive testing framework

#### **Phase 2: Developer Experience (Q2 2024)** ✅

- ✅ Hardhat and Foundry integration
- ✅ Source-map/debug internals for downstream tooling
- ✅ Performance optimization
- ✅ Security analysis features

#### **Phase 3: Production Deployment (Q3 2024)** ✅

- ✅ Audit-ready codebase
- ✅ Performance benchmarking
- ✅ Community testing and feedback
- ✅ MainNet deployment support

#### **Phase 4: Ecosystem Growth (2025-2026)** 🔄

- 🔄 Additional language support (Vyper)
- 🔄 Advanced optimization passes
- 🔄 IDE integrations (VS Code, IntelliJ)
- 🔄 Educational resources and workshops
- 📋 Formal verification tools
- 📋 Multi-chain support

## 🏆 **Examples Gallery**

### **Real-World Contracts**

We've included production-oriented implementations of popular contract patterns:

#### **🪙 [ERC20 Token](./examples/ERC20Token.sol)** (420 lines)

- Complete standard implementation
- Advanced features: minting, burning, pausing
- Owner management and emergency functions
- Batch operations and token recovery
- Comprehensive event logging

#### **🎨 [ERC721 NFT](./examples/ERC721Token.sol)** (850 lines)

- Note: this example includes EVM-specific patterns (inline assembly + `.selector`) and is not currently supported end-to-end; prefer `examples/new/NFT.sol` or `devpack/examples/CompleteNEP11NFT.sol` for Neo N3.
- Full NFT implementation with metadata
- Enumerable extension for token discovery
- Royalty support (EIP-2981)
- Batch minting and advanced features
- Gas-optimized storage patterns

#### **🏦 [Uniswap V2 Pair](./examples/UniswapV2Pair.sol)** (650 lines)

- Complete AMM implementation
- Liquidity provision and swapping
- Price oracle functionality
- Fee collection and governance
- Advanced mathematical operations

#### **🔐 [MultiSig Wallet](./examples/MultiSigWallet.sol)** (720 lines)

- Neo-adapted: uses native GAS (NEP-17) transfers and accepts deposits via `onNEP17Payment`.
- Smaller Neo-native example: `examples/new/MultiSigWalletNEP17.sol`.
- Multi-signature transaction approval
- Owner management and daily limits
- Emergency stop functionality
- Batch operations support
- Comprehensive security features

#### **🗳️ [Governance Token](./examples/GovernanceToken.sol)** (980 lines)

- Neo-adapted: proposals cannot attach native value (`values[]` must be `0`); use NEP-17 transfers and a Neo-compatible timelock contract instead.
- ERC20 with voting capabilities
- Delegation and vote tracking
- Proposal creation and execution
- Timelock integration
- Advanced governance features

#### **💾 [Simple Storage](./examples/SimpleStorage.sol)** (170 lines)

- Basic storage read/write operations
- Key-value mapping storage
- Owner access control
- Increment/decrement functions
- Ideal for learning NeoVM storage

#### **🔒 [Escrow](./examples/Escrow.sol)** (280 lines)

- Secure fund escrow service
- Time-locked releases
- Multi-party dispute resolution
- Arbiter-based conflict handling
- Fee collection system

#### **🎰 [Lottery](./examples/Lottery.sol)** (320 lines)

- Multi-round lottery system
- Ticket purchase and tracking
- Pseudo-random winner selection
- Prize pool management
- Operator fee collection

#### **📈 [Staking](./examples/Staking.sol)** (310 lines)

- Token staking with rewards
- Configurable lock periods
- APY calculation
- Emergency withdraw function
- Reward distribution tracking

#### **🏷️ [Name Service](./examples/NameService.sol)** (350 lines)

- Decentralized name registration
- Address resolution
- Text record storage
- Name transfer and renewal
- Similar to ENS for Neo N3

#### **🏛️ Famous DeFi/Web3 Contracts** (`examples/famous/`)

Ports of iconic Ethereum DeFi protocols adapted for Neo N3:

- **[WGAS](./examples/famous/WGAS.sol)** — Wrapped GAS (WETH9-style NEP-17 wrapper)
- **[FlashLoan](./examples/famous/FlashLoan.sol)** — Aave V2-style flash loan pool
- **[SimpleAMM](./examples/famous/SimpleAMM.sol)** — Uniswap V2-style constant-product AMM
- **[TokenVesting](./examples/famous/TokenVesting.sol)** — OpenZeppelin-style linear vesting with cliff
- **[SimpleLending](./examples/famous/SimpleLending.sol)** — Compound-style lending with liquidation
- **[SimpleDAO](./examples/famous/SimpleDAO.sol)** — Governor-style DAO with staking and timelock

See [`examples/famous/README.md`](./examples/famous/README.md) for full details and Neo N3 adaptation notes.

### **Usage Examples**

```bash
# Compile ERC20 token
neo-solc examples/ERC20Token.sol -O3 -o build/ERC20Token

# Deploy to Neo TestNet
neo-cli contract deploy build/ERC20Token.nef build/ERC20Token.manifest.json

# Verify deployment
neo-cli contract invoke <hash> balanceOf [<address>]

# Run the ERC20-style integration coverage
cargo test test_erc20_like_contract
```

## 🆘 **Support & Community**

### **Getting Help**

- **📖 Documentation**: Complete guides and API reference
- **💬 Discord**: Join our [Discord server](https://discord.gg/r3e-network)
- **🐛 Issues**: Report bugs on [GitHub Issues](https://github.com/r3e-network/neo-devpack-solidity/issues)
- **📧 Email**: Technical support at jimmy@r3e.network

### **Community Resources**

- **🎥 Video Tutorials**: [YouTube Channel](https://youtube.com/r3e-network)
- **📝 Blog Posts**: [Development Blog](https://r3e.network/blog)
- **🎓 Workshops**: Monthly community workshops
- **📱 Twitter**: [@R3ENetwork](https://twitter.com/r3enetwork) for updates

### **Contributing**

We welcome contributions from the community! Check out our:

- **👥 [Contributing Guide](./CONTRIBUTING.md)**
- **🎯 [Good First Issues](https://github.com/r3e-network/neo-devpack-solidity/labels/good%20first%20issue)**
- **🏗️ [Testing and Local Validation](./TESTING.md)**
- **🔐 [Security Policy](./SECURITY.md)**

## 📄 **License**

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 **Acknowledgments**

- **Neo Global Development Team** for blockchain infrastructure
- **Ethereum Foundation** for Solidity language specification
- **Rust Community** for excellent tooling and libraries
- **Open Source Contributors** who made this project possible

---

<div align="center">

**Built with ❤️ by R3E Network**

[Website](https://r3e.network) • [Documentation](https://docs.r3e.network) • [Discord](https://discord.gg/r3e-network) • [Twitter](https://twitter.com/r3enetwork)

_Bringing Ethereum's developer ecosystem to Neo blockchain_

</div>
