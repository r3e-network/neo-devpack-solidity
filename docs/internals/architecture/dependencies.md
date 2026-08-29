---
title: "Architecture: Dependencies"
description: "Dependencies from Architecture."
---

# Dependencies

[Back to Architecture](/internals/architecture)

## Core

| Crate                      | Purpose                                            |
| -------------------------- | -------------------------------------------------- |
| `solang-parser`            | Solidity parsing (AST generation)                  |
| `clap`                     | CLI argument parsing                               |
| `serde` / `serde_json`     | JSON serialization (manifest, standard-json)       |
| `sha2` / `sha3` / `ripemd` | Cryptographic hash functions                       |
| `thiserror` / `anyhow`     | Error handling                                     |
| `once_cell`                | Lazy static initialization (opcode/syscall tables) |
| `hex`                      | Hex encoding/decoding                              |

## Dev / Test

| Crate       | Purpose                            |
| ----------- | ---------------------------------- |
| `criterion` | Benchmarking                       |
| `proptest`  | Property-based testing             |
| `tempfile`  | Temporary file utilities for tests |
