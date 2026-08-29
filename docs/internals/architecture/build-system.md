---
title: "Architecture: Build System"
description: "Build System from Architecture."
---

# Build System

[Back to Architecture](/internals/architecture)

## Overview

```bash
# Debug build
cargo build

# Release build (optimized binary)
cargo build --release

# Run all tests
cargo test --workspace

# Run specific test suite
cargo test runtime_  # Runtime tests
cargo test e2e_      # End-to-end compilation tests

# Format code
cargo fmt

# Lint
cargo clippy

# Benchmark
cargo bench
```

The release binary is at `target/release/neo-solc`.
