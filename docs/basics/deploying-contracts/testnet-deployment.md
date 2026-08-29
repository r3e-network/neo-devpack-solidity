---
title: "Deploying Contracts: TestNet Deployment"
description: "TestNet Deployment from Deploying Contracts."
---

# TestNet Deployment

[Back to Deploying Contracts](/basics/deploying-contracts)

## Prerequisites

- Neo CLI (`neo-cli`) installed and synced with Neo N3 TestNet
- A funded TestNet account (get test GAS from the Neo TestNet faucet)

## Deploy via neo-cli

```bash
neo-cli contract deploy build/MyContract.nef build/MyContract.manifest.json
```

For contracts with constructor arguments:

```bash
neo-cli contract deploy build/MyContract.nef build/MyContract.manifest.json -- [1000]
```

## Verify Deployment

```bash
# Invoke a view method to confirm the contract is live
neo-cli contract invoke <contract-hash> getNumber
```
