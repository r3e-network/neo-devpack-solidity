---
title: "Deploying Contracts: Local Deployment with Neo-Express"
description: "Local Deployment with Neo-Express from Deploying Contracts."
---

# Local Deployment with Neo-Express

[Back to Deploying Contracts](/basics/deploying-contracts)

[Neo-Express](https://github.com/neo-project/neo-express) provides a local Neo N3 blockchain for development. It supports single-node chains, account management, contract deployment, and method invocation.

## Prerequisites

- Neo-Express installed (see [Installation](/basics/installing-the-compiler))
- `jq` for JSON inspection
- A compiled contract (`.nef` + `.manifest.json`)

## Step 1: Create a Local Chain

```bash
NEOXP=./build/dotnet-tools/neoxp

# Create a fresh single-node chain
$NEOXP create -f -o chain.neo-express
```

The `-f` flag forces overwrite if a chain file already exists.

## Step 2: Fund the Deployer Account

Neo-Express creates a `genesis` account with all NEO and GAS. Transfer GAS to the deployer:

```bash
$NEOXP transfer -i chain.neo-express 100 GAS genesis node1
```

This gives `node1` enough GAS to cover deployment and invocation costs.

## Step 3: Deploy the Contract

```bash
$NEOXP contract deploy -i chain.neo-express build/MyContract.nef node1
```

The output includes:

- `contract-hash` -- the deployed contract's script hash
- `tx-hash` -- the deployment transaction hash

To capture the contract hash programmatically:

```bash
DEPLOY_OUT="$($NEOXP contract deploy -i chain.neo-express build/MyContract.nef node1 -j)"
CONTRACT_HASH="$(echo "$DEPLOY_OUT" | jq -r '.["contract-hash"]')"
echo "Deployed at: $CONTRACT_HASH"
```

## Step 4: Invoke Methods

Create a `.neo-invoke.json` file for each invocation:

```json
{
    "contract": "0x<contract-hash>",
    "operation": "methodName",
    "args": [42, "hello"]
}
```

**Write invocation** (creates a transaction, costs GAS):

```bash
$NEOXP contract invoke -i chain.neo-express invoke.neo-invoke.json node1
```

**Read-only invocation** (no transaction, no GAS cost):

```bash
$NEOXP contract invoke -r -j -i chain.neo-express invoke.neo-invoke.json node1
```

The `-r` flag runs the invocation as a test (read-only). The `-j` flag outputs JSON for programmatic parsing.

## Step 5: Verify Transaction Results

After a write invocation, inspect the transaction's application log:

```bash
TX_HASH="0x<transaction-hash>"
$NEOXP show transaction -i chain.neo-express "$TX_HASH"
```

Check the VM state and notifications:

```bash
APP_LOG="$($NEOXP show transaction -i chain.neo-express "$TX_HASH")"

# VM state should be "HALT" (success)
echo "$APP_LOG" | jq '.["application-log"].executions[0].vmstate'

# Check emitted events
echo "$APP_LOG" | jq '.["application-log"].executions[0].notifications'
```
