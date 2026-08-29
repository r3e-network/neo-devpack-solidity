---
title: "Syscalls: Syscall Categories"
description: "Syscall Categories from Syscalls."
---

# Syscall Categories

[Back to Syscalls](/internals/syscalls)

## Overview

The embedded runtime registry contains 35 syscall names across 5 categories: the 34 Neo N3 syscall names plus the neo-devpack-solidity `System.Runtime.GetMsgValue` extension used for host-injected Solidity `msg.value`. Some host-state queries are intentionally approximated in the embedded runtime; see the category pages for method-level behavior.

| Category | Count | Prefix              | Purpose                             |
| -------- | ----: | ------------------- | ----------------------------------- |
| Storage  |     7 | `System.Storage.*`  | Persistent key-value state          |
| Runtime  |    20 | `System.Runtime.*`  | Execution context, notifications, and the `GetMsgValue` extension |
| Contract |     4 | `System.Contract.*` | Cross-contract calls and accounts   |
| Crypto   |     2 | `System.Crypto.*`   | Signature verification              |
| Iterator |     2 | `System.Iterator.*` | Traversal of storage search results |

::: info
Most cryptographic operations (SHA256, RIPEMD160, keccak256, ECDSA verification, BLS12-381) are exposed through the `CryptoLib` native contract, not through syscalls. The two `System.Crypto.*` syscalls handle only signature verification against the current script container.
:::
