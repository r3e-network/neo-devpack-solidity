# Neo-Solidity review — reconciled findings tracker

Baseline @ a13e58f: clean build, 0 clippy, 1758 tests green. Workflow (73 agents) + independent verification.

## ⛔ FALSE POSITIVE — DO NOT "FIX"
- **GAS native hash (`bytecode_core.rs:48`)** — workflow rated this CRITICAL ("hash wrong, canonical is 0x…48a47fa4c08bc06066"). **This is a shared LLM hallucination.** The code value reverses (LE→BE, mechanical) to `0xd2a4cff31913016155e38e474a2c06d08be276cf` = the REAL Neo N3 mainnet GAS hash (neo-go, explorers, repo tests/docs all agree). Changing it would break every GAS call. **LEAVE AS-IS.** LLMs are unreliable on long hex strings; both verifiers invented the same wrong "canonical". All 11 native hashes verified correct by mechanical LE-reversal.

## STATUS
| # | sev | area | finding | status |
|---|-----|------|---------|--------|
| 1,2,3 | crit/high | binary/compound | narrow-int overflow skipped w/ literal operand; compound & ++/-- | **F2 DONE** ✓ |
| 4,10,11 | high | binary | SHL no narrow width truncation (uint8 200<<1 = 400 not 144) | **F2 DONE** ✓ |
| 5 | high | power.rs | `**` no overflow guard / no truncation | **F2 DONE** ✓ |

## F2 IMPLEMENTATION (DONE) — narrow-integer checked-arithmetic
Shared `emit_arith_with_overflow_ladder(left,right,ctx,instr,op,allow_unchecked_u256_widen)` in binary.rs, called by both `lower_binary_expr` (true) and the compound/++/-- path (false). Key pieces:
- `is_typed_uint256` excludes literals; `is_narrow_result` = narrow operand present AND no genuine uint256/int256 (so `uint256+uint32` stays WIDE — critical: mixed-width is uint256 arithmetic). Gate early-outs in u256/unchecked-u256 gates use `is_narrow_result`.
- New: `should_truncate_unchecked_narrow_u/i`, `emit_truncate_narrow_unsigned/signed`, `shl_narrow_truncation`. Power guard in power.rs (narrow checked range-check / unchecked truncate).
- compound passes `allow_unchecked_u256_widen=false`: the Bug-#16 widen ends in a SUBSTR Buffer that breaks an l-value reused as integer index (`for(...; i++) a[i]` → PICKITEM fault). Regression caught by batch82_fff3; fixed.
- Tests: tests/arith_narrow_literal_tests.rs (18 cases incl. mixed-width). Probe tests/zz_scratch_arith_probe.rs = DELETE before finishing.
- Full-suite regression: re-running (was 1607pass/1fail before mixed-width fix; fixed).
| 6 | high(contested) | optimizer | `x*0 → PUSH 0` leaks multiplicand | **DONE** (confirmed real, removed rule) |
| 8 | high | manifest | NEP-11 explicit validation omits totalSupply/tokensOf | **DONE** ✓ |
| 13 | high | devpack | NEP17 witness-auth transfer reverts on allowance underflow | **DONE** ✓ (cache witness, guard) |
| 14 | high | devpack | VaultPattern share math uses post-deposit balance | **DONE** ✓ (pre-deposit denom) |
| 15,18 | med | getters | struct getter includes mapping/array members (invalid) | **DONE** ✓ (skip Mapping/Array; test updated) |
| 21 | med | devpack | Syscalls.scriptHashToAddress truncates wrong bytes | **DONE** ✓ (`address(uint160(scriptHash))`) |
| 22 | med | devpack | CompleteNEP11NFT.buyToken royalty needs seller witness | **DONE** ✓ (split buyer payment) |
| 28 | low | devpack | NEP17 constructor ignores maxSupply cap | **DONE** ✓ (require) |

## ADDITIONAL FIXES (DONE since first pass)
| 20 | med | binary.rs ladder | narrow signed div `intN.min/-1` Panic(0x11) / unchecked wrap | **DONE** ✓ (tests in arith_narrow_literal_tests.rs) |
| 7,16,19 | high/med | NeoType::canonical_abi_type | selector + event-topic0 struct→tuple / enum→uint8 / uint→uint256 | **DONE** ✓ — added `NeoType::canonical_abi_type()`; used by manifest selector (convert/functions.rs), `.selector` registry (solidity_analyse.rs, unified — verified consistent), and event topic0 (helpers.rs). Tests: selector_struct_enum_tests.rs (7, incl. runtime `.selector` consistency + struct event topic0). |

