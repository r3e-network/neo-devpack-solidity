---
title: "Native Contracts"
description: "Native Contracts section index."
---

# Native Contracts

Neo N3 ships eleven native contracts that are deployed at genesis and provide core platform functionality — token management, governance, cryptography, blockchain queries, and more. Unlike user-deployed contracts, native contracts have deterministic script hashes that are identical across all Neo N3 networks (mainnet, testnet, private chains). The `neo-devpack-solidity` compiler and devpack expose these contracts through the `NativeCalls` library and `Syscalls` wrappers, lowering calls to either `System.Contract.Call` syscalls or optimized `CALLT` method token instructions.

---

## Sections

| Section |
| --- |
| [Overview](/internals/native-contracts/overview) |
| [NEO Token Contract](/internals/native-contracts/neo-token-contract) |
| [GAS Token Contract](/internals/native-contracts/gas-token-contract) |
| [ContractManagement](/internals/native-contracts/contractmanagement) |
| [Policy Contract](/internals/native-contracts/policy-contract) |
| [Oracle Contract](/internals/native-contracts/oracle-contract) |
| [RoleManagement](/internals/native-contracts/rolemanagement) |
| [Notary Contract](/internals/native-contracts/notary-contract) |
| [Treasury Contract](/internals/native-contracts/treasury-contract) |
| [Ledger Contract](/internals/native-contracts/ledger-contract) |
| [CryptoLib](/internals/native-contracts/cryptolib) |
| [StdLib](/internals/native-contracts/stdlib) |
| [Method Token Optimization (CALLT)](/internals/native-contracts/method-token-optimization-callt) |
| [Permission Model](/internals/native-contracts/permission-model) |
| [See Also](/internals/native-contracts/see-also) |
