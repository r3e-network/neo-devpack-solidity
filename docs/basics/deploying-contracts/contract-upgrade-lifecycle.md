---
title: "Deploying Contracts: Contract Upgrade Lifecycle"
description: "Contract Upgrade Lifecycle from Deploying Contracts."
---

# Contract Upgrade Lifecycle

[Back to Deploying Contracts](/basics/deploying-contracts)

Neo N3 supports upgrading deployed contracts through the native `ContractManagement` system. The lifecycle is:

## Deploy

Initial deployment creates the contract on-chain:

```bash
$NEOXP contract deploy -i chain.neo-express build/MyContract.nef node1
```

## Update

Update replaces the contract's NEF and manifest while preserving its storage and hash:

```bash
$NEOXP contract update -i chain.neo-express "$CONTRACT_HASH" build/MyContractV2.nef node1
```

When updating:

- The `_deploy(data, update)` method is called with `update = true`.
- The Solidity constructor logic is skipped (only runs on initial deploy).
- Storage is preserved -- existing state variables retain their values.
- The contract hash remains the same.

## Destroy

Remove a contract from the chain (irreversible):

```bash
# Via devpack: ContractManagement.destroy() in your contract code
# Or via neo-cli / SDK calls to ContractManagement
```

::: danger
`destroy` permanently removes the contract and its storage. This cannot be undone. Only include destroy functionality if your contract genuinely needs it, and protect it with proper access control.
:::

## Update Smoke Test

The repository includes an automated update smoke test:

```bash
make test-deploy-update-smoke
```

This script:

1. Deploys a v1 contract with constructor args
2. Updates it to v2
3. Verifies that the constructor did not re-run on update
4. Verifies that new methods from v2 are callable
