---
title: "Devpack Overview: Usage"
description: "Usage from Devpack Overview."
---

# Usage

[Back to Devpack Overview](/additional-material/neo-devpack)

## Compiling with the devpack

Pass the devpack root as an include path:

```bash
neo-solc MyContract.sol -I devpack -O2 -o build/MyContract
```

## Importing in Solidity

```solidity
import "contracts/NativeCalls.sol";
import "contracts/compat/EVMNativeAssetAdapter.sol";
import "libraries/Storage.sol";
import "standards/NEP17.sol";
```

All imports resolve relative to the `-I` include path. No package manager or remappings required.
