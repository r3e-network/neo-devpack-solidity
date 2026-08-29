# Fuzz Testing

The Neo DevPack for Solidity compiler uses a multi-layered fuzzing strategy to catch crashes, semantic regressions, and runtime parity issues before they reach production.

## Overview

| Layer | Framework | Target | Scope |
|-------|-----------|--------|-------|
| **Proptest** (Rust) | `proptest` | Compiler properties, runtime verification, storage, differential checks | Case count is controlled by `PROPTEST_CASES` and cargo defaults |
| **Cargo-fuzz** | `libFuzzer` | Registered targets in `fuzz/Cargo.toml` | Coverage-guided crash finding across compiler, NEF, manifest, runtime, and Yul surfaces |
| **Differential checks** | `proptest` | Reference crates and expected Neo behavior | Runs as part of `cargo test --test fuzz_tests` |

## Running the Fuzz Suite

### One-Shot Proptest Run

```bash
# Full suite (case counts come from each proptest config unless PROPTEST_CASES overrides them)
cargo test --test fuzz_tests

# Full suite with more cases (slower, deeper)
PROPTEST_CASES=200 cargo test --test fuzz_tests

# Quick smoke (fewer cases, faster feedback)
PROPTEST_CASES=10 cargo test --test fuzz_tests -- --test-threads=4
```

### Continuous Background Fuzzing

```bash
# Start the continuous fuzz loop (proptest + cargo-fuzz, indefinite)
make test-fuzz-start

# Check status, coverage, and crash count
make test-fuzz-status

# Stop the background fuzzer
make test-fuzz-stop
```

The continuous runner executes in rounds:
1. Proptest suite (`PROPTEST_CASES=50` by default in the continuous runner)
2. Every target reported by `cargo +nightly fuzz list`
3. A per-target `CARGO_FUZZ_TIME` burst, defaulting to 300 seconds
4. A short sleep before the next round

### CI / Scheduled Deep Runs

```bash
# Deep proptest run: PROPTEST_CASES=100
make test-fuzz-continuous

# Or run the suite script directly
bash scripts/run_fuzz_suite.sh deep
```

Use `make test-fuzz-start` or `./scripts/run_continuous_fuzz.sh` for the background loop that cycles through cargo-fuzz targets.

A GitHub Actions workflow (`.github/workflows/fuzz.yml`) runs the deep proptest suite nightly at 02:00 UTC, then runs 60-second cargo-fuzz smoke bursts for the workflow's configured target subset.

## What the Fuzz Suite Covers

### Compiler Properties (`optimizer_props.rs`)

- **Semantic equivalence**: O0 vs O3 produce identical execution results for pure arithmetic, loops, recursion, internal calls, modifiers, storage+events
- **NEF format validity**: Generated `.nef` files have correct magic header, checksum, and method tokens
- **Manifest correctness**: Generated `.manifest.json` has valid ABI, permissions, and supported standards
- **Determinism**: Same source produces identical bytecode across compilations

### Runtime Verification (`compiler_props.rs`)

- **Neo N3 native contracts**: Oracle, Policy, Ledger, RoleManagement, ContractManagement
- **Precompile helpers**: sha256, ripemd160, identity, and the typed uint256 modexp helper are checked against reference implementations. The modexp helper is not full EIP-198 raw byte-payload parity, and Neo BLS12-381 adaptations for 0x06-0x08 are not Ethereum BN256 precompile parity.
- **EVM globals**: `gasleft()`, `block.timestamp`, `address.balance`, `selfdestruct`
- **ABI encode/decode**: Roundtrip correctness (test-runtime only; see P0 warning)

### Storage Properties (`storage_props.rs`)

- Isolation between accounts
- Roundtrip preservation
- Key ordering
- Overwrite semantics
- Large value handling
- Empty value handling

### Syntax Resilience (`arithmetic_props.rs`)

- Keyword case variants
- Nested ternary with side effects
- Interleaved comments
- Very large hex literals
- Scientific notation extremes
- Escape sequence edge cases
- Empty/whitespace strings

### Edge Features (`compiler_props.rs`)

- User-defined value types
- Nested struct arrays
- Anonymous events
- Try/catch with custom errors
- Enum casts
- Function pointers
- `type(I).interfaceId`
- Global `using for`

### Precompile Runtime Verification (batches_100_105.rs, Batch #100)

