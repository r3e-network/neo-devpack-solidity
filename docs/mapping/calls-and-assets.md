# Calls and Assets

Neo N3 does not have native Ether attached to accounts. Cross-contract calls,
GAS transfers, and NEP token transfers map to explicit Neo runtime and native
contract behavior.

## Contract Calls

| Solidity Pattern | Neo N3 Mapping | Notes |
| --- | --- | --- |
| `address(target).call(data)` | `System.Contract.Call` | Standard cross-contract invocation path. |
| `address(target).staticcall(data)` | `System.Contract.Call` with read-only flags | Use for read-only calls when possible. |
| `address(target).delegatecall(data)` / `callcode(data)` | Blocked | NeoVM cannot execute in the caller's storage context. |
| Typed contract interface calls | `System.Contract.Call` | ABI selector and arguments are lowered by the compiler. |

## Value Transfers

| Solidity Pattern | Neo N3 Mapping | Notes |
| --- | --- | --- |
| `address.balance` | `GAS.balanceOf(address)` | GAS balance approximation. |
| `payable(to).transfer(amount)` | `GAS.transfer(this, to, amount, data)` | Emits a warning because Ether semantics differ. |
| `payable(to).send(amount)` | `GAS.transfer(this, to, amount, data)` | Emits a warning and should be checked explicitly. |
| `msg.value` | Payment callback amount | Only available in NEP-17/NEP-11 callback contexts. |

## Token Callbacks

Neo token transfers into contracts use explicit callbacks:

| Asset Flow | Callback |
| --- | --- |
| NEP-17 fungible token transfer | `onNEP17Payment(address from, uint256 amount, Any data)` |
| NEP-11 NFT transfer | `onNEP11Payment(address from, uint256 amount, bytes32 tokenId, Any data)` |

If a recipient contract does not implement the required callback for the token
standard, the transfer should fail according to the standard implementation.

## Authorization

Use `Runtime.checkWitness(account)` for user authority. `msg.sender` is still
useful for immediate caller checks, but it does not replace witness validation
for account-controlled state changes.

## More Detail

- [Execution Context](/mapping/execution-context)
- [Standards Mapping](/mapping/standards)
- [Devpack Overview](/additional-material/neo-devpack)
