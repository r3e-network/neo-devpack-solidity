# NeoVM Parity TODO

This file tracks remaining gaps between the embedded runtime implementation and the full Neo N3 VM/syscall surface. Current implementation covers:

## Current Coverage

- Full jump suite (JMP/JMPIF*/JMPEQ/JMPNE/JMPGT/JMPLT/JMPGE/JMPLE), CALL/CALLA/CALLT, TRY/ENDTRY/ENDFINALLY with catch/finally rethrow semantics, ASSERT/ABORT/THROW handling.
- Stack ops (DEPTH/DROP/NIP/XDROP/CLEAR/DUP/OVER/PICK/TUCK/SWAP/ROT/ROLL/REVERSE3/REVERSE4/REVERSEN), locals/args + static slots (INITSSLOT/LDSFLD/STSFLD/LDARG/STARG), type ops (ISNULL/ISTYPE/CONVERT), arrays/maps/buffers, packing/pick/set, SIZE aliases, buffer ops (CAT/SUBSTR/LEFT/RIGHT).
- Arithmetic/logic (SIGN/ABS/NEGATE/INC/DEC/MODMUL/MODPOW/POW/SQRT/min/max/within), logical ops, SHA256/HASH256 opcode handlers, and hash/crypto helpers via syscalls.
- Syscalls routed through a central registry with coverage for storage (Get/Put/Delete/Find/contexts plus Iterator Next/Value), runtime metadata (platform/network/time/gas/random/invocation counter/log/notify/checkWitness/GetMsgValue), crypto witness checks (CheckSig/CheckMultisig), contract calls (Call/GetCallFlags/create account), and native contract dispatch.
- Native contract mapping covers NEO, GAS, Policy, Oracle, ContractManagement, RoleManagement, Notary, Treasury, Ledger, CryptoLib, and StdLib. A stateful in-memory ContractManagement registry tracks deployed contracts and update counters.
- Iterator handles are runtime tokens. `System.Iterator.Dispose` is not exposed by Neo N3 or the embedded runtime; handles remain valid for the execution context that created them.
- Syscall gas table uses per-syscall values (storage/iterator/runtime/crypto/oracle/contract) instead of coarse buckets.
- Collection helpers include NEWARRAY0/NEWSTRUCT/NEWSTRUCT0/NEWMAP, HASKEY/KEYS/VALUES at 0xCB/0xCC/0xCD, PICKITEM0/1, APPEND, SETITEM0/1, REMOVE, CLEARITEMS, POPITEM.

**Status:** Runtime coverage is exercised by the `tests/runtime_*` suites, `tests/unit/runtime_tests.rs`, conformance vectors, and fuzz/property tests. Use the checked-in test inventory rather than this file for exact counts.

## Remaining Items (priority-ordered)

### P1 - High Priority (Correctness)
- **Exception handling**: Stack unwinding and gas effects need spec verification
- **Gas precision**: Dynamic costs for large integers and complex operations remain approximate

### P2 - Medium Priority (Performance)
- **Iterator streaming**: Current implementation materializes all entries (works correctly, but inefficient for large datasets)
- **ByteString vs Buffer**: Currently both treated as ByteArray (functional, but not spec-compliant type distinction)

### P3 - Low Priority (Nice to Have)
- **Blockchain accessors**: Live-chain transaction/block state beyond deterministic embedded Ledger data
- **Additional hash functions**: Beyond the currently modeled CryptoLib surface if Neo N3 expands it
- **Native contract methods**: Full Policy/ContractManagement/Ledger and governance surfaces

## Completed
- ✅ **CheckSig/CheckMultisig**: Real secp256k1 verification with DER and compact signature support
- ✅ **Storage syscalls**: Complete Get/Put/Delete/Find with iterator Next/Value token handling
- ✅ **Runtime syscalls**: Platform, network, time, gas, notifications, checkWitness
- ✅ **Crypto syscalls**: SHA256, RIPEMD160, Keccak256, Murmur32, Hash160, Hash256
- ✅ **Core opcode coverage**: The documented opcode subset has runtime handlers with stack-effect coverage; unsupported opcodes are rejected explicitly
- ✅ **Gas accounting**: Per-opcode and per-syscall tables with approximate production costs

Implementation of the above should follow the authoritative Neo N3 VM and native contract specification (opcode values/stack effects/gas, syscall IDs, method signatures). Replace current stubs with real semantics and remove placeholders once the spec-aligned behavior is defined.
