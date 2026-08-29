---
title: "Troubleshooting: Constructor and Deployment Issues"
description: "Constructor and Deployment Issues from Troubleshooting."
---

# Constructor and Deployment Issues

[Back to Troubleshooting](/advisory-content/troubleshooting)

## Constructor Parameter Mismatch

```
error[E2015]: constructor parameter type mismatch
  --> MyToken.sol:8:5
   |
8  |     NEP17("My Token", "MYT", 8, 100000000, 0)
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   = note: expected (string, string, uint8, uint256, uint256)
```

**Cause:** The base contract constructor expects specific parameter types that don't match what you're passing.

**Solution:** Check the base contract's constructor signature in the devpack:

```solidity
// NEP17 constructor signature:
constructor(
    string memory name_,    // Token name
    string memory symbol_,  // Token symbol
    uint8 decimals_,        // Decimal places (max 18)
    uint256 initialSupply_, // Initial supply in smallest units
    uint256 maxSupply_      // Max supply (0 = unlimited)
)

// NEP11 constructor signature:
constructor(
    string memory name_,    // Collection name
    string memory symbol_,  // Collection symbol
    uint8 decimals_,        // 0 for indivisible, >0 for divisible
    string memory baseURI_, // Base URI for metadata
    uint256 maxSupply_,     // Max supply
    bool divisible_         // Whether tokens are divisible
)
```

## Deployment Fails on Neo-Express

```
error: deployment failed: insufficient GAS for deployment
```

**Solutions:**

1. Ensure your account has enough GAS:

   ```bash
   neo-express transfer GAS genesis alice 1000
   ```

2. Check the NEF file size — large contracts cost more to deploy:

   ```bash
   ls -la build/MyContract/MyContract.nef
   # If > 512KB, consider splitting into multiple contracts
   ```

3. Verify Neo-Express is running:
   ```bash
   neo-express run
   ```

---
