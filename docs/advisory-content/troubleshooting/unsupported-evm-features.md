---
title: "Troubleshooting: Unsupported EVM Features"
description: "Unsupported EVM Features from Troubleshooting."
---

# Unsupported EVM Features

[Back to Troubleshooting](/advisory-content/troubleshooting)

## E3001: UnsupportedFeature

```
error[E3001]: unsupported feature: delegatecall
  --> MyContract.sol:15:9
   |
15 |         address(impl).delegatecall(data);
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   = note: NeoVM has no delegate call mechanism
   = help: use ContractManagement.update() for upgradeable contracts
```

**Blocked or limited constructs and their Neo alternatives:**

| Feature / construct                  | Diagnostic | Neo Alternative                            |
| ------------------------------------ | ---------- | ------------------------------------------ |
| `delegatecall`                       | E3001      | `ContractManagement.update()` for upgrades |
| Unsupported Yul inside `assembly`    | Warning    | Use Solidity or devpack libraries instead  |
| EVM `CREATE` / `CREATE2` opcodes     | E3001      | `ContractManagement.deploy(...)`           |

::: warning
`delegatecall` and EVM create opcodes have no safe 1:1 NeoVM equivalent.
Inline assembly has limited Yul lowering, but EVM-only operations inside it
still need Neo-native replacements. Solidity `new Contract(...)` is not real
deployment on Neo: the compiler currently inlines/simulates constructor-like
logic and returns a zero-address placeholder. `type(X).creationCode` and
`type(X).runtimeCode` are not blocked; they compile to deterministic Neo-shaped
compatibility payloads for hashing, not deployable EVM bytecode.
:::

---
