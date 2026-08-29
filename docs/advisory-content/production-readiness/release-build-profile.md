---
title: "Production Readiness: Release Build Profile"
description: "Release Build Profile from Production Readiness."
---

# Release Build Profile

[Back to Production Readiness](/advisory-content/production-readiness)

## Overview

The Cargo release profile is configured for maximum optimization:

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
```

This produces the smallest and fastest compiler binary. Always use `cargo build --release` for production compilation.
