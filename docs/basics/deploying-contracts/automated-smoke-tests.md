---
title: "Deploying Contracts: Automated Smoke Tests"
description: "Automated Smoke Tests from Deploying Contracts."
---

# Automated Smoke Tests

[Back to Deploying Contracts](/basics/deploying-contracts)

## Smoke Test Commands

The repository ships 16+ deployment smoke tests that validate end-to-end behavior:

```bash
# Basic deploy + invoke
make test-deploy-smoke

# Constructor arguments
make test-deploy-constructor-smoke

# Contract update lifecycle
make test-deploy-update-smoke

# Manifest permissions for native contracts
make test-deploy-permissions-smoke

# abi.encode / abi.decode round-trip
make test-deploy-encoding-smoke

# Revert reason propagation
make test-deploy-abortmsg-smoke

# Low-level calls
make test-deploy-lowlevel-call-smoke

# External member calls
make test-deploy-external-call-smoke

# View/ReadOnly calls
make test-deploy-view-readonly-call-smoke

# All smoke tests
make test-deploy-smoke-full
```

## Execution Model

Each script creates a fresh Neo-Express chain, deploys a purpose-built contract, invokes methods, and validates results. They are safe to run repeatedly and clean up after themselves.
