---
title: "Production Readiness: Pre-Deployment Checklist"
description: "Pre-Deployment Checklist from Production Readiness."
---

# Pre-Deployment Checklist

[Back to Production Readiness](/advisory-content/production-readiness)

## 1. Compile with Production Settings

Use the strictest compilation flags:

```bash
neo-solc contract.sol \
  -I devpack \
  -O3 \
  --callt \
  --deny-wildcard-contracts \
  --deny-wildcard-methods \
  --json-errors \
  --json-warnings \
  -o build/contract
```

Flags explained:

| Flag                        | Purpose                                                              |
| --------------------------- | -------------------------------------------------------------------- |
| `-O3`                       | Maximum optimization for smallest bytecode and lowest execution cost |
| `--callt`                   | Use CALLT instructions for efficient native contract calls           |
| `--deny-wildcard-contracts` | Reject wildcard contract permissions                                 |
| `--deny-wildcard-methods`   | Reject wildcard method permissions                                   |
| `--json-errors`             | Structured error output for CI parsing                               |
| `--json-warnings`           | Structured warning output for CI parsing                             |

## 2. Audit the Manifest

Inspect every field of the generated manifest:

```bash
# Full manifest
jq '.' build/contract.manifest.json

# Contract name
jq '.name' build/contract.manifest.json

# Permissions -- should NOT contain wildcards
jq '.permissions' build/contract.manifest.json

# Supported standards
jq '.supportedstandards' build/contract.manifest.json

# Method signatures
jq '.abi.methods[] | {name, parameters: [.parameters[].type], returntype, safe}' \
  build/contract.manifest.json

# Events
jq '.abi.events[] | {name, parameters: [.parameters[].type]}' \
  build/contract.manifest.json

# Trust settings
jq '.trusts' build/contract.manifest.json
```

## Permission Audit Guide

Review each permission entry:

```bash
jq '.permissions[] | {contract, methods}' build/contract.manifest.json
```

For each entry, verify:

- **contract** is a specific hash (e.g., `0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5`), not `*`.
- **methods** is a specific list (e.g., `["transfer", "balanceOf"]`), not `*`.
- Every listed method is actually called by your contract.
- No unnecessary permissions are included.

If the compiler cannot narrow permissions (e.g., due to dynamic calls), provide an explicit allowlist:

```bash
neo-solc contract.sol \
  -I devpack \
  --manifest-permissions permissions.json \
  --manifest-permissions-mode replace-wildcards \
  --deny-wildcard-contracts \
  --deny-wildcard-methods \
  -o build/contract
```

## 3. Verify the NEF

Check the NEF file size and structure:

```bash
# File size
ls -la build/contract.nef

# Verify NEF magic bytes (should start with 4E 45 46 33)
xxd build/contract.nef | head -1
```

Optionally inspect the NeoVM assembly:

```bash
neo-solc contract.sol -I devpack -O3 -f assembly -o build/contract.asm
cat build/contract.asm
```

## 4. Predict the Contract Hash

If you need to know the contract hash before deployment (e.g., for cross-contract references):

```bash
neo-solc contract.sol \
  -I devpack \
  -O3 \
  --deployer 0x<your-deployer-scripthash> \
  -o build/contract
```

The compiler prints the predicted hash. Verify it matches your expectations.
