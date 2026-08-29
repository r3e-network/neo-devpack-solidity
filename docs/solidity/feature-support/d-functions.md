---
title: "Solidity Feature Support: D. Functions"
description: "D. Functions from Solidity Feature Support."
---

# D. Functions

[Back to Solidity Feature Support](/solidity/feature-support)

| Feature                          | Status | Notes                                                                                                                   |
| -------------------------------- | :----: | ----------------------------------------------------------------------------------------------------------------------- |
| Regular functions                |   ✅   | `public`, `external`, `internal`, `private` visibility.                                                                 |
| File-scope (free) functions      |   ✅   | Callable from contracts. Overloaded free functions are deduplicated by name + arity, so every overload is callable.     |
| Constructor                      |   ✅   | Single constructor. Multiple constructors rejected.                                                                     |
| `view` / `pure`                  |   ✅   | State mutability tracked and enforced at IR level.                                                                      |
| `payable`                        |   ⚠️   | Parsed. `payable` on non-receive functions warns — Neo has no native gas payment in function calls.                     |
| `returns (T)`                    |   ✅   | Single return type.                                                                                                     |
| `returns (T1, T2, ...)`          |   ✅   | Multi-return via NeoVM arrays.                                                                                          |
| Function overloading             |   ⚠️   | Supported with Neo overload mangling. One canonical ABI name is kept; other overloads use generated `neo_name` entries. |
| `modifier`                       |   ✅   | Full modifier expansion with `_` placeholder substitution.                                                              |
| `receive()`                      |   ⚠️   | **Silently remapped** to `onNEP17Payment(address,uint256,bytes)` in the manifest when no explicit `onNEP17Payment` is declared. See detail below. |
| `fallback()`                     |   ⚠️   | Kept as `fallback` in the manifest. Diagnostic `W105` suggests using `onNEP17Payment()` callback instead.               |
| `virtual` / `override`           |   ✅   | Inheritance flattening resolves overrides. Multi-level chains supported.                                                |
| Function selectors (`.selector`) |   ✅   | Computed from canonical parameter types.                                                                                |
| NatSpec comments                 |   ✅   | `@notice`, `@dev`, `@param`, `@return` preserved in metadata.                                                           |

## Partial function details

**`payable`** — Neo does not attach native value to function calls the way EVM does with `msg.value`. The `payable` modifier is accepted for source compatibility, but a warning is emitted on non-receive functions. Token payments on Neo are handled through NEP-17/NEP-11 callbacks.

**Function overloading** — The compiler supports overloaded functions by assigning Neo-visible mangled names (`neo_name`) to overloaded variants. One canonical ABI name is preserved, and other overloads are exported under generated names like `foo(uint256)` or `foo(address)`. The limitation is not compilation, but downstream invocation: Neo callers must use the generated Neo method names when targeting a non-primary overload.

```solidity
// ⚠️ Overload collision in Neo ABI — both produce "transfer" in manifest
function transfer(address to, uint256 amount) public { ... }
function transfer(address to, uint256 amount, bytes calldata data) public { ... }

// ✅ Use distinct names instead
function transfer(address to, uint256 amount) public { ... }
function transferWithData(address to, uint256 amount, bytes calldata data) public { ... }
```

**`receive()` / `fallback()`** — These EVM constructs handle incoming Ether. On Neo, token receipts are handled by explicit callbacks.

- `receive() external payable { ... }` is **silently remapped** to `onNEP17Payment(address from, uint256 amount, bytes data)` in the manifest when the contract does not already declare an explicit `onNEP17Payment`. The body is preserved unchanged; only the ABI entrypoint name and signature are rewritten (see `src/solidity/convert/functions.rs:32`). Ethereum developers should be aware that tooling will see the entrypoint as `onNEP17Payment`, not `receive`.
- `fallback()` is **never remapped** — it keeps its Solidity name in the manifest. Neo has no EVM-style unknown-method fallback; diagnostic `W105` flags it and suggests `onNEP17Payment`.
- When both `receive()` and an explicit `onNEP17Payment` are declared, `receive()` is **not** remapped (it retains its name) and `W105` warns that it has no effect on Neo N3.

**Migration guidance**: declare `onNEP17Payment(address from, uint256 amount, bytes data)` directly when porting from Solidity. It surfaces the sender, amount, and attached data that NEP-17 provides at the transfer boundary.

---
