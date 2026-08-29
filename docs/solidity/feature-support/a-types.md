---
title: "Solidity Feature Support: A. Types"
description: "A. Types from Solidity Feature Support."
---

# A. Types

[Back to Solidity Feature Support](/solidity/feature-support)

| Feature                   | Status | Notes                                                                                                                         |
| ------------------------- | :----: | ----------------------------------------------------------------------------------------------------------------------------- |
| `bool`                    |   ✅   | Maps to NeoVM Boolean.                                                                                                        |
| `int8` .. `int256`        |   ✅   | All widths parsed. NeoVM uses arbitrary-precision BigInteger internally.                                                      |
| `uint8` .. `uint256`      |   ✅   | All widths parsed. NeoVM uses arbitrary-precision BigInteger internally.                                                      |
| `address`                 |   ✅   | Maps to Neo UInt160 (Hash160, 20 bytes).                                                                                      |
| `address payable`         |   ⚠️   | Parsed and canonicalized to `address`. `.transfer()` / `.send()` auto-map to GAS transfer semantics (not EVM-attached value). |
| `bytes1` .. `bytes32`     |   ✅   | Fixed-length byte arrays via `NeoType::ByteArray { fixed_len }`.                                                              |
| `bytes` (dynamic)         |   ✅   | Dynamic byte array.                                                                                                           |
| `string`                  |   ✅   | UTF-8 string type.                                                                                                            |
| `enum`                    |   ✅   | Backed by `uint8`. Converted via `convert_enum`.                                                                              |
| `struct`                  |   ✅   | Full struct support with nested fields. Serialized via `StdLib.serialize`/`StdLib.deserialize` for storage.                   |
| `mapping(K => V)`         |   ✅   | Storage mappings with Neo StorageMap. Key type validation enforced.                                                           |
| `T[]` (dynamic array)     |   ✅   | `new T[](n)` allocation supported via `NEWARRAY`.                                                                             |
| `T[N]` (fixed array)      |   ⚠️   | Parsed. `new T[N]` supported when `N` is a compile-time constant.                                                             |
| `fixed` / `ufixed`        |   ❌   | Not supported. Also unsupported in mainline Solidity compilers.                                                               |
| User-defined value types  |   ✅   | `type X is Y` creates transparent aliases. `wrap`/`unwrap` compile to no-ops.                                                 |
| `bytes.concat(...)`       |   ✅   | Chains NeoVM `CAT` opcodes. Zero args produce an empty byte array.                                                            |
| `string.concat(...)`      |   ✅   | Same implementation as `bytes.concat` via `CAT` opcode chain.                                                                 |
| Contract types (`IERC20`) |   ✅   | Resolved to Neo UInt160 address. Interface types tracked.                                                                     |
| Tuple types               |   ✅   | Represented as NeoVM arrays internally.                                                                                       |
| `transient` state variables |  ⚠️  | Solidity 0.8.28 (EIP-1153). Compiles to regular **persistent** storage — NeoVM has no transient store. Warns `W_TRANSIENT_PERSISTED`. |
| Function types            |   ❌   | `function(...) internal/external` is not representable on NeoVM. State variables, locals, parameters, and return types of function type are rejected with an "unsupported type" diagnostic. Use named functions and inheritance instead of function pointers. |

## Partial type details

**`address payable`** — The type is accepted and treated identically to `address`. On Neo, `.transfer(amount)` / `.send(amount)` compile via GAS NEP-17 transfer lowering (`transfer(from,to,amount,data)`), so behavior is close but not identical to EVM attached-value calls.

**`transient` state variables** — NeoVM has no transient storage (EIP-1153), so a `transient` state variable compiles to ordinary persistent storage and is **not** cleared at the end of the transaction. The compiler emits warning `W_TRANSIENT_PERSISTED`. A transient-style reentrancy guard would stay locked forever unless reset — clear the variable manually at the end of every entry point, or use ordinary storage semantics deliberately.

**`T[N]` (fixed array)** — Fixed-size arrays compile when the size `N` is a compile-time constant. Runtime-computed sizes require dynamic arrays (`T[]`).

```solidity
// ✅ Compiles — size is a compile-time constant
uint256[10] memory arr;

// ✅ Compiles — dynamic array with runtime size
uint256[] memory arr = new uint256[](n);
```

---
