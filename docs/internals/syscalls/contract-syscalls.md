---
title: "Syscalls: Contract Syscalls"
description: "Contract Syscalls from Syscalls."
---

# Contract Syscalls

[Back to Syscalls](/internals/syscalls)

Contract syscalls handle cross-contract invocation and account creation.

| Syscall Name                            | Gas Cost | Description                    | Devpack Wrapper                               |
| --------------------------------------- | -------: | ------------------------------ | --------------------------------------------- |
| `System.Contract.Call`                  |       10 | Cross-contract call            | `Syscalls.contractCall(hash, method, params)` |
| `System.Contract.GetCallFlags`          |       10 | Get current call flags         | `Syscalls.getCallFlags()`                     |
| `System.Contract.CreateStandardAccount` |       10 | Create account from public key | `Syscalls.createStandardAccount(pubkey)`      |
| `System.Contract.CreateMultisigAccount` |       10 | Create multisig account        | `Syscalls.createMultisigAccount(m, pubkeys)`  |

```solidity
import "devpack/contracts/Syscalls.sol";

contract Bridge {
    address constant GAS_CONTRACT = 0xd2a4cff31913016155e38e474a2c06d08be276cf;

    function checkBalance(address account) external returns (bytes memory) {
        // Cross-contract call to the GAS native contract (10 GAS units)
        bytes memory params = abi.encode(account);
        return Syscalls.contractCall(GAS_CONTRACT, "balanceOf", params);
    }

    function createMultisig(uint256 threshold, bytes[] memory pubkeys)
        external
        returns (address)
    {
        // Create a multisig account requiring `threshold` of `pubkeys`
        return Syscalls.createMultisigAccount(threshold, pubkeys);
    }
}
```

## CallFlags

The `System.Contract.GetCallFlags` syscall returns the permission flags for the current execution context. These flags control what operations the called contract is allowed to perform.

| Flag          | Value  | Description                                 |
| ------------- | ------ | ------------------------------------------- |
| `None`        | `0x00` | No permissions                              |
| `ReadStates`  | `0x01` | Can read storage and call other contracts   |
| `WriteStates` | `0x02` | Can write to storage                        |
| `AllowCall`   | `0x04` | Can make cross-contract calls               |
| `AllowNotify` | `0x08` | Can emit notifications                      |
| `All`         | `0x0F` | Full permissions (default for direct calls) |

::: tip
When calling another contract, you can restrict its permissions by passing specific call flags. This is a defense-in-depth mechanism — a called contract with `ReadStates` only cannot modify your storage even if it contains malicious code.
:::
