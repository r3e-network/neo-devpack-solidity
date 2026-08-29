# v0.22 Audit-Fix Validation Report

| Field | Value |
|---|---|
| **Validated** | 2026-06-23 |
| **Source audit** | `AUDIT_REPORT_v0.21.md` (2026-06-17, commit `17664cd`) |
| **Code under review** | v0.22.0 — release commit `ad19dff` (`release: v0.22.0 compiler + devpack v2.2.0`) |
| **Branch / worktree** | `phase1/validate-v0.22` @ `.worktrees/phase1` |
| **Method** | Read every fix-commit diff + the current code at each cited location (did **not** trust commit messages), located the regression test for each finding, ran it, and judged whether the test would fail on pre-fix code. Closed with one full `cargo test --workspace` run. |
| **Tester** | Automated validation pass (PR0, Phase 1) |

---

## 1. Summary

**26 findings** in scope (6 critical S\* minus intentionally-deferred S5; 16 medium; 4 low).

| Verdict | Count | Meaning |
|---|---|---|
| **YES** | 19 | Root cause addressed in code **and** a regression test that would fail on pre-fix code (or an audit-accepted resolution). |
| **PARTIAL** | 7 | Fix lands the headline behavior but has incomplete scope, a test gap, or is documented-only. **Surfaced in §3.** |
| **NO** | 0 | Nothing was found to be unfixed relative to its commit's claim. |

**Bottom line:** Every fix commit does what its message claims. The 7 PARTIALs are **not** false claims — they are either (a) audit issues where the fix intentionally landed only the security-critical half and deferred the rest, (b) correct fixes that lack a behavioral end-to-end test, or (c) low-severity items documented rather than code-fixed. **No finding is NO.** The security-critical correctness fixes (S1 serialize, S2 storage gas, S4 multisig account, S6 storage-write gate, S7 rollback wiring) are real and exercised.

