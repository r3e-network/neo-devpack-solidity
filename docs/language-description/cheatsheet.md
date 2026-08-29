# Cheatsheet

## Order of Precedence of Operators

Neo DevPack for Solidity exactly mimics the Solidity operator precedence rules.

## Global Variables Comparison

A quick reference for mapping EVM global context to NeoVM intrinsics:

For a guided, topic-by-topic version of these mappings, see
[Semantic Mapping](/mapping/).

| EVM Global           | NeoVM Equivalent                   | Notes                                                                              |
| :------------------- | :--------------------------------- | :--------------------------------------------------------------------------------- |
| `msg.sender`         | `Runtime.getCallingScriptHash()`   | Accurate mapping.                                                                  |
| `msg.value`          | `amount` in `onNEP17Payment`       | Only available during transfer hooks.                                              |
| `msg.data`           | `selector \|\| abi.encode(args)`   | In callbacks maps to typed `data` param; outside produces selector + encoded args. |
| `msg.sig`            | Current function selector          | Approximated; internal calls differ.                                               |
| `block.timestamp`    | `Runtime.getTime()`                | Seconds since epoch.                                                               |
| `block.number`       | `Ledger.currentIndex()`            | Blockchain height.                                                                 |
| `block.coinbase`     | `address(0)`                       | dBFT has no miner.                                                                 |
| `block.sha3`         | `Ledger.currentHash`               | Deprecated in Solidity 0.8+.                                                       |
| `tx.origin`          | First signer hash                  | Not recommended for auth. Use witnesses.                                           |
| `this`               | `Runtime.getExecutingScriptHash()` | Accurate mapping.                                                                  |
| `address.balance`    | `GAS.balanceOf(address)`           | Auto-mapped to GAS token.                                                          |
| `address.transfer()` | `GAS.transfer(...)`                | Auto-mapped.                                                                       |
| `gasleft()`          | `Runtime.gasLeft()`                | Execution GAS budget.                                                              |

## Cryptography Intrinsics

| EVM Hash                   | Neo Native Contract Call                                    |
| :------------------------- | :---------------------------------------------------------- |
| `keccak256(data)`          | `CryptoLib.keccak256(data)`                                 |
| `sha256(data)`             | `CryptoLib.sha256(data)`                                    |
| `ecrecover(hash, v, r, s)` | `CryptoLib.recoverSecp256K1(...)`, then `keccak256` + rightmost 20-byte address derivation |

## Security Best Practices

1. **Authorization:** Prefer `Runtime.checkWitness(address)` over `msg.sender == address`.
2. **Wildcards:** Always compile with `--deny-wildcard-contracts` and `--deny-wildcard-methods` for production to restrict the manifest.
3. **Values:** NeoVM stores integers as `BigInteger`, but Neo DevPack for Solidity emits Solidity 0.8 fixed-width overflow guards outside `unchecked` blocks. Use `unchecked` only when wraparound is intentional and tested.
4. **Upgrades:** Use `ContractManagement.update()` instead of Ethereum-style proxy contracts. Proxy storage delegates are not supported.
