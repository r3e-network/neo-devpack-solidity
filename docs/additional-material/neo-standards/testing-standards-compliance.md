---
title: "Standards and Contracts: Testing Standards Compliance"
description: "Testing Standards Compliance from Standards and Contracts."
---

# Testing Standards Compliance

[Back to Standards and Contracts](/additional-material/neo-standards)

## Overview

After compiling, verify your contract meets the target standard:

1. **Check manifest** — confirm `supportedstandards` contains the expected entries:

    ```bash
    cat build/MyToken/MyToken.manifest.json | jq '.supportedstandards'
    ```

2. **Verify method signatures** — ensure all required methods are present with correct parameter counts. The compiler warns on near-misses.

3. **Test transfer with witness** — invoke `transfer` with a valid witness scope and verify `Runtime.checkWitness()` passes:

    ```bash
    neo-express invoke MyToken transfer \
      --account sender -- sender recipient 1000 null
    ```

4. **Test payment callbacks** — transfer tokens to a contract and verify `onNEP17Payment` or `onNEP11Payment` is called.

5. **Integration testing** — use `neo-express` for end-to-end testing on a local private network:
    ```bash
    neo-express create
    neo-express run
    neo-express contract deploy build/MyToken/MyToken.nef
    neo-express invoke MyToken symbol
    ```

::: tip Compiler Warnings Are Your Friend
Address all `W1xx` warnings before deploying. They indicate patterns that may compile but won't produce a standards-compliant manifest or may behave unexpectedly on Neo N3.
:::
