---
title: "Quickstart: Step 4: Compile with Strict Manifest Policy"
description: "Step 4: Compile with Strict Manifest Policy from Quickstart."
---

# Step 4: Compile with Strict Manifest Policy

[Back to Quickstart](/basics/quickstart)

## Overview

For production contracts, you should deny wildcard permissions. This ensures the manifest explicitly lists every contract and method your code calls:

```bash
./target/release/neo-solc MyStorage.sol \
  -I devpack \
  -O2 \
  --deny-wildcard-contracts \
  --deny-wildcard-methods \
  -o build/MyStorageStrict
```

If the contract requires permissions that cannot be narrowed (e.g., dynamic calls), the compiler will fail with an error. In that case, provide an explicit allowlist:

```bash
./target/release/neo-solc MyStorage.sol \
  -I devpack \
  -O2 \
  --manifest-permissions permissions.json \
  --manifest-permissions-mode replace-wildcards \
  --deny-wildcard-contracts \
  --deny-wildcard-methods \
  -o build/MyStorageStrict
```

See [Compile Workflow](/compiler/analysing-the-compiler-output) for full details on manifest hardening.
