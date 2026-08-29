---
title: "Solidity Feature Support: G. Error Handling"
description: "G. Error Handling from Solidity Feature Support."
---

# G. Error Handling

[Back to Solidity Feature Support](/solidity/feature-support)

| Feature                                | Status | Notes                                                                                                               |
| -------------------------------------- | :----: | ------------------------------------------------------------------------------------------------------------------- |
| `require(condition)`                   |   ✅   | Maps to NeoVM `ASSERT`.                                                                                             |
| `require(condition, "msg")`            |   ✅   | `ASSERT` with message.                                                                                              |
| `require(condition, CustomError(...))` |   ✅   | Error name and arg count preserved in NeoVM `THROW` message.                                                        |
| `assert(condition)`                    |   ✅   | Maps to NeoVM `ASSERT`.                                                                                             |
| `revert()`                             |   ✅   | Maps to NeoVM `ABORT`.                                                                                              |
| `revert("message")`                    |   ✅   | `ABORT` with message.                                                                                               |
| `revert CustomError(...)`              |   ✅   | Named revert with arguments.                                                                                        |
| Custom error definitions               |   ✅   | `error X(...)` parsed and used in revert statements.                                                                |
| `try` / `catch`                        |   ✅   | NeoVM `TRY`/`ENDTRY` structured exception handling.                                                                 |
| `try` with return binding              |   ✅   | `try f() returns (uint r) { ... }` supported.                                                                       |
| Multiple catch clauses                 |   ✅   | Lowered with EVM-canonical 4-byte selector guards (Task #103). `catch Panic(uint256)` matches `0x4e487b71`, `catch Error(string)` matches `0x08c379a0`, `catch (bytes)` binds the raw envelope. User-defined named error clauses retain the legacy permissive `ISTYPE` guard. |

## Partial error handling details

**Multiple catch clauses** — Task #103 switched the dispatcher to EVM-canonical selector matching. `catch Panic(uint256)` and `catch Error(string)` now guard on the 4-byte keccak selector (`0x4e487b71`, `0x08c379a0`) at the head of the revert envelope, decoding `code` / `msg` via `StdLib.abiDecode` and `SUBSTR`. User-defined named catches (e.g. `catch CustomErr(uint c)`) retain the pre-#103 ISTYPE guard until a follow-up extends selector-based routing to custom errors.

```solidity
// ✅ Recommended — single catch clause
try target.someFunction() returns (uint256 result) {
    // success
} catch (bytes memory reason) {
    // handle any failure
}

// ⚠️ Works but with caveats — multiple catch clauses
try target.someFunction() returns (uint256 result) {
    // success
} catch Error(string memory reason) {
    // string exceptions routed here
} catch (bytes memory lowLevelData) {
    // everything else
}
```

---
