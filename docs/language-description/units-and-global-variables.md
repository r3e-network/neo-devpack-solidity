# Units and Globally Available Variables

This section explains how Solidity's standard Ether/Time units and global execution variables (`block`, `tx`, `msg`) map to NeoVM semantics.

For a split reader guide focused on execution globals, see
[Execution Context](/mapping/execution-context).

## Ether Units

A literal number can take a suffix of `wei`, `gwei` or `ether` to specify a subdenomination of Ether, where Ether numbers without a postfix are assumed to be Wei.

::: warning
**Neo GAS Difference:** Ether unit literals (`1 ether = 10^18 wei`) are parsed for source compatibility but emit a warning. Neo GAS uses 10^8 decimals, not 10^18. You should adjust your constants to match Neo GAS units:

```solidity
// ⚠️ EVM units — warning emitted
uint256 fee = 0.1 ether; // 10^17 wei — wrong for Neo

// ✅ Neo GAS units
uint256 fee = 10_000_000; // 0.1 GAS (10^7, since GAS has 10^8 decimals)
```

:::

## Time Units

Suffixes like `seconds`, `minutes`, `hours`, `days` and `weeks` after literal numbers can be used to specify units of time where seconds are the base unit.
These map perfectly to NeoVM logic.

## Block and Transaction Properties

The `neo-devpack-solidity` compiler auto-maps EVM globals to approximate Neo equivalents. Auto-mapped features compile successfully but emit a compiler warning when the semantic match is not exact.

| Solidity / EVM                          | NeoVM / Neo N3              | Notes                                                                      |
| --------------------------------------- | --------------------------- | -------------------------------------------------------------------------- |
| `block.timestamp`                       | `System.Runtime.GetTime`    | Returns block time in milliseconds, normalized to seconds by the compiler. |
| `block.number`                          | `Ledger.currentIndex`       | Returns the current block height.                                          |
| `block.chainid`                         | Neo network magic           | Compile-time constant. Mainnet: 860833102, Testnet: 894710606.             |
| `block.coinbase`                        | `address(0)`                | Auto-mapped with warning. dBFT consensus has no block miner.               |
| `block.difficulty` / `block.prevrandao` | `Runtime.getRandom()`       | Auto-mapped with warning. Different randomness model.                      |
| `block.gaslimit`                        | `Policy.getExecFeeFactor()` | Auto-mapped with warning. Different gas accounting model.                  |
| `block.basefee`                         | `Policy.getFeePerByte()`    | Auto-mapped with warning. Different fee model.                             |
| `blockhash(n)`                          | `Ledger.getBlockHash(n)`    | Auto-mapped with warning. Returns Neo block hash.                          |
| `block.sha3`                            | `Ledger.currentHash`        | Auto-mapped with warning. Deprecated in Solidity 0.8+.                     |
| `tx.origin`                             | First signer script hash    | Maps to the first transaction signer. Warning: authorization anti-pattern. |
| `tx.gasprice`                           | `Policy.getFeePerByte()`    | Auto-mapped with warning. Different fee model.                             |

## Message Properties

| Solidity / EVM | NeoVM / Neo N3                        | Notes                                                                                                        |
| -------------- | ------------------------------------- | ------------------------------------------------------------------------------------------------------------ |
| `msg.sender`   | `System.Runtime.GetCallingScriptHash` | Returns the script hash of the calling contract or transaction signer.                                       |
| `msg.value`    | `onNEP17Payment` amount parameter     | Only valid inside NEP-17/NEP-11 payment callbacks. No equivalent outside that context.                       |
| `msg.data`     | —                                     | Approximated as `selector \|\| abi.encode(current args)` outside onNEP17Payment.                            |
| `msg.sig`      | —                                     | Approximated as the current function selector (with warning). Internal-call semantics still differ from EVM. |

## Contract Context

| Solidity / EVM       | NeoVM / Neo N3                          | Notes                                                                                                  |
| -------------------- | --------------------------------------- | ------------------------------------------------------------------------------------------------------ |
| `this`               | `System.Runtime.GetExecutingScriptHash` | Returns the script hash of the current contract.                                                       |
| `gasleft()`          | `System.Runtime.GasLeft`                | Returns remaining GAS for the current invocation.                                                      |
| `selfdestruct(addr)` | `ContractManagement.destroy()`          | Auto-mapped with warning. No refund. Permanent and irreversible.                                       |
| `address.codehash`   | Contract script hash                    | Auto-mapped with warning. Non-contract addresses return `bytes32(0)`.                                  |
| `address.code`       | Contract script bytes                   | Auto-mapped with warning through `ContractManagement.getContract()`. Non-contracts return empty bytes. |
