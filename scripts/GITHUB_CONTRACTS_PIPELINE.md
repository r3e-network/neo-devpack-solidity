# GitHub Contracts → Neo N3 Pipeline

A unified tool to collect Solidity contracts from GitHub, compile them with `neo-solc`, deploy to Neo-Express, and run runtime assertions.

## Prerequisites

- `neo-solc` (will be auto-built from source if missing)
- `neoxp` (will be auto-installed as dotnet tool if missing)
- `git`, `node` (20.19+ or 22.12+), `npm`

## Quick Start

```bash
# 1. Collect contracts from a GitHub repo
node scripts/github_contracts_pipeline.js collect \
  --repo OpenZeppelin/openzeppelin-contracts \
  --branch master \
  --path contracts/token/ERC20 \
  --out ./my-contracts

# 2. Compile them
node scripts/github_contracts_pipeline.js compile \
  --in ./my-contracts \
  --out ./my-build

# 3. Deploy to a fresh Neo-Express chain
node scripts/github_contracts_pipeline.js deploy \
  --in ./my-build \
  --chain ./my-chain.neo-express

# 4. Run tests (see Test Manifest format below)
node scripts/github_contracts_pipeline.js test \
  --chain ./my-chain.neo-express \
  --manifest ./my-tests.json

# Or run the full pipeline end-to-end
node scripts/github_contracts_pipeline.js pipeline \
  --repo OpenZeppelin/openzeppelin-contracts \
  --out ./pipeline-result

# CI gate: fail if any compile/deploy/test item fails
node scripts/github_contracts_pipeline.js pipeline \
  --config ./pipeline-config.json \
  --fail-on-partial
```

## Modes

### `collect` — Download Contracts

**From GitHub:**
```bash
node scripts/github_contracts_pipeline.js collect \
  --repo owner/repo \
  --branch main \
  --path contracts/ \
  --out ./contracts
```

**From NPM:**
```bash
node scripts/github_contracts_pipeline.js collect \
  --npm @openzeppelin/contracts@5.4.0 \
  --path contracts/token/ERC20 \
  --out ./contracts
```

### `compile` — Compile with neo-solc

```bash
node scripts/github_contracts_pipeline.js compile \
  --in ./contracts \
  --out ./build \
  --import ./devpack \
  --Wno W200 \
  --Wno W121
```

Options:
- `--in <dir>` — Directory containing `.sol` files
- `--out <dir>` — Output directory for `.nef` + `.manifest.json`
- `--import <dir>` — Additional Solidity import path (repeatable)
- `--contract <name>` — Compile only specific contracts (repeatable)
- `--Wno <code>` — Suppress warning code (repeatable)
- `--O0`, `--O1`, `--O2`, `--O3` — Optimization level
- `--fail-on-partial` — Exit non-zero if any target fails to compile

### `deploy` — Deploy to Neo-Express

```bash
node scripts/github_contracts_pipeline.js deploy \
  --in ./build \
  --chain ./chain.neo-express \
  --account node1 \
  --data '["arg1","arg2"]' \
  --clear-standards
```

The script automatically:
1. Creates a new Neo-Express chain if one doesn't exist
2. Transfers GAS from `genesis` to the deployer account if balance is low
3. Deploys each compiled contract
4. Verifies deployment succeeded (HALT vmstate)

Options:
- `--in <dir>` — Directory with compiled `.nef`/`.manifest.json`
- `--chain <file>` — Neo-Express chain file path
- `--account <name>` — Deployer account (default: `node1`)
- `--data <json>` — Constructor args as JSON array
- `--clear-standards` — Clear `supportedstandards` in manifest before deploy
- `--fail-on-partial` — Exit non-zero if any compiled contract fails to deploy

### `test` — Run Runtime Assertions

```bash
node scripts/github_contracts_pipeline.js test \
  --chain ./chain.neo-express \
  --manifest ./tests.json \
  --account node1 \
  --fail-on-partial
```

### `pipeline` — End-to-End

```bash
node scripts/github_contracts_pipeline.js pipeline \
  --config ./pipeline-config.json \
  --fail-on-partial
```

By default the tool writes reports and exits zero after partial compile/deploy/test
failures so it can be used for exploratory compatibility audits. For CI and
release gates, pass `--fail-on-partial`, set `NEO_PIPELINE_FAIL_ON_PARTIAL=1`,
or set `"failOnPartial": true` in the config.

## Test Manifest Format

