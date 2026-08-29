---
title: "Quickstart: Step 1: Write a Contract"
description: "Step 1: Write a Contract from Quickstart."
---

# Step 1: Write a Contract

[Back to Quickstart](/basics/quickstart)

## Contract Source

Create a file called `MyStorage.sol`:

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.34;

/// @title MyStorage - A simple storage contract for Neo N3
/// @notice Stores and retrieves a single integer value
contract MyStorage {
    uint256 private value;

    event ValueChanged(uint256 indexed oldValue, uint256 indexed newValue, address indexed setter);

    /// @notice Store a new value
    /// @param newValue The value to store
    function set(uint256 newValue) public {
        uint256 oldValue = value;
        value = newValue;
        emit ValueChanged(oldValue, newValue, msg.sender);
    }

    /// @notice Retrieve the stored value
    /// @return The current stored value
    function get() public view returns (uint256) {
        return value;
    }

    /// @notice Increment the stored value by one
    /// @return The new value after incrementing
    function increment() public returns (uint256) {
        value += 1;
        return value;
    }
}
```

## What It Demonstrates

This contract demonstrates the core patterns you will use in Neo DevPack for Solidity:

- **State variables** (`value`) persist in Neo Storage.
- **Events** (`ValueChanged`) map to `Runtime.Notify` on Neo.
- **`msg.sender`** maps to `Runtime.GetCallingScriptHash()`.
- **`view` functions** are marked `safe` in the Neo manifest, meaning they can be invoked without a transaction.
