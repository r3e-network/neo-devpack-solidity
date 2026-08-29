---
title: Reference
description: Canonical reference material — Solidity feature support matrix, error catalog, runtime spec, NeoVM parity status, and the ERC ↔ Neo standards mirror.
---

# Reference

Canonical reference for the Neo DevPack for Solidity. The pages below
are the source-of-truth tables you check when you need a definitive
answer about what's supported, what an error means, or how a Solidity
construct maps to NeoVM.

## Compiler & Language

- [EVM feature support matrix](/solidity/feature-support) — every
  Solidity 0.8.x feature with its Neo support status (full / partial /
  unsupported / lowered).
- [Language grammar](/language-description/grammar) — the formal
  grammar accepted by `neo-solc`.
- [Cheatsheet](/language-description/cheatsheet) — quick-lookup for
  syntax, operators, and globals.
- [Keyword index](/resources/keyword-index) — every reserved word and
  built-in identifier with a link to its definition.

## Diagnostics & Errors

- [Error reference](/advisory-content/error-reference) — every
  diagnostic the compiler emits, with cause and remediation.
- [Known bugs](/advisory-content/known-bugs) — currently-open issues.
- [Breaking changes](/advisory-content/breaking-changes) — version-to-version
  migration notes.

## Runtime & VM

- [Runtime specification](/internals/runtime-specification) — the
  semantics that compiled code relies on.
- [Native contracts](/internals/native-contracts) — every native Neo
  contract with its method-by-method behavior.
- [Syscalls](/internals/syscalls) — full syscall catalog.
- [Architecture](/internals/architecture) — compiler + runtime layout.
- [NeoVM parity & limitations](/internals/parity-and-limitations) —
  EVM constructs that don't have a clean NeoVM mapping.

## Standards & Mirror

- [Standards mirror — overview](/standards-mirror/) — entry point.
- [Coverage matrix](/standards-mirror/coverage-matrix) — every
  mirrored ERC/EIP with category, status, mapping, and live deploy
  state.
- [Coverage audit & gap report](/standards-mirror/coverage-audit) —
  what's covered, what's missing, and the conventions used in the
  Neo C# samples.
- [TestNet deployments](/standards-mirror/deployments/RESULTS) — 47
  live ERC ↔ Neo pairs with addresses and assertion results.
- [Deferred deployments](/standards-mirror/deployments/DEFERRED) —
  catalog-only entries with deferral rationale.

## Standards by Category

- [Token standards (NEP-11 / NEP-17)](/standards-mirror/tokens)
- [Account & authentication](/standards-mirror/account-and-auth)
- [Infrastructure & patterns](/standards-mirror/infrastructure)
- [DeFi building blocks](/standards-mirror/defi)
- [Protocol-level EIPs](/standards-mirror/protocol-eips)
- [Neo standards (NEP) reference](/additional-material/neo-standards)

## Other

- [Production readiness checklist](/advisory-content/production-readiness)
- [Use cases](/use-cases) — what's already shipped on Neo with this
  compiler.
- [Common patterns](/resources/common-patterns) — idioms across token,
  governance, and proxy standards.
