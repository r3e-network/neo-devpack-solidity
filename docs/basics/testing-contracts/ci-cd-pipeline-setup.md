---
title: "Testing Contracts: CI/CD Pipeline Setup"
description: "CI/CD Pipeline Setup from Testing Contracts."
---

# CI/CD Pipeline Setup

[Back to Testing Contracts](/basics/testing-contracts)

## Recommended CI Sequence

The recommended CI pipeline runs these steps in order:

```bash
# 1. Code formatting check
cargo fmt --all -- --check

# 2. Linting
cargo clippy --all-targets --all-features -- -D warnings

# 3. Release build
cargo build --release

# 4. Full test suite
cargo test --workspace --all-features

# 5. E2E compilation tests (parallel)
cargo test --test e2e_compilation_tests -- --test-threads=4

# 6. Strict compatibility sweep
STRICT_SWEEP_FAIL_ON_UNEXPECTED_WARNINGS=1 make test-compile-strict

# 7. Neo-Express smoke tests (requires .NET)
make test-deploy-smoke-full
```

## One-Command CI Gate

The `production-gate` target runs all of the above:

```bash
make production-gate
```

See [Production Readiness](/advisory-content/production-readiness) for details.

## GitHub Actions Example

The repository's CI workflow (`.github/workflows/ci.yml`) includes:

- Standard Rust CI (fmt, clippy, test) on Ubuntu
- E2E compilation tests
- A dedicated `neoxp-showcases` job that:
    - Installs Rust + .NET 8 + `jq`
    - Runs `examples/test_neoxp_new_showcases_smoke.sh`
    - Validates `UpgradeLifecycleShowcase`, `WitnessGuardShowcase`, and `OracleRelayStrictShowcase` end-to-end
