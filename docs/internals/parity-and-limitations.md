# Runtime Parity and Limitations

This page documents the current state of parity between the Neo DevPack for Solidity compiler's embedded runtime and the production Neo N3 virtual machine, known limitations, and planned improvements.

For a reader-focused summary of exact, approximate, replacement, and unsupported
semantic mappings, see [Parity and Limitations](/mapping/parity-and-limitations).

## What "Parity" Means

Parity refers to how closely the embedded NeoVM runtime in `neo-solc` matches the behavior of the official Neo N3 reference implementation. In a reference VM, full parity would mean identical behavior for all opcodes, syscalls, gas accounting, and edge cases; this embedded runtime is intentionally scoped and documents unsupported opcodes separately.

The embedded runtime is designed for local testing and development. It prioritizes correctness of core functionality over exhaustive edge-case coverage. For production validation, always test on Neo-Express or testnet.

## Current Status Summary

The embedded runtime is covered by runtime unit tests, conformance vectors, and fuzz/property suites. It is suitable for fast local validation of compiler output and core runtime behavior; production gas, oracle, and live-chain behavior still require Neo-Express or TestNet validation.

| Category             | Status        | Accuracy                                         |
| -------------------- | ------------- | ------------------------------------------------ |
| Opcode execution     | Broad subset  | Documented opcode subset; unsupported opcodes are rejected |
| Stack operations     | Complete      | All stack/slot operations                        |
| Arithmetic and logic | Complete      | Including MODMUL, MODPOW, SQRT                   |
| Control flow         | Complete      | All jump variants, CALL/CALLA/CALLT              |
| Exception handling   | Complete      | TRY/CATCH/FINALLY with rethrow                   |
| Storage syscalls     | Complete      | Get/Put/Delete/Find with iterators               |
| Runtime syscalls     | Complete      | Platform, network, time, gas, notifications      |
| Crypto syscalls      | Complete      | SHA256, RIPEMD160, Keccak256, Murmur32, CheckSig |
| Gas accounting       | Approximate   | Per-opcode and per-syscall tables                |
| Iterator handling    | Functional    | Materialized (not streaming)                     |
| Native contracts     | Partial       | Core methods implemented, stubs for others       |
| Blockchain accessors | Partial       | Placeholder/stub data                            |

## Known Gaps by Priority

### P1 -- High Priority (Correctness)

#### Exception Handling Stack Unwinding

Stack unwinding during exception propagation and the associated gas effects need verification against the Neo N3 specification. The current implementation handles the common cases correctly, but edge cases involving nested try/catch/finally blocks with complex stack states may diverge.

**Impact:** Contracts with deeply nested exception handling may behave differently in production.

**Workaround:** Test exception-heavy code paths on Neo-Express.

#### Gas Precision for Complex Operations

Gas accounting is approximate. The main gaps are:

- Dynamic costs for large integer operations (the runtime uses fixed costs instead of size-dependent costs)
- Complex compound operations where gas depends on collection size
- Some syscall costs are approximations of the Neo N3 fee schedule

**Impact:** Gas estimates from the embedded runtime may undercount or overcount for complex operations.

**Workaround:** Use Neo-Express or TestNet for accurate gas measurement before mainnet deployment. Keep a deployment-specific safety margin above embedded runtime estimates.

### P2 -- Medium Priority (Performance and Spec Compliance)

#### Iterator Streaming

`Storage.Find` currently materializes all matching entries into memory before returning the iterator. The production Neo N3 implementation uses lazy streaming, loading entries on demand.

**Impact:** Functional correctness is not affected. Memory usage may be higher than production for large result sets. Performance characteristics differ for contracts that iterate over many storage entries.

**Workaround:** None needed for correctness. Be aware that memory usage in tests may not reflect production behavior for large datasets.

#### ByteString vs Buffer Type Distinction

The embedded runtime treats both `ByteString` and `Buffer` as generic byte arrays. In the Neo N3 specification, these are distinct types with different mutation semantics:

- `ByteString` is immutable (copy-on-write)
- `Buffer` is mutable (in-place modification)

**Impact:** Code that relies on the distinction between mutable and immutable byte sequences may behave differently. Most Solidity-compiled contracts do not depend on this distinction.

