# Mapping & Indexed Storage Lowering Plan

This internal plan is excluded from the VitePress build. For the public,
reader-friendly version, see
[Indexed Storage Lowering](/mapping/indexed-storage-lowering).

This note captures the first-phase implementation plan for supporting Solidity
`mapping` access inside the Neo DevPack for Solidity compiler. It is intentionally scoped to
what is required to compile the NEP‑11/17/24 sample contracts; follow-up work
will extend support to dynamic arrays (struct fields are now wired through the
same lowering path).

## 1. Frontend Metadata
- Teach `convert_state_variable` to recognise `mapping(<K> => <V>)`. Record a new
  `StateVariableKind::Mapping { key: SolidityType, value: SolidityType }`.
- Extend `ParameterMetadata`/`StateVariableMetadata` so mapping value types are
  not rejected during validation.
- Update `NeoType::from_solidity` to return a dedicated `NeoType::Mapping`
  variant (wrapper around `Box<NeoType>` for key/value) instead of `Err`.

## 2. IR Representation
- Add `ValueType::Mapping { key: Box<ValueType>, value: Box<ValueType> }`.
- Introduce new IR instructions:
  - `Instruction::LoadMapping { state_index: usize, keys: Vec<ValueType> }`
  - `Instruction::StoreMapping { state_index: usize, keys: Vec<ValueType> }`
  - `Instruction::AddressOfMapping { state_index: usize, keys: Vec<ValueType> }`
    (needed for compound assignment patterns)
  - `Instruction::LoadStructField { state_index, key_types, field_key, field_type }`
  - `Instruction::StoreStructField { state_index, key_types, field_key, field_type }`
- When lowering `Expression::ArraySubscript`, build the key vector by walking up
  the nested subscripts.

## 3. Storage Key Encoding
- Use the same key derivation the current bytecode helper uses:
  `keccak256(key_bytes || slot_hash)`, where `slot_hash` is the 32-byte hash of
  the state variable name.
- Create helpers in `codegen` to:
  1. Evaluate each key expression and normalise it (padded integers, byte arrays,
     UTF‑8 strings).
  2. Pack keys into a single byte array with the `StdLib.serialize` native
     contract call followed by `CryptoLib.keccak256`.

## 4. Code Generation Hooks
- Extend `emit_ir_function` to translate the new mapping and struct-field
  instructions to the runtime sequence.
- Define helper entry points inside codegen:
  - `emit_serialize_key(bytecode, key_type)`: consumes the key value on the
    stack and leaves a canonical `ByteArray` representing the key.
  - `emit_mapping_slot(bytecode, base_slot_bytes, key_types)`: assumes the stack
    holds the serialized keys (outermost on top) and iteratively hashes
    `SHA256(key || slot)` to produce the final slot hash.
  - `emit_load_mapping(bytecode, module, state_index, key_types)`: orchestrates
    key serialization, slot derivation, pushes the storage context, and invokes
    `System.Storage.Get`.
  - `emit_store_mapping(bytecode, module, state_index, key_types)`: same as
    above but keeps the value on the stack and calls `System.Storage.Put`.
- Stack layout requirement:
  1. After lowering, keys must be pushed in *reverse* order so the outermost key
     is on the top of the evaluation stack.
  2. `emit_load_mapping` expects the stack to look like
     `[..., key_outer, key_inner, ...]` (outermost on top). It consumes all keys
     and leaves the loaded value.
  3. `emit_store_mapping` expects `[..., key_outer, key_inner, value]` and
     leaves nothing (value consumed).
- Key serialization notes:
  - Integers: convert to big-endian byte arrays, padded to their declared bit
    width. Signed integers must be sign-extended; unsigned integers zero-extended.
  - Booleans: single byte `[0x00]` or `[0x01]`.
  - Addresses and fixed-length byte arrays: use raw bytes.
  - Dynamic byte arrays and strings: use UTF-8 bytes; no length prefix (length
    is implicit in hashing loop).
- Hashing loop:
  - Start by pushing the 32-byte state slot hash onto the stack (precomputed in
    `StateVariable.storage_key`).
  - For each key (outermost → innermost):
    1. Serialize key (leaves `key_bytes` on stack).
    2. Swap to get `[... slot key_bytes]`.
    3. Concatenate `key_bytes || slot` (use `CAT` after pushing both).
    4. Call `CryptoLib.keccak256` (returns new slot).
  - Result is the final mapping slot key.
- Storage syscalls:
  - `Load`: push storage context (`System.Storage.GetContext`), reorder stack to
    `[context, slot]`, call `System.Storage.Get`, result left on stack.
  - `Store`: after computing slot, push context and value, reorder to
    `[context, slot, value]`, call `System.Storage.Put`.

## 5. Validation & Diagnostics
- Update `validate_contract` to accept mapping variables and parameters.
- For now, warn when the value type is itself a dynamic array (struct values are
  handled via `LoadStructField`/`StoreStructField`).

## 6. Testing Strategy
- Unit tests:
  - Parsing `mapping(address => uint256)` and nested mappings.
  - Lowering `balances[msg.sender]` into `Instruction::LoadMapping`.
- Integration:
  - Compile `devpack/standards/NEP17.sol` and assert bytecode contains the
    storage key pattern.
  - VM smoke test: transfer tokens, verify balances.

## 7. Follow-up Items
- Dynamic array indexing (reuses much of the machinery).
- Inline caching of storage contexts to avoid repeated syscalls.
- Gas accounting for hashing and serialisation helpers.

This document is a living plan; subsequent work will tick items off and expand
edge-case coverage.

## 4a. Codegen Helper Sketch

- Runtime prerequisites: `StdLib.serialize`, `CryptoLib.keccak256`, concatenation via CAT, storage syscalls.
- emit_serialize_key: call `StdLib.serialize` for the key value.
- emit_mapping_slot: push base slot, iterate keys (outermost -> innermost), CAT and `CryptoLib.keccak256` each step.
- load/store stack shapes spelled out, preserving value for stores.
