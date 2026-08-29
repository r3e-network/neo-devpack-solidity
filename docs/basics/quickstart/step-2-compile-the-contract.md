---
title: "Quickstart: Step 2: Compile the Contract"
description: "Step 2: Compile the Contract from Quickstart."
---

# Step 2: Compile the Contract

[Back to Quickstart](/basics/quickstart)

## Overview

```bash
./target/release/neo-solc MyStorage.sol -I devpack -O2 -o build/MyStorage
```

Flags explained:

| Flag                 | Purpose                                                                                    |
| -------------------- | ------------------------------------------------------------------------------------------ |
| `MyStorage.sol`      | Input Solidity source file                                                                 |
| `-I devpack`         | Add the `devpack/` directory as an import search path (provides Neo N3 library interfaces) |
| `-O2`                | Optimization level 2 (standard optimization: inlining, peephole, DCE, constant folding)    |
| `-o build/MyStorage` | Output prefix -- generates `build/MyStorage.nef` and `build/MyStorage.manifest.json`       |

Verbose output (with `-v`) is diagnostic and may add or reorder lines between compiler releases. Current output reports the resolved inputs, selected optimization level, emitted artifacts, manifest processing, and any warnings:

```bash
./target/release/neo-solc MyStorage.sol -I devpack -O2 -v -o build/MyStorage
```

You should end the command with both artifacts present:

```bash
ls build/MyStorage.nef build/MyStorage.manifest.json
```
