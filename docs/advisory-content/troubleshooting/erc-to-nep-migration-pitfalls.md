---
title: "Troubleshooting: ERC to NEP Migration Pitfalls"
description: "ERC to NEP Migration Pitfalls from Troubleshooting."
---

# ERC to NEP Migration Pitfalls

[Back to Troubleshooting](/advisory-content/troubleshooting)

## Common ERC-20 → NEP-17 Mistakes

1. **Keeping 2-parameter transfer:**

   ```solidity
   // ❌ ERC-20 style — detected as a near miss and warned, not canonical NEP-17
   function transfer(address to, uint256 amount) public returns (bool) { ... }

   // ✅ NEP-17 style — 4 parameters
   function transfer(address from, address to, uint256 amount, Any calldata data)
       public returns (bool) { ... }
   ```

2. **Using msg.sender for authorization:**

   ```solidity
   // ❌ EVM pattern
   require(msg.sender == from, "unauthorized");

   // ✅ Neo pattern — cryptographic witness verification
   require(Runtime.checkWitness(from), "unauthorized");
   ```

3. **Keeping approve/allowance:**

   ```solidity
   // ❌ Not part of NEP-17 spec — compiler warns W103
   function approve(address spender, uint256 amount) public { ... }

   // ✅ Remove — witness model replaces approvals entirely
   ```

4. **Missing onNEP17Payment callback:**

   ```solidity
   // ❌ EVM pattern
   receive() external payable { ... }

   // ✅ NEP-17 callback — required for contracts receiving tokens
   function onNEP17Payment(address from, uint256 amount, Any calldata data) external { ... }
   ```

## Common ERC-721 → NEP-11 Mistakes

1. **Using uint256 token IDs instead of bytes32**
2. **Keeping transferFrom instead of 3-param transfer**
3. **Missing `tokensOf()` or metadata strategy**
4. **Missing `decimals()` returning 0 for indivisible NFTs**

---
