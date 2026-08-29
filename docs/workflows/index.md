---
title: Workflows
description: End-to-end developer workflows for compiling, deploying, testing, debugging, and shipping Solidity contracts on Neo N3.
---

# Workflows

End-to-end developer workflows for the Neo DevPack for Solidity. Each
section threads together the steps you need from a fresh checkout to a
production-deployed contract.

## Build & Compile

- [Installing the compiler](/basics/installing-the-compiler) — get
  `neo-solc` onto your machine.
- [Quickstart](/basics/quickstart) — compile a one-page contract and
  inspect its NEF + manifest output.
- [Using the compiler](/compiler/using-the-compiler) — full CLI
  reference including standard JSON, optimization levels, and source
  maps.
- [Analysing compiler output](/compiler/analysing-the-compiler-output) —
  read the NEF, the ABI, the manifest, and the assembly listing.

## Deploy

- [Deploying contracts](/basics/deploying-contracts) — TestNet and
  MainNet deploy walk-through with `neo-cli` and `neo-express`.
- [Famous contracts on neo-express](/solidity/famous-contracts-neoxp-deploy) —
  deploy matrix verifying the most common production contracts.
- [Standards mirror — TestNet deployments](/standards-mirror/deployments/RESULTS) —
  47 live ERC ↔ Neo pairs with deployed addresses and assertion results.

## Test & Verify

- [Testing contracts](/basics/testing-contracts) — assertion patterns
  and replay loops.
- [Fuzz testing](/compiler/fuzz-testing) — the project's fuzz harness
  and its growing corpus.
- [TestNet runtime verification](/solidity/famous-contracts-testnet-runtime) —
  invocation matrix for live contracts.

## Debug & Diagnose

- [Troubleshooting](/advisory-content/troubleshooting) — common
  symptoms and their fixes.
- [Error reference](/advisory-content/error-reference) — full diagnostic
  catalog with cause and remediation.
- [Known bugs](/advisory-content/known-bugs) — open issues being tracked.

## Ship to Production

- [Production readiness checklist](/advisory-content/production-readiness) —
  pre-deploy gate covering security, manifests, witness scopes, and
  rollback strategy.
- [Security considerations](/advisory-content/security-considerations) —
  patterns and anti-patterns for Neo-specific risks.
- [Breaking changes](/advisory-content/breaking-changes) — migration
  notes between compiler versions.

## See Also

- [Use cases](/use-cases) — what's already shipped on Neo using this
  compiler.
- [Standards mirror](/standards-mirror/) — ERC ↔ NEP migration map.
