---
title: "Production Readiness: Deployment Rehearsal"
description: "Deployment Rehearsal from Production Readiness."
---

# Deployment Rehearsal

[Back to Production Readiness](/advisory-content/production-readiness)

Follow this progression to catch issues at each stage:

## Stage 1: Local Neo-Express

```bash
# Create a fresh chain
NEOXP=./build/dotnet-tools/neoxp
$NEOXP create -f -o chain.neo-express
$NEOXP transfer -i chain.neo-express 100 GAS genesis node1

# Deploy
$NEOXP contract deploy -i chain.neo-express build/contract.nef node1

# Test every public method
# Test every event emission
# Test error conditions (require failures, access control)
# Test constructor arguments if applicable
# Test contract update if applicable
```

## Stage 2: Neo N3 TestNet

1. Obtain test GAS from the Neo TestNet faucet.
2. Deploy the exact same `.nef` and `.manifest.json` files.
3. Run the same test scenarios as local.
4. Verify transaction results on a Neo block explorer.
5. Test with multiple accounts and concurrent transactions.

## Stage 3: Neo N3 MainNet

1. Verify the contract hash matches your prediction.
2. Deploy with a funded MainNet account.
3. Start with small-value transactions to verify behavior.
4. Monitor the contract for unexpected behavior.
