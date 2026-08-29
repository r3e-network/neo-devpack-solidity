---
title: "Native Contracts: RoleManagement"
description: "RoleManagement from Native Contracts."
---

# RoleManagement

[Back to Native Contracts](/internals/native-contracts)

Manages designated node roles in the Neo N3 network. Used by committee members to assign oracle nodes, state validators, and other infrastructure roles.

## Methods

| Method                | Signature                             | Return    | Safe | Description                                             |
| --------------------- | ------------------------------------- | --------- | :--: | ------------------------------------------------------- |
| `designateAsRole`     | `designateAsRole(bytes1,bytes[])`     | `void`    |  ❌  | Assign public keys to a role (committee only).          |
| `getDesignatedByRole` | `getDesignatedByRole(bytes1,uint256)` | `bytes[]` |  ✅  | Get public keys designated for a role at a block index. |

## Role Types

| Value  | Role              | Description                  |
| :----: | ----------------- | ---------------------------- |
| `0x04` | StateValidator    | State root validation nodes. |
| `0x08` | Oracle            | Oracle service nodes.        |
| `0x10` | NeoFSAlphabetNode | NeoFS alphabet nodes.        |
| `0x20` | P2PNotary         | P2P notary service nodes.    |

---
