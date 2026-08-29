# Storage and Mappings

NeoVM uses key-value storage. Neo DevPack for Solidity therefore maps Solidity's slot-based
storage model onto deterministic byte keys instead of contiguous 256-bit slots.

## State Variables

Simple state variables use the hash of the variable name:

```text
storage_key = SHA256(variable_name)
```

Renaming a state variable in an upgrade changes the key and makes previously
stored data unreachable unless migration code moves it.

## Mappings

Mappings use iterative hashing over serialized keys:

```text
slot_hash = SHA256("balances")
storage_key = SHA256(serialize(account) || slot_hash)
```

Nested mappings repeat the process from outer key to inner key:

```text
slot_hash = SHA256("approvals")
level_1 = SHA256(serialize(owner) || slot_hash)
level_2 = SHA256(serialize(spender) || level_1)
```

`level_2` is the final key passed to `System.Storage.Get` or
`System.Storage.Put`.

## Arrays

Storage arrays keep one key for length and one derived key per element:

```text
length_key = SHA256(variable_name)
element_key(i) = SHA256(serialize(i) || SHA256(variable_name))
```

`push()` reads the current length, writes the new element at the derived element
key, and updates the length key.

## Structs

Structs stored as state variables are serialized as a single binary payload with
`StdLib.serialize()`. Reading one field requires loading and deserializing the
whole struct. Use separate state variables for frequently updated hot fields.

## Key Serialization

| Type | Storage-Key Encoding |
| --- | --- |
| Integers | Big-endian bytes padded to declared width; signed values are sign-extended. |
| Booleans | `0x00` for false, `0x01` for true. |
| Address | Raw 20-byte `UInt160`. |
| Fixed bytes | Raw fixed-length bytes. |
| Dynamic bytes / string | Raw bytes, no length prefix. |
| Struct values | `StdLib.serialize()` payload when persisted as values. |

## More Detail

- [Layout of State Variables in Storage](/internals/layout-in-storage)
- [Indexed Storage Lowering](/mapping/indexed-storage-lowering)
- [Syscalls and Devpack](/mapping/syscalls-and-devpack)
