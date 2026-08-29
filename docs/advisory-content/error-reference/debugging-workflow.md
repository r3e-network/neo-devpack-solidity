---
title: "Error Reference: Debugging Workflow"
description: "Debugging Workflow from Error Reference."
---

# Debugging Workflow

[Back to Error Reference](/advisory-content/error-reference)

## Overview

A step-by-step approach to resolving compilation failures:

1. **Compile with full diagnostics:**

   ```bash
   neo-solc contract.sol -I devpack --json-errors --json-warnings -o build/ 2>diag.jsonl
   ```

2. **Fix highest-severity errors first.** Parse errors (E1xxx) block all later stages. Fix them before addressing semantic errors.

3. **Address semantic errors (E2xxx).** These are type and scope issues in your Solidity code.

4. **Review codegen errors (E3xxx).** These usually indicate unsupported EVM features that need refactoring for NeoVM.

5. **Run with manifest safety flags:**

   ```bash
   neo-solc contract.sol -I devpack \
     --deny-wildcard-permissions \
     --deny-wildcard-contracts \
     -o build/
   ```

6. **Validate with Neo-Express smoke tests** before deploying to testnet or mainnet.

::: tip
Use `-v` (verbose) to see optimization statistics and IR details. This can help diagnose unexpected bytecode size or behavior.
:::
