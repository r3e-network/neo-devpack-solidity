# Indexed Storage Lowering

This page documents the compiler lowering path for Solidity mappings and
indexed storage. It is intended for compiler contributors and reviewers.

## Frontend Metadata

- `convert_state_variable` recognizes `mapping(K => V)` and records mapping key
  and value metadata.
- Parameter and state-variable metadata permit mapping values where Solidity
  allows them.
- Solidity mapping types lower to an explicit Neo type shape instead of being
  rejected during validation.

## IR Representation

The lowering path represents mapping access with dedicated IR instructions for:

- loading a mapping value,
- storing a mapping value,
- taking the address of a mapping access for compound assignment patterns,
- loading and storing struct fields reached through indexed storage.

When lowering nested `ArraySubscript` expressions, the frontend walks outward
through the subscript chain and records the key vector in deterministic order.

## Storage Key Encoding

Mapping storage uses the same deterministic key scheme described in
[Storage and Mappings](/mapping/storage-and-mappings):

```text
SHA256(serialized_key || current_slot_hash)
```

The starting slot hash is `SHA256(variable_name)`. Each nested key produces the
next slot hash. The final hash is passed to `System.Storage.Get` or
`System.Storage.Put`.

## Codegen Stack Shape

Mapping load expects the evaluated keys on the stack and leaves the loaded
value. Mapping store expects the evaluated keys plus the value and consumes all
of them after the storage write.

Key serialization must normalize integers, booleans, addresses, fixed bytes,
dynamic bytes, and strings before hashing. Struct values are serialized as
values, not as mapping keys.

## Validation and Tests

Compiler coverage should include:

- parsing simple and nested mappings,
- lowering `balances[msg.sender]` to mapping-load IR,
- compiling NEP-17/NEP-11 sample contracts that use indexed balances or owners,
- VM smoke tests that mutate and read back derived mapping keys.

## More Detail

- [Storage and Mappings](/mapping/storage-and-mappings)
- [Layout of State Variables in Storage](/internals/layout-in-storage)
- [Syscalls and Devpack](/mapping/syscalls-and-devpack)
