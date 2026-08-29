---
title: "Runtime Specification: Native Contract Integration"
description: "Native Contract Integration from Runtime Specification."
---

# Native Contract Integration

[Back to Runtime Specification](/internals/runtime-specification)

The embedded runtime includes a registry of Neo N3 native contracts:

| Contract           | Description                         |
| ------------------ | ----------------------------------- |
| NEO                | NEO governance token                |
| GAS                | GAS utility token                   |
| ContractManagement | Deploy, update, and query contracts |
| Policy             | Network fee policies                |
| Oracle             | Oracle request service              |
| RoleManagement     | Node role management                |
| Notary             | Notary service                      |
| Treasury           | Treasury management                 |
| Ledger             | Blockchain data access              |
| CryptoLib          | Cryptographic functions             |
| StdLib             | Standard library utilities          |

## ContractManagement

The embedded runtime maintains a stateful in-memory contract registry that supports:

- `Deploy` -- register a new contract with NEF and manifest
- `Update` -- update an existing contract (increments update counter)
- `GetContract` -- query contract metadata by hash

This registry persists within a single test execution and is used by both syscall-based and native contract calls.
