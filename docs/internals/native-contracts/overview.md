---
title: "Native Contracts Overview"
description: "Overview from Native Contracts."
---

# Native Contracts Overview

[Back to Native Contracts](/internals/native-contracts)

## Overview

| Name               | Script Hash                                  | Purpose                                     | Devpack Wrapper                                                     |
| ------------------ | -------------------------------------------- | ------------------------------------------- | ------------------------------------------------------------------- |
| NEO                | `0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5` | Governance token, voting, validators        | `NativeCalls.neo*`                                                  |
| GAS                | `0xd2a4cff31913016155e38e474a2c06d08be276cf` | Utility/fee token                           | `NativeCalls.gas*`                                                  |
| ContractManagement | `0xfffdc93764dbaddd97c48f252a53ea4643faa3fd` | Deploy, update, destroy contracts           | `NativeCalls.deployContract` / `updateContract` / `destroyContract` |
| Policy             | `0xcc5e4edd9f5f8dba8bb65734541df7a1c081c67b` | Network policy parameters                   | `NativeCalls.getFeePerByte` / `getExecFeeFactor` / etc.             |
| Oracle             | `0xfe924b7cfe89ddd271abaf7210a80a7e11178758` | Off-chain data requests                     | `NativeCalls.requestOracleData` / `OracleService.sol`               |
| RoleManagement     | `0x49cf4e5378ffcd4dec034fd98a174c5491e395e2` | Node role designation                       | `NativeCalls.designateAsRole` / `getDesignatedByRole`               |
| Ledger             | `0xda65b600f7124ce6c79950c1772a36403104f2be` | Block and transaction queries               | `NativeCalls.currentIndex` / `getBlock` / `getTransaction`          |
| CryptoLib          | `0x726cb6e0cd8628a1350a611384688911ab75f51b` | Cryptographic operations                    | `Syscalls.sha256` / `neoKeccak256` / `verifyWithECDsa`              |
| StdLib             | `0xacce6fd80d44e1796aa0c2c625e9e4e0ce39efc0` | Serialization and encoding utilities        | `Syscalls.serialize` / `base64Encode` / `itoa` / etc.               |
| Notary             | `0xc1e14f19c3e60d0b9244d06dd7ba9b113135ec3b` | Notary service for multi-party transactions | `NativeCalls.notary*`                                               |
| Treasury           | `0x156326f25b1b5d839a4d326aeaa75383c9563ac1` | Treasury management                         | `NativeCalls.treasury*`                                             |

::: info
Script hashes shown above are big-endian hex (the format used by Neo RPC and block explorers). The devpack constants in `NativeCalls.sol` use the same format as Solidity `address` literals.
:::

---
