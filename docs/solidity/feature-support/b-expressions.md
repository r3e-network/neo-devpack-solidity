---
title: "Solidity Feature Support: B. Expressions"
description: "B. Expressions from Solidity Feature Support."
---

# B. Expressions

[Back to Solidity Feature Support](/solidity/feature-support)

| Feature                                       | Status | Notes                                                                                                    |
| --------------------------------------------- | :----: | -------------------------------------------------------------------------------------------------------- |
| Arithmetic (`+`, `-`, `*`, `/`, `%`)          |   ✅   | Binary ops via `try_lower_expression_binary_ops`. Checked by default (Solidity 0.8 `Panic(0x11)`); `unchecked` wraps mod 2^256, including `int256` two's-complement. |
| Comparison (`==`, `!=`, `<`, `>`, `<=`, `>=`) |   ✅   | Via `try_lower_expression_comparisons`.                                                                  |
| Logical (`&&`, `\|\|`, `!`)                   |   ✅   | Short-circuit evaluation.                                                                                |
| Bitwise (`&`, `\|`, `^`, `~`, `<<`, `>>`)     |   ✅   | Full bitwise support. `bytesN << k` / `bytesN >> k` shift the big-endian face value with EVM truncation (all widths). |
| Unary (`++`, `--`, `-`, `!`)                  |   ✅   | Pre/post increment/decrement.                                                                            |
| Ternary (`? :`)                               |   ✅   | `ConditionalOperator` lowered with labels.                                                               |
| Assignment (`=`, `+=`, `-=`, etc.)            |   ✅   | Compound assignments in `assignments/compound.rs`.                                                       |
| `delete`                                      |   ✅   | State vars, mapping entries, locals, array elements, struct fields.                                      |
| Tuple expressions `(a, b, c)`                 |   ✅   | Lowered to NeoVM arrays.                                                                                 |
| Tuple destructuring `(a, b) = f()`            |   ⚠️   | Supported. Some complex nested target forms may require intermediate locals.                             |
| Type casting                                  |   ✅   | Explicit casts between compatible types.                                                                 |
| `type(X).min` / `type(X).max`                 |   ✅   | Supported for integer types.                                                                             |
| `type(T).name`                                |   ✅   | Compile-time string constant.                                                                            |
| `type(I).interfaceId`                         |   ✅   | Computed from selector XOR of interface methods.                                                         |
| `abi.encode(...)`                             |   ⚠️   | Supported in context of `address.call`/`staticcall`. Standalone use is limited.                          |
| `abi.encodePacked(...)`                       |   ⚠️   | Same as `abi.encode` — used for Neo contract call encoding.                                              |
| `abi.encodeWithSignature(...)`                |   ⚠️   | Low-level call payloads rewrite to Neo contract calls; standalone use approximates calldata as `selector \|\| abi.encode(args)`. |
| `abi.encodeWithSelector(...)`                 |   ⚠️   | Low-level call payloads rewrite to Neo contract calls; standalone use approximates calldata as `selector \|\| abi.encode(args)`. |
| `abi.encodeCall(...)`                         |   ✅   | Maps to `StdLib.serialize`.                                                                              |
| `abi.decode(...)`                             |   ✅   | Maps to `StdLib.deserialize`. Type tuple parsed from second argument.                                    |
| Named function call args `f({x: 1})`          |   ✅   | Named args reordered to positional order at IR level.                                                    |

## Partial expression details

**Shifts on `bytesN`** — `bytesN << k` and `bytesN >> k` shift the big-endian face value and truncate the result to `N` bytes, matching EVM semantics for every width from `bytes1` to `bytes32` (e.g. `bytes32(uint256(0x100)) >> 8` yields `0x...01`).

**`unchecked` arithmetic** — inside an `unchecked { ... }` block, `+`, `-`, and `*` wrap two's-complement modulo 2^256 for both `uint256` and `int256` (`type(int256).max + 1 == type(int256).min`). Outside `unchecked`, Solidity 0.8 checked semantics apply and overflow reverts with `Panic(0x11)`.

**Tuple destructuring** — Standard patterns like `(uint a, uint b) = getValues()` work. Deeply nested destructuring targets (e.g., destructuring into struct members or nested tuples in a single statement) may require the compiler to introduce intermediate locals.

**`abi.encode` / `abi.encodePacked` / `abi.encodeWithSignature` / `abi.encodeWithSelector`** — These functions are primarily designed for cross-contract call encoding on Neo, not for producing Ethereum-ABI-compatible byte sequences. When used as arguments to `address.call()` or `address.staticcall()`, the compiler rewrites them into Neo contract-call lowering. Standalone use now returns a Neo-side approximation: `abi.encode*` emits `StdLib.serialize(...)`, while `encodeWithSignature` / `encodeWithSelector` emit `selector || abi.encode(args)`. These byte sequences are useful on Neo, but not guaranteed to be EVM-identical raw calldata.

```solidity
// ✅ Works — encoding for cross-contract call
address(target).call(abi.encodeWithSignature("transfer(address,uint256)", to, amount));

// ⚠️ Limited — standalone encoding may differ from EVM ABI
bytes memory encoded = abi.encode(a, b, c);
bytes memory payload = abi.encodeWithSignature("transfer(address,uint256)", to, amount);
```

---
