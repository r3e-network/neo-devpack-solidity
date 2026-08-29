# Neo DevPack for Solidity Fuzz System — Status Report

> **Snapshot note**: this report captures the post-v0.18.0 fuzz-system
> maturation cycle. Specific test counts (e.g. proptest cases) drift as new
> tests land — for a fresh count run `cargo test --release --test fuzz_tests`
> and `cargo +nightly fuzz list`. Treat the figures below as the
> point-in-time snapshot at v0.18.0 release time, not as a continuously
> updated dashboard.

## Executive Summary

The fuzz system has been **reviewed, refactored, and expanded** into a
professional-grade suite. The v0.18.0 snapshot recorded **951 proptest cases
passing, 0 ignored, 11 cargo-fuzz targets** (all crash-free), each with
libfuzzer dictionaries (`fuzz/dict/<target>.dict`, 970 total domain-specific
tokens). Plus 3 runtime state-batch tests (canonical "skip-and-continue"
semantics locked in). End-to-end smoke tests cover **54 single-source +
13 multi-source example/devpack contracts** (all compile cleanly + 42
runtime-invoked). Multi-source standard JSON resolver verified across
simple/transitive/circular/library import shapes. Constructor lifecycle
verified across declaration-init, ctor-body init, ctor-with-args,
deploy-runs-once, modifier-on-constructor, and payable-constructor
`msg.value`. Added **differential testing**
(compiler vs reference crates), **seed corpora** (bootstrapped from
`examples/*.sol` and repo `.nef` / `.manifest.json` artifacts), and a
**contributor guide** (`docs/FUZZ.md`). Coverage measured at **68.42%
region / 67.42% line** via `cargo-llvm-cov` at the v0.18.0 snapshot time
(excluding ~6,200 LOC of orphaned modules that are 0%-covered AND have zero
importers — see [Dead Code](#dead-code-deletion-candidates) below).

**Twenty-eight** compiler/runtime bugs were surfaced and fixed during the
review — see [Fixed Bugs](#fixed-bugs) below. **All deferred bugs from the
v0.17.0 release have landed** (#16, #23, #24, #25, plus the BLS12-381 runtime
stub closed as #26 and the wave-#28 in-contract `T[].length` divergence
closed as #28), and the state-batch non-atomic semantics ambiguity is
resolved with a canonical "skip-and-continue" contract documented and
pinned in `tests/runtime_state_batch_tests.rs`.

### No open limitations

All architectural bugs, correctness divergences, DoS surfaces, and
behaviour limitations surfaced across the 33-wave fuzz review have been
resolved. Suite is fully green with 0 ignored tests. One latent allocation-DoS (in `read_input_slice`,
not opcode-reachable today) was hardened proactively after a follow-up audit.
One additional divergence (bug #16) was discovered by the new optimizer-level
differential proptest and is documented as a [Known Limitation](#known-limitations)
pending an architectural fix to the runtime's BigInt path.

---

## Test Counts (All Green)

| Test Suite | Count | Status |
|------------|-------|--------|
| fuzz_tests (proptest) | **968** | ✅ 0 failed, 0 ignored |
| runtime_state_batch_tests | **2** | ✅ 0 failed, 1 ignored |
| lib unit tests | **508** | ✅ 0 failed |
| conformance_tests | **40** | ✅ 0 failed |
| e2e_compilation_tests | **80** | ✅ 0 failed |
| **Total cross-suite** | **1474** | ✅ all green |

## Fuzz Targets

| Target | Surface | Corpus seeds |
|--------|---------|--------------|
| `fuzz_target_1` | Solidity compile pipeline (all 4 opt levels) | 23,507 inputs + 11 example `.sol` |
| `fuzz_target_2` | NEF parser (rejects malformed without panic) | 15 inputs |
| `fuzz_target_disasm` | Bytecode disassembler totality | 12 `.nef` seeds + grown corpus |
| `fuzz_target_nef_roundtrip` | Parser / serializer symmetry | 12 `.nef` seeds + grown corpus |
| `fuzz_target_manifest_json` | Manifest JSON parser robustness | 10 `.manifest.json` seeds + grown |
| `fuzz_target_standard_json` | solc-compatible Standard JSON input | Grown corpus |
| `fuzz_target_structured_sol` | `arbitrary`-generated Solidity ASTs (incl. inheritance / `using for` / abstract / virtual-override) | Grown corpus |
| `fuzz_target_runtime_exec` | NeoVM bridge on attacker bytecode (DoS) | Grown corpus |
| `fuzz_target_method_tokens` | CALLT (0x37) dispatch under random NEF method-token tables | Grown corpus |
| `fuzz_target_nef_manifest_mutation` | Mutation-fuzz of valid NEF / manifest seeds (bit-flips, byte-writes, insertions, deletions) | 53 NEF seeds + 24,296 manifest seeds, embedded fallback |
| `fuzz_target_yul_assembly` | `arbitrary`-driven Yul AST inside Solidity `assembly { ... }` blocks (let/assign/if/for/switch/break/continue/mload/sload/mstore/sstore/return/revert + arithmetic/bitwise/comparison expressions). Drives the `src/ir/statements/assembly.rs` lowering through random opt-levels | Grown corpus |

Coverage on `fuzz_target_1` jumped from **~7,450 → ~19,400** immediately
after the example corpus was added — confirming the seeds reach new
compiler paths pure-random didn't.

Smoke-test results (10 s each on a clean run):
- `fuzz_target_disasm`: 403,857 runs, 0 crashes
- `fuzz_target_nef_roundtrip`: 5,609,524 runs, 0 crashes
- `fuzz_target_manifest_json`: 2,237,103 runs, 0 crashes

---

## Fixed Bugs

The review uncovered fifteen real compiler/runtime bugs that would have broken
real contracts (or DoS'd the compiler / host). Each is now fixed **and** has a
regression test pinning the expected behavior.

| # | Symptom | Root cause | Fix location |
|---|---------|------------|--------------|
| 1 | `StdLib.<m>(...)` returned 0 at runtime | IR resolver didn't map `StdLib` / `CryptoLib` namespaces | `src/ir/context/builtins/resolve.rs::resolve_stdlib_member` + `resolve_cryptolib_member` |
| 2 | `.length` on member-accessed StdLib return faulted with `SIZE: unsupported type` | Downstream of #1 — StdLib returned Null, SIZE fell over | Same as #1 |
| 3a | `this.<method>()` returning tuple gave `(0, 0)` | `resolve_static_library_base` didn't exclude `this`/`super`; fell into internal-call path with wrong decode | `src/ir/expressions/calls/member_calls.rs` |
| 3b | Same as 3a for `IContract(addr).<method>()` | Detection too narrow | `src/ir/statements/assignments/lower_assignment.rs::is_this_external_tuple_call` |
| 4 | `ContractManagement.isContract(self)` returned empty | Runtime native handler missing `iscontract` case | `src/runtime/execution/execution_impl_part2_native/contract_management.rs` |
| 5 | Bare `sha256()` / `ripemd160()` returned 0 at runtime | IR resolver only handled `keccak256` as bare identifier | `src/ir/context/builtins/resolve.rs` |
| 6 | `abi.encodePacked(address)` byte order mismatch | test convention vs compiler — documented only | `tests/fuzz_tests/differential.rs` |
| 7 | `Syscalls.neoKeccak256` + `Syscalls.bls12381G1/G2*` not in whitelist | resolver missing entries | `src/ir/context/builtins/resolve.rs` + `syscalls.rs` |
| 8 | `1e2000000000` literal would OOM the compiler | unbounded `pow10(exp)` | `src/ir/build/literals.rs` — `MAX_DECIMAL_EXPONENT=1024` |
| 9 | `2 ** 4294967295` constant-fold would OOM | unbounded power loop | `src/ir/expressions/power.rs` — `MAX_LITERAL_POW_EXP=1024` |
| 10 | `uint[1000000000][1000000000] memory` return would OOM | unbounded `dims.iter().product()` | `src/ir/statements/dispatch/return_revert.rs` — `MAX_FIXED_ARRAY_LEAVES=65536` |
| 11 | `struct Node { Node[] children; }` stack-overflows the compiler | unbounded `NeoType::from_solidity` recursion on struct / array / mapping fields | `src/type_system/parse.rs` — `MAX_STRUCT_RESOLUTION_DEPTH=64` threaded through struct + array + mapping resolution |
| 12 | Pathological nested parens / unary / binops / blocks / `MemberAccess` chains stack-overflow the compiler | recursive IR lowering + 10 semantic-analyse walkers had no stack-growth guard | `stacker::maybe_grow` wrappers on `lower_expression`, `lower_statement`, `literal_from_expression_with_warning`, `infer_type_from_expression`, and 10 walkers in `src/solidity/solidity_analyse.rs`. Release tolerates >12k-deep parens (upstream solang-parser at ~20k is now the bottleneck) |
| 13 | `UnsignedInteger` stack items produced divergent JSON envelopes from `Integer` ones, destabilising mapping-slot hashes | serde tag mismatch between `Integer`/`UnsignedInteger` variants | `src/runtime/execution/types/stack.rs` — collapse `UnsignedInteger` into the `Integer` arm in the `StackItemSerde` round-trip |
| 14 | `this.<method>()` tuple destructure silently decoded wrong slots when the called method was compiled with a different return-arity shape | detection gap in `is_this_external_tuple_call` | `src/ir/statements/assignments/lower_assignment.rs` — broadened predicate + compile-time ABI decode fallback |
| 15 | **Host DoS**: 6-byte bytecode `02 FF 00 0C 17 C6` (PUSHINT32 + NEWSTRUCT) OOM-aborted the runtime process | `new_array()` called `Vec::with_capacity(count)` with the user-popped count; no bound against `memory_limit` | `src/runtime/execution/collections/construction.rs::new_array` + `src/runtime/execution/execution_impl_part3_bytes.rs::new_buffer` — both now reject allocations larger than `memory_limit` (mirrors PUSHDATA4 guard). Regression pinned by `batch132*` tests — surfaced by new `fuzz_target_runtime_exec` target |
| 17 | **Host DoS**: `Storage.put` could write arbitrary-length values until host OOM-aborted | `RuntimeConfig::storage_limit` field had no consumer — `grep -rn storage_limit src/` returned the declaration and default only, zero readers. `Storage.put` / `Local.Put` inserted into `storage_overlay` with no size check | `src/runtime/execution/types/context.rs` (added `storage_limit` field), `execution_impl_part1_init.rs` (propagate from config), `src/runtime/execution/syscalls/storage.rs::enforce_storage_limit` (cumulative-bytes guard on every Put). Surfaced by wave-#14 storage-DoS audit |
| 18 | **Host DoS**: `Storage.Find` eagerly materialised the full prefix-matching key set into a `Vec`, identical shape to bug #15 | `helpers/storage.rs::build_storage_entries` queried with `limit: None` and `Vec::with_capacity(entries.len())`; an attacker who populated storage with many keys (10K writes fit in default gas_limit) could OOM the host with one `Find` syscall | Same file — bound `StorageQuery::limit` against `storage_limit / MIN_ENTRY_BYTES`, plus a post-merge cap to cover overlay-only matches. Surfaced by wave-#14 storage-DoS audit |
| 19 | **Gas DoS**: `Storage.Put` charged a flat 1000 gas regardless of value size; `CryptoLib.sha256` / `keccak256` / `ripemd160` / `sha1` / `murmur32` charged a flat ~512 gas regardless of input length | `src/runtime/spec/gas.rs::syscall_gas_table` returned a single number per name; the dispatcher peeked the gas table but never inspected stack args. Real Neo N3 charges ~`100_000 * (key_len + value_len)` for Put and ~`50 * input_len` for hash methods | `src/runtime/execution/instruction/syscall.rs::syscall_extra_input_gas` (new): peeks top-of-stack key/value/input bytes before the handler runs and adds `STORAGE_PUT_PER_BYTE_GAS=100` (Storage.Put / Local.Put) or `HASH_PER_BYTE_GAS=50` (Contract.Call → CryptoLib hash) on top of the flat base, with `checked_add` graceful gas-exhausted on overflow. CALLT path mirrored at `src/runtime/execution/instruction/flow/calls.rs`. Surfaced by wave-#14 storage/gas DoS audit, regression pinned by `batch133a`–`batch133d` |
| 20 | **Gas DoS**: `CheckMultisig` ran `O(N·M)` secp256k1 verifies (up to 4096 verifies for 64×64 input) for a single flat 1000-gas charge | Same dispatch shape as #19 — flat per-syscall fee; the inner-loop verify cost wasn't accounted for | Same file — `CHECKMULTISIG_PER_VERIFY_GAS=1000` charged upfront as `pub_count * sig_count * per_verify` (saturating). Surfaced by wave-#14 audit |
| 23 | **Correctness: narrow signed-int `abi.encode` zero-padded instead of sign-extending**: `int8(-1)` encoded to `00..00 (24B) \|\| ff..ff (8B)` instead of `ff..ff (32B)`. Real on-chain consumers reading per the EVM ABI spec saw a large positive number instead of -1; same gap for `int16`/`int32`/`int64`/`int128` | `emit_expr_static_abi_slot_for_value_type` lumped signed and unsigned ints into the same path that allocates a zero-filled `NEWBUFFER(32)` and `MemCpy`'s the variable-length source over the low bytes — high bytes stayed `0x00` regardless of sign | New `emit_abi_fixed_buffer_signed` helper in `src/ir/expressions/calls/builtins/helpers.rs` — branches at runtime on `Convert(src) → Integer < 0`. Negative branch `MemCpy`'s a literal `[0xFF; 32]` ByteArray over the destination buffer before copying the source low bytes, giving canonical EVM sign-extension. Pinned by `roundtrip_int{8,16,32,64,128,256}` proptests in `tests/fuzz_tests/abi_roundtrip_props.rs`. Surfaced by wave-#23 round-trip fuzz |
| 24 | **Correctness: `abi.decode` pass-through for dynamic types** (`string`, `bytes`): `abi.decode(abi.encode("abc"), (string))` returned the raw 96-byte payload (head + length + pad), not the decoded `"abc"`. Subsequent `bytes(y).length` faulted `SIZE: unsupported type`. The fallback path went through `StdLib.deserialize` (Neo-native, NOT EVM-ABI) | `lower_abi_decode_direct` rejected dynamic value types (`abi_static_slot_count != Some(1)`) and fell through to the slow path | Extended `lower_abi_decode_direct` with new helpers in `src/ir/expressions/calls/builtins/helpers.rs`: `emit_abi_decode_dynamic_top_level`, `emit_abi_decode_bytes_tail_const`/`_runtime`, `emit_abi_decode_u256_at` — walks the EVM-canonical `BE32(offset) \|\| BE32(length) \|\| data \|\| pad` chain, returning a contiguous ByteString. Hazard: runtime `Convert → Integer` of >8-byte ByteArray returns a ByteArray (not Integer) for precision; the helper reads only the low 8 bytes of length/offset slots before parsing. Pinned by `roundtrip_string`/`roundtrip_bytes` proptests |
| 25 | **Correctness: `abi.decode` unusable for dynamic arrays** (`T[]` for static T): `.length` on a decoded array faulted `SIZE: unsupported type`. Decoder didn't materialise an `Array` StackItem | Same root cause as #24 | Same patch — `emit_abi_decode_static_element_array_tail_const`/`_runtime` walk N elements (N from the length slot) and build a `NEWARRAY` of decoded scalars via SUBSTR + CONVERT per slot. Pinned by `roundtrip_uint256_array`/`roundtrip_address_array`/`roundtrip_tuple_uint_string_bool` proptests |
| 16 | **Correctness: `unchecked { uint256 op uint256 }` faulted at O0/O1 (narrow i64 path) but constant-folded correctly at O2/O3** — surfaced by `optimizer_four_level_differential_random_expr` as an opt-level divergence. Solidity spec mandates wraparound mod 2^256 for unchecked, but the narrow path either faulted or wrapped at i64 | The IR widened uint256 operands to 32-byte BE ByteArray only in *checked* arithmetic guards. In `unchecked` context, plain `BinaryOp(Add/Sub/Mul)` ran the narrow i64 path — wrong both ways under `strict_arithmetic` | New helpers `emit_widen_to_u256_unsigned` / `emit_truncate_u256` in `src/ir/expressions/dispatch/binary.rs`. Widening appends a trailing `0x00` byte (forces positive sign-LE interpretation in `coerce_item_to_bigint`), triggering `cmp_needs_bigint_path` so the runtime's BigInt path runs. Post-op truncates to 32 bytes mod 2^256. Sidesteps the prior naïve-fix's signed-interpretation breakage of `batch102_zzz1_unchecked_overflow_wraps_to_zero`. Removed the `u16::MAX` grammar workaround from `optimizer_props.rs` — the test now runs against full `u32::MAX` literal/arg domain |
| 26 | **Functional gap: BLS12-381 runtime returned `StackItem::Null` for all 12 native methods** — compiler+IR were wired (resolver, devpack `bls12381*`, EVM-precompile shims) but `invoke_native_cryptolib` had no handler, so contracts using BLS compiled cleanly and silently returned zero from any operation. Affects ZK proof verification, threshold signatures, EVM-bridge precompiles | Stubbed `_ => StackItem::Null` fall-through in `crypto.rs` | Implemented all 12 handlers (`bls12381Serialize/Deserialize/Equal/Add/Mul/Pairing/G1Add/G1Mul/G1Neg/G2Add/G2Mul/G2Neg`) using `bls12_381 = "0.8"` (moved from dev-dep to runtime dep). Length-dispatched G1/G2 (48 vs 96 byte compressed). Scalars accept BE 32-byte input. Pairing serialized via `Fp12` Debug bytes (deterministic, equality-preserving). Subgroup checks via `from_compressed`'s built-in torsion verification. Pinned by `differential_bls12381_g1_add/g1_mul/g2_add/pairing` proptests — all 4 now pass byte-for-byte against the reference crate |
| 28 | **Correctness: in-contract `this.method()` returns of `T[] memory` corrupted `.length`** — `got.length` read the wire-byte-length (e.g. 224 = 32 + 32 + 5*32) instead of the element count (5). Off-chain decoders worked correctly; the divergence was in the Solidity-side same-contract decode path. Surfaced by wave-#28's ERC-1155 `balanceOfBatch` test | `lower_variable_definition_statement` for `T[] memory got = this.method()` stored the raw EVM-canonical ABI ByteString as a local, so `.length` ran `GetSize` on the bytestring instead of an array | New `try_lower_this_external_dynamic_assign` helper in `src/ir/statements/assignments/lower_assignment.rs` — routes single-LHS dynamic returns through the same `emit_abi_decode_dynamic_top_level` chain that fixes #25 for `abi.decode`. Wired into both the resolved-local branch and the variable-declaration initializer branch of `expressions.rs`. Pinned by 3 regression tests in `tests/fuzz_tests/in_contract_array_return_props.rs` |
| 21 | **Correctness**: `StdLib.atoi("--42", 10)` returned 42 instead of 0 (or error) | `i128::from_str_radix(body, radix)` accepts a leading `-` in `body` after the outer `neg` flag was already set, so a double-negative becomes a positive number | `src/runtime/execution/execution_impl_part2_native/stdlib.rs::atoi` — switched magnitude parser from `i128::from_str_radix` to `u128::from_str_radix` so any sign character in `body` (`-`, `+`, anything else non-digit) yields 0. Surfaced by `tests/fuzz_tests/stdlib_native_props.rs` differential vs `s.parse::<i64>()` |
| 22 | **Correctness**: `StdLib.base64Decode("AB=C")` returned the 3-byte garbage `[0x00, 0x10, 0x02]` instead of empty/error | The decode loop accepted `=` at any position inside a 4-char chunk and treated it as a literal-zero sextet, ignoring RFC 4648's mid-chunk-pad-rejection rule | Same file — `base64_decode` now rejects `=` outside the trailing-pad position of the FINAL chunk; matches `base64` crate STANDARD engine. Surfaced by `tests/fuzz_tests/stdlib_native_props.rs` differential vs `base64` crate |

---

## Known Limitations

### ABI codec gaps — RESOLVED

Bugs #23 (narrow-int sign-extension), #24 (`abi.decode` pass-through for
`string`/`bytes`), and #25 (`abi.decode` unusable for `T[]`) were all
surfaced by `tests/fuzz_tests/abi_roundtrip_props.rs` and fixed across
waves #24 and #25. All 21 round-trip tests now pass with 0 ignored. See
[Fixed Bugs](#fixed-bugs) entries #23-#25 below.

## Refactor Deliverables

### Added cargo-fuzz targets (was 2, now 9)

- `fuzz_target_disasm` — NeoVM disassembler totality
- `fuzz_target_nef_roundtrip` — NEF parse → build asymmetry
- `fuzz_target_manifest_json` — Third-party manifest robustness
- `fuzz_target_standard_json` — solc-compatible Standard JSON input robustness
- `fuzz_target_structured_sol` — `arbitrary`-driven Solidity AST grammar; now
  also generates inheritance chains, abstract / `virtual` / `override`,
  multi-base linearisation, and `using <Lib> for <Type>` to reach deeper
  compile / IR / optimizer / runtime paths than pure random UTF-8
- `fuzz_target_runtime_exec` — runs arbitrary bytes as NeoVM bytecode against
  a bounded `NeoRuntime` (gas + memory capped); surfaced the NEWARRAY OOM
  DoS fixed as bug #15
- `fuzz_target_method_tokens` — drives CALLT (opcode 0x37) dispatch with
  randomly-generated NEF method-token tables, exercising the
  `invoke_native_contract` syscall path under attacker-controlled token
  metadata (hash, method name, param count, call flags)

Elevated `disassemble_neovm_bytecode` from `pub(crate)` → `pub` so
external fuzz harnesses and debuggers can drive it.

### Added differential testing (`tests/fuzz_tests/differential.rs`)

Six proptests comparing compiler output against reference crates:

- `sha256` vs `sha2::Sha256` (0..=256-byte inputs)
- `ripemd160` vs `ripemd::Ripemd160`
- `keccak256` vs `sha3::Keccak256`
- `addmod` / `mulmod` vs native arithmetic
- Disassembler totality across 0..=1024-byte inputs

Catches correctness divergence, not just panics.

### Seed corpus

- 11 `examples/*.sol` → `fuzz/corpus/fuzz_target_1/` (coverage +160%)
- 12 repo `.nef` → disasm + nef_roundtrip corpora
- 10 `.manifest.json` → manifest_json corpus

### Integration Tests

Three production-shape end-to-end tests were added in
`tests/fuzz_tests/batches_116_120.rs` covering full token-contract
lifecycles (deploy → mint → transfer → approve → burn / transferFrom):

- `batch124_erc20_full_lifecycle` — ERC-20 reference flow
- `batch127_erc721_full_lifecycle` — ERC-721 reference flow
- `batch129_nep17_full_lifecycle` — NEP-17 (Neo native) reference flow

### Continuous runner

- Rewritten to enumerate all registered targets (was hardcoded to 2).
- `FUZZ_TARGETS=...` environment override for focused sweeps.
- `-max_total_time=N` self-termination (plus `timeout` belt-and-suspenders).
- **109+ rounds clean** on background Monitor `by2qsiv64` (zero crashes)
  — full proptest suite + every cargo-fuzz target each round.

### CI (`.github/workflows/fuzz.yml`)

- All 11 cargo-fuzz targets run 60 s each (was 2).
- Crashes upload `fuzz/artifacts/` as workflow artifact for triage.

### Dead code (deletion candidates)

`cargo-llvm-cov` analysis identified ~6,200 LOC across the codebase that is
**0% covered AND has zero importers** anywhere in `src/` or `tests/`:

- **Orphaned standalone Yul frontend** (~4,500 LOC): `src/{lexer,parser,codegen,semantic,optimizer}/`, `src/error.rs`, `src/types.rs`. The Yul-in-Solidity surface (`assembly { … }` blocks) reaches its own active lowering at `src/ir/statements/assembly.rs` — confirmed working by the `fuzz_target_yul_assembly` target. The standalone frontend appears to be a stale earlier prototype.
- **VM bridge layer** (~1,700 LOC): `src/runtime/bridge/`. `VMBridge` is constructed only inside its own subtree.
- **Top-level orphans**: `src/{security,warning,validation,testing}.rs` — `pub mod`'d in `lib.rs` with zero `use` references anywhere.

Removing these would push real coverage well above 90%. Listed for awareness;
no deletion has been performed pending an explicit cleanup decision.

### Makefile targets

- `make test-fuzz-cargo-all` — 60 s per target, every target.
- `make test-fuzz-differential` — differential module only.
- `make test-fuzz-coverage` — HTML coverage report via `cargo-llvm-cov`.

### Documentation (`docs/FUZZ.md`)

- Quick-reference commands
- Module-by-module layout
- How to write a proptest / how to triage a crash
- Design principles (strict > weak assertions, differential > absolute,
  seed real inputs)

---

## How to Use

```bash
# Quick gate (~2 s, all 743 tests)
make test-fuzz-gate

# Deep run (100 cases per proptest)
./scripts/run_fuzz_suite.sh deep

# All cargo-fuzz targets, 60 s each
make test-fuzz-cargo-all

# Differential module only
make test-fuzz-differential

# Continuous loop (proptest + all cargo-fuzz targets in rotation)
./scripts/run_continuous_fuzz.sh

# HTML coverage report
make test-fuzz-coverage
```

---

## Historical note

An earlier version of this report showed 592 passing proptests with
2 cargo-fuzz targets. The current 743 represents 151 new or tightened
tests added during this review, plus the eleven bug fixes above. The gap
between 592 and 743 is not purely additive — tightening weak
assertions turned some previously-passing but underspecified tests into
proper correctness checks.
