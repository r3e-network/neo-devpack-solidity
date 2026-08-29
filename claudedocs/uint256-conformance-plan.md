# uint256 ≥ 2^255 conformance — implementation plan (#12)

## ✅ LANDED — uint256 >= 2^255 conformance is complete (full suite green, 0 clippy)

The atomic change is done and committed. uint256 values in `[2^255, 2^256-1]` are
stored as their conformant **32-byte two's-complement** end to end:
- **Literals** emit two's-complement PUSHINT256 (`ops_and_literals.rs`).
- **Comparison** `< > <= >=` uses the unsigned XOR-2^255 routine
  (`emit_u256_unsigned_compare`, gated `is_typed_uint256 && !is_int256`).
- **Checked + unchecked `+ - *`** use inline-IR limb routines over a shared scratch
  pool; overflow/underflow → `Panic(0x11)` via carry/borrow (not `GetSize>32`).
- **`/` `%`** use the unsigned divmod routine (`emit_u256_divmod_ir`).
- **`int256(x)`** for 256-bit is a no-op reinterpret (`type_constructors.rs`).
- **ABI decode** reads the slot as two's-complement (removed the `0x00`
  positive-magnitude sign-byte append in `emit_le_buffer_to_unsigned_integer`).
- **Runtime**: `bigint_to_stack_item` / `u256_twos_complement_item` emit 32-byte
  two's-complement for negatives; `shift_right` sign-extends narrow negatives;
  narrow arith overflow promotes to BigInt (NeoVM is arbitrary precision).
- New `tests/fuzz_tests/uint256_conformance.rs` (13 tests) pins the behavior;
  whole suite green, clippy clean.

The history below is the original plan/analysis, kept for reference.

## WORK IN PROGRESS — atomic landing (uncommitted; last commit is green)

Commit only when the whole suite is green. **DONE & validated (working tree):**
- `runtime/.../helpers/bitwise.rs`: `u256_twos_complement_item` → 32-byte
  two's-complement; **negatives emit a fixed 32-byte ByteArray** (so a uint256
  >= 2^255 stays a distinguishable 256-bit word, not a narrow i64).
- `runtime/.../arithmetic/basic_ops.rs`: `bigint_to_stack_item` same 32-byte rule
  for negatives; **narrow add/sub/mul overflow now PROMOTES to BigInt** (NeoVM
  integers are arbitrary precision; the i64 fault was a simulator bug).
- `runtime/.../bitwise.rs` `shift_right`: narrow NEGATIVE >> >=64 now sign-extends
  to -1 (was wrongly 0) — the limb routines need it.
- `ops_and_literals.rs`: uint256 literals `[2^255,2^256-1]` → two's-complement PUSHINT256.
- `ir/expressions/dispatch/binary.rs`: uint256 `< > <= >=` → `emit_u256_unsigned_compare`;
  **checked + unchecked `+ - *`** → inline-IR limb routines (`emit_u256_unchecked_{add,sub,mul}_ir`,
  `emit_u256_checked_arith`) over a shared scratch pool (`ctx.u256_scratch_locals`),
  overflow/underflow → `emit_panic(0x11)`. The old `GetSize>32` guard and the
  zero-pad widen are bypassed for uint256.
- `lowering_context.rs`: `u256_scratch` pool (reused across arith sites; no local blowup).
- Migrated unit pins: `test_bitwise_not_uint256_zero_returns_all_ones`,
  `push_integer_bigint_coerces_out_of_range_unsigned_literals`.
- NEW `tests/fuzz_tests/uint256_conformance.rs`: **11/11 GREEN** — literal round-trip,
  unsigned compare, equality, checked add/sub/mul overflow→Panic(0x11), unchecked
  wrap, and the **ERC-20 infinite-approval pattern**, all end-to-end conformant.

**REMAINING — 12 suite tests red (the only blockers to the green commit):**
1. **uint256 `/` `%`** still native (signed) → wire the unsigned divmod routine
   (Hacker's Delight; inline-IR using the add/sub helpers, own scratch range
   `s[15..]` so it does not clobber `s[0..3]`). Fixes `arith_scope_uint256_mul_overflow`.
2. **ABI round-trip of >=2^255** (7 tests: `abi_decode_*`, `roundtrip_uint256/int256/tuple`).
   `abi_pad32_be` (`execution_impl_part2_native/stdlib.rs`) treats a 32-byte ByteArray
   as natural BE, but uint256 is stored LITTLE-ENDIAN two's-complement — non-symmetric
   values get the wrong byte order. Needs integer-typed LE→BE handling (type is erased
   after PACK, so likely encode-side: emit a reverse for integer args, or a decode that
   matches).
