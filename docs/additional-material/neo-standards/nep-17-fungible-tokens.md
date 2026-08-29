---
title: "Standards and Contracts: NEP-17: Fungible Tokens"
description: "NEP-17: Fungible Tokens from Standards and Contracts."
---

# NEP-17: Fungible Tokens

[Back to Standards and Contracts](/additional-material/neo-standards)

NEP-17 is Neo's fungible token standard. It replaces the ERC-20 approve/allowance authorization model with cryptographic witness verification, eliminating entire classes of vulnerabilities (infinite approvals, front-running attacks) at the protocol level.

**Spec:** [NEP-17 Proposal](https://github.com/neo-project/proposals/blob/master/nep-17.mediawiki)

## Required Interface

| Method        | Parameters                                           | Return    | Safe | Description             |
| ------------- | ---------------------------------------------------- | --------- | ---- | ----------------------- |
| `symbol`      | —                                                    | `String`  | Yes  | Token symbol            |
| `decimals`    | —                                                    | `Integer` | Yes  | Decimal places (max 18) |
| `totalSupply` | —                                                    | `Integer` | Yes  | Total supply            |
| `balanceOf`   | `Hash160 account`                                    | `Integer` | Yes  | Account balance         |
| `transfer`    | `Hash160 from, Hash160 to, Integer amount, Any data` | `Boolean` | No   | Transfer tokens         |

Required event:

```
Transfer(Hash160 from, Hash160 to, Integer amount)
```

::: info The `Any` Type
The `data` parameter uses the Neo DevPack for Solidity `Any` type, which maps to NeoVM's unconstrained `StackItem`. This allows callers to pass any serializable value to the recipient's `onNEP17Payment` callback. In the devpack, `Any` is defined as `type Any is bytes;`.
:::

## Authorization Model

The most significant difference between ERC-20 and NEP-17 is the authorization model:

|                            | ERC-20 (Ethereum)                              | NEP-17 (Neo)                          |
| -------------------------- | ---------------------------------------------- | ------------------------------------- |
| **Auth mechanism**         | `msg.sender` + approve/allowance               | `Runtime.checkWitness(from)`          |
| **Delegated transfers**    | `approve()` → `transferFrom()`                 | Not needed — witness proves ownership |
| **Approval storage**       | On-chain `mapping(owner => spender => amount)` | None                                  |
| **Infinite approval risk** | Yes — common vulnerability                     | Eliminated by design                  |
| **Front-running attacks**  | Possible on `approve()`                        | Not applicable                        |

Neo transactions include **witness scopes** that cryptographically prove the caller controls an address. `Runtime.checkWitness(from)` verifies this at the VM level — no off-chain signatures, no nonce tracking, no approval state.

```solidity
// ERC-20: relies on msg.sender identity
function transfer(address to, uint256 amount) public returns (bool) {
    require(msg.sender == from); // implicit via msg.sender
    // ...
}

// NEP-17: cryptographic witness verification
function transfer(address from, address to, uint256 amount, Any calldata data)
    public returns (bool)
{
    require(Runtime.checkWitness(from), "NEP17: unauthorized");
    // ...
}
```

::: warning No approve/allowance
NEP-17 does not define `approve()`, `transferFrom()`, or `allowance()`. The devpack's `NEP17.sol` includes them as optional ERC-20 compatibility extensions, but they are **not** part of the NEP-17 spec and do not contribute to standard detection.
:::

## Payment Callbacks

When tokens are transferred to a contract, the receiving contract's `onNEP17Payment` callback is invoked automatically:

```solidity
interface INEP17Receiver {
    function onNEP17Payment(address from, uint256 amount, Any calldata data) external;
}
```

|                      | Solidity (EVM)             | Neo DevPack for Solidity (NeoVM)                  |
| -------------------- | -------------------------- | ------------------------------------- |
| **Callback**         | `receive()` / `fallback()` | `onNEP17Payment(from, amount, data)`  |
| **Trigger**          | ETH sent to contract       | NEP-17 tokens transferred to contract |
| **Failure behavior** | Transfer reverts           | Transfer reverts                      |
| **Data passing**     | `msg.data`                 | Explicit `data` parameter             |

::: tip
Any contract that receives NEP-17 tokens **must** implement `onNEP17Payment`. If the recipient is a contract and does not implement this callback, the transfer reverts with `NEP17InvalidReceiver`.
:::

## Implementation Example

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "standards/NEP17.sol";
import "libraries/Runtime.sol";

/// @title MyToken — A production NEP-17 fungible token
contract MyToken is NEP17 {
    using Runtime for *;

    address private _admin;

    constructor()
        NEP17(
            "My Token",     // name
            "MYT",          // symbol
            8,              // decimals
            100_000_000,    // initial supply (1 MYT = 1e8 units)
            0               // max supply (0 = unlimited)
        )
    {
        _admin = msg.sender;
    }

    /// @dev NEP-17 4-parameter transfer with witness authorization
    function transfer(
        address from,
        address to,
        uint256 amount,
        Any calldata data
    ) public override returns (bool) {
        require(Runtime.checkWitness(from), "MyToken: unauthorized");
        return super.transfer(from, to, amount, data);
    }

    /// @dev Callback for receiving NEP-17 tokens
    function onNEP17Payment(address from, uint256 amount, Any calldata data) external {
        // Accept all incoming token payments
        // Add custom logic here (e.g., token swap, staking deposit)
    }

    /// @dev Mint new tokens — restricted to admin with witness check
    function mintTokens(address to, uint256 amount) external {
        require(Runtime.checkWitness(_admin), "MyToken: not admin");
        mint(to, amount);
    }
}
```

::: info Manifest Output
Compiling this contract produces `supportedstandards: ["NEP-17"]` in the manifest. The compiler detects all 5 required methods (`symbol`, `decimals`, `totalSupply`, `balanceOf`, `transfer`) and confirms `ownerOf` is absent (distinguishing it from NEP-11).
:::

## ERC-20 Migration Checklist

1. Replace 2-param `transfer(to, amount)` with 4-param `transfer(from, to, amount, data)`
2. Replace `require(msg.sender == ...)` with `require(Runtime.checkWitness(...))`
3. Remove `approve()`, `transferFrom()`, `allowance()` — not needed in NEP-17 spec
4. Add `onNEP17Payment(from, amount, data)` callback for receiving tokens
5. Replace `receive()` / `fallback()` with `onNEP17Payment`
6. Use the `Any` type for the `data` parameter
7. Verify manifest shows `supportedstandards: ["NEP-17"]`
