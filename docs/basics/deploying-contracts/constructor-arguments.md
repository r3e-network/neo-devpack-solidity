---
title: "Deploying Contracts: Constructor Arguments"
description: "Constructor Arguments from Deploying Contracts."
---

# Constructor Arguments

[Back to Deploying Contracts](/basics/deploying-contracts)

Solidity constructors map to Neo's `_deploy(data, update)` entry point. The compiler automatically generates this entry point for contracts with constructors.

## How It Works

1. The compiler emits a `_deploy(data, update)` method in the NEF.
2. On initial deployment (`update = false`), the deploy stub deserializes `data` and passes the values to your Solidity constructor logic.
3. On contract update (`update = true`), the constructor is skipped.

## Passing Constructor Arguments

**Neo-Express / CLI tooling:** Pass a JSON array string as the `-d` flag:

```bash
# Constructor: constructor(uint256 initialValue)
$NEOXP contract deploy -i chain.neo-express -d '[7]' build/Counter.nef node1

# Constructor: constructor(uint256 amount, string name)
$NEOXP contract deploy -i chain.neo-express -d '[1000, "MyToken"]' build/Token.nef node1
```

**Neo SDKs (C#, Python, JS):** Pass a StackItem Array directly through the SDK's deploy API.

**Contract-to-contract deployment:** Use `abi.encode(...)` to produce StdLib.serialize bytes, which the deploy stub can also accept.

## Manifest Requirements

Contracts with parameterized constructors require these permissions in the manifest:

- `StdLib.jsonDeserialize`
- `StdLib.deserialize`

The compiler adds these automatically. You can verify:

```bash
jq '.permissions' build/Counter.manifest.json
```

::: warning
If you use `--deny-wildcard-methods` and the compiler cannot narrow the StdLib permissions, compilation will fail. In that case, provide an explicit `--manifest-permissions` file that includes the required StdLib methods.
:::
