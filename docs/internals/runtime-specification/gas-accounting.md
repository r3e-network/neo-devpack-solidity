---
title: "Runtime Specification: Gas Accounting"
description: "Gas Accounting from Runtime Specification."
---

# Gas Accounting

[Back to Runtime Specification](/internals/runtime-specification)

The embedded runtime tracks gas consumption to approximate production Neo N3 behavior.

## How Gas Works

Every opcode and syscall has an associated gas cost. The runtime deducts gas before executing each instruction. If gas reaches zero, execution halts with an out-of-gas error.

## Opcode Gas Costs

Gas costs by opcode category:

| Category             | Typical Gas | Notes                    |
| -------------------- | ----------- | ------------------------ |
| Constants (PUSH\*)   | 1-2         | PUSHDATA variants cost 2 |
| Flow control (JMP\*) | 2           | CALL variants cost 512   |
| Stack operations     | 2           | All stack manipulation   |
| Slot operations      | 2-3         | INITSLOT costs 3         |
| Splice/buffer        | 4           | All buffer operations    |
| Bitwise/logic        | 2-3         |                          |
| Numeric (basic)      | 3           | ADD, SUB, comparisons    |
| Numeric (complex)    | 5-8         | MUL, DIV, POW, MODPOW    |
| Collections          | 4           | All compound operations  |
| Type operations      | 2           | ISNULL, ISTYPE, CONVERT  |
| RET                  | 0           | Free                     |

## Syscall Gas Costs

| Syscall Category                           | Gas Cost | Notes                           |
| ------------------------------------------ | -------- | ------------------------------- |
| Storage context (GetContext, AsReadOnly)   | 1        | Cheap metadata operations       |
| Storage read (Get, Find)                   | 100      | Per-operation                   |
| Storage write (Put)                        | 1,000    | Most expensive common operation |
| Storage delete (Delete)                    | 100      |                                 |
| Iterator (Next, Value)                     | 1        |                                 |
| Runtime metadata (Platform, GetTime, etc.) | 1        |                                 |
| Runtime.CheckWitness                       | 200      | Signature verification          |
| Runtime.GetRandom                          | 50       |                                 |
| Runtime.Log / Notify                       | 1        |                                 |
| Crypto.CheckSig / CheckMultisig            | 1,000    | Cryptographic verification      |
| Contract.Call                              | 10       |                                 |
| Contract.Create\*Account                   | 10       |                                 |

## Current Accuracy

Gas accounting is an embedded-runtime approximation of production Neo N3 costs. Known gaps:

- Dynamic costs for large integer operations are approximated
- Some complex operations use fixed costs instead of size-dependent costs
- Edge cases in exception handling gas effects need spec verification

::: tip
For precise gas estimation, deploy to Neo-Express or TestNet and measure actual consumption. The embedded runtime is useful for development feedback but should not be used as the sole source of gas budgeting for mainnet.
:::
