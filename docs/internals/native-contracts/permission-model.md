---
title: "Native Contracts: Permission Model"
description: "Permission Model from Native Contracts."
---

# Permission Model

[Back to Native Contracts](/internals/native-contracts)

Every native contract call generates a permission entry in the compiled manifest's `permissions` array. The compiler infers these permissions from the IR and emits explicit `contract + methods` entries.

## Fixed vs Wildcard Permissions

| Call Pattern                     | Manifest Permission                                     |
| -------------------------------- | ------------------------------------------------------- |
| `NativeCalls.neoBalanceOf(addr)` | `{ "contract": "0xef40...", "methods": ["balanceOf"] }` |
| `NativeCalls.gasTransfer(...)`   | `{ "contract": "0xd2a4...", "methods": ["transfer"] }`  |
| `Syscalls.sha256(data)`          | `{ "contract": "0x726c...", "methods": ["sha256"] }`    |
| Dynamic contract call            | `{ "contract": "*", "methods": "*" }`                   |

Using the devpack's fixed wrappers (`NativeCalls.*`, `Syscalls.*`) produces precise permission entries. Dynamic calls through `Syscalls.contractCall()` with runtime-computed targets or method names may force wildcard permissions.

## Hardening

Reject wildcard permissions in production builds:

```bash
neo-solc contract.sol \
  --callt \
  --deny-wildcard-contracts \
  --deny-wildcard-methods \
  -o build/contract
```

::: warning
Wildcard permissions (`"contract": "*"` or `"methods": "*"`) allow the contract to call any contract or method on the network. Always audit the generated manifest before deployment. Use `--deny-wildcard-contracts --deny-wildcard-methods` to make the compiler reject any code path that would require wildcards.
:::

---
