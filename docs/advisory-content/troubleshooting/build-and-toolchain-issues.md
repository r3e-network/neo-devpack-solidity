---
title: "Troubleshooting: Build and Toolchain Issues"
description: "Build and Toolchain Issues from Troubleshooting."
---

# Build and Toolchain Issues

[Back to Troubleshooting](/advisory-content/troubleshooting)

## VitePress Build Failures (Documentation)

```bash
# Build the documentation site
npm run docs:build

# Common fix: dead links
# Check the error output for the specific file and link path
# Ensure all internal links use absolute VitePress paths: /section/page
```

## Cargo Build Failures (Compiler)

```bash
# Clean and rebuild
cargo clean && cargo build --release

# If solang-parser fails, check Rust version
rustc --version  # Requires 1.88+

# Run tests to verify
cargo test --workspace
```

## Neo-Express Issues

```bash
# Reset if state is corrupted
neo-express reset

# Create fresh network
neo-express create
neo-express run

# Check if running
neo-express show balances
```

---
