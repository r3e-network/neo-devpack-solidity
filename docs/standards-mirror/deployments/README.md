# Standards Mirror — TestNet Deployment Kit

This directory contains paired Solidity + Neo C# implementations of standards from
the ERC ↔ Neo mirror, plus an automation script that:

1. Compiles the Solidity sources with `neo-solc` (built from this repo) and the C#
   sources with `nccs` (Neo.Compiler.CSharp v3.9+).
2. Deploys the produced NEFs to Neo N3 TestNet.
3. Runs an invocation matrix (read + write) against each pair and asserts the
   Solidity and C# implementations behave equivalently.
4. Writes structured results to `results.json` and a Markdown summary to
   `RESULTS.md`.

## Quick Start

```bash
# Build the Solidity compiler if not already built:
cargo build --release --bin neo-solc

# Install nccs if needed:
dotnet tool install -g neo.compiler.csharp

# Provide a funded TestNet wallet WIF and run:
NEO_TESTNET_WIF=<your-wif> node scripts/standards_mirror_testnet.js
```

The runner is idempotent — already-deployed contracts are detected via the
"Contract Already Exists" error and reused for subsequent assertion runs.

## Structure

```
deployments/
├── manifest.json          ← list of pairs + per-pair test cases
├── results.json           ← machine-readable last-run output
├── RESULTS.md             ← human summary
├── erc-20/
│   ├── solidity/DemoToken.sol         (compiles via neo-solc)
│   └── csharp/DemoToken.{cs,csproj}   (compiles via nccs)
├── erc-721/
│   ├── solidity/DemoNFT.sol
│   └── csharp/DemoNFT.{cs,csproj}
├── erc-2981/
│   ├── solidity/RoyaltyNFT.sol
│   └── csharp/RoyaltyNFT.{cs,csproj}
└── erc-3525/
    ├── solidity/Bond.sol
    └── csharp/Bond.{cs,csproj}
```

## Adding More Pairs

1. Create a new `<standard-id>/solidity/Contract.sol` and `<standard-id>/csharp/Contract.{cs,csproj}`.
2. Append a pair entry to `manifest.json` with the test invocation matrix.
3. Re-run the script. New deploys will pick up; existing ones are reused.

## Compatibility Notes

A few neo-solc 0.18 quirks surfaced during this work; we document them so future
contract authors avoid them:

- **`emit Event(...)` faults during deploy.** When the event signature's keccak256
  starts with byte `0xDD` (e.g. `Transfer(address,address,uint256)`), the runtime
  trips a UTF-8 decoder error. Workaround: avoid `emit` in `_deploy`-reached code
  paths. The C# implementations don't have this issue.
- **`uint256(uint8 var)` cast in constructor faults at deploy.** Use a literal
  pre-computed constant instead.
- **`msg.sender` at constructor time is the ManagementContract**, not the
  deploying user. Solidity owner-init patterns like `deployer = msg.sender` won't
  work at deploy. Use `Runtime.checkWitness(...)` from the devpack post-deploy
  instead.
