---
title: "Quickstart: Step 7: Constructor Arguments"
description: "Step 7: Constructor Arguments from Quickstart."
---

# Step 7: Constructor Arguments

[Back to Quickstart](/basics/quickstart)

## Constructor Model

If your contract has a parameterized constructor, Neo handles it through the `_deploy(data, update)` entry point. The compiler automatically generates this entry point.

## Example Contract

Example contract with a constructor:

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.34;

contract Counter {
    uint256 private count;

    constructor(uint256 initialCount) {
        count = initialCount;
    }

    function get() public view returns (uint256) {
        return count;
    }
}
```

## Compile

Compile:

```bash
./target/release/neo-solc Counter.sol -I devpack -O2 -o build/Counter
```

## Deploy with Arguments

Deploy with constructor arguments via Neo-Express:

```bash
$NEOXP contract deploy -i chain.neo-express -d '[100]' build/Counter.nef node1
```

The `-d '[100]'` flag passes a JSON array as the `data` parameter to `_deploy`. The compiler's deploy stub deserializes this array and passes the values to your Solidity constructor.

::: warning Manifest Permissions
Contracts with parameterized constructors require `StdLib.jsonDeserialize` and `StdLib.deserialize` permissions in the manifest. The compiler adds these automatically.
:::
