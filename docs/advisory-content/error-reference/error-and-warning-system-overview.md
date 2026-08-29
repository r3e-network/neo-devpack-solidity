---
title: "Error Reference: Error and Warning System Overview"
description: "Error and Warning System Overview from Error Reference."
---

# Error and Warning System Overview

[Back to Error Reference](/advisory-content/error-reference)

## Overview

The compiler uses a structured diagnostic system with four severity levels:

| Severity    | Behavior                                                                                          |
| ----------- | ------------------------------------------------------------------------------------------------- |
| **error**   | Compilation fails. Must be fixed before artifacts can be generated.                               |
| **warning** | Compilation succeeds, but the issue should be reviewed. Can be promoted to error with `--Werror`. |
| **info**    | Informational note. Does not affect compilation.                                                  |
| **hint**    | Suggestion for improvement. Does not affect compilation.                                          |

Each diagnostic includes:

- A numeric error code (format `E<NNNN>`)
- A human-readable message
- Source location (file, line, column) when available
- Optional fix suggestions
