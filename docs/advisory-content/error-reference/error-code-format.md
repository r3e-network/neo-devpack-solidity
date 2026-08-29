---
title: "Error Reference: Error Code Format"
description: "Error Code Format from Error Reference."
---

# Error Code Format

[Back to Error Reference](/advisory-content/error-reference)

## Overview

Error codes follow the pattern `E<NNNN>` where the first digit indicates the category:

| Range   | Category                                            |
| ------- | --------------------------------------------------- |
| `E1xxx` | Parse errors -- syntax and tokenization             |
| `E2xxx` | Semantic errors -- types, scopes, validation        |
| `E25xx` | Type errors -- overflow, bounds, casts              |
| `E3xxx` | Codegen errors -- bytecode generation, NeoVM limits |
| `E4xxx` | I/O errors -- files, imports, permissions           |
| `E5xxx` | Security warnings -- reentrancy, unsafe patterns    |
