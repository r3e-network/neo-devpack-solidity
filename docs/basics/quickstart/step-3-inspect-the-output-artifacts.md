---
title: "Quickstart: Step 3: Inspect the Output Artifacts"
description: "Step 3: Inspect the Output Artifacts from Quickstart."
---

# Step 3: Inspect the Output Artifacts

[Back to Quickstart](/basics/quickstart)

## NEF File

The `.nef` file is a binary format. You can verify it was created and check its size:

```bash
ls -la build/MyStorage.nef
```

The NEF (Neo Executable Format) contains:

- **Magic bytes** (`0x3346454E`) -- identifies the file as a NEF
- **Compiler metadata** -- compiler name and version
- **Method tokens** -- references to native contract methods (when using `--callt`)
- **Script** -- the NeoVM bytecode
- **Checksum** -- SHA-256 hash for integrity verification

## Manifest File

The `.manifest.json` file is human-readable. Inspect it with `jq`:

```bash
jq '.' build/MyStorage.manifest.json
```

Example output (abbreviated):

```json
{
    "name": "MyStorage",
    "groups": [],
    "features": {},
    "supportedstandards": [],
    "abi": {
        "methods": [
            {
                "name": "set",
                "parameters": [{ "name": "newValue", "type": "Integer" }],
                "returntype": "Void",
                "offset": 0,
                "safe": false
            },
            {
                "name": "get",
                "parameters": [],
                "returntype": "Integer",
                "offset": 42,
                "safe": true
            },
            {
                "name": "increment",
                "parameters": [],
                "returntype": "Integer",
                "offset": 56,
                "safe": false
            }
        ],
        "events": [
            {
                "name": "ValueChanged",
                "parameters": [
                    { "name": "oldValue", "type": "Integer" },
                    { "name": "newValue", "type": "Integer" },
                    { "name": "setter", "type": "Hash160" }
                ]
            }
        ]
    },
    "permissions": [
        {
            "contract": "*",
            "methods": "*"
        }
    ],
    "trusts": [],
    "extra": null
}
```

Key fields to understand:

| Field                | Description                                                                                                       |
| -------------------- | ----------------------------------------------------------------------------------------------------------------- |
| `name`               | Contract name, derived from the Solidity contract name                                                            |
| `abi.methods`        | Each public/external function becomes a method entry. `safe: true` means the method is read-only (`view`/`pure`). |
| `abi.events`         | Each Solidity `event` becomes an event entry with typed parameters.                                               |
| `permissions`        | Declares which other contracts and methods this contract may call. Wildcards (`*`) mean unrestricted.             |
| `supportedstandards` | Lists NEP standards the contract implements (e.g., `["NEP-17"]`).                                                 |

::: tip Inspect Specific Fields
Quick commands to extract common fields:

```bash
# Contract name
jq '.name' build/MyStorage.manifest.json

# Number of methods
jq '.abi.methods | length' build/MyStorage.manifest.json

# Method names
jq '[.abi.methods[].name]' build/MyStorage.manifest.json

# Event names
jq '[.abi.events[].name]' build/MyStorage.manifest.json

# Permissions
jq '.permissions' build/MyStorage.manifest.json
```

:::

## NeoVM Assembly (Optional)

To see the generated NeoVM opcodes:

```bash
./target/release/neo-solc MyStorage.sol -I devpack -O2 -f assembly -o build/MyStorage.asm
cat build/MyStorage.asm
```

This produces a human-readable disassembly showing each NeoVM instruction.
