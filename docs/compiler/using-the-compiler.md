# Using the Compiler

## Using the Commandline Compiler

One of the build targets of the Neo DevPack for Solidity repository is `neo-solc`, the commandline compiler. Using `neo-solc --help` provides you with an explanation of all options. The compiler can produce various outputs, ranging from simple binaries and JSON manifests over assembly to standard JSON configurations.

### Command Syntax

```bash
neo-solc <source...> [options]
```

The compiler accepts one or more Solidity source files and produces Neo N3 deployment artifacts (`.nef` + `.manifest.json`). Standard JSON mode uses the Solidity-compatible JSON interface and can read from stdin or from `--input`.

```bash
# Single file
neo-solc MyContract.sol -o build/

# Multiple files
neo-solc contracts/*.sol -o build/

# With devpack imports
neo-solc MyToken.sol -I devpack -o build/

# Standard JSON from a file
neo-solc --standard-json --input input.json --output output.json
```

### Complete Flag Summary

| Flag                          | Short | Argument   | Default    | Description                                                                          |
| ----------------------------- | ----- | ---------- | ---------- | ------------------------------------------------------------------------------------ |
| `<source...>`                 |       | positional | required   | Input Solidity file(s); omitted when `--standard-json` is used                       |
| `--standard-json`             |       |            |            | Enable Solidity Standard JSON input/output mode                                      |
| `--input`                     |       | `FILE`     | stdin      | Standard JSON input file; requires `--standard-json`                                 |
| `--output`                    | `-o`  | `FILE`     | source stem | Output prefix for single-file mode; directory/prefix handling for multi-file mode; JSON output file for Standard JSON |
| `--optimize`                  | `-O`  | `LEVEL`    | `2`        | Optimization level (0-3)                                                             |
| `--format`                    | `-f`  | `FORMAT`   | `complete` | Output format: `nef`, `manifest`, `complete`, `assembly`, or `json`                  |
| `--include-path`              | `-I`  | `DIR`      |            | Import search path (repeatable)                                                      |
| `--contract`                  |       | `NAME`     |            | Emit only named contract (repeatable)                                                |
| `--verbose`                   | `-v`  |            |            | Enable verbose output                                                                |
| `--callt`                     |       |            |            | Emit CALLT + method tokens for native calls                                          |
| `--analyze`                   |       |            |            | Print a JSON upgrade/readiness analysis instead of compiling artifacts               |
| `--nef-source`                |       | `STRING`   |            | Override the source string embedded in generated NEF metadata                        |
| `--deployer`                  |       | `HASH160`  |            | Deployer account hash used for deterministic contract hash calculation               |
| `--json-errors`               |       |            |            | Emit diagnostics as JSON errors                                                      |
| `--json-warnings`             |       |            |            | Emit diagnostics as JSON warnings                                                    |
| `--Wno`                       |       | `CODE`     |            | Disable a warning code (repeatable)                                                  |
| `--Werror`                    |       | `CODE`     |            | Treat a warning code as an error (repeatable)                                        |
| `--deny-wildcard-permissions` |       |            |            | Fail if manifest needs full wildcard permission                                      |
| `--deny-wildcard-contracts`   |       |            |            | Fail if manifest needs wildcard contract permission                                  |
| `--deny-wildcard-methods`     |       |            |            | Fail if manifest needs wildcard method permission                                    |
| `--manifest-permissions`      |       | `FILE`     |            | JSON file with manifest permissions                                                  |
| `--manifest-permissions-mode` |       | `MODE`     | `merge`    | How to apply permission overrides: `merge` or `replace-wildcards`                    |

### Base Path and Include Paths

The compiler resolves `import` directives by looking into the current directory and any directories provided via `--include-path` (or `-I`).

```bash
neo-solc contract.sol -I devpack -I ./lib -o build/
```

## Setting the EVM Version to Target

::: tip 💡 NeoVM Difference
Neo DevPack for Solidity targets Neo N3 (`NeoVM`) exclusively. Standard EVM version targets (like `istanbul`, `paris`, `cancun`) do not apply and are safely ignored by the compiler if provided through standard JSON configurations.
:::

Because Neo N3 relies on standard execution mechanics and syscall interfaces, versioning is primarily handled at the node runtime level rather than requiring specific hardfork flags during compilation.

## Compiler Input and Output JSON Description

These JSON formats are used by the compiler API as well as the standard JSON interface of the commandline compiler. These interfaces are recommended for any programmatic interactions (like IDEs and framework toolchains).

To use the standard JSON interface from the command line, run:
```bash
neo-solc --standard-json --input input.json > output.json
```

### Input JSON Description

The input JSON format is completely compatible with the Ethereum Solidity Standard JSON format.

```json
{
  "language": "Solidity",
  "sources": {
    "MyContract.sol": {
      "content": "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\n..."
    }
  },
  "settings": {
    "optimizer": {
      "enabled": true,
      "level": 2
    },
    "outputSelection": {
      "*": {
        "*": ["abi", "nef", "manifest"]
      }
    }
  }
}
```

### Output JSON Description

The output JSON includes any compiler diagnostics (errors/warnings) and the compiled artifacts.

::: tip 💡 NeoVM Difference
In EVM output, contracts produce `evm.bytecode` and `abi` fields. In Neo DevPack for Solidity, the output produces a Base64-encoded `nef` string and the structural Neo `manifest`.
:::

```json
{
  "contracts": {
    "MyContract.sol": {
      "MyContract": {
        "abi": { 
            "methods": [ ... ],
            "events": [ ... ]
        },
        "nef": "<base64-encoded NEF>",
        "manifest": { ... }
      }
    }
  },
  "errors": [
    {
      "component": "neo-devpack-solidity",
      "severity": "warning",
      "code": "COMPILER_WARNING",
      "message": "...",
      "formattedMessage": "...",
      "location": { "file": "MyContract.sol", "line": 10, "column": 1 }
    }
  ]
}
```
