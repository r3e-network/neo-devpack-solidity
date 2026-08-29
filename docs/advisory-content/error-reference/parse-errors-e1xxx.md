---
title: "Error Reference: Parse Errors (E1xxx)"
description: "Parse Errors (E1xxx) from Error Reference."
---

# Parse Errors (E1xxx)

[Back to Error Reference](/advisory-content/error-reference)

These errors occur during Stage 1 (Frontend) when the `solang-parser` processes Solidity source code. Parse errors block all subsequent compilation stages.

| Code    | Name              | Description               | Common Cause                             |
| ------- | ----------------- | ------------------------- | ---------------------------------------- |
| `E1001` | UnexpectedToken   | Unexpected token in input | Typo, missing operator, wrong syntax     |
| `E1002` | UnexpectedEof     | Unexpected end of file    | Unclosed brace, missing semicolon at EOF |
| `E1003` | InvalidSyntax     | Invalid syntax            | Unsupported Solidity syntax construct    |
| `E1004` | MissingSemicolon  | Missing semicolon         | Statement not terminated                 |
| `E1005` | MissingBrace      | Missing brace             | Unclosed `{` or extra `}`                |
| `E1006` | InvalidNumber     | Invalid number literal    | Malformed hex, overflow in literal       |
| `E1007` | InvalidString     | Invalid string literal    | Unclosed string, invalid escape sequence |
| `E1008` | InvalidIdentifier | Invalid identifier        | Reserved word used as identifier         |

::: tip
Parse errors must be fixed first. The compiler cannot proceed to semantic analysis or code generation until all parse errors are resolved.
:::

## Example: E1005 MissingBrace

```
error[E1005]: missing closing brace
  --> MyContract.sol:42:1
   |
42 | function transfer(address to, uint256 amount) public {
   |                                                       ^ expected matching '}'
```

**Fix:** Ensure every `{` has a corresponding `}`. Check for accidentally deleted lines or copy-paste errors.
