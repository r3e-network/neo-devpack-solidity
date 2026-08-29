---
title: "Devpack Overview"
description: "Devpack Overview section index."
---

# Devpack Overview

The devpack provides Solidity-facing libraries, standard contracts, and compiler intrinsics for Neo N3 development. It bridges the gap between Solidity syntax and Neo blockchain capabilities, giving developers access to native contracts, syscalls, storage, runtime services, and NEP token standards through familiar Solidity interfaces.

For a reader-focused map from Solidity constructs to syscalls and devpack
wrappers, see [Syscalls and Devpack](/mapping/syscalls-and-devpack).

::: info Compiler Intrinsics
These libraries are primarily compiler intrinsics surfaces — the `neo-solc` compiler recognizes supported members and lowers them directly to NeoVM opcodes and syscalls. Unsupported members produce a compile-time diagnostic listing available intrinsics. You write Solidity; the compiler emits NeoVM bytecode.
:::

## Sections

| Section |
| --- |
| [Directory Layout](/additional-material/neo-devpack/directory-layout) |
| [Usage](/additional-material/neo-devpack/usage) |
| [Core Contracts](/additional-material/neo-devpack/core-contracts) |
| [Libraries](/additional-material/neo-devpack/libraries) |
| [EVM Compatibility Layer](/additional-material/neo-devpack/evm-compatibility-layer) |
| [Token Standards](/additional-material/neo-devpack/token-standards) |
| [Compiler Intrinsics](/additional-material/neo-devpack/compiler-intrinsics) |
| [Permission-Conscious Development](/additional-material/neo-devpack/permission-conscious-development) |
| [Building Custom Contracts](/additional-material/neo-devpack/building-custom-contracts) |
| [Native Contract Hash Reference](/additional-material/neo-devpack/native-contract-hash-reference) |
| [See Also](/additional-material/neo-devpack/see-also) |
