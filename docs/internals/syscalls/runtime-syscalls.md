---
title: "Syscalls: Runtime Syscalls"
description: "Runtime Syscalls from Syscalls."
---

# Runtime Syscalls

[Back to Syscalls](/internals/syscalls)

## Runtime API

Runtime syscalls provide access to execution context, authorization, notifications, and platform metadata.

| Syscall Name                            | Gas Cost | Description                              | Devpack Wrapper                            |
| --------------------------------------- | -------: | ---------------------------------------- | ------------------------------------------ |
| `System.Runtime.GetTrigger`             |        1 | Get execution trigger type               | `Syscalls.getTrigger()`                    |
| `System.Runtime.Platform`               |        1 | Get platform string (`"NEO"`)            | `Syscalls.getPlatform()`                   |
| `System.Runtime.GetNetwork`             |        1 | Get network magic number                 | `Syscalls.getNetwork()`                    |
| `System.Runtime.GetAddressVersion`      |        1 | Get address version byte                 | `Syscalls.getAddressVersion()`             |
| `System.Runtime.GetTime`                |        1 | Get current block timestamp (ms)         | `Syscalls.getTime()`                       |
| `System.Runtime.GetScriptContainer`     |        1 | Get transaction container                | `Syscalls.getScriptContainer()`            |
| `System.Runtime.GetExecutingScriptHash` |        1 | Get current contract hash                | `Syscalls.getExecutingScriptHash()`        |
| `System.Runtime.GetCallingScriptHash`   |        1 | Get caller contract hash                 | `Syscalls.getCallingScriptHash()`          |
| `System.Runtime.GetEntryScriptHash`     |        1 | Get entry point hash                     | `Syscalls.getEntryScriptHash()`            |
| `System.Runtime.LoadScript`             |        1 | Registered but intentionally unsupported in the embedded runtime; use `System.Contract.Call` | `Syscalls.loadScript(script, flags, args)` |
| `System.Runtime.CheckWitness`           |      200 | Verify witness authorization             | `Syscalls.checkWitness(hash)`              |
| `System.Runtime.GetInvocationCounter`   |        1 | Get call count for current contract      | `Syscalls.getInvocationCounter()`          |
| `System.Runtime.GetRandom`              |       50 | Get deterministic random number          | `Syscalls.getCurrentRandom()`              |
| `System.Runtime.Log`                    |        1 | Emit log message                         | `Syscalls.log(message)`                    |
| `System.Runtime.Notify`                 |        1 | Emit notification (event)                | `Syscalls.notify(data)`                    |
| `System.Runtime.GetNotifications`       |        1 | Embedded runtime returns `[]`; notification lookup state is not exposed | `Syscalls.getNotifications()`              |
| `System.Runtime.GasLeft`                |        1 | Get remaining GAS                        | `Syscalls.gasLeft()`                       |
| `System.Runtime.BurnGas`                |        1 | Burn specified GAS amount                | `Syscalls.burnGas(amount)`                 |
| `System.Runtime.CurrentSigners`         |        1 | Embedded runtime returns `[]`; transaction signers are not modeled | `Syscalls.getCurrentSigners()`             |
| `System.Runtime.GetMsgValue`            |        1 | Get host-injected Solidity `msg.value`   | Compiler runtime intrinsic                 |

## Example Usage

```solidity
import "devpack/contracts/Syscalls.sol";

contract Guarded {
    address public owner;

    modifier onlyOwner() {
        // CheckWitness verifies the transaction includes a valid
        // signature for the given address (200 GAS units)
        require(Syscalls.checkWitness(owner), "unauthorized");
        _;
    }

    function sensitiveAction() external onlyOwner {
        // Only executes if the transaction signer controls `owner`
        Syscalls.log("sensitiveAction executed");
    }

    function getExecutionInfo() external view returns (
        address executing,
        address caller,
        uint256 timestamp,
        uint256 remainingGas
    ) {
        executing = Syscalls.getExecutingScriptHash();  // address(this)
        caller = Syscalls.getCallingScriptHash();        // msg.sender
        timestamp = Syscalls.getTime();                  // block.timestamp
        remainingGas = Syscalls.gasLeft();               // gasleft()
    }
}
```

## Gas Cost Guidance

::: warning CheckWitness Gas Cost
`System.Runtime.CheckWitness` costs 200 GAS units — the most expensive runtime syscall. It performs cryptographic signature verification against the transaction's witness list. Avoid calling it in tight loops. Cache the result in a local variable when you need to check the same witness multiple times within a single execution.
:::
