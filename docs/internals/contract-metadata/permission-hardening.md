---
title: "Manifest Specification: Permission Hardening"
description: "Permission Hardening from Manifest Specification."
---

# Permission Hardening

[Back to Manifest Specification](/internals/contract-metadata)

## CLI Flags

Use these flags to reject manifests containing wildcard permissions at compile time:

| Flag                          | Effect                                                  |
| ----------------------------- | ------------------------------------------------------- |
| `--deny-wildcard-permissions` | Reject full wildcard (`contract="*"` AND `methods="*"`) |
| `--deny-wildcard-contracts`   | Reject any entry with `contract="*"`                    |
| `--deny-wildcard-methods`     | Reject any entry with `methods="*"`                     |

```bash
# Production build — reject all wildcards
neo-solc contract.sol \
  --deny-wildcard-permissions \
  --deny-wildcard-contracts \
  --deny-wildcard-methods \
  -I devpack -o build/
```

::: warning
When a deny flag is triggered, compilation fails with a `Manifest` error. The error message identifies which wildcard type was detected. Use permission override files to replace wildcards with explicit entries.
:::

## Permission Override Files

Override files let you replace compiler-inferred wildcard permissions with explicit entries:

```bash
neo-solc contract.sol \
  --manifest-permissions permissions.json \
  --manifest-permissions-mode replace-wildcards \
  -I devpack -o build/
```

The override file is a JSON array of permission entries:

```json
[
    {
        "contract": "0xd2a4cff31913016155e38e474a2c06d08be276cf",
        "methods": ["transfer", "balanceOf"]
    },
    {
        "contract": "0x0102030405060708090a0b0c0d0e0f1011121314",
        "methods": ["ping"]
    }
]
```

Alternatively, wrap the array in an object with a `permissions` key:

```json
{
    "permissions": [
        {
            "contract": "0xd2a4cff31913016155e38e474a2c06d08be276cf",
            "methods": ["transfer"]
        }
    ]
}
```

**Override modes:**

| Mode                | Behavior                                                                                              |
| ------------------- | ----------------------------------------------------------------------------------------------------- |
| `replace-wildcards` | Replace only wildcard entries with override entries. Non-wildcard inferred permissions are preserved. |
| `merge`             | Merge override entries into the inferred permissions. Existing entries are extended, not replaced.    |

::: tip
Combine `--deny-wildcard-contracts` with `--manifest-permissions-mode replace-wildcards` to enforce that all wildcard contracts are replaced by your override file while keeping precise native contract permissions intact.
:::
