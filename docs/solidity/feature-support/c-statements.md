---
title: "Solidity Feature Support: C. Statements"
description: "C. Statements from Solidity Feature Support."
---

# C. Statements

[Back to Solidity Feature Support](/solidity/feature-support)

| Feature                   | Status | Notes                                                                                                        |
| ------------------------- | :----: | ------------------------------------------------------------------------------------------------------------ |
| `if` / `else`             |   ✅   | Standard conditional branching.                                                                              |
| `for` loop                |   ✅   | Init, condition, post, body all lowered.                                                                     |
| `while` loop              |   ✅   | Condition + body.                                                                                            |
| `do...while` loop         |   ✅   | Body + condition.                                                                                            |
| `break`                   |   ✅   | Loop break.                                                                                                  |
| `continue`                |   ✅   | Loop continue.                                                                                               |
| `return`                  |   ✅   | Single and multi-value returns.                                                                              |
| `emit Event(...)`         |   ✅   | Maps to `Runtime.Notify`. Indexed params supported.                                                          |
| `revert(...)`             |   ✅   | Maps to NeoVM `ABORT` with message.                                                                          |
| `revert CustomError(...)` |   ✅   | Named revert with args.                                                                                      |
| Variable declaration      |   ✅   | Local variable definitions with optional initializer.                                                        |
| Block `{ ... }`           |   ✅   | Scoped statement blocks.                                                                                     |
| `unchecked { ... }`       |   ✅   | Suppresses Solidity 0.8 checked-arithmetic guards inside the block; wrapping semantics are preserved for supported integer arithmetic. |
| `assembly { ... }`        |   ⚠️   | Limited Yul subset lowering; unsupported EVM-only operations warn and emit no assembly logic for that block. |
| `try` / `catch`           |   ✅   | Maps to NeoVM `TRY`/`ENDTRY`. Single catch clause preferred.                                                 |
| `catch Error(string)`     |   ✅   | Named catch with parameter binding.                                                                          |
| `catch Panic(uint256)`    |   ✅   | Matches the EVM-canonical `keccak256("Panic(uint256)")[..4]` (= `0x4e487b71`) selector on the revert envelope and decodes the 32-byte BE code into the catch binding (Task #103). |
| `catch (bytes)`           |   ✅   | Low-level catch with raw bytes.                                                                              |

## Partial statement details

**`catch Panic(uint256)`** — The compiler emits the EVM-canonical revert envelope (`selector || abi.encode(code)`) for `assert(false)` (0x01), checked-arithmetic overflow and underflow (0x11), div/mod by zero (0x12), enum-cast range violations (0x21), empty-array `.pop()` (0x31), and `abi.decode` short-buffer faults (0x41). The `catch Panic(uint code)` dispatcher guards on the 4-byte selector and decodes `code` via `StdLib.abiDecode`, so panic codes such as `0x11` and `0x12` are reachable with Ethereum-compatible semantics.

**`unchecked { ... }`** — Checked arithmetic outside an `unchecked` block emits fixed-width range guards and reverts with `Panic(0x11)` on overflow or underflow. Inside `unchecked`, those overflow guards are suppressed and supported fixed-width arithmetic wraps according to Solidity semantics. `unchecked` does not suppress non-overflow panics such as division by zero (`Panic(0x12)`).

```solidity
unchecked {
    uint256 result = a + b; // wraps instead of reverting on overflow
}
```

**`assembly { ... }`** — Inline assembly parses as Yul and lowers a compatibility
subset for common library patterns: Yul locals and assignments, reads from
Solidity locals and parameters, bounded `mstore`/`mload`/`return`, transient
`tstore`/`tload`, `returndatasize`, guarded `returndatacopy`, arithmetic,
bitwise, shift, comparison helpers, and simple nested control flow. EVM-only
operations such as raw `sload`, `sstore`, `call`, and `create` should be
rewritten with Solidity or Neo devpack/native-contract helpers.

---
