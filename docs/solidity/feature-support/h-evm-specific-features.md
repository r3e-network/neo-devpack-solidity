---
title: "Solidity Feature Support: H. EVM-Specific Features"
description: "H. EVM-Specific Features from Solidity Feature Support."
---

# H. EVM-Specific Features

[Back to Solidity Feature Support](/solidity/feature-support)

These features reference EVM runtime concepts. The compiler maps them to Neo equivalents where possible, blocks them where no safe equivalent exists, and emits warnings for approximate mappings.

| Feature                                 | Status | Neo Mapping                                                                                                          |
| --------------------------------------- | :----: | -------------------------------------------------------------------------------------------------------------------- |
| `msg.sender`                            |   ✅   | `Runtime.GetCallingScriptHash()`                                                                                     |
| `msg.value`                             |   ⚠️   | Only mapped inside `onNEP17Payment` callback.                                                                        |
| `msg.data`                              |   ⚠️   | Approximated as `selector \| abi.encode(current args)` outside onNEP17Payment.                                      |
| `msg.sig`                               |   ⚠️   | Approximated as the current function selector with warning; internal-call propagation still differs from EVM.        |
| `block.timestamp`                       |   ✅   | `Runtime.GetTime()` (normalized to seconds).                                                                         |
| `block.number`                          |   ✅   | `Ledger.CurrentIndex()`.                                                                                             |
| `block.chainid`                         |   ✅   | Neo network magic number.                                                                                            |
| `block.coinbase`                        |   ✅   | Auto-mapped to `address(0)` with warning (dBFT has no miner).                                                        |
| `block.difficulty` / `block.prevrandao` |   ✅   | Auto-mapped to `Runtime.getRandom()` with warning.                                                                   |
| `block.gaslimit`                        |   ✅   | Auto-mapped to `Policy.getExecFeeFactor()` with warning.                                                             |
| `block.basefee`                         |   ✅   | Auto-mapped to `Policy.getFeePerByte()` with warning.                                                                |
| `tx.origin`                             |   ⚠️   | Parsed. Warning about authorization risks. Maps to first signer script hash.                                         |
| `tx.gasprice`                           |   ✅   | Auto-mapped to `Policy.getFeePerByte()` with warning.                                                                |
| `gasleft()`                             |   ✅   | `System.Runtime.GasLeft` syscall.                                                                                    |
| `blockhash(n)`                          |   ✅   | Auto-mapped to `Ledger.getBlockHash()` with warning.                                                                 |
| `block.sha3`                            |   ⚠️   | Auto-mapped to `Ledger.currentHash`. Deprecated in Solidity 0.8+.                                                    |
| `keccak256(...)`                        |   ✅   | `CryptoLib.keccak256`.                                                                                               |
| `sha256(...)`                           |   ✅   | `CryptoLib.sha256`.                                                                                                  |
| `ecrecover(...)`                        |   ✅   | `CryptoLib.recoverSecp256K1`, followed by `keccak256` and rightmost-20-byte address derivation.                      |
| `selfdestruct(addr)`                    |   ✅   | Auto-mapped to `ContractManagement.destroy()` with warning.                                                          |
| `address.call(...)`                     |   ⚠️   | `System.Contract.Call`; return wrapping and ABI payload parity differ from EVM and need Neo-Express validation.      |
| `address.staticcall(...)`               |   ⚠️   | `System.Contract.Call` with read-only flag; return wrapping and ABI payload parity differ from EVM.                  |
| `address.delegatecall(...)` / `callcode(...)` |   🚫   | Blocked at compile time; NeoVM has no equivalent caller-storage execution semantics.                                 |
| `address.transfer(amount)`              |   ✅   | Auto-mapped to `GAS.transfer(from,to,amount,data)`; aborts on transfer failure.                                      |
| `address.send(amount)`                  |   ✅   | Auto-mapped to `GAS.transfer(from,to,amount,data)`; returns bool.                                                    |
| `address.balance`                       |   ✅   | Auto-mapped to `GAS.balanceOf(address)`.                                                                             |
| `address.code`                          |   ⚠️   | Returns Neo contract script bytes via `ContractManagement.getContract()`; non-contract addresses return empty bytes. |
| `address.codehash`                      |   ✅   | Auto-mapped to contract script hash with warning. Non-contract returns `bytes32(0)`.                                 |
| Ether units (`wei`, `gwei`, `ether`)    |   ⚠️   | Parsed. Warning that Neo uses GAS token (10^8 decimals).                                                             |
| Time units (`seconds`, `minutes`, etc.) |   ✅   | Compile-time constants normalized to seconds.                                                                        |
| `this` keyword                          |   ✅   | `Runtime.GetExecutingScriptHash()`.                                                                                  |
| `type(X).creationCode`                  |   ⚠️   | Emits deterministic NEF3-shaped bytes for hashing compatibility; not EVM bytecode.                                    |
| `type(X).runtimeCode`                   |   ⚠️   | Same deterministic Neo-shaped payload model as `creationCode`; not production bytecode introspection.                 |