**Workaround:** Avoid relying on ByteString immutability guarantees in tests. Verify byte mutation behavior on Neo-Express if your contract uses low-level byte operations.

### P3 -- Low Priority (Nice to Have)

#### Blockchain Accessors

Ledger and contract-management accessors return deterministic embedded data, registry entries, or null/default values rather than live blockchain state. The embedded runtime does not maintain a full simulated chain.

**Impact:** Contracts that read blockchain metadata (block height, transaction data, other contract state) will receive fixed test values.

**Workaround:** Use Neo-Express for integration tests that depend on blockchain state.

#### Additional Hash Functions

The runtime implements SHA256, RIPEMD160, Keccak256, and Murmur32. Additional hash functions may be needed if the Neo N3 CryptoLib native contract surface expands.

**Impact:** None for current contracts. Future Neo N3 protocol updates may add hash functions not yet supported.

#### Native Contract Method Surface

The full method surface of Policy, ContractManagement, and Ledger native contracts is not completely implemented. Core methods (Deploy, Update, GetContract, fee queries) work; less common methods return stub values.

**Impact:** Contracts calling uncommon native contract methods may receive incorrect results in tests.

**Workaround:** Test native contract interactions on Neo-Express.

## Completed Items

The following items were previously tracked as gaps and have been resolved:

- **CheckSig / CheckMultisig** -- Real secp256k1 verification with DER and compact signature support
- **Storage syscalls** -- Complete Get/Put/Delete/Find with iterator Next/Value token handling
- **Runtime syscalls** -- Platform, network, time, gas, notifications, checkWitness
- **Crypto syscalls** -- SHA256, RIPEMD160, Keccak256, Murmur32, Hash160, Hash256
- **Core opcode coverage** -- The documented opcode subset has runtime handlers with stack-effect coverage; unsupported opcodes are rejected rather than silently emulated
- **Gas accounting** -- Per-opcode and per-syscall approximation tables

## Compiler-Level Limitations

These are not runtime parity issues but fundamental limitations of the Solidity-to-NeoVM compilation target.

### Blocked EVM Features

The following Solidity/EVM constructs are blocked or restricted because they do not have a safe 1:1 NeoVM equivalent:

| Feature / Construct                  | Diagnostic                     | Reason                                                       |
| ------------------------------------ | ------------------------------ | ------------------------------------------------------------ |
| Inline assembly (`assembly { ... }`) | Warning / limited lowering     | Only selected Yul operations are lowered; EVM opcode parity is not available |
| EVM assembly `extcodesize` / `extcodecopy` | Blocked in unsupported assembly paths | Use `address.code.length` or `ContractManagement.getContract()` for Neo contract-script checks |
| `CREATE2` / EVM create opcodes       | Blocked                        | Neo deployment uses `ContractManagement.deploy()` and Neo contract hashes |

### Auto-Mapped Features (with Warnings)

These EVM features compile successfully but emit warnings because their Neo mappings have different semantics:

| Feature                                 | Neo Mapping                        | Warning Reason                                                  |
| --------------------------------------- | ---------------------------------- | --------------------------------------------------------------- |
| `delegatecall` / `callcode`             | Blocked at compile time            | NeoVM has no equivalent caller-storage execution context |
| `staticcall`                            | `System.Contract.Call` (read-only) | Uses call flags instead of EVM's storage read restriction       |
| `selfdestruct(addr)`                    | `ContractManagement.destroy()`     | No refund mechanism; permanent on Neo                           |
| `block.difficulty` / `block.prevrandao` | `Runtime.getRandom()`              | dBFT consensus has no PoW difficulty                            |
| `tx.gasprice`                           | `Policy.getFeePerByte()`           | Neo uses different fee model                                    |
| `block.coinbase`                        | `address(0)`                       | dBFT has no block miner                                         |
| `block.sha3`                            | `Ledger.currentHash`               | Deprecated in Solidity 0.8+; uses current block hash            |
| `block.gaslimit`                        | `Policy.getExecFeeFactor()`        | Neo uses GAS token fees, not gas limits                         |
| `block.basefee`                         | `Policy.getFeePerByte()`           | Neo does not use EIP-1559                                       |
| `blockhash(n)`                          | `Ledger.getBlockHash()`            | Semantic match but different chain                              |
| `msg.value`                             | `0` (outside callbacks)            | No attached value on Neo; use NEP-17 callbacks                  |
| `msg.data`                              | `selector \|\| abi.encode(args)`   | Approximated calldata; use explicit params                      |
| `msg.sig`                               | Current function selector          | Differs across internal calls                                   |
| `tx.origin`                             | First signer script hash           | Neo uses multi-sig witnesses                                    |
| `address.code`                          | Contract script bytes              | Neo script, not EVM bytecode                                    |
| `address.codehash`                      | Contract script hash               | Returns bytes32(0) for non-contract                             |
| `gasleft()`                             | `System.Runtime.GasLeft`           | Direct syscall mapping                                          |