3. **cast/storage 33-byte normalization** (`batch78`, `batch39`, `batch80`, `task108`):
   `int256(uint256.max)` returns a 33-byte `-(2^256+1)` instead of `-1` — a cast/storage
   path produces an un-normalized (>32-byte) two's-complement; normalize mod 2^256.

The CORE defect (literal + compare + checked/unchecked arithmetic + overflow — the
on-chain fault for `type(uint256).max` and the max-approval pattern) is RESOLVED and
validated; the remainder is div + the ABI/serialization byte-order + cast normalization.

## Status (2026-06-15)

**Phase 1 — software 256-bit routines: DONE & validated.**
`src/cli/bytecode/uint256_ops.rs` emits conformant NeoVM bytecode for the full
uint256 operation set, each operating on the **32-byte two's-complement**
representation with **no intermediate exceeding 32 bytes** (so it runs on a real
Neo node, unlike today's 33-byte literals / signed comparisons):

| Op | Function(s) | Technique |
|----|-------------|-----------|
| `<` `>` `<=` `>=` | `emit_uint256_unsigned_{lt,gt,le,ge}` | XOR-2^255 order-preserving map |
| `+` | `emit_uint256_{unchecked,checked}_add` | 128-bit limbs; carry-out = overflow (Panic 0x11) |
| `-` | `emit_uint256_{unchecked,checked}_sub` | 128-bit limbs; final borrow = underflow |
| `*` | `emit_uint256_{unchecked,checked}_mul` | 64-bit-limb schoolbook; high-column ≠ 0 = overflow |
| `>>` | `emit_uint256_logical_shr` | logical (native SHR is arithmetic) |
| `<<` | `emit_uint256_shl` | mask-before-shift limbs, wraps mod 2^256 |
| `/` `%` | `emit_uint256_divmod_body` | unsigned reduction (HD 9-3); CALLs add/sub; Panic 0x12 on /0 |

Validated by 15 unit tests against a **faithful reference VM** (signed
two's-complement, 32-byte fault, call frames) in the same file — including
values ≥ 2^255, limb boundaries, full wrap, and overflow/underflow/÷0 panics.
`type(uint256).max`, the ERC-20 max-approval pattern, `1<<255` and large-balance
`/1e18` are all covered by these primitives.

**Phase 2 — wiring: NOT DONE. Blast radius MEASURED (was over-estimated).**
The routines are not yet wired into lowering. An earlier estimate of "~1800 test
migrations / full StackItem rewrite" was an **assumption and was wrong**. Direct
measurement (flip `u256_bigint_to_stack_item` in `runtime/.../helpers/bitwise.rs`
to 32-byte two's-complement, run the whole suite) breaks **exactly 4 tests**:
`runtime::tests::test_bitwise_not_uint256_zero_returns_all_ones` (the new `~0 =
-1` all-ones IS what a node computes — test asserted the old model),
`abi_roundtrip_props::{roundtrip_uint256, roundtrip_tuple_uint_addr}` (the
`abi.encode` path must read a two's-complement uint256 as **unsigned**, i.e.
sign-extend to 32 bytes), and `batch57_..._uint256_cast_of_int256` (cast
reinterpretation). The simulator's **decode** (`coerce_item_to_bigint`) and
**arithmetic encode** (`bigint_to_stack_item`) are *already* faithful
two's-complement; only the bitwise/shift encode and a few serialize-to-bytes
paths assume positive-magnitude. So the model is **reachable**, not a rewrite.

**Why it is still all-or-nothing (the genuine coupling).** NeoVM is *untyped*:
the simulator (like a real node) cannot know a stack `Integer` is `uint` vs
`int`, so its wide comparison is **signed**. Flipping the representation to
two's-complement WITHOUT simultaneously emitting the unsigned-compare routine
makes `~uint256(0) < 5` evaluate `true` (silent wrong) where it is correctly
`false` today — and the suite's thin ≥2^255 coverage does not catch it (the
4-test measurement misses it). Likewise, changing literal emission to
two's-complement makes `max*2` native-`MUL` wrap to `-2` so the `GetSize>32`
overflow guard never trips (`arith_scope_uint256_mul_overflow` regresses) —
checked-overflow detection requires the software high-half. So **literals +
unsigned comparison + checked arithmetic (software, as CALL helpers) + ABI
encode/cast must flip together**; a partial flip trades a fail-loud on-chain
revert for silent miscomputation, which is strictly worse.

**Gating item:** the checked software arithmetic must be emitted as **CALL helper
functions** (each routine has its own `INITSLOT`, so it cannot be inlined into a
caller that already has one). The codegen infra is ready: `generate_contract_bytecode`
(`bytecode_core.rs`) already resolves `CALL_L` patches by target *name* against an
`offset_map`, so synthetic helper functions can be appended after the methods and
named-CALLed from uint256 op sites (op sites get type info from
`ir/expressions/dispatch/binary.rs`). Route via new IR instructions lowered in
`bytecode_emit_ir.rs`; reserve no extra method locals (helpers have own frames).

**Measured migration set (the second flip — simulator two's-complement encode +
conformant literal — run against the whole suite).** Exactly **15** tests, all
named, all expected-value migrations or coupled-wiring, *not* a rewrite:
- ABI encode/decode must read a two's-complement uint256 as **unsigned**
  (sign-extend to 32B): `abi_decode_uint256_max_roundtrip`,
  `abi_decode_uint256_half_roundtrip`,
  `abi_decode_uint256_array_with_high_bit_elements_roundtrip`,
  `abi_decode_struct_with_negative_and_max_fields_roundtrip`,
  `abi_roundtrip_props::{roundtrip_uint256, roundtrip_tuple_uint_addr,
  roundtrip_int256}`.
- Old-model unit pins to update: `test_bitwise_not_uint256_zero_returns_all_ones`
  (`~0 = -1` all-ones IS the node result), `push_integer_bigint_coerces_out_of_range_unsigned_literals`.
- Ternary/cast/storage reinterpretation: `batch39_n2_ternary_typed_arms_and_signed_widening`,
  `batch57_gg2_uint256_cast_of_int256_two_complement_reinterpretation`,
  `batch78_bbb4_storage_uint_to_signed_reinterpret_max_equals_minus_one`
  (its very name — "max equals minus one" — is the two's-complement identity),
  `batch80_ddd4_ternary_type_coercion_int_uint_mixed`,
  `task108_catch_panic_int256_min_div_neg_one_routes_canonical`.
- Overflow detection (the arith coupling): `arith_scope_uint256_mul_overflow` —
  needs the software checked-mul (native MUL wraps so `GetSize>32` never trips).

**Validated en route:** with the faithful (two's-complement) simulator encode, the
unsigned-comparison routine now executes **correctly** in the runtime (the XOR-2^255
trick requires two's-complement, which the old magnitude encode denied it). So
comparison wiring is unblocked.

This is a bounded, de-risked, multi-step landing (15 migrations + ABI unsigned-read
+ comparison wiring + arithmetic CALL-helpers + ≥2^255 contract tests), not a
rewrite — but the pieces are coupled (a partial flip silently breaks ≥2^255
comparison, which the suite under-covers) so they must land together, which is why
it cannot be a single green-in-one-session commit.

## Problem

NeoVM `Integer` stack items are **signed two's-complement and capped at 32 bytes**
(range `[-2^255, 2^255-1]`). Solidity `uint256` spans `[0, 2^256-1]`. Values in
`[2^255, 2^256-1]` — `type(uint256).max`, all-ones masks, `uint256(keccak256(...))`
(high bit set ~50% of the time), sentinels — **cannot be a positive NeoVM
integer**. The only ≤32-byte form is the two's-complement (the value mod 2^256,
which "looks negative"). Critically, even *adding two large valid uint256 values*
yields a 33-byte intermediate that NeoVM rejects.

The current compiler emits a **33-byte** push for such literals
(`push_integer_bigint` fallback, `bytecode_helpers/ops_and_literals.rs`) and uses
an unsigned-magnitude representation whose checked-overflow guard relies on
`GetSize > 32`. This works only because the **runtime simulator uses unbounded
`BigInt`** and never enforces NeoVM's 32-byte limit — so it is invisible to the
~1800-test suite but **faults on a real Neo node**.

## Empirical finding (why it's not a local fix)

Changing only the literal emission to 32-byte two's-complement (Step 1) was
attempted and **reverted**: it broke `arith_scope_uint256_mul_overflow`
(expected `Panic(0x11)`, got `Returned(2)`). With `max = -1`, `max * 2 = -2`
(1 byte) → `GetSize > 32` never trips → overflow undetected. This proves the
**overflow detection is entangled with the representation**: a conformant
representation requires reworking checked-arithmetic to detect overflow without a
33-byte intermediate — i.e. multi-word (software) arithmetic. Therefore the change
is **all-or-nothing**: representation + comparison + arithmetic-overflow + return
encoding + test updates must land together for a green checkpoint.

## Target representation

`uint256` = a **32-byte two's-complement** NeoVM integer (the value mod 2^256).
Values ≥ 2^255 appear negative. This is the only conformant form.

## Per-operation lowering

| Op | Strategy | Notes |
|----|----------|-------|
| literal ≥2^255 | PUSHINT256 with low 32 bytes (two's-complement) | small |
| `==`, `!=` | native EQUAL | byte-equal; already correct |
| `&`, `\|`, `^`, `~` | native | bitwise on two's-complement is correct |
| storage put/get | native | store the 32-byte value verbatim |
| `<`,`>`,`<=`,`>=` | **unsigned**: `(a XOR 2^255) <s (b XOR 2^255)` | gate on uint256/uint operands only; narrow uints (<2^255) keep native signed. `2^255` constant = PUSHINT256 `00..0080` (LE). XOR is ≤32-byte. |
| `>>` | **logical** for uint256 (native SHR is arithmetic/sign-extending) | mask high bits, or branch on sign |
| `<<` | mask result to 256 bits (already clamped for u256) | ensure ≤32 bytes |
| `+`, `-` | software byte-wise add/sub with carry/borrow → 32-byte result + carry-out flag | carry-out = checked-overflow signal (Panic 0x11); unchecked drops it |
| `*` | software schoolbook 32×32-byte → 64-byte product; overflow = high 32 bytes ≠ 0; low 32 bytes = result | expensive; emit as a CALL helper |
| `/`, `%` | software **unsigned** long division (native DIV is signed) | complex; emit as CALL helper |
| `**` | square-and-multiply over the software `*` with overflow check | reuses `*` |
| return/ABI encode | unsigned BE-pad to 32 bytes (read two's-complement as unsigned) | |

Overflow detection moves from `GetSize > 32` to the **carry/high-word** signals
produced by the software add/mul, so it no longer needs a 33-byte intermediate.

## Runtime simulator — ALSO non-conformant (second empirical finding)

The simulator does **not** faithfully model NeoVM's two's-complement integers; it
uses an **unsigned-magnitude** representation. `PUSHINT256` pushes a `ByteArray`,
and `u256_bigint_to_stack_item` (helpers/bitwise.rs) masks results to
`value & (2^256-1)` and re-emits them as a **positive** magnitude (appending a
`0x00` sign byte when the high bit is set). Verified by running a correct
XOR-trick comparison routine through the VM: `5 XOR 2^255` came back as
`+（2^255+5)` instead of the two's-complement negative a real node yields, so
`uint256.max < 5` evaluated to `true`. The routine is correct **on a real Neo
node** but cannot be validated against this simulator.

Consequence: the conformance work must **also** flip the simulator's uint256
representation to 32-byte two's-complement (`PUSHINT256` → Integer, XOR/AND/OR/NOT
and arithmetic dropping the unsigned-magnitude masking), which itself breaks the
existing uint256 tests that encode today's positive-magnitude behavior. So the
landing is a **three-way coordinated change** — compiler lowering + simulator
representation + test migration — not a compiler-only edit. Add an opt-in
`enforce_integer_size_limit` differential mode once the representation is flipped.

## Test strategy

1. Implement the software routines (`emit_uint256_checked_add/sub/mul`,
   `emit_uint256_unsigned_cmp`, `emit_uint256_unsigned_divmod`,
   `emit_uint256_logical_shr`) as bytecode-emitting functions with **isolated
   unit tests** (emit → run via simulator with values incl. ≥2^255 → assert) —
   green and independent of the default lowering.
2. Wire them into `lower_binary_expr` / comparison / literal emission as ONE
   reviewed change; update the ~dozen tests that encode today's 33-byte /
   positive-BigInt behavior (e.g. `arith_scope_uint256_mul_overflow`,
   `abi_roundtrip_props::roundtrip_int256`, ternary coercion tests,
   `return type(uint256).max`, `type(uint256).max + b`).
3. Turn on the simulator's 32-byte enforcement in a differential suite.
4. Validate against neo-go / a Neo N3 testnet (the simulator is not authoritative
   for the 32-byte limit).

## Rollout / effort

This is a **dedicated, multi-session feature** (estimate: software bignum routines
+ wiring + test migration + testnet validation). The representation flip is a
big-bang within the compiler (cannot be partially applied — see Empirical
finding), so land it on a branch with the routines unit-tested first, then the
wiring + test migration as a single reviewed PR. Until then, the compiler emits a
**compile-time warning** for unrepresentable literals (already shipped) so the
limitation is visible, not silent.
