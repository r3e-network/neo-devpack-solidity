---
title: "Manifest Specification: Manifest Warnings"
description: "Manifest Warnings from Manifest Specification."
---

# Manifest Warnings

[Back to Manifest Specification](/internals/contract-metadata)

## Overview

The compiler emits structured warnings for manifest-related issues. In standard JSON mode, these appear in the `errors` array with `severity: "warning"`.

| Code                         | Type                 | Trigger                                                 |
| ---------------------------- | -------------------- | ------------------------------------------------------- |
| `COMPILER_WARNING`           | `CompilerWarning`    | General compiler warning (default code)                 |
| `NEF_SOURCE_TRUNCATED`       | `NefSourceTruncated` | `--nef-source` value exceeds 256 bytes                  |
| `MANIFEST_FULL_WILDCARD`     | `Validation`         | Both `contract="*"` and `methods="*"` in a single entry |
| `MANIFEST_WILDCARD_CONTRACT` | `Validation`         | `contract="*"` in a permission entry                    |
| `MANIFEST_WILDCARD_METHODS`  | `Validation`         | `methods="*"` in a permission entry                     |

::: info
Wildcard warnings are informational by default. They become hard errors when the corresponding `--deny-wildcard-*` flag is set.
:::
