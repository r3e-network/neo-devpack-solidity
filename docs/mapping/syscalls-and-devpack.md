# Syscalls and Devpack

The compiler emits Neo syscalls and native contract calls for common Solidity
constructs. The devpack exposes the same runtime surface for explicit contracts.

## Solidity to Neo Runtime Calls

| Solidity Construct | Neo Runtime Call | Notes |
| --- | --- | --- |
| `msg.sender` | `System.Runtime.GetCallingScriptHash` | Immediate caller script hash or transaction signer; some internal/self-offset runtime paths inject a direct-caller override. |
| `address(this)` | `System.Runtime.GetExecutingScriptHash` | Current contract script hash. |
| `block.timestamp` | `System.Runtime.GetTime` | Neo milliseconds normalized to seconds. |
| `block.number` | `Ledger.currentIndex` | Native contract call through `System.Contract.Call`. |
| `gasleft()` | `System.Runtime.GasLeft` | Remaining invocation GAS. |
| State variable read | `System.Storage.Get` | Key derived from state variable name. |
| State variable write | `System.Storage.Put` | Key derived from state variable name. |
| `mapping[key]` read/write | `System.Storage.Get` / `System.Storage.Put` | Key derived by iterative SHA256 hashing. |
| `emit Event(...)` | `System.Runtime.Notify` | Event name plus serialized arguments. |
| `Runtime.checkWitness(addr)` | `System.Runtime.CheckWitness` | Account witness verification. |
| `address(target).call(...)` | `System.Contract.Call` | Cross-contract invocation. |
| `keccak256(...)` | `CryptoLib.keccak256` | Native contract call. |
| `sha256(...)` | `CryptoLib.sha256` | Native contract call. |

## Devpack Layers

| Layer | Import | Use |
| --- | --- | --- |
| Low-level syscalls | `devpack/contracts/Syscalls.sol` | Typed wrappers for direct Neo N3 syscall access. |
| Runtime helpers | `devpack/libraries/Runtime.sol` | Witness checks, time, logging, notifications, and execution context. |
| Storage helpers | `devpack/libraries/Storage.sol` | Typed storage accessors, iterator helpers, mapping and array keys. |
| Native contract helpers | `devpack/libraries/Neo.sol` and related wrappers | GAS, NEO, Ledger, Policy, Oracle, CryptoLib, and ContractManagement calls. |
| Standards | `devpack/standards/*.sol` | NEP token and lifecycle standard implementations. |

## When to Use Devpack Calls

Use normal Solidity syntax for common constructs such as state variables,
events, and typed contract calls. Use devpack wrappers when your contract needs
Neo-specific functionality, direct native contract access, or explicit witness
and storage utilities.

## More Detail

- [Syscalls](/internals/syscalls)
- [Devpack Overview](/additional-material/neo-devpack)
- [Storage and Mappings](/mapping/storage-and-mappings)
