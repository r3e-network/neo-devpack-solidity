---
title: "Deploying Contracts: Manifest Verification Before Deploy"
description: "Manifest Verification Before Deploy from Deploying Contracts."
---

# Manifest Verification Before Deploy

[Back to Deploying Contracts](/basics/deploying-contracts)

## Overview

Before deploying to TestNet or MainNet, verify the manifest:

```bash
# Check contract name
jq '.name' build/MyContract.manifest.json

# Check permissions are not overly broad
jq '.permissions' build/MyContract.manifest.json

# Check supported standards
jq '.supportedstandards' build/MyContract.manifest.json

# Check all method signatures
jq '.abi.methods[] | {name, parameters: [.parameters[].type], returntype, safe}' \
  build/MyContract.manifest.json

# Check events
jq '.abi.events[] | {name, parameters: [.parameters[].type]}' \
  build/MyContract.manifest.json
```

::: tip
For production contracts, always compile with `--deny-wildcard-contracts --deny-wildcard-methods` to ensure the manifest has minimal permissions. See [Compile Workflow](/compiler/analysing-the-compiler-output) for details.
:::
