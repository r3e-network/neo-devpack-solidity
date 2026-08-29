---
title: "Deploying Contracts: MainNet Considerations"
description: "MainNet Considerations from Deploying Contracts."
---

# MainNet Considerations

[Back to Deploying Contracts](/basics/deploying-contracts)

## Overview

Before deploying to MainNet:

1. **Test thoroughly on TestNet.** Deploy the exact same artifacts and run all your test scenarios.
2. **Audit the manifest permissions.** Ensure no wildcard permissions remain unless explicitly justified.
3. **Review the contract hash.** Use `--deployer` to predict the hash and verify it matches your expectations.
4. **Check GAS costs.** MainNet GAS is real value. Estimate deployment and invocation costs on TestNet first.
5. **Plan for upgrades.** If your contract needs to be upgradeable, ensure the update mechanism is tested.
6. **Run the production gate.** `make production-gate` validates the entire toolchain.

See [Production Readiness](/advisory-content/production-readiness) for the complete pre-deployment checklist.
