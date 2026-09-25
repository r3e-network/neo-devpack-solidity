# neo-devpack-solidity v0.31.0 — Release

**Release date:** 2026-09-26
**Compiler / CLI / workspace:** **v0.31.0**
**Target Neo N3 node:** **v3.10.0**

> Canonical change list: [`CHANGELOG.md`](./CHANGELOG.md) `[v0.31.0]`.

---

# neo-devpack-solidity v0.30.3 — Audit Hardening & Dead-Code Removal

**Release date:** 2026-08-29
**Compiler / CLI / workspace:** **v0.30.3**
**Target Neo N3 node:** **v3.10.0**

> Canonical change list: [`CHANGELOG.md`](./CHANGELOG.md) `[v0.30.3]`. The
> sections below summarize the latest full-codebase audit + refactor pass.

## TL;DR — v0.30.3 hardening pass

- **Dead-code removal (~1,600 lines)** — the `VMBridge` `instruction_mapping` /
  `system_calls` dispatch cluster (every `handle_*` opcode handler under
  `bridge/logic`, `bridge/stack`, `bridge/bridge_impl_arithmetic`, the
  arithmetic/bitwise/comparison/shifts stack-item helpers, and
  `bridge/bridge_impl_syscalls` + `bridge_helpers`) was initialized but never
  invoked: the live execution path is `VMBridge::new → execute →
  ExecutionContext::step()`, which already implements every opcode. The orphaned
  cluster was deleted; `VMBridge`, `VMBridgeError`, and `execute` remain the
  public surface. Full test suite (1,019 tests) unchanged → behavior preserved.
- **Import-resolution parity fix** — the standard-JSON resolver and the on-disk
  CLI resolver each carried a private copy of `import_aliases` that had
  silently drifted: only the CLI applied Foundry `@scope/pkg@x.y.z` version-pin
  aliases, so an identical project could compile via the CLI but fail import
  resolution via `--standard-json`. Both now share one canonical helper.
- **IR lowering perf** — the `Panic(uint256)` ABI selector is hoisted to a
  const (was a fresh Keccak hash at ~41 lowering sites), guarded by a unit test
  that recomputes the digest; the externally-callable-function lookup no longer
  allocates a `String` per query.
- **DevPack consistency** — removed six never-reachable `bls12381G1*/G2*`
  wrappers from the monolith `Syscalls.sol` (kept in sync with
  `SyscallsCrypto.sol`; the builtin whitelist already rejects them) and dropped
  an unused `SyscallsTypes.sol` import in `SyscallsOracle.sol`.
- **Toolchain hygiene** — `cargo fmt --all`, `cargo clippy --all-targets
  --all-features` (now zero warnings), and `cargo test --workspace
  --all-features` are all green.

---

# neo-devpack-solidity v0.27.0 — Real-World Compatibility & Performance

**Release date:** 2026-07-02
**Compiler / CLI / workspace:** **v0.27.0**
**devpack (`@neo-devpack-solidity/contracts`):** **v0.27.0**
**Target Neo N3 node:** **v3.10.0** (Gorgon-prep; no hardfork activation)

