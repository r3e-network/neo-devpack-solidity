---
title: "Deploying Contracts: Neo N3 Deployment Model"
description: "Neo N3 Deployment Model from Deploying Contracts."
---

# Neo N3 Deployment Model

[Back to Deploying Contracts](/basics/deploying-contracts)

## Overview

On Neo N3, contract deployment is a transaction that calls the native `ContractManagement.deploy` method. The transaction payload includes:

1. The NEF binary (bytecode + method tokens + checksum)
2. The manifest JSON (ABI + permissions + standards)
3. Optional deployment data (passed to the contract's `_deploy` method)

The Neo node validates the manifest, computes a contract hash from the sender address + NEF checksum + manifest name, and stores the contract on-chain. After deployment, the contract is callable by its hash.

::: info Contract Hash
The contract hash is deterministic: it is derived from `hash160(sender_scripthash + nef_checksum + contract_name)`. You can predict it before deployment using `neo-solc --deployer <sender_hash>`.
:::
