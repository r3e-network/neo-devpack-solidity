---
title: "Error Reference: Internal Error Types"
description: "Internal Error Types from Error Reference."
---

# Internal Error Types

[Back to Error Reference](/advisory-content/error-reference)

## Overview

The compiler uses several internal error categories that appear in JSON diagnostic output:

| Type                 | Source                           | Description                              |
| -------------------- | -------------------------------- | ---------------------------------------- |
| `CompilerError`      | Validation and semantic analysis | Standard compilation errors with E-codes |
| `IrGeneration`       | IR lowering (Stage 5)            | Errors during IR generation with context |
| `ManifestGeneration` | Artifact builder (Stage 8)       | Manifest permission policy violations    |
| `Generic`            | Various stages                   | Catch-all for unclassified errors        |

IR generation errors include additional context fields in JSON output:

```json
{
  "component": "neo-devpack-solidity",
  "severity": "error",
  "type": "IrGeneration",
  "message": "unsupported feature: inline assembly",
  "functionName": "computeHash",
  "errorCode": "E3001",
  "suggestion": "Use CryptoLib.sha256() from the devpack instead"
}
```
