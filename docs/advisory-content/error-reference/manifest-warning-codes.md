---
title: "Error Reference: Manifest Warning Codes"
description: "Manifest Warning Codes from Error Reference."
---

# Manifest Warning Codes

[Back to Error Reference](/advisory-content/error-reference)

These are string-based warning codes emitted during manifest generation (Stage 8):

| Code                         | Description                  | Trigger                                            |
| ---------------------------- | ---------------------------- | -------------------------------------------------- |
| `COMPILER_WARNING`           | General compiler warning     | Various conditions                                 |
| `NEF_SOURCE_TRUNCATED`       | NEF source field truncated   | `--nef-source` value exceeds 240 bytes             |
| `MANIFEST_FULL_WILDCARD`     | Full wildcard permission     | Manifest requires `{"contract":"*","methods":"*"}` |
| `MANIFEST_WILDCARD_CONTRACT` | Wildcard contract permission | Manifest requires `{"contract":"*",...}`           |
| `MANIFEST_WILDCARD_METHODS`  | Wildcard method permission   | Manifest requires `{...,"methods":"*"}`            |

## Constructor Deploy Warning

When a contract has a parameterised constructor, the compiler emits a warning if the manifest does not include `StdLib.jsonDeserialize` and `StdLib.deserialize` permissions. The injected deploy prologue uses these methods to parse constructor arguments.

```
warning: contract 'MyToken' has a parameterised constructor; deploy it by passing
constructor args through `_deploy(data, update)`. Neo-Express: pass a JSON array
string via `-d '[7]'`; SDKs that support StackItems may pass an Array directly.
```

**Fix:** Ensure the manifest permissions include StdLib access, or pass constructor arguments as a StackItem Array directly from your SDK.
