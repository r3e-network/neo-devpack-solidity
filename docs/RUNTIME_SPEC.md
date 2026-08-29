# NeoVM Runtime Specification (Current Emulator State)

This document captures the fidelity of the embedded Neo N3 runtime at
`src/runtime/execution.rs` plus its split implementation modules under
`src/runtime/execution/` and `src/runtime/runtime_parts/` against the official
VM/syscall surface.

## Opcodes
- Opcode table is derived from the canonical Neo N3 enum (see `src/runtime/spec.rs`).
- Supported families: constants, flow control (JMP/JMPIF*/JMPEQ/JMPNE/JMPGT/JMPLT/JMPGE/JMPLE, CALL/CALL_L/CALLA/CALLT, TRY/ENDTRY/ENDFINALLY), stack ops, slots (LD*/ST* for args/locals/static), splice/buffer, bitwise, numeric, compound (PACK/MAP/STRUCT/NEWMAP/NEWARRAY_T/NEWSTRUCT), types (ISNULL/ISTYPE/CONVERT), extensions (ABORTMSG/ASSERTMSG).
- Structured exceptions: try/catch/finally stack with rethrow semantics; ENDFINALLY rethrows pending errors when required.
- Type codes: ISTYPE/CONVERT understand booleans (0x20), integers (0x21/0x22), ByteString/Buffer (0x28/0x30 treated as bytes), arrays/structs (0x40/0x41), maps (0x48), interop/iterator handles (0x60/0x80 best-effort).

## Syscalls
- Registry and IDs are generated via SHA-256(name) (see `src/runtime/spec.rs`).
- Registered syscall categories:
  - Storage: GetContext/GetReadOnlyContext/AsReadOnly, Get/Put/Delete, Find, Iterator.Next/Value.
  - Runtime: GetTrigger/Platform/GetScriptContainer/GetNetwork/GetTime/GasLeft/GetInvocationCounter/GetCallingScriptHash/GetEntryScriptHash/GetExecutingScriptHash/CheckWitness/Log/Notify/GetNotifications/BurnGas/CurrentSigners/GetRandom/GetMsgValue.
  - Crypto: CheckSig/CheckMultisig (best-effort secp256k1).
  - Blockchain/Ledger metadata is exposed through native-contract flow, not runtime-callable blockchain syscalls.
  - Contract: Call/GetCallFlags/CreateStandardAccount/CreateMultisigAccount.
- Native contract handlers are available through `System.Contract.Call`: CryptoLib hash/signature helpers, StdLib serialization/encoding helpers, ContractManagement deploy/update/introspection subset, Oracle request/price subset, Policy fee/storage/blocklist/whitelist subset, NEO/GAS token and governance subsets, RoleManagement, Notary, Treasury, and deterministic Ledger metadata.
- Syscall gas hints: per-syscall table approximating Neo pricing (storage/iterator/runtime/crypto/oracle/contract) applied before execution; out-of-gas halts.

## Iterators
- `Storage.Find` produces a handle (byte token) tied to materialized entries; `Iterator.Next` advances the index and `Iterator.Value` reads the current entry.
- Iterator handles are validated by ISTYPE (type code 0x80) while alive.

## Native Contracts
- Hash table in `spec.rs` covers NEO, GAS, ContractManagement, Policy, Oracle, RoleManagement, Notary, Treasury, Ledger, CryptoLib, and StdLib.
- In-memory ContractManagement registry backs Deploy/Update/GetContract and is reused by native `contractmanagement.*` calls.
- Neo-side ABI serialization fallbacks are implemented through the `StdLib` native contract (`serialize` / `deserialize`), not `System.Runtime.Serialize` or `System.Runtime.Deserialize` syscalls.

## Gas & Stack Effects
- Syscall gas: per-syscall hints as above.
- Opcode gas: table in `spec.rs` mirrors Neo opcode surface (values are emulator-scaled; tighten as needed).

## Known Gaps (see `docs/NEO_VM_PARITY_TODO.md` for detail)
- Exact opcode gas/stack effects, ByteString vs Buffer mutation semantics, streaming iterators, richer native contract surfaces, and strict CheckSig/CheckMultisig parity remain to be implemented.
