---
title: "Production Readiness: Production Gate"
description: "Production Gate from Production Readiness."
---

# Production Gate

[Back to Production Readiness](/advisory-content/production-readiness)

## Overview

The single most important command before any production deployment:

```bash
make production-gate
```

This runs the full validation pipeline:

| Step               | Command                                                               | What It Checks                                           |
| ------------------ | --------------------------------------------------------------------- | -------------------------------------------------------- |
| 1. Formatting      | `cargo fmt --all -- --check`                                          | Code style consistency                                   |
| 2. Linting         | `cargo clippy --all-targets --all-features -- -D warnings`            | Rust code quality and common mistakes                    |
| 3. Release build   | `cargo build --release`                                               | Compilation succeeds with optimizations                  |
| 4. Full test suite | `cargo test --workspace --all-features`                               | Rust unit, integration, manifest, runtime, and E2E tests |
| 5. Tooling tests   | `make tooling-test`                                                   | TypeScript tooling packages build and test               |
| 6. Tooling lint    | `make tooling-lint`                                                   | Tooling ESLint checks pass                               |
| 7. Runtime tests   | `make runtime-test`                                                   | Maintained optional C# runtime test slice passes         |
| 8. E2E tests       | `cargo test --test e2e_compilation_tests -- --test-threads=4`         | All example contracts compile correctly                  |
| 9. Strict sweep    | `STRICT_SWEEP_FAIL_ON_UNEXPECTED_WARNINGS=1 make test-compile-strict` | No unexpected warnings in devpack or examples            |
| 10. Smoke tests    | `make test-deploy-smoke-full`                                         | All 16+ deploy/invoke scenarios pass on Neo-Express      |

If `make production-gate` passes, the compiler toolchain is verified end-to-end.

::: warning
Never skip the production gate before deploying to TestNet or MainNet. It catches regressions that unit tests alone cannot detect.
:::