**Two things the owner must read before PR1–PR4:**
1. **§3 — the 7 PARTIALs.** Decide which are acceptable tradeoffs vs. follow-up work items.
2. **§5 — the commit-map discrepancy + the still-open structural gap.** v0.22 does **not** wire the 28 Neo-Express real-chain scripts into CI (audit M-TEST1/2/3, the report's #1 recommendation). That remains the largest open risk and is the only true root-cause remedy for the whole S1–S7 class.

---

## 2. Per-finding table

Legend — **Root cause?** = is the audit's named defect actually closed in code. **Reg test?** = a test exists that targets the fixed behavior. **Exercises bug?** = that test would plausibly fail on the pre-fix code.

| Finding | Fix commit | Root cause? | Reg test? | Exercises bug? | Notes |
|---|---|---|---|---|---|
| **S1** serialize JSON→binary | `5c2da83` | YES | YES (5 integ + 7 unit) | YES | Caveat A: Integer tag emits fixed 8-byte LE, real Neo emits `varint(len)`+minimal LE. Byte-identical for ByteArray/Boolean/Null/Array/Map; **diverges for integer-containing values**. |
| **S2** storage gas 1000× low | `8eea4dd` | YES | YES | YES | `STORAGE_PUT_PER_BYTE_GAS` 100→100_000; default `gas_limit` 10M→1B (under mainnet 2×10⁹ cap). |
| **S3** CheckSig synthetic hash | `9d16f23` | PARTIAL | YES | YES (injected arm) | Injectable `override_signing_hash()` API added (audit's explicit ask). **Default execution still uses the synthetic fallback** — correctness is opt-in. See §3. |
| **S4** CreateMultisigAccount | `9d16f23` | YES | YES (2) | YES | Now builds the verification script + `RIPEMD160(SHA256(script))`. Test recomputes the hash independently and guards against the old `SHA256(m‖pubkeys)[..20]` stub. |
| **S6** CallFlags bypass | `fea83cd` | PARTIAL | YES (3) | YES (storage arm) | `GetCallFlags` no longer hardcoded; `Storage.Put/Delete` fault without `WriteStates`. **Notify/AllowCall gating + manifest checks deferred; restricted context is host-armed, not auto-propagated on nested `Contract.Call`.** See §3. |
| **S7** no storage rollback on inner revert | `2459cdc` | YES (code) | PARTIAL | PARTIAL | Snapshot/restore wired through `handle_contract_call → CallFrame.storage_snapshot → dispatch_exception`. Tests cover the snapshot/restore **mechanism in isolation**; **no end-to-end faulting-inner-call-storage-rollback test.** See §3. |
| **M-IR** mulmod/addmod signed MOD | `6336e40` | PARTIAL | YES (3) | YES | `% m` routed through `emit_u256_divmod_ir` (fixes signed-MOD for modulus ≥2²⁵⁵). **512-bit `mulmod` intermediate product still wraps to 256 bits — explicitly TODO at the call site; the audit named this half too.** See §3. |
| **M-FE1** receive+onNEP17 dead code | `b63e361` | YES | YES | YES | Promoted W105→**E105 hard error** when `receive()` survives convert alongside an explicit `onNEP17Payment`. Detection uses convert's invariant (sound). |
| **M-FE2** fallback catch-all | `b63e361` | YES* | YES | weak | Stays at W105 (audit listed "loud warning" as an accepted option). "Loud" = slightly expanded message text, not a new severity. *Audit-acceptable; treatment is inconsistent with M-FE1's escalation. |
| **M-FE3** onNEP17 case-sensitive | `5bab6d5` ⚠ | YES | — | — | `eq_ignore_ascii_case`. **Not in `bc06f0c` as the task map claimed — see §5.** No dedicated test in the commit. |
| **M-FE4** ETHER_UNIT_RE false positives | `5bab6d5` ⚠ | YES | — | — | Regex now requires `(?:\d|\))` prefix, so `// whether` / `uint ether =` no longer match. **Not in `bc06f0c` as the task map claimed — see §5.** No dedicated test in the commit. |
| **M-FE5** state-var type clash | `bc06f0c` | YES | YES (suite) | YES | Inheritance-flatten path now emits **E122** for same-name+different-type non-constants (same-name+same-type stays W122). |
| **M-BC1** 3 unchecked slice writes | `5bab6d5` ⚠ | YES | partial | partial | `emit_ir_function` + `apply_method_tokens` now `Result`; unresolved labels/calls + out-of-range patches + u16 overflow → hard `Err` (were silent fall-through / panic / zero-operands). Error-path hardening; no test feeds a malformed IR to trigger each arm. |
| **M-BC2** `x==true→x` unsound opt | `5bab6d5` ⚠ | YES | YES | YES | Rewrite deleted from `-O3`; stale test updated. |
| **M-BC3** `MethodToken::serialize` assert | `5bab6d5` ⚠ | YES | — | — | Returns `Result` instead of `assert!`. No dedicated test. |
| **M-DEV1** NEP-11 self-escrow | `63c1efa` | YES | NO | NO | `to != address(this) && to.code.length > 0` guard added to `_transfer` + `_mint` (mirrors NEP-17). **Compile-verified only; no behavioral test that a self-transfer now succeeds.** See §3. |
| **M-DEV2** NEP-24 tokenId bytes32 | `bc06f0c` | YES | YES (compile) | — | `royaltyInfo` + `_tokenRoyalty` mapping + event all `bytes`. |
| **M-DEV3** NEP-17 hybrid auth | `bc06f0c` | YES | — | — | Inline doc explains the witness∥allowance model. **Audit's ask was literally "should be documented" — fully satisfied.** |
| **M-RT1** GetNotifications empty | `bc06f0c` | YES | — | — | `Notify` records `(src,name,state)`; `GetNotifications` returns the filtered list. Source simplified to `default_account_bytes`. No dedicated test in the commit. |
| **M-RT2** GetRandom deterministic | `bc06f0c` | YES* | — | — | `override_random_seed()` host API added (audit: "at least inject an configurable seed"). **Default still deterministic.** |
| **M-RT3** CheckWitness semantics | `bc06f0c` | YES* | — | — | `add_witness_signer()` host API (audit: "support injecting witness list"). **Default fallback unchanged.** |
| **M-RT4** revert/fault by substring | `bc06f0c` | YES | — | — | Discriminate by `revert_payload` marker, not `rendered.contains("THROW")`. |
| **L-FE1** `parse_source` silent drop | `bc06f0c` | PARTIAL | — | — | **Documented (comment), not fixed** — audit asked for a warning to be emitted; explicitly tracked as a follow-up. See §3. |
| **L-FE2** W121 collision | `bc06f0c` (pre-plan ✓) | YES | YES | YES | Library diagnostic renumbered **W121→W124**; W121 now lives only at `state_variables.rs:162`. |
| **L-BC** CALLT `u16::MAX` alias | `5bab6d5` (pre-plan ✓) | YES | — | — | `u16::try_from(index).map_err(...)?` at `bytecode_core.rs:261-265`. |
| **L-DEV** `getGovernanceInfo` dead iterator | `bc06f0c` | PARTIAL | — | — | **Documented as a showcase TODO, not fixed** (`activeProposals`/`executedProposals` still always 0). See §3. |

⚠ = commit is **not** in the task's fix-map table; see §5.

\* = audit-accepted resolution (warning / host-injection API) rather than a default-behavior change.

---

## 3. PARTIAL findings — detail

These are the items to read before approving PR1–PR4. None is a false claim, but each has a real caveat.

### S3 — CheckSig correctness is opt-in
`get_current_message_hash()` prefers an injected hash when armed, else falls back to the synthetic `SHA256(bytecode‖account‖counter)`. The injection API the audit asked for exists and is correctly drained per-execution. **But the default execution path still verifies signatures against a hash that matches no real signature** — so any test that does not call `override_signing_hash()` gets meaningless CheckSig results, exactly as in v0.21. This is a deliberate backward-compat choice (keeps 594 proptests green), and the audit's literal request ("provide an injection API") is satisfied. **Owner decision:** is opt-in correctness acceptable, or should the default flip (with the proptest migration cost)?

### S6 — only the storage-write flag is enforced
Done: `GetCallFlags` returns the real active flags; `Storage.Put/Delete` fault without `WriteStates` (the staticcall safety guarantee). **Deferred** (commit message is explicit): `Notify`/`Log` (`AllowNotify`) and nested `Contract.Call` (`AllowCall`) are not flag-gated, and manifest permission checks are not added at all. Additionally the restricted context is only settable via the `override_call_flags()` host API — the runtime does **not** auto-propagate a restricted flag when a compiler-emitted `staticcall` enters a nested `Contract.Call`, so the protection only fires in host-driven tests. The audit's manifest-permission complaint remains open. **Owner decision:** is storage-write gating a sufficient v0.22 scope, or are Notify/AllowCall + manifest required before further work?

### S7 — correct wiring, missing end-to-end test
The code is right: snapshot taken at the contract-call frame push (`execution_impl_part2_contract_call.rs`), restored when that frame unwinds in `dispatch_exception` (`try_frames.rs`). But both regression tests (`snapshot_restore_discards_callee_writes_keeps_caller_state`, `snapshot_is_not_empty_when_overlay_has_entries`) call `snapshot_storage_overlay`/`restore_storage_snapshot` **directly** on a hand-seeded overlay. They prove the mechanism; they do **not** execute `Contract.Call → Storage.Put → THROW → catch` and assert the storage was rolled back. An off-by-one in the wiring (snapshot on the wrong frame, restore on the wrong unwind iteration) would leave these tests green. **Recommend:** add one end-to-end test before PR1–PR4 touch the runtime.

### M-IR — signed-MOD fixed, 512-bit mulmod is TODO
`% m` now goes through the conformant `emit_u256_divmod_ir`, fixing the wrong-residue bug for modulus ≥ 2²⁵⁵ (3 tests, RED-verified pre-fix). **But** the intermediate `a*b` for `mulmod` is still emitted as a native `Mul` that truncates to 256 bits, so `mulmod` is wrong whenever `a*b ≥ 2²⁶⁶`. The audit called this out by name ("中间积 `a*b` 还可能在窄路径上静默回绕"); the fix acknowledges it inline as `TODO(full-512-bit-mulmod)`. `addmod` is fully conformant (sum fits). **Recommend:** file a follow-up issue; until then document the mulmod input range in the devpack.

### M-DEV1 — correct fix, no behavioral test
The 2-line `to != address(this)` guard mirrors the proven NEP-17 pattern and is obviously correct. The commit only verifies "CompleteNEP11NFT.sol compiles end-to-end" — there is **no test that transfers/mints an NFT to `address(this)` and asserts success** (the exact scenario the audit said was hard-blocked). Low risk because the change is trivial, but a behavioral test is the proper guard against a future regression that removes the clause.

### L-FE1 — documented, not fixed
`frontend_parse.rs:94` still uses `_ => {}` to silently drop unrecognized `SourceUnitPart` variants. The fix adds a comment explaining the catch-all; it does **not** emit a diagnostic (the audit's suggestion). Explicitly tracked as a follow-up ("threading a diagnostics sink through this function"). Genuine low severity — today every supported variant is handled — but the audit item itself is not closed.

### L-DEV — documented, not fixed
`CompleteNEP17Token.getGovernanceInfo()` still iterates a raw `"proposal"` prefix that yields nothing (`activeProposals`/`executedProposals` always 0). The fix documents why and sketches the correct enumeration-index design; it does not implement it. Showcase-example only; no on-chain impact for devpack consumers.

---

## 4. Pre-plan validation (re-confirmed in code)

Both items claimed fixed during pre-plan are genuinely fixed in the v0.22 tree:

- **L-FE2 (W121 collision) — CLOSED.** W121 now appears only at `src/solidity/validate/contract/state_variables.rs:162` (the sibling-merge constant-duplicate warning). The library external-visibility diagnostic at `src/solidity/validate/contract/library.rs:63` emits **W124**. The two unrelated diagnostics are now distinguishable by code. Test: `analyzer_cli_tests.rs` expects `warning[W124]`.
- **L-BC (CALLT `u16::MAX` alias) — CLOSED.** `src/cli/bytecode/bytecode_core.rs:261-265` uses `u16::try_from(index).map_err(|_| ...)?` propagation; the old `unwrap_or(u16::MAX)` aliasing to token #511 is gone. (This site is also claimed under M-BC1 in commit `5bab6d5`.)

---

## 5. Commit-map discrepancy (important) + structural gap

### The task's fix-map table is wrong for the polish cluster
The task description maps the entire polish cluster (M-FE3/4/5 + M-BC1/2/3 + M-DEV2/3 + M-RT1/2/3/4 + L-\*) to a single commit `bc06f0c`. **That is not what the repo shows.** Two commits did this work:

- **`5bab6d5`** `fix(bytecode+frontend): M-BC1/2/3 + M-FE3/4` — fixes **M-BC1, M-BC2, M-BC3, M-FE3, M-FE4**. This commit is **not in the task's fix-map table at all.** It IS an ancestor of the v0.22 release (`git merge-base --is-ancestor 5bab6d5 ad19dff` → YES), so v0.22 contains the fixes; the task map just failed to cite the commit.
- **`bc06f0c`** `fix(validate+runtime+devpack): M-FE5/M-DEV2/3 + M-RT1/2/3/4 + L-* polish` — the actual scope is **M-FE5, M-DEV2, M-DEV3, M-RT1, M-RT2, M-RT3, M-RT4, L-FE2 (renamed W124), and documentation-only for L-FE1 + L-DEV.** It does **not** touch M-FE3/M-FE4/M-BC1/M-BC2/M-BC3.

**Impact on this report:** every listed finding is still validated above against its *real* commit. The discrepancy is a task-mapping error, not a missing fix. Future phases should cite `5bab6d5` for the bytecode/frontend cluster.

### Structural gap still open: Neo-Express real-chain oracle is NOT in CI
The audit's #1 recommendation (§5 and §7.P0-2) — wire the 28 existing Neo-Express real-chain smoke scripts (`make test-deploy-smoke-full`) into CI as a required gate — is **not addressed by v0.22.** None of the 9 fix commits touch CI config for this. This is the audit's named root-cause remedy for the entire "passes in the simulator, fails on-chain" class (S1–S7). The v0.22 runtime fixes above make the simulator *more* faithful, but without a real-chain oracle in CI, regressions of this class can still land silently. The audit's M-TEST1 (e2e tests compile-only), M-TEST2 (conformance targets internal consistency), M-TEST3 (optimizer diff ignores Notify/storage) are likewise still open — they were not in the v0.22 fix set. **Strongest recommendation in this report: land the Neo-Express CI gate before Phase 7.**

### Other audit items not claimed by v0.22 (for completeness)
M-IR2 (logical-OP right operand bool normalization), M-IR3 (same-sign different-width overload match), and M-TEST1/2/3 (above) are in the v0.21 audit but were not in v0.22's fix set. They remain open and should be triaged into a later phase.

---

## 6. Test evidence

Full workspace run (`cargo test --workspace`) is **green**. Targeted re-runs for the audit-fix tests:

| Test target | Result |
|---|---|
| `runtime_stdlib_binary_serialize_tests` (S1) | **5/5 pass** |
| `runtime_syscall_tests` (S3, S4, S6) | **19/19 pass** |
| `runtime_gas_tests` (S2) | **6/6 pass** |
| `storage_ops::s7_tests` (S7 mechanism) | **2/2 pass** |
| `semantics::arithmetic` addmod/mulmod (M-IR) | **3/3 pass** |
| `metadata::receive_with_existing_onnep17payment_is_a_hard_error` (M-FE1) | pass |
| `ir_opt::neovm_simplify` (M-BC2) | **2/2 pass** |
| `uint256_ops_tests` (prototype internal, see §7) | **15/15 pass** |
| full `cargo test --workspace` | green across all bins |

---

## 7. `uint256_ops.rs` decision

**Recommendation: DELETE (file a follow-up cleanup task — does NOT block PR1–PR4).**

### Evidence
- **File:** `src/cli/bytecode/uint256_ops.rs`, 1491 LOC, every function `#[allow(dead_code)]`. Textually `include!`'d from `src/cli/bytecode.rs:39` (so it compiles + its 15 internal tests run), but **no production caller exists** — `rg "uint256_ops"` finds only the `include!` plus 4 **comment** references (`ir/expressions/dispatch/binary.rs:649,1168`, `bytecode_helpers/ops_and_literals.rs:104`, `runtime/execution/helpers/bitwise.rs:23`) describing it as the historical origin of the routines.
- **Git activity:** last touched `edf6589` on **2026-06-15** ("feat(bytecode): unsigned 256-bit division and modulo") — the "Phase 1 — software routines" prototype. No commit since the inline-IR path landed.
- **The migration was abandoned in favor of a different implementation.** `claudedocs/uint256-conformance-plan.md` begins:
  > "✅ LANDED — uint256 ≥ 2^255 conformance is complete (full suite green, 0 clippy). … `-`/`%` use the unsigned divmod routine (`emit_u256_divmod_ir`); comparison uses `emit_u256_unsigned_compare`; checked/unchecked `+ - *` use inline-IR limb routines … New `tests/fuzz_tests/uint256_conformance.rs` (13 tests) pins the behavior."
  The shipped path is the **inline-IR** routines in `ir/expressions/dispatch/binary.rs` + `runtime/`, **not** the standalone bytecode-emitting prototype in `uint256_ops.rs`. The M-IR fix (`6336e40`) routes through `emit_u256_divmod_ir`, confirming the inline-IR path is live.
- **Shipped path is correct:** audit §6.2 confirms the BigInteger/inline-IR path is conformant post-`6336e40`, and the plan doc's "✅ LANDED" section asserts full-suite green + clippy clean (re-verified: `uint256_conformance.rs` is in the green workspace run).

### Why DELETE, not WIRE-IN or DOCUMENT-AND-KEEP
- **Not WIRE-IN:** the wiring already happened via a different (inline-IR) design. The prototype's "Phase 2 — wiring" in the plan doc is marked superseded. There is no active plan to wire `uint256_ops.rs`; the only recent git activity is the prototype itself.
- **Not DOCUMENT-AND-KEEP:** the 4 "consumers" are comments, not dependencies. They describe history ("originated as a bytecode-level prototype … now emitted inline as IR"), not a living contract. Keeping 1491 LOC of `#[allow(dead_code)]` code + 15 tests that validate **unshipped** routines is a maintenance burden and provides **misleading coverage** (green tests on code that isn't in the production path).
- **Default-to-DELETE rule applies** (task criterion: "1491 LOC of dormant code is a maintenance burden").

### Caveat / follow-up
The one piece of genuinely unfinished uint256 work is the **full-512-bit `mulmod`** (see M-IR above). The prototype's 64-bit-limb schoolbook mul (`emit_uint256_{unchecked,checked}_mul`) is a useful reference for that future task — but "useful reference" lives in git history, not in 1491 LOC of compiled dead code. **Follow-up task:** delete `src/cli/bytecode/uint256_ops.rs`, remove the `include!` at `src/cli/bytecode.rs:39`, and convert the 4 comment references to past-tense (point at the inline-IR routines and, for the mul reference, at the commit history). This is a ~30-line cleanup PR; it does not gate PR1–PR4.

---

## 8. Recommendation for the phase plan

- **PR1–PR4 may proceed.** No finding is NO; the security-critical runtime fixes (S1/S2/S4/S6/S7) are real and tested.
- **Before PR1 lands,** add the two small tests flagged in §3: (a) S7 end-to-end faulting-inner-call storage rollback, (b) M-DEV1 NEP-11 self-transfer behavioral test. Both are low-effort and close the only two "fix correct but untested" gaps in the security path.
- **Track as explicit follow-ups** (do not block): S3 default-flip decision, S6 Notify/AllowCall + manifest gating, M-IR 512-bit mulmod, L-FE1 diagnostic, L-DEV showcase iterator, `uint256_ops.rs` delete.
- **Land the Neo-Express CI gate before Phase 7.** This is the single highest-leverage remaining action and the audit's #1 priority; v0.22 does not address it.