### Partial Features with Neo-Specific Semantics

| Feature                    | Neo Behavior                                                                 |
| -------------------------- | ---------------------------------------------------------------------------- |
| `msg.value`                | Returns `amount` inside `onNEP17Payment`; returns `0` with warning elsewhere |
| Function overloading       | Supported with Neo ABI name mangling for same-arity overloads                |
| Dynamic arrays in storage  | Mapped to storage arrays with `.push()` / `.pop()`                           |
| `receive()` / `fallback()` | Mapped to Neo entry points; use `onNEP17Payment` for value receipt           |
| `payable`                  | Accepted with warning; use `onNEP17Payment` for NEP-17 token receipt         |
| `address.balance`          | Maps to `GAS.balanceOf()`                                                    |
| `address.transfer/send`    | Maps to `GAS.transfer()` with abort/bool semantics                           |

### Dynamic Call Sites and Wildcard Permissions

Contracts that use dynamic contract addresses (computed at runtime rather than hardcoded) require wildcard permissions in the manifest. This is a security concern because wildcard permissions grant the contract the ability to call any other contract.

Use the `--deny-wildcard-*` flags and `--manifest-permissions` to enforce explicit permission policies. See [CLI Reference](/compiler/using-the-compiler) for details.

## Intentionally Different Behaviors

Some behaviors are intentionally different from the production Neo N3 runtime:

| Behavior         | Embedded Runtime        | Production               | Reason                   |
| ---------------- | ----------------------- | ------------------------ | ------------------------ |
| Random numbers   | Deterministic seed      | Block-based entropy      | Reproducible tests       |
| Block timestamps | Fixed test value        | Real timestamps          | Reproducible tests       |
| Network magic    | Test value              | Mainnet/testnet magic    | Isolation                |
| Contract hashes  | Computed from test data | Computed from deployment | Test isolation           |
| Oracle responses | Deterministic pseudo ID | Real oracle network      | No external dependencies |

These differences are by design and ensure that tests are deterministic and reproducible without requiring a running Neo node.

## Planned Improvements

The following improvements are planned but not yet scheduled:

- **Gas precision tightening** -- Align dynamic gas costs with the Neo N3 specification for large integer and collection operations
- **Streaming iterators** -- Replace materialized iterator implementation with lazy streaming
- **ByteString/Buffer distinction** -- Implement proper mutation semantics
- **Differential testing framework** -- Automated comparison of embedded runtime output against Neo-Express
- ✅ **Fuzzing framework** -- Property-based tests plus the registered cargo-fuzz target suite cover storage, compiler, runtime, manifest, NEF, and edge-case paths

## Reporting Issues

If you encounter a behavior difference between the embedded runtime and production Neo N3:

1. Verify the behavior on Neo-Express to confirm it is a runtime parity issue
2. Check this page to see if the gap is already documented
3. Report the issue with a minimal reproduction case

Issue tracker: [github.com/r3e-network/neo-devpack-solidity/issues](https://github.com/r3e-network/neo-devpack-solidity/issues)

Include:

- The Solidity source code that demonstrates the issue
- Expected behavior (from Neo-Express or the Neo N3 specification)
- Actual behavior from the embedded runtime
- Compiler version (`neo-solc --version`)

## See Also

- [Runtime Specification](/internals/runtime-specification) -- full runtime documentation
- [Architecture](/internals/architecture) -- compiler pipeline
- [Error Reference](/advisory-content/error-reference) -- diagnostic codes
- [CLI Reference](/compiler/using-the-compiler) -- compiler options
- [Troubleshooting](/advisory-content/troubleshooting) -- common issues and solutions