```json
{
  "variables": {
    "GENESIS_HASH": "0x26bade52b48383031255b1d8f4004feaf5175443",
    "NODE1_HASH": "0x495b9cb2a27f536be389e3e07253b4b513f7b9de"
  },
  "cases": [
    {
      "contract": "MyContract",
      "contractHash": "0x...",
      "assertions": [
        {
          "name": "get() returns 0",
          "kind": "read",
          "operation": "get",
          "args": [],
          "expect": { "type": "Integer", "value": 0 }
        },
        {
          "name": "set(42)",
          "kind": "write",
          "operation": "set",
          "args": [42],
          "gas": 10000000
        }
      ]
    }
  ]
}
```

- `kind: "read"` — Read-only invocation (`-r` flag)
- `kind: "write"` — Transaction invocation (sends actual tx)
- Variables prefixed with `$` in args are resolved from the `variables` map

## Pipeline Config Format

```json
{
  "collect": {
    "repo": "OpenZeppelin/openzeppelin-contracts",
    "branch": "master",
    "path": "contracts/token/ERC20"
  },
  "compile": {
    "Wno": ["W200", "W121"]
  },
  "deploy": {
    "account": "node1",
    "clear-standards": true
  },
  "failOnPartial": true,
  "test": {
    "manifest": "./tests.json"
  }
}
```

## Known Limitations

### Write Operations on Some Contracts

Contracts that modify storage (state variables, mappings) or depend on EVM ABI
byte-for-byte behavior may fail at runtime with `InvalidCastException` or
similar errors. This is due to known compiler gaps (see
`docs/internals/parity-and-limitations.md`):

- **P0**: `abi.encode` with dynamic args, `abi.decode`, `abi.encodePacked` — these paths are only partially compatible and must be validated on real Neo N3/Neo-Express
- Storage serialization for certain type combinations may fail

**Workaround**: Test read-only (`pure`/`view`) functions first. For storage-heavy contracts, use the existing `scripts/famous_contracts_neoxp_runtime.js` which tests contracts known to work.

### Type Compatibility

- `uint256`/`int256` arithmetic: Some operations may fault due to BigInteger casting issues
- `string` storage: Works in embedded runtime but verify on Neo-Express
- `bytes` dynamic arrays: Use with caution

## Examples

### Test a Simple Counter Contract

```bash
# Create a simple contract
cat > /tmp/Counter.sol << 'EOF'
pragma solidity ^0.8.19;
contract Counter {
    uint256 private count;
    function get() public view returns (uint256) { return count; }
}
EOF

# Compile
node scripts/github_contracts_pipeline.js compile \
  --in /tmp --contract Counter --out /tmp/counter-build

# Deploy
node scripts/github_contracts_pipeline.js deploy \
  --in /tmp/counter-build --chain /tmp/counter-chain.neo-express

# Test
cat > /tmp/counter-test.json << 'EOF'
{
  "cases": [{
    "contract": "Counter",
    "contractHash": "<hash-from-deploy-report>",
    "assertions": [
      { "name": "get() == 0", "kind": "read", "operation": "get", "args": [],
        "expect": { "type": "Integer", "value": 0 } }
    ]
  }]
}
EOF
node scripts/github_contracts_pipeline.js test \
  --chain /tmp/counter-chain.neo-express --manifest /tmp/counter-test.json
```

### Run Existing Famous Contracts Suite

The project already includes scripts for famous contracts:

```bash
# Vendor dependencies
node scripts/vendor_famous_contracts.js

# Audit compilation
node scripts/famous_contracts_audit.js

# Deploy to neoxp
node scripts/famous_contracts_neoxp_deploy.js

# Runtime tests
node scripts/famous_contracts_neoxp_runtime.js
```

## Troubleshooting

### "Insufficient GAS" on deploy

The script auto-transfers GAS from `genesis` to the deployer account. If this fails:
- Ensure the chain file is not corrupted
- Try deleting the chain and re-creating it

### "Called Contract Does Not Exist"

- Ensure all neoxp commands use the same chain file
- Check `HOME` env var — neoxp stores blockchain data in `$HOME/.neo-express/`
- Use `neoxp contract list -i chain.neo-express` to verify deployment

### Contract deployment succeeds but write operations fail

This is likely a compiler/runtime compatibility issue. Check:
- `docs/internals/parity-and-limitations.md` for known gaps
- Try simplifying the contract (remove complex types, use `pure`/`view` functions)
- Test with the existing `examples/` contracts first
