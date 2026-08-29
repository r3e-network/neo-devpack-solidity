---
title: "Deploying Contracts: Common Deployment Errors"
description: "Common Deployment Errors from Deploying Contracts."
---

# Common Deployment Errors

[Back to Deploying Contracts](/basics/deploying-contracts)

## Overview

| Error                        | Cause                                                   | Solution                                                       |
| ---------------------------- | ------------------------------------------------------- | -------------------------------------------------------------- |
| `FAULT` on deploy            | Manifest permissions insufficient for constructor logic | Add required permissions or use `--manifest-permissions`       |
| `contract already exists`    | Contract with same hash already deployed                | Use `update` instead of `deploy`, or change the sender account |
| `insufficient GAS`           | Deployer account lacks GAS                              | Transfer more GAS to the deployer                              |
| `invalid NEF`                | Corrupted or incompatible NEF file                      | Rebuild with `cargo build --release && neo-solc ...`           |
| `manifest validation failed` | Manifest has invalid fields                             | Check `features` is `{}`, verify JSON structure                |
