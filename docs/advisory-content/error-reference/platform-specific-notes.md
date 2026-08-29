---
title: "Error Reference: Platform-Specific Notes"
description: "Platform-Specific Notes from Error Reference."
---

# Platform-Specific Notes

[Back to Error Reference](/advisory-content/error-reference)

## Linux

Ensure `libclang` is installed for the `solang-parser` dependency:

```bash
apt-get install libclang-dev
```

## macOS

Xcode command-line tools are required:

```bash
xcode-select --install
```

## Windows

Visual Studio Build Tools and the Rust MSVC target are required.
