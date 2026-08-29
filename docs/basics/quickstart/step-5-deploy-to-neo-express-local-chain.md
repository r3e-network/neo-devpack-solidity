---
title: "Quickstart: Step 5: Deploy to Neo-Express (Local Chain)"
description: "Step 5: Deploy to Neo-Express (Local Chain) from Quickstart."
---

# Step 5: Deploy to Neo-Express (Local Chain)

[Back to Quickstart](/basics/quickstart)

Neo-Express provides a local Neo N3 blockchain for testing. This section walks through a manual deployment.

## 5a. Create a Local Chain

```bash
NEOXP=./build/dotnet-tools/neoxp

# Create a fresh single-node chain
$NEOXP create -f -o chain.neo-express

# Transfer GAS to the deployer account
$NEOXP transfer -i chain.neo-express 100 GAS genesis node1
```

## 5b. Deploy the Contract

```bash
$NEOXP contract deploy -i chain.neo-express build/MyStorage.nef node1
```

The output includes the contract hash and transaction hash. Note the contract hash -- you will need it to invoke methods.

## 5c. Invoke a Method (Write)

Create an invocation file `invoke-set.neo-invoke.json`:

```json
{
    "contract": "0x<your-contract-hash>",
    "operation": "set",
    "args": [42]
}
```

Invoke it:

```bash
$NEOXP contract invoke -i chain.neo-express invoke-set.neo-invoke.json node1
```

## 5d. Invoke a Method (Read)

Create `invoke-get.neo-invoke.json`:

```json
{
    "contract": "0x<your-contract-hash>",
    "operation": "get",
    "args": []
}
```

Invoke with `-r` for read-only (no transaction, no GAS cost):

```bash
$NEOXP contract invoke -r -j -i chain.neo-express invoke-get.neo-invoke.json node1
```

Expected output:

```json
{
    "state": "HALT",
    "stack": [
        {
            "type": "Integer",
            "value": "42"
        }
    ]
}
```

## 5e. Run the Automated Smoke Test

Instead of manual steps, you can run the repository's built-in smoke test:

```bash
make test-deploy-smoke
```

This script automatically:

1. Builds `neo-solc` if needed
2. Compiles a test contract
3. Creates a fresh Neo-Express chain
4. Deploys the contract
5. Invokes `set(7)`, verifies the `ValueSet` event
6. Invokes `get()`, verifies it returns `7`
7. Invokes `height()`, verifies it returns an integer (tests native contract calls)
