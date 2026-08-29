---
title: "Troubleshooting: Gas and Execution Issues"
description: "Gas and Execution Issues from Troubleshooting."
---

# Gas and Execution Issues

[Back to Troubleshooting](/advisory-content/troubleshooting)

## Insufficient GAS at Runtime

```
error: execution failed: insufficient GAS
```

**Cause:** The transaction's system fee doesn't cover the execution cost.

**Diagnosis:**

1. Check gas consumption with `gasleft()`:

   ```solidity
   function expensiveOperation() external {
       uint256 gasBefore = gasleft();
       // ... operation ...
       uint256 gasUsed = gasBefore - gasleft();
       Runtime.log(string(abi.encodePacked("Gas used: ", gasUsed)));
   }
   ```

2. Use the `withGasLimit` modifier from `FrameworkBase`:
   ```solidity
   function criticalOp() external withGasLimit(50000000) {
       // Reverts early if < 0.5 GAS remaining
   }
   ```

**Common gas-heavy operations:**

| Operation                      | Approximate Cost       |
| ------------------------------ | ---------------------- |
| `System.Storage.Put` (new key) | 200,000+ GAS units     |
| `System.Storage.Put` (update)  | 100,000+ GAS units     |
| `System.Contract.Call`         | 32,768 GAS units       |
| `CryptoLib.verifyWithECDsa`    | 1,000,000+ GAS units   |
| Large array iteration          | Scales with array size |

::: warning
Neo GAS uses 10^8 decimals (not 10^18 like Ethereum). `1 GAS = 100,000,000` fractional units. Adjust your constants accordingly.
:::

---
