---
title: "Solidity Feature Support: F. Storage and Memory"
description: "F. Storage and Memory from Solidity Feature Support."
---

# F. Storage and Memory

[Back to Solidity Feature Support](/solidity/feature-support)

| Feature                          | Status | Notes                                                                  |
| -------------------------------- | :----: | ---------------------------------------------------------------------- |
| State variables                  |   ✅   | Mapped to Neo Storage with prefix-based keys.                          |
| `constant`                       |   ✅   | Compile-time constants inlined.                                        |
| `immutable`                      |   ✅   | Tracked via `is_immutable` flag. Modification blocked at compile time. |
| `memory` keyword                 |   ✅   | Parsed. NeoVM is stack-based so memory is implicit.                    |
| `storage` keyword                |   ✅   | Storage references for mappings and state variables.                   |
| `calldata` keyword               |   ✅   | Parsed. Treated as `memory` — NeoVM has no calldata region.            |
| Nested mappings                  |   ✅   | `mapping(K1 => mapping(K2 => V))` with composite storage keys.         |
| Struct in storage                |   ✅   | Serialized/deserialized via `StdLib.serialize`/`StdLib.deserialize`.   |
| Array `.push()` / `.pop()`       |   ✅   | Storage array operations supported.                                    |
| Array `.length`                  |   ✅   | Both memory and storage arrays. Fixed-size storage arrays `T[N]` report the declared bound `N`. |
| `new bytes(n)` / `new string(n)` |   ✅   | Buffer allocation via `NEWBUFFER`.                                     |
| `new T[](n)`                     |   ✅   | Dynamic array allocation via `NEWARRAY`.                               |
| `new Contract(...)`              |   ⚠️   | Does not deploy on Neo; constructor-like logic is inlined/simulated and a zero-address placeholder is produced. Use `ContractManagement.deploy(...)` for real deployment. |
| `new X{salt: s}()`               |   ⚠️   | CREATE2 salted-creation syntax compiles; the salt is evaluated then ignored with a warning — Neo has no CREATE2. Same inline/simulate lowering as `new Contract(...)`. |

## Contract creation via `new`

`new Contract(...)` is accepted for source compatibility, but it does not perform Neo contract deployment. The current lowering inlines/simulates constructor-like logic when the contract is available in the compilation graph and returns a zero-address placeholder. For real child-contract deployment, compile the target contract separately and call `ContractManagement.deploy(nef, manifest, data)`.

The CREATE2 salted form `new X{salt: s}()` also compiles — the salt expression is evaluated (so its side effects still run) and then ignored, with a warning, because Neo has no CREATE2 deterministic-address mechanism. Contracts written against CREATE2 patterns build unchanged; the lowering is otherwise identical to `new Contract(...)`.

## Storage key derivation

State variables are stored in Neo Storage using deterministic key derivation. For simple state variables, the key is derived from the variable name. For mappings, the key is computed as:

```
SHA256(key_bytes || slot_hash)
```

Where `slot_hash` is `SHA256(variable_name)`. Nested mappings iterate this process for each key level. See [Types](/language-description/types) for the full storage lowering specification.

---
