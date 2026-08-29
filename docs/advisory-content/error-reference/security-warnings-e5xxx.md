---
title: "Error Reference: Security Warnings (E5xxx)"
description: "Security Warnings (E5xxx) from Error Reference."
---

# Security Warnings (E5xxx)

[Back to Error Reference](/advisory-content/error-reference)

| Code    | Name                | Description                        | Recommendation                                    |
| ------- | ------------------- | ---------------------------------- | ------------------------------------------------- |
| `E5001` | ReentrancyRisk      | Potential reentrancy vulnerability | Use checks-effects-interactions pattern           |
| `E5002` | UncheckedCall       | Unchecked external call            | Check return value of external calls              |
| `E5003` | TxOriginUsage       | `tx.origin` used for authorization | Use `msg.sender` instead                          |
| `E5004` | UnsafeDelegate      | Unsafe delegatecall                | `delegatecall` is blocked on NeoVM; refactor      |
| `E5005` | IntegerOverflowRisk | Potential integer overflow         | Use SafeMath or Solidity 0.8.x checked arithmetic |

::: warning
Security warnings (E5xxx) should be treated as errors in production builds. Use `--Werror E5` to enforce this in your CI pipeline.
:::

## Example: E5001 ReentrancyRisk

```
warning[E5001]: potential reentrancy vulnerability in function 'withdraw'
  --> MyContract.sol:20:5
   |
20 |     payable(msg.sender).transfer(balance);
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: move state changes before external calls (checks-effects-interactions pattern)
```

**Fix:** Reorder operations so state changes happen before external calls:

```solidity
// Before (vulnerable)
function withdraw() public {
    uint256 balance = balances[msg.sender];
    payable(msg.sender).transfer(balance);  // external call first
    balances[msg.sender] = 0;               // state change after
}

// After (safe)
function withdraw() public {
    uint256 balance = balances[msg.sender];
    balances[msg.sender] = 0;               // state change first
    payable(msg.sender).transfer(balance);  // external call after
}
```
