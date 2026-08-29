---
title: "Runtime Specification: Embedded Runtime vs Production Neo N3"
description: "Embedded Runtime vs Production Neo N3 from Runtime Specification."
---

# Embedded Runtime vs Production Neo N3

[Back to Runtime Specification](/internals/runtime-specification)

## What the Embedded Runtime Provides

- Full NeoVM opcode execution with correct stack semantics
- Storage read/write/find with iterator support
- Event notification capture
- Gas tracking (approximate)
- Exception handling with try/catch/finally
- Native contract call routing
- Crypto operations (SHA256, RIPEMD160, Keccak256, CheckSig)

## Differences from Production

| Aspect                 | Embedded Runtime                | Production Neo N3           |
| ---------------------- | ------------------------------- | --------------------------- |
| Storage                | In-memory, ephemeral            | Persistent on-chain         |
| Gas costs              | Approximate static tables       | Exact per specification     |
| Blockchain data        | Stub/placeholder values         | Real chain state            |
| Signature verification | Best-effort secp256k1           | Full Neo crypto stack       |
| Iterator behavior      | Materialized (eager)            | Streaming (lazy)            |
| ByteString vs Buffer   | Both treated as byte array      | Distinct mutation semantics |
| Oracle                 | Returns deterministic pseudo ID | Real oracle network         |
| Network state          | Fixed test values               | Live network data           |

## When to Use Neo-Express

Use the embedded runtime for:

- Unit testing individual contract functions
- Verifying compilation output correctness
- Quick iteration during development

Use Neo-Express or testnet for:

- Integration testing with multiple contracts
- Accurate gas measurement
- Testing with real blockchain state
- Verifying oracle and cross-contract interactions
- Pre-mainnet validation
