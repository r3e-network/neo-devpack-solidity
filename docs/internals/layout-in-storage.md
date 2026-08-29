# Layout of State Variables in Storage and Transient Storage

Statically-sized variables (everything except mapping and dynamically-sized array types) are laid out in storage consecutively starting from position `0` in Ethereum.

::: tip 💡 NeoVM Difference: Key-Value Architecture
EVM uses a linear, sequential 256-bit slot model for storage where variables occupy contiguous 32-byte slots. **NeoVM uses a dynamic Key-Value storage model.** Keys are arbitrary-length byte arrays. Neo DevPack for Solidity computes a unique, deterministic hash for every state variable using `SHA256` rather than assigning sequential slots.
:::

For a split reader guide to mappings, arrays, structs, and key derivation, see
[Storage and Mappings](/mapping/storage-and-mappings).

## Transient Storage

::: tip 💡 NeoVM Difference: Transient Keyword
Ethereum Solidity v0.8.24 introduced the `transient` data location keyword (EIP-1153) to store data that is discarded after every transaction. **NeoVM does not currently have a native transient storage mechanism.**

Using the `transient` keyword in Neo DevPack for Solidity is currently parsed for compatibility but will emit a compilation warning or error depending on the use-case, as the runtime does not support reverting state without a transaction failure.
:::

## Deterministic Key Derivation

The Neo DevPack for Solidity compiler derives storage keys by utilizing the `SHA256` hashing algorithm.

For simple, non-dynamic variables, the storage key is the hash of the variable name:
```
storage_key = SHA256(variable_name)
```

Because keys are derived by name, renaming a state variable between contract upgrades will cause the contract to lose access to the previously stored data.

## Mappings and Dynamic Arrays

Mappings and arrays on Neo use iterative hashing to produce deterministic, collision-free keys, similar in concept to EVM's `keccak256(key . slot)`, but adapted for Neo's key-value infrastructure.

For mappings, keys are derived iteratively:

```
// Simple mapping: mapping(address => uint256) balances;
slot_hash = SHA256("balances")
storage_key = SHA256(serialize(key) || slot_hash)

// Nested mapping: mapping(address => mapping(uint256 => bool)) approvals;
slot_hash   = SHA256("approvals")
level_1     = SHA256(serialize(outer_key) || slot_hash)
level_2     = SHA256(serialize(inner_key) || level_1)
// level_2 is the final storage key
```

### Array Storage Layout

Storage arrays use a length key and per-element keys:

```
length_key = SHA256(variable_name)
element_key(i) = SHA256(serialize(i) || SHA256(variable_name))
```

The `.push()` operation retrieves the length, increments it, and writes the new element to `element_key(i)`.

### Struct Storage Layout

Structs stored in state variables are serialized as a single blob using `StdLib.serialize()`. The entire struct is read and written atomically.

```solidity
struct Config {
    address admin;
    uint256 fee;
    bool paused;
}

Config public config;
// In storage: mapped to a single key: SHA256("config")
// Value: the binary serialized output of the struct
```

*Note: Because struct storage is atomic, reading a single field requires deserializing the entire struct. For frequently accessed individual fields, consider using separate state variables.*

## Storage Serialization Formats

When encoding values to create storage keys or persisting them to storage, Neo DevPack for Solidity uses specific binary formats:

| Type | Serialization Strategy |
| --- | --- |
| **Integers** | Big-endian byte array, padded to declared bit width. Signed integers are sign-extended; unsigned are zero-extended. |
| **Booleans** | Single byte: `0x00` (false) or `0x01` (true). |
| **Address** | Raw 20 bytes (UInt160), no padding. |
| **Bytes/String** | Raw UTF-8 bytes, no length prefix. |
| **Structs** | `StdLib.serialize()` binary payload. |

Because NeoVM handles the underlying `Get` and `Put` syscalls using these arbitrary byte arrays, you do not have to worry about EVM tight-packing optimizations (like packing two `uint128` variables into a single 256-bit slot). The NeoVM bills storage byte-by-byte based on the actual size of the payload.
