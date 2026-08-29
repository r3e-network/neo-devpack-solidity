# @neo-devpack-solidity/cli-tools

CLI utility scaffolding for the Neo DevPack for Solidity toolchain. The package ships lightweight Node CLIs (`solc-neo`, `neo-sol`) that shell out to the Rust `neo-solc` binary, plus a reusable CLI framework. The compile-oriented flows are still scaffold-level, but the framework itself is usable and returns structured results instead of terminating the process on command failures.

## Status

| Command / API | Status |
| --- | --- |
| `solc-neo` / `neo-sol` compile (`CompilerCLI`) | ⚠️ Constructs JSON input and shells out to `neo-solc`, but the surrounding task runner (artifact persistence, plugin integration) is still under construction |
| CLI framework (`NeoDevpackSolidityCLI`) | ✅ Usable to register commands/log/spinners and returns structured success/failure results |
| Utility helpers in `src/cli-framework.ts` | ✅ Logging/spinner/progress abstractions |
| Higher-level forge/cast/anvil flows | ⚠️ Live in `@neo-devpack-solidity/neo-foundry`, not this package. There, `neo-forge test` delegates to the native `neo-test` runner; the remaining flows still print placeholder output and throw “not implemented yet”. |

## Installation

```bash
npm install -g @neo-devpack-solidity/cli-tools
```

## Usage

```bash
# Compile contracts (scaffold-level wrapper around `neo-solc`)
solc-neo compile contracts/**/*.sol --optimize --gas-model hybrid
# or
neo-sol compile contracts/**/*.sol --optimize --gas-model hybrid

# Show version (delegates to the discovered neo-solc binary)
solc-neo --version
```

The `CompilerCLI` module is also exportable so tool authors can integrate it programmatically:

```ts
import { CompilerCLI } from "@neo-devpack-solidity/cli-tools";

const cli = new CompilerCLI();
await cli.compile(["contracts/MyToken.sol"], { optimize: true });
```

## Roadmap

1. Wire compiler output into the artifact format consumed by `@neo-devpack-solidity/hardhat-solc-neo` and Neo Foundry.
2. Move `neo-forge`/`neo-cast` commands into this package once they perform real RPC transactions.
3. Provide a `neo-anvil` CLI that launches a real Neo Express instance or VM sandbox.

Until the higher-level flows land, consider the compiler wrapper experimental. The reusable CLI framework is already suitable for local scripting and package composition.
