---
title: "Syscalls: Gas Cost Reference"
description: "Gas Cost Reference from Syscalls."
---

# Gas Cost Reference

[Back to Syscalls](/internals/syscalls)

All registered embedded-runtime syscalls sorted by gas cost, grouped into cost tiers.

## Tier 1 — 1 GAS unit

Metadata reads, logging, and iterator traversal. Cheap and safe to call frequently.

| Syscall Name                            | Category |
| --------------------------------------- | -------- |
| `System.Storage.GetContext`             | Storage  |
| `System.Storage.GetReadOnlyContext`     | Storage  |
| `System.Storage.AsReadOnly`             | Storage  |
| `System.Runtime.GetTrigger`             | Runtime  |
| `System.Runtime.Platform`               | Runtime  |
| `System.Runtime.GetNetwork`             | Runtime  |
| `System.Runtime.GetAddressVersion`      | Runtime  |
| `System.Runtime.GetTime`                | Runtime  |
| `System.Runtime.GetScriptContainer`     | Runtime  |
| `System.Runtime.GetExecutingScriptHash` | Runtime  |
| `System.Runtime.GetCallingScriptHash`   | Runtime  |
| `System.Runtime.GetEntryScriptHash`     | Runtime  |
| `System.Runtime.LoadScript`             | Runtime  |
| `System.Runtime.GetInvocationCounter`   | Runtime  |
| `System.Runtime.Log`                    | Runtime  |
| `System.Runtime.Notify`                 | Runtime  |
| `System.Runtime.GetNotifications`       | Runtime  |
| `System.Runtime.GasLeft`                | Runtime  |
| `System.Runtime.BurnGas`                | Runtime  |
| `System.Runtime.CurrentSigners`         | Runtime  |
| `System.Runtime.GetMsgValue`            | Runtime  |
| `System.Iterator.Next`                  | Iterator |
| `System.Iterator.Value`                 | Iterator |

## Tier 2 — 10 GAS units

Contract interaction syscalls.

| Syscall Name                            | Category |
| --------------------------------------- | -------- |
| `System.Contract.Call`                  | Contract |
| `System.Contract.GetCallFlags`          | Contract |
| `System.Contract.CreateStandardAccount` | Contract |
| `System.Contract.CreateMultisigAccount` | Contract |

## Tier 3 — 50 GAS units

| Syscall Name               | Category |
| -------------------------- | -------- |
| `System.Runtime.GetRandom` | Runtime  |

## Tier 4 — 100 GAS units

Storage read, delete, and search operations.

| Syscall Name                  | Category |
| ----------------------------- | -------- |
| `System.Storage.Get`          | Storage  |
| `System.Storage.Delete`       | Storage  |
| `System.Storage.Find`         | Storage  |

## Tier 5 — 200 GAS units

| Syscall Name                  | Category |
| ----------------------------- | -------- |
| `System.Runtime.CheckWitness` | Runtime  |

## Tier 6 — 1,000 GAS units

Storage writes and cryptographic verification. The most expensive operations.

| Syscall Name                  | Category |
| ----------------------------- | -------- |
| `System.Storage.Put`          | Storage  |
| `System.Crypto.CheckSig`      | Crypto   |
| `System.Crypto.CheckMultisig` | Crypto   |

::: info Gas Accuracy
The neo-devpack-solidity embedded runtime tracks syscall gas costs with static approximation tables. The primary gaps are in storage pricing (which depends on the dynamic `Policy.getStoragePrice()` value on mainnet) and opcode-level metering. Always perform final gas estimation against a Neo N3 testnet node before mainnet deployment.
:::

::: warning LoadScript
`System.Runtime.LoadScript` is present in the syscall registry for Neo N3 surface parity, but the embedded runtime rejects it at execution time. Use `System.Contract.Call` for inter-contract calls.
:::
