---
title: "Devpack Overview: Building Custom Contracts"
description: "Building Custom Contracts from Devpack Overview."
---

# Building Custom Contracts

[Back to Devpack Overview](/additional-material/neo-devpack)

## Contract Source

Minimal contract using devpack features:

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "contracts/NativeCalls.sol";
import "libraries/Storage.sol";
import "libraries/Runtime.sol";
import "contracts/FrameworkBase.sol";

contract Vault is FrameworkBase {
    event Deposit(address indexed from, uint256 amount);
    event Withdraw(address indexed to, uint256 amount);

    function deposit() external withWitness {
        uint256 balance = NativeCalls.gasBalanceOf(address(this));
        Storage.put("balance", abi.encode(balance));
        emit Deposit(msg.sender, balance);
    }

    function withdraw(address to, uint256 amount) external onlyOwner withWitness {
        Runtime.requireWitness(to);
        bool ok = NativeCalls.gasTransfer(address(this), to, amount, "");
        require(ok, "Vault: transfer failed");

        uint256 remaining = NativeCalls.gasBalanceOf(address(this));
        Storage.put("balance", abi.encode(remaining));
        emit Withdraw(to, amount);
    }

    function getBalance() external view returns (uint256) {
        bytes memory data = Storage.get("balance");
        if (data.length == 0) return 0;
        return abi.decode(data, (uint256));
    }
}
```

## Compile and Deploy

Compile and deploy:

```bash
# Compile with strict permissions
neo-solc Vault.sol -I devpack -O2 \
  --deny-wildcard-contracts \
  --deny-wildcard-methods \
  -o build/Vault

# Inspect generated manifest permissions
cat build/Vault/Vault.manifest.json | jq '.permissions'

# Deploy to testnet
neo-solc deploy build/Vault --network testnet
```
