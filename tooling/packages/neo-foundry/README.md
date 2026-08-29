# @neo-devpack-solidity/neo-foundry

Neo Foundry is the companion CLI that mirrors Foundry/Hardhat ergonomics for the Neo chain. `neo-forge init` creates a working project layout and config, `neo-forge test` delegates to the native `neo-test` runner (see `docs/TESTING.md` in the repository root), and the package can be imported as a library without executing the CLI parser as a side effect. The remaining flows are still scaffold-level.

## Current State (v0.27.0)

| Command | Status |
| --- | --- |
| `neo-forge init` | ✅ Creates `neo-foundry.toml`, `src/Counter.sol`, and `test/Counter.t.sol` |
| `neo-forge test` | ✅ Delegates to the native `neo-test` runner (build it with `cargo build --release --bin neo-test`, or point `NEO_TEST` at the binary); supports `--match-test`, `--gas-report`, `-v`; `--coverage`/`--fork-*` are ignored with a warning |
| `neo-forge clean` | ✅ Removes the profile’s `out` and cache directories |
| `neo-forge build` | ⚠️ Scaffold-only; stops with an explicit “not implemented yet” error (compile with `neo-solc` or the Hardhat plugin) |
| `neo-cast` (`call`, `send`, `balance`, etc.) | ⚠️ Reads configs and validates inputs but does not submit real transactions |
| `neo-anvil` | ⚠️ In-memory mock chain with a JSON-RPC façade; useful for CLI demos, not for executing NEF code |
| `neo-foundry` programmatic API (`NeoForge`, `NeoCast`, `NeoAnvil`) | ⚠️ `NeoForge.test`/`clean`/`init` work; the remaining members are typed stubs raising “not implemented” errors when called |

## Installation

```bash
npm install -g @neo-devpack-solidity/neo-foundry
```

The CLI exposes three entry points:

- `neo-forge` – project/build/test workflows (`init` and `test` implemented — `test` runs via the native `neo-test` runner; `build` still placeholder)
- `neo-cast` – contract interaction helper (placeholder)
- `neo-anvil` – mock RPC server for local development demos

## Configuration

The package understands a Foundry-like `neo-foundry.toml` via `ConfigManager`, and `neo-forge init` writes that file plus a starter project skeleton. Future work will connect the same config to real compiler runs, RPC signing, and artifact resolution.

## Roadmap

1. Wire `neo-forge build` to the Neo DevPack for Solidity compiler pipeline and persist artifacts.
2. ~~Implement `neo-forge test` by driving the Neo VM test harness.~~ Done — `neo-forge test` delegates to the native `neo-test` runner.
3. Turn `neo-cast send/deploy` into real RPC transactions using the shared `NeoRpcClient`.
4. Replace `neo-anvil` with an actual Neo VM sandbox (or connect to Neo Express).

Until the rest of the roadmap ships, expect the remaining commands to terminate with an explicit “feature not implemented” message. This documentation keeps expectations clear for anyone installing the package from npm.
