---
title: "Troubleshooting: Manifest and Permission Issues"
description: "Manifest and Permission Issues from Troubleshooting."
---

# Manifest and Permission Issues

[Back to Troubleshooting](/advisory-content/troubleshooting)

## Wildcard Permission Rejection

```
error: wildcard contract permission detected
  --> manifest generation
   |
   = note: contract call to dynamic target requires {"contract":"*","methods":"*"}
   = help: use --manifest-permissions to supply explicit overrides, or remove --deny-wildcard-contracts
```

**Cause:** Your code calls a contract through a dynamic address (not a compile-time constant), and you compiled with `--deny-wildcard-contracts`.

**Solutions:**

1. Replace dynamic calls with fixed-target wrappers:

   ```solidity
   // ❌ Dynamic target — forces wildcard permission
   Syscalls.contractCall(dynamicAddr, "transfer", ...);

   // ✅ Fixed target — generates specific permission
   NativeCalls.gasTransfer(from, to, amount, "");
   ```

2. If dynamic dispatch is unavoidable, supply explicit permissions:

   ```bash
   neo-solc MyContract.sol -I devpack \
     --manifest-permissions '{"contract":"0xabcd...","methods":["transfer"]}' \
     -o build/MyContract
   ```

3. Remove the strict flags for development (not recommended for production):
   ```bash
   neo-solc MyContract.sol -I devpack -o build/MyContract
   # Without --deny-wildcard-contracts, wildcards are allowed with a warning
   ```

## Standards Not Detected

```
info: contract does not implement any recognized NEP standard
  --> manifest generation
   |
   = note: supportedstandards will be empty
```

**Cause:** The compiler's auto-detection didn't find a complete set of required methods.

**Checklist for NEP-17 detection:**

- [ ] `symbol()` — public/external, returns string
- [ ] `decimals()` — public/external, returns integer
- [ ] `totalSupply()` — public/external, returns integer
- [ ] `balanceOf(address)` — public/external, returns integer
- [ ] `transfer(address, address, uint256, Any)` — public/external, 4 parameters
- [ ] `ownerOf` must NOT be present (its presence triggers NEP-11 instead)

**Checklist for NEP-11 detection:**

- [ ] `balanceOf(address)` AND `ownerOf(bytes32)` both present
- [ ] At least one of: `transfer`, `transferFrom`, `tokensOf`

::: tip
After compilation, verify detection results:

```bash
cat build/MyContract/MyContract.manifest.json | jq '.supportedstandards'
```

:::

---
