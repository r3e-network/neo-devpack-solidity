---
title: "Error Reference: Diagnostic Output Formats"
description: "Diagnostic Output Formats from Error Reference."
---

# Diagnostic Output Formats

[Back to Error Reference](/advisory-content/error-reference)

## Human-Readable (default)

```
error[E2002]: type mismatch: expected uint256, found address
  --> MyContract.sol:15:5
   |
15 |     uint256 x = msg.sender;
   |     ^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: use explicit cast: uint256(uint160(msg.sender))
```

## JSON Lines (`--json-errors`, `--json-warnings`)

Each diagnostic is a single JSON object on one line of stderr:

```json
{
  "component": "neo-devpack-solidity",
  "severity": "error",
  "type": "CompilerError",
  "code": "E2002",
  "message": "type mismatch: expected uint256, found address",
  "formattedMessage": "type mismatch: expected uint256, found address",
  "location": {
    "file": "MyContract.sol",
    "line": 15,
    "column": 5
  }
}
```
