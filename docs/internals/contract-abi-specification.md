# Contract ABI Specification

The Application Binary Interface (ABI) is the standard way to interact with contracts in the ecosystem. It describes the data structures, method signatures, and events exposed by the smart contract to the outside world.

::: tip 💡 NeoVM Difference: JSON Manifest vs Binary Encoding
Ethereum relies on a tightly-packed binary encoding scheme and 4-byte `keccak256` function selectors. Contracts parse this binary payload internally.

**Neo uses a structured JSON Manifest ABI and dispatches methods natively by their string names.** The NeoVM natively understands arguments like `Integer`, `Hash160`, and `ByteArray`, meaning the contract does not need to manually decode a packed byte payload.
:::

## Basic Design

When you compile a contract, Neo DevPack for Solidity generates a `.manifest.json` file. The `abi` object inside this manifest dictates exactly how clients and other contracts interact with your code.

### Method Dispatch

In EVM, `transfer(address,uint256)` hashes to the selector `0xa9059cbb`. 

On Neo, the function name itself (`"transfer"`) is the dispatch mechanism. The NeoVM executes a `System.Contract.Call` syscall, providing the target contract hash and the exact string `"transfer"`.

**Overload Collisions:**
Because Neo dispatches by name, overloading functions with the same name but different parameters will cause collisions in the manifest unless handled carefully.

```solidity
// ⚠️ Both produce "transfer" in Neo manifest — collision
function transfer(address to, uint256 amount) public { }
function transfer(address to, uint256 amount, bytes calldata data) public { }

// ✅ Distinct names — no collision
function transfer(address to, uint256 amount) public { }
function transferWithData(address to, uint256 amount, bytes calldata data) public { }
```

When a collision occurs, the `neo-devpack-solidity` compiler retains one canonical method in the ABI and mangles the names of the others, emitting a warning. Downstream tools calling the contract must use the mangled name if they intend to hit the overloaded variant.

## Argument Encoding and Types

Because NeoVM execution relies on strongly-typed StackItems rather than a raw byte array, the ABI maps high-level Solidity types into one of the fundamental Neo ABI types.

| Solidity Type | Neo ABI Type | NeoVM Representation |
| --- | --- | --- |
| `bool` | `Boolean` | Native boolean stack item. |
| `int<M>` / `uint<M>` | `Integer` | Arbitrary-precision `BigInteger`. |
| `address` | `Hash160` | 20-byte `ByteArray`. |
| `bytes<M>` | `ByteArray` | `M`-byte `ByteArray`. |
| `bytes32` | `Hash256` | 32-byte `ByteArray`. |
| `bytes` | `ByteArray` | Dynamic `ByteArray`. |
| `string` | `String` | UTF-8 encoded `ByteString`. |
| `T[]` (arrays) | `Array` | Ordered collection of StackItems. |
| `struct` | `Array` | Ordered collection of StackItems matching struct field order. |
| `mapping` | N/A | Mappings cannot be passed as arguments or returned. |

### The `Any` Type

Neo DevPack for Solidity introduces a special type, `Any`, which directly maps to the Neo ABI `Any` type. This represents a completely unconstrained StackItem. It is strictly used for Neo-native interface compliance, such as the `data` parameter in `NEP-17`'s `transfer` method.

## Function Selector and Argument Encoding

When interacting with the contract via Neo-Express or an SDK (like `neon-js` or `mamba`), you must pass arguments matching the **Neo ABI Type**, not the raw EVM types.

For example, when calling `transfer(address,uint256)`, the SDK handles packaging the arguments as an `Integer` and a `Hash160`. The NeoVM pushes these items onto the execution stack before jumping to your contract's entry point.

## Events

Solidity events map directly to Neo's `Runtime.Notify` syscall. The ABI includes an `events` array that lists all declared events.

```json
"events": [
  {
    "name": "Transfer",
    "parameters": [
      { "name": "from", "type": "Hash160" },
      { "name": "to", "type": "Hash160" },
      { "name": "amount", "type": "Integer" }
    ]
  }
]
```

Unlike Ethereum, **Neo does not use "topics" for indexed parameters.** All event parameters, whether marked `indexed` or not, are appended to the notification's state array sequentially.

## The `safe` Flag

Neo's ABI includes a `safe` boolean for every method. This indicates whether a method is read-only.
* `view` and `pure` functions in Solidity are compiled with `"safe": true`.
* All other functions are compiled with `"safe": false`.

Methods marked as safe can be called via RPC test invocations without requiring a transaction signature or incurring GAS fees. This is identical in purpose to EVM's `eth_call`.