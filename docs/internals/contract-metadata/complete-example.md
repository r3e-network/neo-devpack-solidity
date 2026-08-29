---
title: "Manifest Specification: Complete Example"
description: "Complete Example from Manifest Specification."
---

# Complete Example

[Back to Manifest Specification](/internals/contract-metadata)

Below is the full `manifest.json` output for the `GoldToken` contract above:

## Manifest JSON

```json
{
    "name": "GoldToken",
    "groups": [],
    "features": {},
    "supportedstandards": ["NEP-17"],
    "abi": {
        "methods": [
            {
                "name": "symbol",
                "parameters": [],
                "returntype": "String",
                "offset": 0,
                "safe": true
            },
            {
                "name": "decimals",
                "parameters": [],
                "returntype": "Integer",
                "offset": 0,
                "safe": true
            },
            {
                "name": "totalSupply",
                "parameters": [],
                "returntype": "Integer",
                "offset": 0,
                "safe": true
            },
            {
                "name": "balanceOf",
                "parameters": [{ "name": "account", "type": "Hash160" }],
                "returntype": "Integer",
                "offset": 0,
                "safe": true
            },
            {
                "name": "transfer",
                "parameters": [
                    { "name": "from", "type": "Hash160" },
                    { "name": "to", "type": "Hash160" },
                    { "name": "amount", "type": "Integer" },
                    { "name": "data", "type": "Any" }
                ],
                "returntype": "Boolean",
                "offset": 0,
                "safe": false
            }
        ],
        "events": [
            {
                "name": "Transfer",
                "parameters": [
                    { "name": "from", "type": "Hash160" },
                    { "name": "to", "type": "Hash160" },
                    { "name": "amount", "type": "Integer" }
                ]
            }
        ]
    },
    "permissions": [
        {
            "contract": "0xacce6fd80d44e1796aa0c2c625e9e4e0ce39efc0",
            "methods": ["serialize", "deserialize"]
        },
        {
            "contract": "0x726cb6e0cd8628a1350a611384688911ab75f51b",
            "methods": ["keccak256"]
        }
    ],
    "trusts": ["0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5"],
    "extra": {
        "Author": "Acme Corp",
        "Description": "Solidity contract 'GoldToken' compiled to NeoVM",
        "Version": "0.19.0.0",
        "Compiler": "neo-devpack-solidity-0.19.0",
        "Repository": "https://github.com/acme/gold-token",
        "Build": { "commit": "abc123", "branch": "main" }
    }
}
```

## Key Observations

- `supportedstandards` is `["NEP-17"]` — the compiler detected all five required methods and the `Transfer` event.
- `permissions` includes `StdLib.serialize` and `CryptoLib.keccak256` — these are required by the `mapping` storage access pattern, not by any explicit cross-contract call in the source.
- `trusts` reflects the NatSpec override, restricting callers to the NEO native contract.
- `safe: true` on `symbol`, `decimals`, `totalSupply`, and `balanceOf` because they are `pure` or `view` functions.
- `extra` merges compiler defaults with NatSpec overrides. The `Author` field is overridden by the NatSpec tag.
