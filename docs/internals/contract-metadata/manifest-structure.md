---
title: "Manifest Specification: Manifest Structure"
description: "Manifest Structure from Manifest Specification."
---

# Manifest Structure

[Back to Manifest Specification](/internals/contract-metadata)

The manifest is a JSON file containing all metadata the Neo N3 runtime needs to validate and dispatch calls to your contract. The compiler emits every required top-level field:

```json
{
    "name": "ContractName",
    "groups": [],
    "features": {},
    "supportedstandards": [],
    "abi": {
        "methods": [],
        "events": []
    },
    "permissions": [],
    "trusts": [],
    "extra": {
        "Author": "Jimmy <jimmy@r3e.network>",
        "Description": "Solidity contract 'ContractName' compiled to NeoVM",
        "Version": "0.19.0.0",
        "Compiler": "neo-devpack-solidity-0.19.0"
    }
}
```

## name

The contract name used for on-chain identification. Defaults to the Solidity contract name.

Override with NatSpec:

```solidity
/// @custom:neo.manifest.name MyCustomName
contract Token { }
```

## groups

Array of group objects for contract grouping and multi-signature verification. Defaults to an empty array.

Override with NatSpec:

```solidity
/// @custom:neo.manifest.groups [{"pubkey":"03b209...","signature":"abc123..."}]
contract Token { }
```

## features

Must be an empty object `{}` for Neo N3. The Neo runtime rejects any contract with populated feature keys. The compiler enforces this constraint — non-empty custom values are ignored with a warning.

::: warning
Do not attempt to set features to a non-empty object. Neo N3 reserves this field for future use and will reject deployment of contracts with populated features.
:::

## supportedstandards

Array of standard identifier strings (e.g. `["NEP-17"]`). The compiler auto-detects supported standards from your contract's method signatures and events. See [Standards Auto-Detection](/internals/contract-metadata/standards-auto-detection) for the full detection logic.

Override with NatSpec:

```solidity
/// @custom:neo.manifest.supportedstandards ["NEP-17"]
contract Token { }
```

## abi

The contract's Application Binary Interface, containing method and event declarations.

### Methods

Each public or external function in your Solidity contract becomes a method entry in the manifest ABI:

```json
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
```

The compiler maps Solidity types to Neo manifest types according to this table:

| Solidity Type       | Manifest Type | Notes                          |
| ------------------- | ------------- | ------------------------------ |
| `address`           | `Hash160`     | 20-byte Neo address            |
| `uint8`..`uint256`  | `Integer`     | All unsigned integer widths    |
| `int8`..`int256`    | `Integer`     | All signed integer widths      |
| `bool`              | `Boolean`     |                                |
| `string`            | `String`      |                                |
| `bytes`             | `ByteArray`   | Dynamic byte array             |
| `bytes1`..`bytes31` | `ByteArray`   | Fixed-size byte arrays         |
| `bytes32`           | `Hash256`     | 32-byte hash                   |
| `bytes20`           | `Hash160`     | Treated as address-width       |
| `T[]`               | `Array`       | Any dynamic array              |
| `mapping(K => V)`   | `Map`         | Key-value mapping              |
| `struct`            | `Array`       | Structs serialize as arrays    |
| `enum`              | `Integer`     | Enums are integer-backed       |
| Neo `Any` type      | `Any`         | Explicit any-type from devpack |

::: info
When a NeoType annotation is present (via the devpack), it takes precedence over the Solidity type inference. For example, a parameter annotated with `NeoType.Address` always maps to `Hash160` regardless of its Solidity declaration.
:::

The `safe` flag is set to `true` for `view` and `pure` functions. Safe methods can be invoked without a transaction (read-only calls) and do not consume GAS on-chain.

When a function returns multiple values, the manifest return type is `Array`. Functions with no return value use `Void`.

**Overloaded functions:** Neo N3 dispatches methods by name and does not support Solidity-style overload resolution. The compiler emits the canonical name for the "primary" overload (highest arity) and uses a mangled `neo_name` for secondary overloads, guaranteeing unique ABI names.

### Events

Each Solidity `event` becomes an event entry with a name and typed parameter array:

```json
{
    "name": "Transfer",
    "parameters": [
        { "name": "from", "type": "Hash160" },
        { "name": "to", "type": "Hash160" },
        { "name": "amount", "type": "Integer" }
    ]
}
```

Event parameters use the same type mapping table as method parameters.

## permissions

Array of permission entries declaring which contracts and methods this contract may call. Neo N3 enforces these at runtime — any call not covered by a permission entry will fault.

Explicit permission (specific contract and method):

```json
{
    "contract": "0xd2a4cff31913016155e38e474a2c06d08be276cf",
    "methods": ["transfer"]
}
```

Wildcard permissions (use with caution):

```json
{ "contract": "*", "methods": "*" }
```

```json
{ "contract": "0xd2a4cff31913016155e38e474a2c06d08be276cf", "methods": "*" }
```

```json
{ "contract": "*", "methods": ["ping"] }
```

See [Permission Inference](/internals/contract-metadata/permission-inference) and [Permission Hardening](/internals/contract-metadata/permission-hardening) for details on how the compiler generates and restricts these entries.

## trusts

Declares which contracts are trusted to call this contract. Defaults to an empty array `[]` (no external callers trusted).

Override with NatSpec:

```solidity
/// @custom:neo.manifest.trusts "*"
contract OpenContract { }
```

::: warning
Setting `trusts` to `"*"` means any contract on the network can call yours. Use this only when your contract is designed as a public utility. For most contracts, leave the default empty array or specify explicit contract hashes.
:::

## extra

Arbitrary metadata attached to the contract. The compiler populates baseline fields automatically:

| Key           | Value                                            |
| ------------- | ------------------------------------------------ |
| `Author`      | Compiler author identifier                       |
| `Description` | `"Solidity contract '<Name>' compiled to NeoVM"` |
| `Version`     | Compiler version in `major.minor.patch.0` format |
| `Compiler`    | Compiler ID string (e.g. `neo-devpack-solidity-0.19.0`)  |

Add custom fields via NatSpec `@custom:neo.manifest.extra.*` tags. Values are parsed as JSON when valid, otherwise stored as plain strings.
