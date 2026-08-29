# Types and Values

Neo DevPack for Solidity enforces Solidity typing at compile time, then lowers values to
NeoVM stack items, storage payloads, and ABI types.

## Value Types

| Solidity Type | NeoVM Representation | Notes |
| --- | --- | --- |
| `bool` | `Boolean` | Direct stack item mapping. |
| `uint*` / `int*` | `BigInteger` | Width is enforced by the compiler; NeoVM integers are arbitrary precision. |
| `address` | `UInt160` / 20-byte script hash | Neo addresses represent accounts or contracts by script hash. |
| `address payable` | `UInt160` plus compatibility members | `.transfer()` and `.send()` map to GAS transfer warnings. |
| `bytes1` ... `bytes32` | Fixed-length `ByteArray` | `bytes32` maps to Neo ABI `Hash256` and is common for NEP-11 token IDs. |
| `string` / dynamic `bytes` | `ByteArray` payload | Used as raw bytes in storage-key derivation. |
| `enum` | `BigInteger` backed by `uint8` | Solidity enum conversion rules still apply. |
| User-defined value type | Underlying value type | Zero-cost Solidity abstraction. |

## Reference Types

| Solidity Type | NeoVM Representation | Notes |
| --- | --- | --- |
| Memory arrays | NeoVM `Array` | Runtime stack representation. |
| Storage arrays | Neo storage keys | Length and element keys are derived deterministically. |
| Structs in memory | NeoVM `Array` | Fields keep their declared order. |
| Structs in storage | Serialized binary blob | Read/write is atomic for the full struct value. |
| `calldata` | Treated like memory | Accepted for source compatibility. |
| `mapping(K => V)` | Deterministic storage-key derivation | No length, no enumeration, and no "set" marker. |

## Integer Behavior

NeoVM represents integers as arbitrary-precision `BigInteger` values. Solidity
width and signedness are type-checked by the compiler, but NeoVM does not impose
fixed-width overflow boundaries at runtime. Code that depends on wrapping should
use explicit modulo arithmetic.

## Mapping Keys

Mapping keys can be built-in value types, `bytes`, `string`, contracts, or
enums. Complex key types such as structs, arrays, and mappings are not valid.
Mapping values can be any supported value, including nested mappings, arrays,
and structs.

## More Detail

- [Types](/language-description/types)
- [Storage and Mappings](/mapping/storage-and-mappings)
- [Contract Metadata](/internals/contract-metadata)
