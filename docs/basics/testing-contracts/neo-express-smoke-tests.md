---
title: "Testing Contracts: Neo-Express Smoke Tests"
description: "Neo-Express Smoke Tests from Testing Contracts."
---

# Neo-Express Smoke Tests

[Back to Testing Contracts](/basics/testing-contracts)

Smoke tests deploy contracts to a real local Neo-Express chain and invoke methods to validate on-chain behavior. Each script is self-contained: it creates a fresh chain, deploys, invokes, and cleans up.

## Prerequisites

- Neo-Express installed (see [Installation](/basics/installing-the-compiler))
- `jq` and `hexdump` available
- `neo-solc` built (scripts auto-build if needed)

## Individual Smoke Tests

```bash
# Basic deploy + invoke (storage, events, native calls)
make test-deploy-smoke

# Constructor arguments via _deploy(data, update)
make test-deploy-constructor-smoke

# Contract update lifecycle (deploy v1, update to v2)
make test-deploy-update-smoke

# Manifest permissions for native contracts (StdLib, CryptoLib)
make test-deploy-permissions-smoke

# abi.encode / abi.decode round-trip (StdLib.serialize/deserialize)
make test-deploy-encoding-smoke

# Revert reason propagation (abort message)
make test-deploy-abortmsg-smoke

# Low-level call (System.Contract.Call)
make test-deploy-lowlevel-call-smoke

# Low-level call failure handling
make test-deploy-lowlevel-call-failure-smoke

# External member-call dispatch
make test-deploy-external-call-smoke

# View/ReadOnly external calls
make test-deploy-view-readonly-call-smoke

# Compound assignment operators (+=, -=, etc.)
make test-deploy-compound-assignment-smoke

# Struct array element access
make test-deploy-struct-array-element-smoke

# Nested struct storage
make test-deploy-nested-struct-smoke

# Delete operator
make test-deploy-delete-smoke

# New showcase contracts (UpgradeLifecycle, WitnessGuard, OracleRelay)
make test-deploy-new-showcases-smoke
```

## Run All Smoke Tests

```bash
make test-deploy-smoke-full
```

This runs all 16+ individual smoke tests sequentially. Each test is independent and creates its own temporary chain.

## Famous DeFi Contract Smoke Tests

The repository also includes smoke tests for ports of iconic DeFi protocols:

```bash
# Wrapped GAS (WETH9-style)
make test-deploy-wgas-smoke

# Flash loan (Aave V2-style)
make test-deploy-flashloan-smoke

# AMM (Uniswap V2-style)
make test-deploy-amm-smoke

# Token vesting (OpenZeppelin-style)
make test-deploy-vesting-smoke

# Lending (Compound-style)
make test-deploy-lending-smoke

# DAO (Governor-style)
make test-deploy-dao-smoke

# All famous contract tests
make test-deploy-famous-all
```

## What Each Smoke Test Validates

The basic deploy smoke test (`test_neoxp_deploy.sh`) is representative of the pattern:

1. Resolves `neo-solc` binary (builds if needed)
2. Resolves `neoxp` binary (installs if needed)
3. Creates a temporary working directory
4. Writes a test contract inline
5. Compiles with `neo-solc`
6. Creates a fresh Neo-Express chain
7. Transfers GAS to the deployer
8. Deploys the contract
9. Invokes `sender()` and verifies it returns the deployer's script hash
10. Invokes `set(7)` and verifies the `ValueSet` event fires
11. Invokes `get()` and verifies it returns `7`
12. Invokes `height()` and verifies it returns an integer (tests native contract calls)
13. Cleans up the temporary directory
