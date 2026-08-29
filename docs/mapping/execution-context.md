# Execution Context

This page maps Solidity execution globals to the Neo N3 runtime calls or native
contract calls used by Neo DevPack for Solidity.

## Block and Transaction Context

| Solidity / EVM | Neo N3 Mapping | Fidelity |
| --- | --- | --- |
| `block.timestamp` | `System.Runtime.GetTime`, normalized from milliseconds to seconds | Exact enough for Solidity time logic. |
| `block.number` | `Ledger.currentIndex` native contract call | Exact block height mapping. |
| `block.chainid` | Neo network magic | Compile-time network value. |
| `blockhash(n)` | `Ledger.getBlockHash(n)` | Neo block hash, not Ethereum block hash. |
| `block.coinbase` | `address(0)` | Approximate; Neo dBFT has no miner account. |
| `block.difficulty` / `block.prevrandao` | `Runtime.getRandom()` | Approximate; randomness model differs. |
| `block.gaslimit` | `Policy.getExecFeeFactor()` | Approximate; Neo fee accounting differs. |
| `block.basefee` | `Policy.getFeePerByte()` | Approximate; Neo does not use EIP-1559 base fee semantics. |
| `tx.origin` | First transaction signer script hash | Available, but do not use for authorization. |
| `tx.gasprice` | `Policy.getFeePerByte()` | Approximate fee signal. |

## Message Context

| Solidity / EVM | Neo N3 Mapping | Fidelity |
| --- | --- | --- |
| `msg.sender` | `System.Runtime.GetCallingScriptHash` | Caller script hash or transaction signer, depending on invocation path; internal/self-offset routes can inject a direct-caller override. |
| `msg.value` | Payment callback amount parameter | Only meaningful inside NEP-17/NEP-11 payment callbacks. |
| `msg.data` | Current selector plus ABI-encoded arguments | Approximate outside payment callbacks; callback data is typed. |
| `msg.sig` | Current function selector | Approximate for internal-call semantics. |

## Contract Context

| Solidity / EVM | Neo N3 Mapping | Fidelity |
| --- | --- | --- |
| `this` / `address(this)` | `System.Runtime.GetExecutingScriptHash` | Exact current contract script hash. |
| `gasleft()` | `System.Runtime.GasLeft` | Neo GAS remaining for the current invocation. |
| `address.code` | `ContractManagement.getContract()` script bytes | Empty for non-contract addresses. |
| `address.codehash` | Contract script hash | `bytes32(0)` for non-contract addresses. |
| `selfdestruct(addr)` | `ContractManagement.destroy()` | Permanent destroy, no refund semantics. |

## Authorization Note

On Ethereum, many contracts gate privileged paths with `msg.sender`. On Neo,
state-changing user authority should normally be checked with
`Runtime.checkWitness(address)`. `msg.sender` identifies the immediate caller,
while witnesses prove which account authorized the transaction.

## More Detail

- [Units and Globally Available Variables](/language-description/units-and-global-variables)
- [Syscalls](/internals/syscalls)
- [Parity and Limitations](/mapping/parity-and-limitations)
