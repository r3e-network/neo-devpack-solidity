---
title: "Architecture Overview"
description: "Overview from Architecture."
---

# Architecture Overview

[Back to Architecture](/internals/architecture)

## Overview

The Neo DevPack for Solidity compiler (`neo-solc`) is a Rust-based toolchain that translates Solidity 0.8.x smart contracts into NeoVM bytecode. The output is a `.nef` (Neo Executable Format) file and a `.manifest.json` deployment manifest, ready for deployment to the Neo N3 blockchain.

The compiler is built as a single Rust binary with no external runtime dependencies. It uses the `solang-parser` crate for Solidity parsing and `clap` for CLI argument handling.