## F1 — function-pointer PUSHINT32+CALLA → PUSHA+CALLA — **DONE** ✓
`bytecode_emit_ir.rs` now emits `PUSHA` (0x0A, signed relative offset) for `PushFunctionOffset` instead of `PUSHINT32` (0x02 absolute); `bytecode_core.rs` AbsoluteOffset fixup computes the relative offset; the emulator `push.rs` PUSHA reads the relative offset and computes the absolute target. Real NeoVM `CALLA` now receives a proper `Pointer`. Tests: function_pointer_calla_tests.rs (runs to 84; asserts PUSHA+CALLA emitted).

## FINAL STATE
Build clean (release), `cargo clippy --all-targets` = 0 warnings, all 43 test binaries pass (~1787 tests, 0 failures). New test files: arith_narrow_literal_tests.rs (21), selector_struct_enum_tests.rs (8), function_pointer_calla_tests.rs (2); updated fix_abi_tests.rs, batches_31_45.rs, batches_18_30.rs to reflect corrected (EVM-canonical) behavior.

## RUNTIME-FIDELITY FIXES (DONE since the hook feedback)
| 9 | high | runtime crypto.rs | CryptoLib.verifyWithECDsa was a `=> false` stub | **DONE** ✓ — real ECDSA verification for all 4 NamedCurveHash values (secp256r1 via new `p256` dep for 23/123, secp256k1 via existing crate for 22/122; SHA256/Keccak256 message hashing). Tests: runtime_verify_ecdsa_tests.rs (valid sig → true, tampered → false, malformed → false). |
| 25 | low | runtime conversion.rs | CONVERT silently succeeded on Array/Map→Integer | **DONE** ✓ — faults (matches NeoVM). |
| 27 | low | runtime modular.rs | MODMUL/MODPOW signed path used `rem_euclid` | **DONE** ✓ — truncated `%` remainder (matches NeoVM/C# `%`). |

## #12 (uint256 literals ≥2^255) — COMPILE-TIME DIAGNOSTIC ADDED; full fix is architectural
NeoVM integers are signed and capped at 32 bytes (range `[-(2^255), 2^255-1]`), so Solidity `uint256` values in `[2^255, 2^256-1]` (notably `type(uint256).max`) CANNOT be a single positive NeoVM integer. The current emitter falls back to a 33-byte push (`push_integer_bigint`, ops_and_literals.rs) that faults on a real node once the value enters an integer op. **Now surfaced as a compile-time WARNING** (`neovm_integer_limit_warning` in ir/build/literals.rs, wired into raw-literal lowering and `type(...).max` in member_access/type_bounds.rs) so it is no longer SILENT — visible in CLI + standard-json output before deploy. Tests: uint256_literal_warning_tests.rs.

**Why not a full fix here:** the entire ≥2^255 path is entangled — the checked-arithmetic OVERFLOW guard relies on the unsigned-magnitude (`GetSize > 32`) representation; switching literals to 32-byte two's-complement (the only conformant form) makes them negative, which (a) breaks the overflow guard, (b) makes ordered comparison (`<`,`>`) and DIVISION/modulo silently WRONG (NeoVM ops are signed), and (c) diverges from the simulator. A correct fix requires unsigned-aware uint256 lowering throughout: 32-byte two's-complement representation + sign-flip (XOR 2^255) comparison + software unsigned division + a non-size-based overflow check + matching the simulator. That is a dedicated, testnet-validated effort; a partial fix would turn today's DETECTABLE on-chain fault into SILENT wrong results — a regression. Documented here as the headline architectural item.

## REMAINING (after the hook-driven pass)
| 12 | high | uint256 ≥2^255 | FULL fix deferred (architectural — see section above); compile-time WARNING now surfaces it. |
| 24 | low | manifest standards.rs | divisible NEP-11 (5-param transfer) not auto-detected. Low. |
| 17,23,26 | contested | runtime/storage | split adversarial verdicts — need deeper investigation before action. |

All other findings (F1, F2, #1–11, #13–16, #18–20, #25, #27, etc.) are FIXED and validated.
