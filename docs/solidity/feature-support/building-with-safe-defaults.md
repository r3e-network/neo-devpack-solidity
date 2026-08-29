---
title: "Solidity Feature Support: Building with Safe Defaults"
description: "Building with Safe Defaults from Solidity Feature Support."
---

# Building with Safe Defaults

[Back to Solidity Feature Support](/solidity/feature-support)

## Overview

Always compile with strict flags in production to avoid unintended wildcard permissions:

```bash
neo-solc MyContract.sol \
  --deny-wildcard-contracts \
  --deny-wildcard-methods \
  -o build/MyContract
```
