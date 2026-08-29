---
title: "Error Reference: Semantic Errors (E2xxx)"
description: "Semantic Errors (E2xxx) from Error Reference."
---

# Semantic Errors (E2xxx)

[Back to Error Reference](/advisory-content/error-reference)

These errors occur during Stage 3 (Semantic Analysis) when the compiler validates types, scopes, and contract structure.

| Code    | Name                  | Description                   | Common Cause                                                 |
| ------- | --------------------- | ----------------------------- | ------------------------------------------------------------ |
| `E2001` | UndefinedVariable     | Undefined variable            | Typo in variable name, missing declaration                   |
| `E2002` | TypeMismatch          | Type mismatch                 | Assigning incompatible types (e.g., `address` to `uint256`)  |
| `E2003` | DuplicateDefinition   | Duplicate definition          | Two functions/variables with the same name in the same scope |
| `E2004` | UndefinedFunction     | Undefined function            | Calling a function that does not exist                       |
| `E2005` | UndefinedType         | Undefined type                | Using a struct or contract name that is not defined          |
| `E2006` | InvalidAssignment     | Invalid assignment            | Assigning to a constant or read-only value                   |
| `E2007` | ImmutableModification | Cannot modify immutable value | Writing to an `immutable` or `constant` variable             |
| `E2008` | VisibilityError       | Visibility error              | Calling a `private` function from outside its contract       |
| `E2009` | InvalidOverride       | Invalid override              | Override without `virtual`/`override` keywords               |
| `E2010` | MissingReturn         | Missing return statement      | Non-void function path without `return`                      |
| `E2011` | UnreachableCode       | Unreachable code              | Code after `return`, `revert`, or `require(false)`           |
| `E2012` | UnusedVariable        | Unused variable               | Declared variable never read                                 |
| `E2013` | UnusedFunction        | Unused function               | Internal function never called                               |
| `E2014` | ShadowedVariable      | Variable shadows outer scope  | Local variable hides a state variable                        |
| `E2015` | InvalidModifier       | Invalid modifier              | Modifier used incorrectly or on wrong function type          |

## Example: E2002 TypeMismatch

```
error[E2002]: type mismatch: expected uint256, found address
  --> MyContract.sol:15:5
   |
15 |     uint256 x = msg.sender;
   |     ^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: use explicit cast: uint256(uint160(msg.sender))
```

**Fix:** Correct the type usage. In Neo DevPack for Solidity, addresses and integers are distinct types just as in standard Solidity. Use explicit casts when conversion is intentional.

## Example: E2003 DuplicateDefinition

```
error[E2003]: duplicate definition: function 'transfer' already defined
  --> MyContract.sol:25:5
```

**Fix:** Rename one of the conflicting definitions, or use function overloading with different parameter signatures.