- **ecrecover** (0x01): compilation and dispatch verification
- **SHA-256** (0x02): digest matches Rust sha2 reference
- **RIPEMD-160** (0x03): digest matches Rust ripemd reference
- **Identity** (0x04): data passthrough roundtrip
- **ModExp** (0x05): compilation and dispatch verification
- **BLS12-381 adaptations** (0x06-0x08): Neo-native curve helpers only; not Ethereum BN256/alt_bn128 parity
- **BLAKE2f** (0x09): unavailable on Neo N3 and expected to revert if called through the devpack helper

### Neo N3 Syscall Verification (batches_100_105.rs, Batch #101)

- `Runtime.checkWitness` — dispatch and return verification
- `Runtime.getNetwork` — network magic number
- `Runtime.getPlatform` — platform string
- `Runtime.getEntryScriptHash` — script hash verification
- `Storage.find` — iterator-based prefix search

### Solidity Feature Runtime (batches_100_105.rs, Batches #102-#105)

- `unchecked` overflow wrap semantics
- `fallback()` dispatch routing
- `msg.sig` selector verification
- `immutable` write rejection
- `receive()` → `onNEP17Payment` manifest remapping
- NEF/manifest `supportedstandards` auto-detection
- Manifest `groups` and `permissions` shape
- Multi-contract manifest isolation
- NEF bytecode determinism
- `abi.encodePacked` output shape
- `abi.encodeWithSignature` selector prefix
- `bytes.concat` / `string.concat` runtime
- try/catch with string return binding

### Native Contract & OOP (batches_106_110.rs, Batches #106-#110)

- ContractManagement.getContract/hasMethod/getMinimumDeploymentFee
- Policy.getExecFeeFactor/getStoragePrice
- Storage multi-key-type mappings, delete-then-get, nested mappings
- Struct with mixed fields (uint/address/bool)
- Array push/pop/length
- Event emission with indexed params
- Custom error revert with ABI-encoded args
- `require` with string and custom error messages
- `assert` failure (Panic 0x01)
- Inheritance with `super` calls
- Interface implementation and dispatch
- Abstract contract rejection
- `using for` directive
- Constructor chaining across 3 levels
- Empty contract deployment
- Event-only contract
- Long function names (100 chars)
- 50 state variables with auto-getters
- 5 contracts in one source file

## Interpreting Coverage

Cargo-fuzz prints coverage after each run:

```
cov: 7356                  ← edges covered in the target
ft: 7356                   ← features (same as cov for libfuzzer)
corp: 127                  ← corpus size (seed inputs)
```

For **`fuzz_target_1`**, coverage grows as the fuzzer discovers new Solidity syntax constructs. Saturation (single-digit gains per round) is expected after several hours.

For **`fuzz_target_2`**, coverage plateaus early because the NEF parser is strict and most random bytes are rejected at the magic header check.

Other registered targets cover bytecode disassembly, NEF roundtrips, manifest JSON, Standard JSON, structured Solidity input generation, runtime execution, method tokens, NEF/manifest mutation, and Yul assembly handling.

## Crash Triage

If the fuzzer finds a crash:

1. **Locate the crash artifact**:
   ```bash
   ls fuzz/artifacts/<target>/crash-*
   ```

2. **Reproduce with the artifact**:
   ```bash
   cargo +nightly fuzz run <target> fuzz/artifacts/<target>/crash-<hash>
   ```

3. **Minimize the crash input**:
   ```bash
   cargo +nightly fuzz tmin <target> fuzz/artifacts/<target>/crash-<hash>
   ```

4. **Convert to a regression test** in the appropriate `tests/` module.

## Known Fuzz Gaps

These areas are not yet deeply fuzzed and represent future expansion targets:

- **Differential testing** — compare neo-solc output against a reference EVM compiler
- **Gas oracle differential** — compare embedded runtime gas against Neo-Express
- **Cross-contract call sequences** — multi-contract interaction fuzzing
- **Manifest permission inference** — fuzz dynamic call sites to stress permission analysis
- **Dynamic ABI encoding** — `abi.encode` with dynamic types (string, bytes, arrays) falls back to pseudo-native StdLib which faults on real Neo N3
- **Full NEP-17 transfer flow** — end-to-end token transfer with onNEP17Payment callback chain
- **ContractManagement.deploy/update** — dynamic contract deployment and upgrade lifecycle
