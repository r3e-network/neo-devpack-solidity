---
title: Manifest
description: Neo N3 contract manifest format — ABI, permissions, supportedstandards, trusts, and the metadata your compiled contract advertises on chain.
---

# Manifest

Every Neo N3 contract ships with a `*.manifest.json` file alongside its
`*.nef` bytecode. The manifest is the contract's self-description — what
it exposes, what it can call, what standards it claims to implement, and
what witness scopes it accepts. Wallets, explorers, and other contracts
read the manifest before invoking your code.

## Structure

- [Contract metadata](/internals/contract-metadata) — the high-level
  shape: name, ABI, permissions, trusts, supportedstandards, extra,
  features, and groups.
- [Contract ABI specification](/internals/contract-abi-specification) —
  the canonical wire-format for methods, parameters, return types, and
  events.

## Permissions & Trust

- [Permission-conscious development](/additional-material/neo-devpack/permission-conscious-development) —
  authoring permissions that don't over-grant.
- [Native contract hash reference](/additional-material/neo-devpack/native-contract-hash-reference) —
  the well-known hashes you'll declare as permissions.
- [Core contracts](/additional-material/neo-devpack/core-contracts) —
  GAS, NEO, ContractManagement, Policy, RoleManagement, Oracle.

## Standards Declaration

- [Supported standards](/additional-material/neo-standards/supported-standards) —
  declaring NEP-11, NEP-17, NEP-24, NEP-26, NEP-27, NEP-29, NEP-30 in
  the manifest.
- [Interface detection — EIP-165 vs Manifest](/additional-material/neo-standards/interface-detection-eip-165-vs-manifest) —
  why Neo uses `supportedstandards` instead of EIP-165 reflection.
- [Standards auto-detection](/additional-material/neo-standards/standards-auto-detection) —
  how the compiler infers manifest entries from your code.
- [Compiler diagnostics for standards](/additional-material/neo-standards/compiler-diagnostics-for-standards) —
  warnings and errors when the manifest disagrees with the
  implementation.
- [Testing standards compliance](/additional-material/neo-standards/testing-standards-compliance) —
  assertion harness for verifying NEP conformance.

## Manifest in the Compiler

- [Compiler intrinsics](/additional-material/neo-devpack/compiler-intrinsics) —
  attributes (`[Safe]`, `[DisplayName]`, `[ContractPermission]`,
  `[SupportedStandards]`) that the compiler bakes into the manifest.
- [NatSpec format](/additional-material/natspec-format) — comments that
  flow through to the manifest's `extra` field.

## See Also

- [Standards mirror](/standards-mirror/) — every ERC's mirror page
  shows the canonical manifest declarations for that standard.
- [Reference — keyword index](/resources/keyword-index) — find
  attributes by name.
