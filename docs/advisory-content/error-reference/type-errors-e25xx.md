---
title: "Error Reference: Type Errors (E25xx)"
description: "Type Errors (E25xx) from Error Reference."
---

# Type Errors (E25xx)

[Back to Error Reference](/advisory-content/error-reference)

| Code    | Name             | Description               | Common Cause                                          |
| ------- | ---------------- | ------------------------- | ----------------------------------------------------- |
| `E2501` | IntegerOverflow  | Integer overflow          | Compile-time constant exceeds type range              |
| `E2502` | IntegerUnderflow | Integer underflow         | Compile-time constant below type minimum              |
| `E2503` | DivisionByZero   | Division by zero          | Compile-time division by zero constant                |
| `E2504` | ArrayOutOfBounds | Array index out of bounds | Compile-time index exceeds fixed array length         |
| `E2505` | InvalidCast      | Invalid type cast         | Incompatible explicit cast (e.g., `string` to `uint`) |
| `E2506` | NullReference    | Null reference            | Accessing member on potentially null reference        |

## Example: E2501 IntegerOverflow

```
error[E2501]: integer overflow: value 999999999999999999999 exceeds uint256 range
  --> MyContract.sol:8:20
```

**Fix:** Use a value within the valid range for the target type, or split the computation across multiple steps.
