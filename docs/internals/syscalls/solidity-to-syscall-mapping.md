---
title: "Syscalls: Solidity to Syscall Mapping"
description: "Solidity to Syscall Mapping from Syscalls."
---

# Solidity to Syscall Mapping

[Back to Syscalls](/internals/syscalls)

## Automatic Lowering

The compiler automatically translates standard Solidity constructs to the corresponding NeoVM syscalls. No manual syscall invocation is needed for these patterns.

For a shorter reader guide that separates runtime calls from devpack usage, see
[Syscalls and Devpack](/mapping/syscalls-and-devpack).

## Mapping Table

| Solidity Construct                    | Syscall / Neo Call                       | Notes                                       |
| ------------------------------------- | ---------------------------------------- | ------------------------------------------- |
| `msg.sender`                          | `System.Runtime.GetCallingScriptHash`    | Returns caller's script hash (UInt160)      |
| `address(this)`                       | `System.Runtime.GetExecutingScriptHash`  | Returns current contract's script hash      |
| `block.timestamp`                     | `System.Runtime.GetTime`                 | Milliseconds, normalized to seconds         |
| `block.number`                        | `Ledger.currentIndex` (native call)      | Not a syscall — uses `System.Contract.Call` |
| `gasleft()`                           | `System.Runtime.GasLeft`                 | Remaining GAS in current invocation         |
| State variable read                   | `System.Storage.Get`                     | Key derived from variable name via SHA256   |
| State variable write                  | `System.Storage.Put`                     | Key derived from variable name via SHA256   |
| `mapping[key]` read                   | `System.Storage.Get`                     | Key = `SHA256(serialize(key) \|\| slot)`    |
| `mapping[key]` write                  | `System.Storage.Put`                     | Same derived key                            |
| `emit Event(...)`                     | `System.Runtime.Notify`                  | Event name + args as notification           |
| `require(Runtime.checkWitness(addr))` | `System.Runtime.CheckWitness`            | Witness verification (200 GAS)              |
| `address(target).call(...)`           | `System.Contract.Call`                   | Cross-contract invocation (10 GAS)          |
| `address(target).staticcall(...)`     | `System.Contract.Call` (read-only flags) | Read-only cross-contract call               |
| `keccak256(...)`                      | `CryptoLib.keccak256` (native call)      | Via `System.Contract.Call`, not a syscall   |
| `sha256(...)`                         | `CryptoLib.sha256` (native call)         | Via `System.Contract.Call`, not a syscall   |

## Example Lowering

```solidity
// What you write:
contract Example {
    mapping(address => uint256) public balances;
    event Transfer(address indexed from, address indexed to, uint256 amount);

    function transfer(address to, uint256 amount) external {
        require(Runtime.checkWitness(msg.sender), "unauthorized");
        balances[msg.sender] -= amount;
        balances[to] += amount;
        emit Transfer(msg.sender, to, amount);
    }
}

// What the compiler emits (pseudocode):
// 1. System.Runtime.CheckWitness(callingScriptHash)  — 200 GAS
// 2. System.Storage.GetContext()                      — 1 GAS
// 3. System.Storage.Get(ctx, derived_key_sender)      — 100 GAS
// 4. System.Storage.Put(ctx, derived_key_sender, new) — 1,000 GAS
// 5. System.Storage.Get(ctx, derived_key_to)          — 100 GAS
// 6. System.Storage.Put(ctx, derived_key_to, new)     — 1,000 GAS
// 7. System.Runtime.Notify("Transfer", [from, to, amount]) — 1 GAS
// Total: ~2,402 GAS units (syscalls only, excludes opcode costs)
```