## Partial EVM feature details

**`msg.value`** — Neo does not attach native value to contract calls. The `msg.value` expression is only meaningful inside `onNEP17Payment()` callbacks, where it maps to the `amount` parameter. Outside that context, the compiler emits a warning (W111) and returns `0` at runtime.

```solidity
// ✅ Works — msg.value inside payment callback
function onNEP17Payment(address from, uint256 amount, bytes memory data) external {
    require(msg.value >= minDeposit, "insufficient deposit");
    // msg.value maps to the `amount` parameter
}

// ⚠️ Warning — msg.value outside payment context returns 0
function deposit() public payable {
    balances[msg.sender] += msg.value; // msg.value is always 0 on Neo
}
```

**`tx.origin`** — Maps to the first signer's script hash in the Neo transaction. The compiler emits a warning because `tx.origin`-based authorization is considered an anti-pattern on both EVM and Neo. Use `msg.sender` (which maps to `Runtime.GetCallingScriptHash()`) or `Runtime.checkWitness()` instead.

**Ether units** — The literal multipliers (`1 ether = 10^18`, `1 gwei = 10^9`, etc.) are parsed for source compatibility, but a warning is emitted because Neo GAS uses 10^8 decimals, not 10^18. Adjust your constants accordingly.

**`address.code`** — On Neo this now lowers to the contract script bytes fetched through `ContractManagement.getContract()`. This is closer to EVM runtime bytecode access than the old empty-byte placeholder, but it is still Neo contract script rather than EVM bytecode. Non-contract addresses return empty bytes. `address.code.length` remains the fast contract-existence approximation (0 for non-contract, 1 for contract).

**`address.delegatecall(...)` / `callcode(...)`** — On NeoVM, each contract has isolated storage. These EVM operations execute callee code in the caller's storage context, which has no Neo equivalent. The compiler rejects them instead of compiling them with unsafe changed semantics. Redesign proxy and library patterns around explicit calls, inlined libraries, or `ContractManagement.update()`.

```solidity
// 🚫 Rejected by neo-solc
(bool success, bytes memory data) = target.delegatecall(abi.encodeWithSignature("foo()"));
// Use explicit calls or ContractManagement.update() depending on the upgrade pattern.
```

**`type(X).creationCode` / `type(X).runtimeCode`** — Neo does not expose EVM bytecode. For compatibility with CREATE2-style hashing patterns, the compiler emits deterministic NEF3-shaped byte payloads that are stable per referenced contract/type when the target is available in the compilation graph. Treat these bytes as hashing artifacts, not deployable EVM bytecode.

```solidity
bytes32 childCodeHash = keccak256(type(Child).creationCode);
```

## Auto-mapping warnings

Several EVM globals are auto-mapped to approximate Neo equivalents. The compiler emits warnings for each to ensure developers understand the semantic differences:

| EVM Global                              | Auto-Mapped To                 | Why It Warns                                              |
| --------------------------------------- | ------------------------------ | --------------------------------------------------------- |
| `block.coinbase`                        | `address(0)`                   | dBFT consensus has no block miner.                        |
| `block.difficulty` / `block.prevrandao` | `Runtime.getRandom()`          | Different randomness model.                               |
| `block.gaslimit`                        | `Policy.getExecFeeFactor()`    | Different gas accounting.                                 |
| `block.basefee`                         | `Policy.getFeePerByte()`       | Different fee model.                                      |
| `tx.gasprice`                           | `Policy.getFeePerByte()`       | Different fee model.                                      |
| `blockhash(n)`                          | `Ledger.getBlockHash()`        | Semantic match but different chain.                       |
| `block.sha3`                            | `Ledger.currentHash`           | Deprecated in Solidity 0.8+; Neo uses current block hash. |
| `selfdestruct(addr)`                    | `ContractManagement.destroy()` | No refund mechanism. Permanent.                           |
| `address.codehash`                      | Contract script hash           | Non-contract addresses return `bytes32(0)`.               |

---
