---
title: "Devpack Overview: Permission-Conscious Development"
description: "Permission-Conscious Development from Devpack Overview."
---

# Permission-Conscious Development

[Back to Devpack Overview](/additional-material/neo-devpack)

## Overview

Neo N3 manifests declare which contracts and methods your contract is allowed to call. The compiler infers these permissions from your code. Using fixed-target wrappers produces minimal permissions:

```
NativeCalls.gasTransfer(...)
  → manifest permission: {"contract":"0xd2a4cff3...","methods":["transfer"]}

Syscalls.contractCall(dynamicTarget, dynamicMethod, ...)
  → manifest permission: {"contract":"*","methods":"*"}
```

::: warning Wildcard Permissions
Wildcard permissions (`"*"`) are a security anti-pattern. They allow your contract to call any contract and any method, which increases the attack surface. Always prefer fixed-target wrappers.
:::

Compile with strict flags to reject wildcard permissions at build time:

```bash
neo-solc MyContract.sol -I devpack \
  --deny-wildcard-contracts \
  --deny-wildcard-methods \
  -o build/MyContract
```

If truly dynamic dispatch is unavoidable, supply explicit permission overrides:

```bash
neo-solc MyContract.sol -I devpack \
  --manifest-permissions '{"contract":"0xabcd...","methods":["specificMethod"]}' \
  -o build/MyContract
```
