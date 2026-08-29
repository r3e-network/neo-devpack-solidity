# Phase 1 — Validate v0.22 Audit Fixes + Close Outstanding Audit Items

**Status:** Approved 2026-06-23
**Scope:** Compiler correctness & safety hardening — first phase of a 7-phase
risk-ascending refactor of the Neo DevPack for Solidity compiler.
**Out of scope:** Architecture consolidation, god-object splits, public API
cleanup, performance work, the `include!` → `mod` migration. Those are
Phases 2–7 and each gets its own spec.

## Context

A full codebase exploration produced an initial report claiming "5
user-reachable `panic!`s". Re-verification against the actual source showed
this was wrong: **all 5 are inside `#[cfg(test)]` modules**. A more rigorous
audit found:

- **0** production `panic!`s in `src/`
- **0** unjustified production `unsafe` blocks (the 2 in
  `runtime/execution/helpers/storage_ops.rs` carry `// Safety:` comments and
  sit on the runtime's FFI boundary)
- **~5** real production `unwrap()`s outside test code (10 of the originally
  reported 21 are `Regex::new(literal).unwrap()` in `upgrade.rs` — provably
  safe because the patterns are compile-time constants; 6 more were test
  helpers under `cli/tests/` misclassified by the file path)
- **4** `TODO` comments in `src/`, all about the same already-tracked
  `mulmod`/`addmod` 512-bit gap

The authoritative prior review is **`AUDIT_REPORT_v0.21.md`** (dated
2026-06-17). The v0.22 git log (`git log 17664cd..ad19dff`) shows that
virtually every finding in that audit was addressed in v0.22:

| Finding | v0.22 fix commit |
| --- | --- |
| S1 (StdLib.serialize JSON vs binary) | `5c2da83` |
| S2 (storage gas ~1000× too low) | `8eea4dd` |
| S3 + S4 (CheckSig hash, multisig account) | `9d16f23` |
| S6 (CallFlags on storage writes) | `fea83cd` |
| S7 (storage rollback on inner revert) | `2459cdc` |
| CI Neo-Express smoke gate | `4495994` |
| M-IR (mulmod/addmod software divmod) | `6336e40` |
| M-FE1 / M-FE2 (receive vs onNEP17Payment; fallback) | `b63e361` |
| M-DEV1 (NEP-11 self-escrow short-circuit) | `63c1efa` |
| M-FE3 / M-FE4 / M-FE5 / M-BC1 / M-BC2 / M-BC3 / M-DEV2 / M-DEV3 / M-RT1 / M-RT2 / M-RT3 / M-RT4 / L-* polish | `bc06f0c` |
| `release: v0.22.0` | `ad19dff` |

**S5 (BLS12-381 Gt serialization) is intentionally not "fixed"** — it is
acknowledged as differential-test-only and gets a documentation PR instead.

This Phase 1 has two jobs:

1. **Validate** — do not trust commit messages. Read the actual diffs for each
   claimed fix, confirm the root cause is addressed (not just a symptom),
   confirm a regression test exists, confirm the test actually exercises the
   bug. Report any incomplete fixes.
2. **Close** — pick up the 7 small outstanding items the audit identified but
   v0.22 did not address, plus decide the fate of the dormant
   `uint256_ops.rs`.

## Deliverables

### PR0 — Validation report + uint256_ops recommendation (informational)

**File:** `docs/audits/AUDIT_v0.22_validation.md`

No code changes. The report contains:

1. One row per v0.21 finding (S1–S7 except S5, all M-*, the L-* items claimed
   in `bc06f0c`) with columns:
   - **Finding** — code + one-line description
   - **Fix commit** — SHA + one-line summary
   - **Root cause addressed?** — YES / PARTIAL / NO, with citation to the
     specific lines changed
   - **Regression test?** — test file + test name, or "NONE"
   - **Test exercises bug?** — YES / NO (does the test actually fail on
     pre-fix code? verified by reading the test, not by running it both ways
     unless ambiguous)
   - **Notes** — regressions, follow-ups, anything surprising
2. A summary section listing any finding marked PARTIAL or NO, with a
   recommended action.
3. A `uint256_ops.rs decision` section: investigate, then recommend DELETE /
   WIRE-IN / DOCUMENT-AND-KEEP with reasoning.

**Verification:** the report is read by the user before any fix PRs land. It
is a research artifact; its "tests" are reading code carefully.

### PR1 — Fix L-FE1: warn on unrecognized `SourceUnitPart`

**Source:** `AUDIT_REPORT_v0.21.md` §4.L-FE1
**File:** `src/frontend/frontend_parse.rs:94` (the `_ => {}` arm in
`parse_source`)

**Change:** Replace the silent `_ => {}` fall-through arm with a warning that
names the unrecognized construct kind. Use the existing `Diagnostic::warning`
machinery (the same one `solidity::Diagnostic` already flows through). Pick a
new diagnostic code (suggested: W130 — check
`solidity/solidity_errors.rs` for the next free code).

**Why:** Today, if solang-parser gains a new `SourceUnitPart` variant (or
returns something unexpected), the compiler silently compiles an empty
contract for that source unit. The user sees no signal.

**Out of scope:** promoting the warning to a hard error; that decision waits
until we've seen how often it fires on real-world inputs.

**Test:** add a unit test that constructs a `SourceUnitPart` variant not
handled by the match (or a syntactically exotic source) and asserts the
warning fires.

### ~~PR2 — L-FE2 (W121 collision)~~ — DROPPED, already fixed in v0.22

**Pre-plan validation (2026-06-23):** W121 now appears at exactly one site
(`src/solidity/validate/contract/state_variables.rs:162`). The library
diagnostic was reassigned to **W124**
(`src/solidity/validate/contract/library.rs:63`). The W-code sequence is
clean: W120 → W121 → W122 → W123 → W124 → W130. No work needed; record in
PR0's validation report as "already-closed, verify-only".

### ~~PR3 — L-BC (CALLT `unwrap_or(u16::MAX)`)~~ — DROPPED, already fixed in v0.22

**Pre-plan validation (2026-06-23):**
`src/cli/bytecode/bytecode_core.rs:261-265` now reads:

```rust
let token_index = u16::try_from(index).map_err(|_| {
    format!("bytecode emission: method-token index {index} exceeds u16 range ...")
})?;
```

The M-BC1 fix in commit `bc06f0c` covered this. No work needed; record in
PR0's validation report as "already-closed, verify-only".

### PR2 — Fix L-DEV: `CompleteNEP17Token.getGovernanceInfo()` iteration is empty

**Source:** `AUDIT_REPORT_v0.21.md` §4.L-DEV
**File:** `devpack/examples/CompleteNEP17Token.sol` — `getGovernanceInfo()`
around line 709.

**Bug:** The function iterates over an `abi.encode("proposal")` prefix, but
proposals are stored under a Solidity keccak slot, not under that prefix.
The iterator never matches, so `activeProposals` and `executedProposals` are
always 0.

**Change:** Derive the correct prefix (the same one the proposal-storage
writer uses — read the writer to confirm) and use it in the read path. If
the writer is also inconsistent, fix both.

**Why:** Example contracts that don't work undermine the "production-ready"
claim. Users copy-pasting this pattern inherit the bug.

**Test:** This is in `devpack/examples/`, which is covered by
`make test-deploy-smoke-full`. Add or extend a smoke test that creates a
proposal, calls `getGovernanceInfo()`, and asserts non-zero counts. If
extending the smoke suite is heavy, at minimum add a Solidity-level assert
inside a new example harness.

### PR3 — Document S5: BLS12-381 Gt serialization is differential-only

**Source:** `AUDIT_REPORT_v0.21.md` §2.S5
**File:** `src/runtime/execution/execution_impl_part2_native/crypto.rs:307-317`
(the `bls_serialize_gt` function using `format!("{gt:?}")`).

**Change:** This is a **small documentation + test-guard PR**, not a fix. The
v0.21 audit acknowledges the encoding is only useful for differential testing
within a single simulator run; making it canonical Neo N3 wire format is out
of scope.

The function `bls_serialize_gt` at
`src/runtime/execution/execution_impl_part2_native/crypto.rs:307-317` already
carries a thorough rustdoc comment (lines 307-314) explaining the
differential-only invariant — added in v0.22. Remaining work:

1. Verify no caller-visible entry point that consumes pairing output is
   undocumented.
2. Add a `#[cfg(test)]` round-trip guard that asserts the encoding is stable
   for a fixed known Gt value — this locks the encoding so a future "fix"
   doesn't silently break differential testing.

**Out of scope:** actually implementing canonical BLS12-381 Gt serialization
(that would be a follow-up phase; needs spec from the Neo N3 reference).

### PR4 — Close M-TEST3: optimizer differential must cover storage + events

**Source:** `AUDIT_REPORT_v0.21.md` §3.M-TEST3
**File:** `tests/fuzz_tests/optimizer_props.rs:230-278` — the
`optimizer_semantic_equivalence_storage_and_events` proptest.

**Bug:** The test compiles a contract that writes storage and emits an event
at O0 and O3, calls the method on each, and then compares results via
`assert_results_equivalent(&res0.unwrap(), &res3.unwrap())`. That helper
only compares `return_data`. A peephole pass that reorders `PUT` and
`Notify` (observable on-chain) would be invisible to this assertion.

**Change:** Extend the differential for at least one storage-writing and one
event-emitting test case to compare:
1. The final storage state (all key/value pairs written) across O0/O1/O2/O3.
2. The full `Notify` payload sequence (in order) across optimization levels.

Use the existing runtime simulator's storage snapshot + notification log.
If the simulator doesn't expose these (it should, post-S7 / M-RT1), document
the gap in the validation report and do as much as the API allows.

**Why:** The optimizer is the highest-risk component for silent
miscompilation. The current test gives false confidence.

**Test:** the new assertions are themselves the test. They should pass
trivially at O0=O0 and O0=O3 for the chosen cases (no known optimizer bug
here); their value is regression protection.

### `uint256_ops.rs` decision (output goes in PR0)

**File:** `src/cli/bytecode/uint256_ops.rs` — 1491 LOC, every function
`#[allow(dead_code)]`.

**Investigation steps:**
1. Read the file header comment (lines 1-14) which describes the migration
   plan ("flipping the simulator to two's-complement, and migrating the test
   suite is the remaining coordinated change").
2. Locate the referenced validation tests (the comment says "validated
   against a reference VM").
3. `git log -- src/cli/bytecode/uint256_ops.rs` to see whether the migration
   has had any recent activity or is abandoned.
4. Check whether any issue tracker references exist (search the repo).
5. Determine: is the BigInteger code path (currently shipped) measurably
   wrong on any case the dormant file handles correctly?

**Decision criteria:**
- DELETE if: migration abandoned, BigInteger path is correct, and the file
  is purely a maintenance burden.
- WIRE-IN if: there's an active plan to migrate, the file is referenced by
  recent issues/PRs, or the BigInteger path has known correctness gaps the
  file would close. (Wiring in becomes a follow-up phase, not Phase 1.)
- DOCUMENT-AND-KEEP if: the file is referenced externally (e.g., academic
  citation, downstream consumer) but unused internally; remove
  `#[allow(dead_code)]` in favor of a `#[doc(hidden)]` and a comment
  explaining the status.

The recommendation lands as a section of the PR0 validation report. If the
recommendation is DELETE, the actual deletion is a tiny follow-up PR
(`PR0b`). If WIRE-IN, it spawns a new phase. If DOCUMENT-AND-KEEP, the
doc-only change is `PR0b`.

## Sequencing

```
PR0 (validation report + uint256_ops recommendation)
  ↓ user reviews
PR1 (L-FE1 warning)         ──┐
PR2 (L-DEV governance fix)  ├── all independent of each other
PR3 (S5 BLS test guard)     ├── may land in any order
PR4 (M-TEST3 optimizer diff)──┘
  ↓
[PR0b if uint256_ops recommendation is DELETE or DOCUMENT]
```

PR0 records L-FE2 and L-BC as "already closed in v0.22, verified" so the
audit trail is complete. Each of PR1-PR4 is small enough for a focused
review. None depend on the others except PR0b which depends on PR0's
decision.

## Per-PR Verification Gate

Every code-touching PR (PR1–PR6, PR0b) must pass before landing:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

If the change touches anything that could affect on-chain output
(PR3 bytecode emission, PR4 devpack contract, PR6 optimizer coverage),
also run:

```bash
make test-deploy-smoke-full
```

PR0 (validation report only) needs no test gate; it is markdown.

## Risks

1. **Validation reveals an incomplete v0.22 fix.** Probability: low
   (commits look focused, v0.22 was a deliberate correctness release) but
   non-zero. *Mitigation:* if any finding marks PARTIAL or NO, stop and
   re-plan before opening fix PRs; the incomplete fix becomes its own PR.
2. **uint256_ops investigation surfaces a load-bearing dependency.**
   Probability: low (file is `#[allow(dead_code)]`). *Mitigation:* if
   WIRE-IN is the recommendation, it spawns a follow-up phase; Phase 1
   scope does not expand.
3. **PR6 discovers the simulator doesn't expose storage/notification state
   cleanly.** Probability: low (post-M-RT1 the simulator has
   GetNotifications). *Mitigation:* if blocked, document the gap in PR0's
   validation report and ship a partial PR6 (only what the API allows).
4. **"Many small PRs" review overhead.** *Mitigation:* each PR is genuinely
   small (one diagnostic code, one Err propagation, one doc comment);
   review cost per PR is minutes.

## Out of Scope (explicitly deferred)

- Phases 2–7 of the overall refactor (dead-code removal, error
  consolidation, god-object split, public API, performance, `include!` →
  `mod`).
- The S5 canonical-encoding fix (only documentation here).
- The actual `uint256_ops.rs` wiring-in if the decision is WIRE-IN.
- Extending the test pyramid to bring in Neo N3 node consensus vectors
  (M-TEST2) — that is a Phase 1 follow-up candidate but not in this spec.
- Any change to the 4 known `mulmod`/`addmod` `TODO` comments beyond
  confirming they were addressed by `6336e40`.

## Success Criteria

Phase 1 is done when:

1. `docs/audits/AUDIT_v0.22_validation.md` exists, covers every v0.21
   finding, and marks each YES / PARTIAL / NO with a cited test. L-FE2 and
   L-BC are recorded as "already closed in v0.22, verified during pre-plan
   validation".
2. The `uint256_ops.rs` decision is documented with reasoning.
3. PR1, PR2, PR3, PR4 have landed (4 PRs, down from 6 in the original
   design — see the pre-plan validation notes above), each passing the
   verification gate.
4. If PR0 surfaces any PARTIAL / NO findings beyond L-FE2 / L-BC, those
   have their own follow-up PRs scheduled.
5. `cargo test --workspace --all-features` is green on `main` after the
   last PR lands, and `make production-gate` is green.
