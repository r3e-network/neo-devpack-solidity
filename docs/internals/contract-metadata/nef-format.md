---
title: "Manifest Specification: NEF Format"
description: "NEF Format from Manifest Specification."
---

# NEF Format

[Back to Manifest Specification](/internals/contract-metadata)

## Overview

The Neo Executable Format (NEF) is the binary container for NeoVM bytecode. The compiler produces a `.nef` file alongside the manifest for every compiled contract.

| Field         | Size     | Description                                                     |
| ------------- | -------- | --------------------------------------------------------------- |
| Magic         | 4 bytes  | `NEF3` — identifies the file as a Neo N3 executable             |
| Compiler ID   | 64 bytes | UTF-8 string, zero-padded (e.g. `neo-devpack-solidity-0.19.0`)          |
| Source URL    | variable | Length-prefixed string, max 256 bytes. Set via `--nef-source`   |
| Reserved      | 1 byte   | Must be `0x00`                                                  |
| Method Tokens | variable | Varint count + up to 128 token entries for `CALLT` optimization |
| Reserved      | 2 bytes  | Must be `0x0000`                                                |
| Script        | variable | Length-prefixed NeoVM bytecode (max 512 KB)                     |
| Checksum      | 4 bytes  | First 4 bytes of `SHA256(SHA256(header + script))`              |

::: tip
The `--nef-source` flag embeds a URL or identifier into the NEF header. This is useful for linking deployed contracts back to their source repository. If the value exceeds 256 bytes it is silently truncated to the nearest valid UTF-8 boundary and a `NEF_SOURCE_TRUNCATED` warning is emitted.
:::