> Canonical change list: [`CHANGELOG.md`](./CHANGELOG.md) `[v0.27.0]`.
> Previous releases: see the
> [GitHub releases page](https://github.com/r3e-network/neo-devpack-solidity/releases)
> and `CHANGELOG.md` history (previous: v0.26.0).

---

## TL;DR

- **Real-world contract compatibility** — driven by a ~600-contract
  famous-contract evaluation (full dependency trees), the merge/inheritance
  gaps that blocked flagship protocols are fixed: Aave v3 went **71% → 85%**
  (every deployable implementation — `Pool`, `PoolConfigurator`, `AToken` —
  compiles), the Compound v2 core stack is unblocked, and USDC/MasterChef/ENS
  -style inherited-event patterns resolve. `new X{salt: s}()` (CREATE2 sugar)
  is accepted.
- **Latest-Solidity semantics** — a 168-probe runtime audit of 0.8.x features
  drove fixes for `bytesN` shifts (big-endian face value), unchecked `int256`
  two's-complement wrap, fixed-size array `.length`, and free-function
  overloads — plus **loud warnings** where NeoVM cannot honor EVM semantics
  (`W_USER_DEFINED_OPERATOR` for 0.8.19 operator overloading,
  `W_TRANSIENT_PERSISTED` for 0.8.28 transient storage) instead of silent
  miscompiles.
- **~1.9× faster in-tree NeoVM** on compute-heavy workloads (per-instruction
  stack-clone/String-alloc elimination, static opcode tables, allocation-free
  slot loads) plus compiler-side inference and sibling-merge complexity wins.
- **33-contract hermetic sample corpus** (`third_party/famous-contracts/samples/`)
  asserted at 100% by `cargo test --release --test famous_samples_compile`.
- **Zero-regression validation** — a 63-repro matrix re-verified every issue
  identified across the campaign: **54 verified fixed, 0 regressions**; all
  deferred items fail exactly as documented.

---

## 🌍 Real-world contract compatibility

The famous-contract evaluation compiled ~600 real Ethereum contracts with
their full dependency trees (~87% end-to-end; zero-knowledge **100%**,
solmate **100%**, solady **99%**, Uniswap-v3 **93%**, OpenZeppelin **89%**,
Aave v3 **85%**). The fixes in this release close the highest-frequency
gaps it found:

- **Library-declared events** are carried into the consuming contract, so
  Aave v3's `library ReserveLogic { event ReserveDataUpdated(...) }` pattern
  compiles (previously: "emit references event … which has no resolved
  declaration").
- **Abstract-typed fields** — a concrete contract holding a field of an
  `abstract contract` type (Compound v2's `PriceOracle` /
  `InterestRateModel`) is no longer forced to implement that abstract's
  virtuals.
- **Inherited events through cross-contract `new`** — the sibling-merge now
  walks the full base/interface closure, so `contract A is IERC20` emitting
  `IERC20.Approval` resolves when a factory does `new A()` (USDC FiatToken,
  SushiSwap MasterChef, ENS PublicResolver).
- **`new X{salt: s}(args)`** — CREATE2-style salted creation compiles; the
  salt is evaluated and ignored with a warning (Neo N3 has no CREATE2).

A curated, dependency-free subset — **33 self-contained famous contracts**
across DeFi / NFT / GameFi / zero-knowledge / infrastructure-DAO, spanning
Solidity **0.5.x–0.8.x** — now lives in-tree with a hermetic-compile test
asserting 100%.

## 🎯 Latest-Solidity semantics

A 168-probe **runtime-semantic** audit (assertions on EVM-correct values,
not just "does it parse") drove this wave:

- `bytesN << k` / `>> k` shift the **big-endian face value** with EVM
  truncation (previously faulted for N<32 or read the bytes little-endian).
- `unchecked { int256 }` Add/Sub/Mul wraps two's-complement mod 2^256
  (`type(int256).max + 1 == type(int256).min`); checked mode still
  Panics(0x11).
- Fixed-size storage arrays report `.length == N` (previously 0 — silently
  skipping `for (i = 0; i < a.length; i++)` loops).
- File-scope free-function **overloads** are all callable (previously only
  the first survived).
- **Honest diagnostics over silent miscompiles**: user-defined operators
  (0.8.19) warn `W_USER_DEFINED_OPERATOR` (raw arithmetic is emitted, not
  the bound function); `transient` state variables (0.8.28) warn
  `W_TRANSIENT_PERSISTED` (persisted — NeoVM has no EIP-1153 store).

## ⚡ Performance

- **In-tree NeoVM interpreter ~1.9× faster** on a compute-heavy benchmark
  (1.91s → 0.99s): the per-instruction hot path no longer clones the entire
  evaluation stack nor allocates an opcode-name String (debug-tracing only,
  now gated behind `enable_debugging`); opcode gas/name lookups index static
  `[_; 256]` tables; slot loads (`LDLOC`/`LDARG`/`LDSFLD`) are
  allocation-free on the success path.
- Compiler front-end: subscript type-inference no longer re-descends the
  same sub-tree up to 4× (exponential in `a[i][j][k]` depth); the
  sibling-merge transitive closure computes each contract's reference walk
  once per pass (byte-identical output verified).

## ✅ Validation

- **63-repro validation matrix** re-ran every issue identified across the
  bug-hunt backlog, the feature audit, and the famous-contract evaluation
  against this release: **54 verified fixed, 0 regressions, 0 surprises**;
  8 deferred items fail exactly as documented (each root-caused in-tree for
  follow-up), 1 known simulator-model artifact behaves as documented.
- Full suite: **1,964 tests across 55 targets** green (integration + lib
  unit + cargo-fuzz/proptest); `cargo clippy -D warnings` + `cargo fmt`
  clean; CI (Core + Neo-Express Differential + Fuzz) green.

### Known limitations (tracked, documented in-tree)

User-defined operator **dispatch** (0.8.19) and true transient storage
(EIP-1153) need type-system/VM work and currently warn; `abi.encodeCall` /
`abi.encodeWithSignature` produce Neo-native call descriptors rather than
raw EVM calldata; `bytesN` bitwise compound-assign (`r |= x`) stores a
byte-reversed value (bitwise reads are correct); nested struct-array-member
`push`, one inherited-getter dispatch shape, and `abi.encode` of fixed-size
arrays remain deferred with root causes documented.

---

## 📊 Headline numbers

| Metric | v0.26.0 | v0.27.0 |
| --- | --- | --- |
| Famous-contract compilation (full deps) | ~87% baseline established | merge gaps fixed; **Aave 71% → 85%** |
| Hermetic sample corpus | — | **33 contracts, 100% compile-tested** |
| Interpreter speed (compute-heavy) | baseline | **~1.9×** |
| Latest-Solidity semantic fixes | — | bytesN shifts, int256 wrap, `T[N].length`, free-fn overloads |
| Silent-miscompile guards | — | `W_USER_DEFINED_OPERATOR`, `W_TRANSIENT_PERSISTED` |
| Fix-validation matrix | — | **63 repros, 54 fixed, 0 regressions** |
| Total tests | 1,952 | **1,964** (55 targets) |
| Test failures | 0 | **0** |

---

## 🚀 How to upgrade

Backwards-compatible release — no public-API breaks or renames. Cargo and
npm users pick up `v0.27.0` on their next dependency resolution.

**CLI users**: rebuild with `cargo install --path . --version 0.27.0`
or download the prebuilt binary from the
[release page](https://github.com/r3e-network/neo-devpack-solidity/releases/tag/v0.27.0).

**Library users** consuming `neo_devpack_solidity` from another crate:
pick up `0.27.0` via `cargo update`.

**devpack users**: `npm install @neo-devpack-solidity/contracts@0.27.0`.

**Sample corpus**: browse `third_party/famous-contracts/samples/` and verify
locally with `cargo test --release --test famous_samples_compile`.

---

## 🧪 Verification

- `cargo test --release` ✅ **1,964 tests / 55 targets / 0 failed**
- `cargo clippy --all-targets --all-features -- -D warnings` ✅
- `cargo fmt --all -- --check` ✅
- 63-repro fix-validation matrix ✅ **0 regressions**
