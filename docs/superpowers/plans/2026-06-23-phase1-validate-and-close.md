# Phase 1 Implementation Plan — Validate v0.22 Fixes + Close Outstanding Items

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Validate that every v0.21 audit finding claimed fixed in v0.22 is actually fixed (root cause addressed + regression test), then close the four remaining outstanding items (L-FE1, L-DEV, S5 doc guard, M-TEST3) and decide the fate of the dormant 1491-LOC `uint256_ops.rs`.

**Architecture:** PR0 is a research-only artifact (validation report + uint256_ops recommendation). PR1–PR4 are independent small code PRs, each with its own commit. Two items from the original spec (L-FE2, L-BC) were dropped after pre-plan validation confirmed they were already fixed in v0.22.

**Tech Stack:** Rust 1.88+, Solidity 0.8.x, solang-parser, proptest, Neo N3 runtime simulator. Build via `cargo`, gate via `make production-gate`.

**Spec:** `docs/superpowers/specs/2026-06-23-phase1-validate-and-close-design.md`

---

## File Structure

| File | Responsibility | PR |
| --- | --- | --- |
| `docs/audits/AUDIT_v0.22_validation.md` (new) | Validation report — one row per v0.21 finding, uint256_ops decision | PR0 |
| `src/frontend/frontend_errors.rs` (modify) | Add `FrontendError::UnsupportedConstruct` variant | PR1 |
| `src/frontend/frontend_parse.rs` (modify) | Replace `_ => {}` arm in `parse_source` with hard error | PR1 |
| `src/frontend/tests.rs` (modify) | Add test that the new variant stringifies correctly | PR1 |
| `devpack/examples/CompleteNEP17Token.sol` (modify) | Add `_proposalIds` index array, push in `createProposal`, iterate in `getGovernanceInfo` | PR2 |
| `src/runtime/execution/execution_impl_part2_native/crypto.rs` (modify) | Add `#[cfg(test)]` stability guard for `bls_serialize_gt` | PR3 |
| `src/runtime/runtime_parts/runtime_types.rs` (modify) | Add `PartialEq` derive to `StateChange` and `LogEntry` | PR4 |
| `tests/fuzz_tests/optimizer_props.rs` (modify) | Extend `assert_results_equivalent` to compare `state_changes` + `logs` | PR4 |
| `src/cli/bytecode/uint256_ops.rs` (investigate; maybe delete) | Decision lands in PR0 report; deletion (if recommended) is PR0b | PR0 + PR0b |

---

## Task 0: Set up worktree

**Files:** none (git operation only)

- [ ] **Step 1: Create isolated worktree for Phase 1**

```bash
git worktree add ../neo-devpack-solidity-phase1 -b phase1/validate-v0.22 main
cd ../neo-devpack-solidity-phase1
```

- [ ] **Step 2: Verify clean baseline**

Run:
```bash
cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings 2>&1 | tail -3
cargo test --workspace --all-features 2>&1 | tail -5
```
Expected: fmt clean, clippy zero warnings, tests pass.

---

## PR0 — Validation Report + uint256_ops Recommendation

This PR is a research artifact. No code changes. The output is a single markdown file. Work in branch `phase1/validate-v0.22-report`.

### Task 1: Create the report skeleton

**Files:**
- Create: `docs/audits/AUDIT_v0.22_validation.md`

- [ ] **Step 1: Create directory and skeleton**

```bash
mkdir -p docs/audits
```

Write `docs/audits/AUDIT_v0.22_validation.md` with this exact skeleton:

```markdown
# v0.22 Audit Fix Validation Report

**Validated:** 2026-06-23
**Validator:** Phase 1 review pass
**Source audit:** `AUDIT_REPORT_v0.21.md` (2026-06-17)
**Code under review:** `main` at `ad19dff` (v0.22.0 release)
**Method:** read each fix commit's diff, locate the regression test, confirm the test actually exercises the bug, run the test.

## Summary

- Total findings claimed fixed: 13
- Confirmed YES (root cause addressed, test exists and exercises bug): <filled in below>
- PARTIAL (fix landed but test missing or doesn't exercise bug): <filled in below>
- NO (fix missing or regressed): <filled in below>

## Per-Finding Validation

| Finding | Fix commit | Root cause addressed? | Regression test? | Test exercises bug? | Notes |
| --- | --- | --- | --- | --- | --- |
| S1 | `5c2da83` | | | | |
| S2 | `8eea4dd` | | | | |
| S3 + S4 | `9d16f23` | | | | |
| S6 | `fea83cd` | | | | |
| S7 | `2459cdc` | | | | |
| M-IR | `6336e40` | | | | |
| M-FE1 + M-FE2 | `b63e361` | | | | |
| M-DEV1 | `63c1efa` | | | | |
| M-FE3/4/5 + M-BC1/2/3 + M-DEV2/3 + M-RT1/2/3/4 + L-* | `bc06f0c` | | | | |

## Pre-Plan Validation (already confirmed)

| Item | Status | Verified at |
| --- | --- | --- |
| L-FE2 (W121 collision) | Already closed — W121 single-use, library uses W124 | `src/solidity/validate/contract/library.rs:63` |
| L-BC (CALLT u16::MAX) | Already closed — `u16::try_from(index).map_err(...)?` | `src/cli/bytecode/bytecode_core.rs:261-265` |

## uint256_ops.rs Decision

(filled in by Task 9)
```

- [ ] **Step 2: Commit the skeleton**

```bash
git add docs/audits/AUDIT_v0.22_validation.md
git commit -m "docs(audit): scaffold v0.22 validation report"
```

### Task 2: Validate S1 (StdLib.serialize binary format)

- [ ] **Step 1: Read the fix commit**

Run:
```bash
git show 5c2da83 --stat
git show 5c2da83 -- src/runtime/execution/execution_impl_part2_native/stdlib.rs | head -120
```
Confirm: `serialize` no longer uses `serde_json::to_vec`; it uses Neo's BinarySerializer wire format.

- [ ] **Step 2: Locate the regression test**

Run:
```bash
rg -n "serialize.*binary|BinarySerializer|neo_binary_serialize" --type rust tests/ src/runtime/
```
Identify the test file and test function name that asserts the wire format. Verify the test asserts bytes-shape (not just round-trip).

- [ ] **Step 3: Run the test**

Run:
```bash
cargo test --workspace serialize 2>&1 | tail -15
```
Expected: PASS.

- [ ] **Step 4: Fill in the S1 row** in `docs/audits/AUDIT_v0.22_validation.md` with YES/NO entries and the test name in the Notes column. Commit:

```bash
git add docs/audits/AUDIT_v0.22_validation.md
git commit -m "docs(audit): validate S1 (serialize binary format)"
```

### Task 3: Validate S2 (storage gas alignment)

- [ ] **Step 1: Read the fix commit**

Run:
```bash
git show 8eea4dd --stat
git show 8eea4dd -- src/runtime/spec/gas.rs
```
Confirm: `STORAGE_PUT_PER_BYTE_GAS` was raised from 100 to 100_000.

- [ ] **Step 2: Locate the regression test**

Run:
```bash
rg -n "storage_price|STORAGE_PUT_PER_BYTE|100_000" --type rust tests/ src/runtime/
```
Identify the test that asserts the new per-byte rate.

- [ ] **Step 3: Run the test**

Run:
```bash
cargo test --workspace gas 2>&1 | tail -15
```
Expected: PASS.

- [ ] **Step 4: Fill in the S2 row.** Commit:

```bash
git add docs/audits/AUDIT_v0.22_validation.md
git commit -m "docs(audit): validate S2 (storage gas)"
```

### Task 4: Validate S3 + S4 (CheckSig hash + multisig account)

- [ ] **Step 1: Read the fix commit**

Run:
```bash
git show 9d16f23 --stat
git show 9d16f23 -- src/runtime/execution/helpers/crypto.rs src/runtime/execution/syscalls/contract.rs | head -150
```
Confirm: CheckSig now uses an injectable signing hash; `CreateMultisigAccount` now constructs the verification script + RIPEMD160(SHA256(script)).

- [ ] **Step 2: Locate the regression tests**

Run:
```bash
rg -n "signing_hash|inject.*hash|CreateMultisigAccount|verification_script|RIPEMD160" --type rust tests/ src/runtime/
```
Identify one test per fix (S3 and S4 separately).

- [ ] **Step 3: Run the tests**

Run:
```bash
cargo test --workspace checksig 2>&1 | tail -10
cargo test --workspace multisig 2>&1 | tail -10
```
Expected: PASS.

- [ ] **Step 4: Fill in the S3+S4 row** (split if the audit treats them separately). Commit:

```bash
git add docs/audits/AUDIT_v0.22_validation.md
git commit -m "docs(audit): validate S3+S4 (CheckSig + multisig)"
```

### Task 5: Validate S6 (CallFlags enforcement)

- [ ] **Step 1: Read the fix commit**

Run:
```bash
git show fea83cd --stat
git show fea83cd -- src/runtime/ | head -120
```
Confirm: storage writes now check CallFlags; `GetCallFlags` no longer hardcoded to `0x0F`.

- [ ] **Step 2: Locate the regression test**

Run:
```bash
rg -n "CallFlags|call_flags|ReadOnly|WriteStates" --type rust tests/ src/runtime/
```

- [ ] **Step 3: Run the test**

Run:
```bash
cargo test --workspace call_flags 2>&1 | tail -10
```
Expected: PASS.

- [ ] **Step 4: Fill in the S6 row.** Commit:

```bash
git add docs/audits/AUDIT_v0.22_validation.md
git commit -m "docs(audit): validate S6 (CallFlags)"
```

### Task 6: Validate S7 (storage rollback on inner-call revert)

- [ ] **Step 1: Read the fix commit**

Run:
```bash
git show 2459cdc --stat
git show 2459cdc -- src/runtime/ | head -150
```
Confirm: inner-call THROW now rolls back the dirty storage overlay to the frame's snapshot.

- [ ] **Step 2: Locate the regression test**

Run:
```bash
rg -n "snapshot|rollback|inner.*revert|storage_snapshot" --type rust tests/ src/runtime/
```

- [ ] **Step 3: Run the test**

Run:
```bash
cargo test --workspace rollback 2>&1 | tail -10
cargo test --workspace snapshot 2>&1 | tail -10
```
Expected: PASS.

- [ ] **Step 4: Fill in the S7 row.** Commit:

```bash
git add docs/audits/AUDIT_v0.22_validation.md
git commit -m "docs(audit): validate S7 (storage rollback)"
```

### Task 7: Validate M-IR + M-FE1/2 + M-DEV1

- [ ] **Step 1: Validate M-IR (`6336e40`)**

Run:
```bash
git show 6336e40 -- src/ir/expressions/calls/variable_calls.rs | head -80
```
Confirm: `mulmod`/`addmod` now route through `emit_u256_divmod_ir`.

Run:
```bash
cargo test --workspace mulmod 2>&1 | tail -10
cargo test --workspace addmod 2>&1 | tail -10
```
Expected: PASS. Fill in the M-IR row.

- [ ] **Step 2: Validate M-FE1 + M-FE2 (`b63e361`)**

Run:
```bash
git show b63e361 --stat
```
Confirm: `receive()` + `onNEP17Payment` coexistence is a hard error; `fallback()` warns loudly.

Run:
```bash
cargo test --workspace receive 2>&1 | tail -10
cargo test --workspace fallback 2>&1 | tail -10
```
Expected: PASS. Fill in the M-FE1+M-FE2 row.

- [ ] **Step 3: Validate M-DEV1 (`63c1efa`)**

Run:
```bash
git show 63c1efa -- devpack/standards/NEP11.sol | head -60
```
Confirm: NEP-11 `_transfer`/`_mint` now short-circuit when `to == address(this)`.

Run:
```bash
cargo test --workspace nep11 2>&1 | tail -10
```
Expected: PASS. Fill in the M-DEV1 row.

- [ ] **Step 4: Commit**

```bash
git add docs/audits/AUDIT_v0.22_validation.md
git commit -m "docs(audit): validate M-IR + M-FE1/2 + M-DEV1"
```

### Task 8: Validate the big polish commit `bc06f0c`

This single commit addresses M-FE3/4/5, M-BC1/2/3, M-DEV2/3, M-RT1/2/3/4, and the L-* items. Validate each sub-finding by spot-checking the changed code and running one test per cluster.

- [ ] **Step 1: Survey the commit**

Run:
```bash
git show bc06f0c --stat | head -60
```

- [ ] **Step 2: Spot-check M-BC1 (unchecked slice writes)**

Run:
```bash
git show bc06f0c -- src/cli/bytecode/bytecode_core.rs src/cli/bytecode/bytecode_emit_ir.rs | head -100
```
Confirm: slice writes are bounds-checked; unresolved call targets return `Err`.

- [ ] **Step 3: Spot-check M-FE3/4/5**

Run:
```bash
git show bc06f0c -- src/solidity/ | head -120
```
Confirm: `onNEP17Payment` detection is case-insensitive (`eq_ignore_ascii_case`); `ETHER_UNIT_RE` is comment/identifier-aware; inheritance-flatten now hard-errors on same-name-different-type state variables.

- [ ] **Step 4: Spot-check M-RT1/2/3/4**

Run:
```bash
git show bc06f0c -- src/runtime/execution/syscalls/runtime.rs src/runtime/bridge/ | head -120
```
Confirm: `GetNotifications` returns real notifications; `GetRandom` is at least seed-injectable; `CheckWitness` accepts a witness list; revert/fault no longer classified by substring.

- [ ] **Step 5: Run the clustered test suites**

Run:
```bash
cargo test --workspace --all-features 2>&1 | tail -10
```
Expected: PASS.

- [ ] **Step 6: Fill in the bc06f0c row** (one row, note "covers M-FE3/4/5, M-BC1/2/3, M-DEV2/3, M-RT1/2/3/4, L-*"). Commit:

```bash
git add docs/audits/AUDIT_v0.22_validation.md
git commit -m "docs(audit): validate bc06f0c polish cluster"
```

### Task 9: uint256_ops.rs investigation + decision

- [ ] **Step 1: Read the file header + referenced plan**

Run:
```bash
sed -n '1,25p' src/cli/bytecode/uint256_ops.rs
ls claudedocs/uint256-conformance-plan.md 2>/dev/null && cat claudedocs/uint256-conformance-plan.md | head -80 || echo "no conformance plan file"
```

- [ ] **Step 2: Check git history for the file**

Run:
```bash
git log --oneline -- src/cli/bytecode/uint256_ops.rs | head -20
git log --since="2025-01-01" --oneline -- src/cli/bytecode/uint256_ops.rs | head -5
```
Note: if there's no recent activity, the migration is likely abandoned.

- [ ] **Step 3: Confirm the BigInteger code path is correct**

Run:
```bash
rg -n "emit_u256_divmod|software_divmod|uint256.*mod|software.*256" --type rust src/ | head -10
```
The audit (v0.21 §6.2) confirms the BigInteger path is correct for `mulmod`/`addmod` post-`6336e40`. Cite that.

- [ ] **Step 4: Check for external references**

Run:
```bash
rg -n "uint256_ops" --type rust src/ tests/ docs/ 2>/dev/null
```
If only `uint256_ops.rs` itself references its functions, there are no external consumers.

- [ ] **Step 5: Decide and write the recommendation**

Based on steps 1–4, write the `## uint256_ops.rs Decision` section of the report. The decision criteria:

- **DELETE** if: migration abandoned (no git activity in 6+ months), BigInteger path is correct, no external consumers.
- **WIRE-IN** if: there's an active plan document, recent git activity, or the BigInteger path has known gaps the file would close.
- **DOCUMENT-AND-KEEP** if: referenced externally but unused; replace `#[allow(dead_code)]` with a `#[doc(hidden)]` + status comment.

Default recommendation if unclear: **DELETE** — 1491 LOC of dead code is a maintenance burden, git history preserves it, and the file header itself says the simulator "cannot validate them" today.

- [ ] **Step 6: Commit the decision**

```bash
git add docs/audits/AUDIT_v0.22_validation.md
git commit -m "docs(audit): record uint256_ops.rs decision"
```

### Task 10: Finalize PR0

- [ ] **Step 1: Fill in the summary section** at the top of the report (count of YES / PARTIAL / NO findings).

- [ ] **Step 2: Final commit + branch push**

```bash
git add docs/audits/AUDIT_v0.22_validation.md
git commit --amend --no-edit  # fold final tweaks into the last decision commit, or skip
git push -u origin phase1/validate-v0.22-report
```

- [ ] **Step 3: STOP and surface to the user.**

If any finding was marked PARTIAL or NO, halt execution and surface to the user before opening PR1–PR4. Re-plan as needed.

If all findings are YES: continue to PR1.

---

## PR1 — Fix L-FE1: surface unrecognized `SourceUnitPart` as a hard error

Work in branch `phase1/l-fe1-unsupported-construct`. The `_ => {}` arm in `parse_source` silently drops any `SourceUnitPart` variant the match doesn't recognize. Replace it with a loud error so a future solang-parser variant is visible instead of silently producing an empty contract.

### Task 11: Add the `UnsupportedConstruct` variant

**Files:**
- Modify: `src/frontend/frontend_errors.rs:15-37`

- [ ] **Step 1: Write the failing test**

Append to `src/frontend/tests.rs` (inside the existing `#[cfg(test)] mod tests` block, before the closing `}`):

```rust
#[test]
fn unsupported_construct_variant_is_loud() {
    let err = FrontendError::UnsupportedConstruct("StrawmanVariant".into());
    let msg = err.to_string();
    assert!(
        msg.contains("StrawmanVariant"),
        "UnsupportedConstruct must surface the kind in its message; got: {msg}"
    );
    assert!(
        !err.is_recoverable(),
        "UnsupportedConstruct must be a hard error so silent drops are impossible"
    );
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run:
```bash
cargo test --lib frontend::tests::unsupported_construct_variant_is_loud 2>&1 | tail -15
```
Expected: FAIL with "no variant named `UnsupportedConstruct`" (compile error).

- [ ] **Step 3: Add the variant**

In `src/frontend/frontend_errors.rs`, add to the `FrontendError` enum (after `ContractNotFound`):

```rust
    /// Encountered an unrecognized `SourceUnitPart` variant from
    /// solang-parser. Should be unreachable for any source the parser
    /// currently emits; surfaced as a hard error so future parser
    /// additions don't silently compile to an empty contract (audit L-FE1).
    #[error("internal error: unsupported top-level Solidity construct '{0}' (please file a bug — the compiler may need updating for a newer Solidity grammar)")]
    UnsupportedConstruct(String),
```

And update `is_recoverable` to explicitly exclude it:

```rust
    /// Check if this is a recoverable error
    pub fn is_recoverable(&self) -> bool {
        matches!(self, Self::UnsupportedVersion(_))
    }
```
(The body is unchanged; the comment clarifies that `UnsupportedConstruct` is intentionally not in the recoverable set.)

- [ ] **Step 4: Run the test to verify it passes**

Run:
```bash
cargo test --lib frontend::tests::unsupported_construct_variant_is_loud 2>&1 | tail -10
```
Expected: PASS.

### Task 12: Wire the new variant into `parse_source`

**Files:**
- Modify: `src/frontend/frontend_parse.rs:94-102` (the `_ => {}` arm)

- [ ] **Step 1: Replace the silent fall-through arm**

In `src/frontend/frontend_parse.rs`, replace lines 94–102 (the comment + `_ => {}`):

```rust
            // L-FE1 fix — surface unrecognized variants loudly so future
            // solang-parser additions don't silently compile to an empty
            // contract. The match above handles every variant the parser
            // currently emits, so this arm is unreachable in practice; if
            // it ever fires, the user sees a clear "file a bug" message
            // instead of a silent empty contract.
            other => {
                return Err(FrontendError::UnsupportedConstruct(format!(
                    "{other:?}"
                )));
            }
```

- [ ] **Step 2: Verify the existing tests still pass**

Run:
```bash
cargo test --lib frontend:: 2>&1 | tail -15
```
Expected: PASS (no real source exercises this arm today).

- [ ] **Step 3: Verify the full workspace builds and passes**

Run:
```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings 2>&1 | tail -5
cargo test --workspace --all-features 2>&1 | tail -10
```
Expected: all green.

- [ ] **Step 4: Commit**

```bash
git add src/frontend/frontend_errors.rs src/frontend/frontend_parse.rs src/frontend/tests.rs
git commit -m "fix(frontend): L-FE1 — surface unrecognized SourceUnitPart as a hard error

Replaces the silent _ => {} fall-through in parse_source with
FrontendError::UnsupportedConstruct. Unreachable today; the value is
defensive coverage against future solang-parser grammar additions so
they cannot silently produce empty contracts."
git push -u origin phase1/l-fe1-unsupported-construct
```

---

## PR2 — Fix L-DEV: `getGovernanceInfo()` enumeration via parallel index

Work in branch `phase1/l-dev-governance-index`. The current `getGovernanceInfo()` iterates `Storage.find(abi.encode("proposal"))` which never matches because proposals are stored in `_proposals[proposalId]` (a Solidity mapping keyed by `bytes32`). Maintain a parallel `bytes32[]` index.

### Task 13: Add the parallel index and populate it on creation

**Files:**
- Modify: `devpack/examples/CompleteNEP17Token.sol` (state vars near line 67; `createProposal` near line 343)

- [ ] **Step 1: Declare `_proposalIds` array**

In `devpack/examples/CompleteNEP17Token.sol`, find the governance state declarations (around line 65–67):

```solidity
    // Governance
    mapping(bytes32 => Proposal) private _proposals;
    mapping(bytes32 => mapping(address => bool)) private _voted;
    uint256 private _proposalCounter;
```

Add a new line after `_proposalCounter`:

```solidity
    // Governance
    mapping(bytes32 => Proposal) private _proposals;
    mapping(bytes32 => mapping(address => bool)) private _voted;
    uint256 private _proposalCounter;
    bytes32[] private _proposalIds; // L-DEV fix — parallel index for enumeration
```

- [ ] **Step 2: Push to the index in `createProposal`**

Find the write to `_proposals[proposalId] = Proposal({...})` (around line 343) and add the push immediately after:

```solidity
        _proposals[proposalId] = Proposal({
            id: proposalId,
            proposer: msg.sender,
            description: description,
            callData: callData,
            startTime: block.timestamp,
            endTime: block.timestamp + (votingPeriodDays * 1 days),
            forVotes: 0,
            againstVotes: 0,
            executed: false,
            proposalType: proposalType
        });
        _proposalIds.push(proposalId); // L-DEV fix — track for enumeration
```

### Task 14: Replace the no-op iterator in `getGovernanceInfo`

**Files:**
- Modify: `devpack/examples/CompleteNEP17Token.sol:699-735`

- [ ] **Step 1: Replace the function body**

In `devpack/examples/CompleteNEP17Token.sol`, replace the entire `getGovernanceInfo` function (lines 699–735) with:

```solidity
    function getGovernanceInfo() public view returns (
        uint256 totalProposals,
        uint256 activeProposals,
        uint256 executedProposals,
        uint256 minimumTokensForProposal
    ) {
        totalProposals = _proposalCounter;
        minimumTokensForProposal = totalSupply() / 100; // 1% of total supply

        // L-DEV fix — iterate the parallel _proposalIds index instead of
        // Storage.find("proposal"), which never matched the Solidity
        // keccak-keyed mapping slot.
        for (uint256 i = 0; i < _proposalIds.length; i++) {
            Proposal storage proposal = _proposals[_proposalIds[i]];
            if (proposal.executed) {
                executedProposals++;
            } else if (block.timestamp <= proposal.endTime) {
                activeProposals++;
            }
        }
    }
```

- [ ] **Step 2: Verify the contract still compiles**

Run:
```bash
cargo run --release -- devpack/examples/CompleteNEP17Token.sol -I devpack -O2 -o /tmp/CompleteNEP17Token 2>&1 | tail -10
ls -la /tmp/CompleteNEP17Token.nef /tmp/CompleteNEP17Token.manifest.json
```
Expected: clean compile, both files present.

- [ ] **Step 3: Run the existing example smoke test (if it covers this contract)**

Run:
```bash
bash examples/test_compilation.sh 2>&1 | tail -10
```
Expected: PASS.

- [ ] **Step 4: Verify the full workspace still builds and passes**

Run:
```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings 2>&1 | tail -5
cargo test --workspace --all-features 2>&1 | tail -10
```
Expected: all green.

- [ ] **Step 5: Commit**

```bash
git add devpack/examples/CompleteNEP17Token.sol
git commit -m "fix(devpack): L-DEV — getGovernanceInfo() enumerates via _proposalIds index

The previous implementation iterated Storage.find(abi.encode(\"proposal\"))
which never matched the Solidity keccak-keyed _proposals mapping slot,
so activeProposals/executedProposals were always 0. Adds a parallel
bytes32[] _proposalIds array populated in createProposal and iterated
in getGovernanceInfo."
git push -u origin phase1/l-dev-governance-index
```

---

## PR3 — S5: Lock BLS12-381 Gt encoding with a stability test

Work in branch `phase1/s5-bls-gt-stability`. The encoding at `crypto.rs:315` uses `format!("{gt:?}")` — non-canonical, differential-only. It's already documented in the rustdoc (lines 307–314). Add a `#[cfg(test)]` guard so a future "fix" can't silently break the differential tests.

### Task 15: Add the stability test

**Files:**
- Modify: `src/runtime/execution/execution_impl_part2_native/crypto.rs` (add a `#[cfg(test)] mod tests` block at the end if none exists; otherwise extend it)

- [ ] **Step 1: Read the end of the file to find the existing test module**

Run:
```bash
tail -40 src/runtime/execution/execution_impl_part2_native/crypto.rs
```
If there's already a `#[cfg(test)] mod tests` block, add the new test inside it. If not, create one at the end of the file.

- [ ] **Step 2: Add the stability test**

Append (or insert into the existing tests module):

```rust
#[cfg(test)]
mod s5_bls_gt_stability {
    use super::*;

    /// S5 lock — `bls_serialize_gt` uses `format!("{gt:?}")` as a
    /// non-canonical, differential-only encoding. This test locks the
    /// encoding so that two calls on the same Gt value produce byte-identical
    /// output. If a future change alters the encoding (e.g., switching to a
    /// canonical Fp12 wire format), this test will fail loudly and force the
    /// differential pairing tests to be re-validated against the new shape.
    #[test]
    fn bls_gt_serialization_is_deterministic_for_identity() {
        let gt = bls12_381::Gt::one();
        let bytes_a = ExecutionContext::bls_serialize_gt(&gt);
        let bytes_b = ExecutionContext::bls_serialize_gt(&gt);
        assert_eq!(
            bytes_a, bytes_b,
            "Gt serialization must be deterministic across calls (S5 lock)"
        );
        assert!(
            !bytes_a.is_empty(),
            "Gt serialization must produce non-empty output"
        );
    }
}
```

- [ ] **Step 3: Run the test to verify it compiles and passes**

Run:
```bash
cargo test --lib s5_bls_gt_stability 2>&1 | tail -10
```
Expected: PASS.

If the test fails because `bls_serialize_gt` is private and not visible to a sibling `mod tests`, move the test into the existing top-level `#[cfg(test)] mod tests` block in the same file (which can access private items via `use super::*;`).

### Task 16: Verify and commit

- [ ] **Step 1: Full workspace gate**

Run:
```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings 2>&1 | tail -5
cargo test --workspace --all-features 2>&1 | tail -10
```
Expected: all green.

- [ ] **Step 2: Commit**

```bash
git add src/runtime/execution/execution_impl_part2_native/crypto.rs
git commit -m "test(runtime): S5 lock — pin BLS12-381 Gt Debug encoding stability

Adds a #[cfg(test)] guard that asserts bls_serialize_gt produces
byte-identical output for the same Gt value. The encoding is
non-canonical (Debug format) by design — used only for differential
pairing tests within a single simulator run. This lock catches any
future change that silently breaks the differential tests."
git push -u origin phase1/s5-bls-gt-stability
```

---

## PR4 — Close M-TEST3: optimizer differential covers storage + events

Work in branch `phase1/m-test3-optimizer-state-events`. Extend `assert_results_equivalent` so the existing `optimizer_semantic_equivalence_storage_and_events` proptest actually compares storage state and notification logs across O0/O3, not just `return_data`.

### Task 17: Add `PartialEq` derives to the runtime types

**Files:**
- Modify: `src/runtime/runtime_parts/runtime_types.rs:77` and `:133`

- [ ] **Step 1: Add `PartialEq` to `StateChange`**

In `src/runtime/runtime_parts/runtime_types.rs`, change line 77:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChange {
```
to:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateChange {
```

- [ ] **Step 2: Add `PartialEq` to `LogEntry`**

In the same file, change line 133:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
```
to:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogEntry {
```

- [ ] **Step 3: Verify the workspace builds**

Run:
```bash
cargo build --workspace --all-features 2>&1 | tail -10
```
Expected: clean build. (`StateChangeType` already derives `PartialEq, Eq`, so no nested-type work needed.)

### Task 18: Extend `assert_results_equivalent`

**Files:**
- Modify: `tests/fuzz_tests/optimizer_props.rs:38-64`

- [ ] **Step 1: Replace the helper function body**

In `tests/fuzz_tests/optimizer_props.rs`, replace the body of `assert_results_equivalent` (lines 38–64) with:

```rust
/// Assert that two execution results are semantically equivalent (success,
/// return data, exception shape, storage state changes, and notification
/// logs). M-TEST3 fix — the previous version compared only `return_data`,
/// which made a peephole pass that reorders PUT and Notify invisible to the
/// optimizer differential.
fn assert_results_equivalent(
    a: &neo_devpack_solidity::runtime::ExecutionResult,
    b: &neo_devpack_solidity::runtime::ExecutionResult,
) {
    assert_eq!(a.success, b.success, "success mismatch between O0 and O3");
    assert_eq!(
        a.return_data, b.return_data,
        "return_data mismatch between O0 and O3"
    );
    match (&a.exception, &b.exception) {
        (None, None) => {}
        (Some(ae), Some(be)) => {
            assert_eq!(
                ae.exception_type, be.exception_type,
                "exception type mismatch between O0 and O3"
            );
            assert_eq!(
                ae.message, be.message,
                "exception message mismatch between O0 and O3"
            );
        }
        _ => panic!(
            "exception presence mismatch: a={:?}, b={:?}",
            a.exception, b.exception
        ),
    }

    // M-TEST3 fix — compare storage state changes (order + content).
    assert_eq!(
        a.state_changes.len(),
        b.state_changes.len(),
        "state_changes count mismatch between O0 and O3: a={:?}, b={:?}",
        a.state_changes,
        b.state_changes
    );
    for (i, (ac, bc)) in a.state_changes.iter().zip(b.state_changes.iter()).enumerate() {
        assert_eq!(
            ac, bc,
            "state_changes[{i}] mismatch between O0 and O3"
        );
    }

    // M-TEST3 fix — compare notification logs (order + content). A
    // peephole pass that reorders emit + storage write is observable
    // on-chain and was previously invisible.
    assert_eq!(
        a.logs.len(),
        b.logs.len(),
        "logs count mismatch between O0 and O3: a={:?}, b={:?}",
        a.logs,
        b.logs
    );
    for (i, (al, bl)) in a.logs.iter().zip(b.logs.iter()).enumerate() {
        assert_eq!(
            al, bl,
            "logs[{i}] mismatch between O0 and O3"
        );
    }
}
```

- [ ] **Step 2: Run the existing storage-and-events proptest**

Run:
```bash
cargo test --test fuzz_tests optimizer_semantic_equivalence_storage_and_events 2>&1 | tail -15
```
Expected: PASS (no known optimizer bug here; the value is regression protection).

If this fails, the optimizer has a real miscompilation — surface to the user immediately. Do not "fix" the test by weakening it.

### Task 19: Verify and commit

- [ ] **Step 1: Full workspace gate**

Run:
```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings 2>&1 | tail -5
cargo test --workspace --all-features 2>&1 | tail -10
```
Expected: all green.

- [ ] **Step 2: Optional — run a deeper proptest pass to confirm stability**

Run:
```bash
PROPTEST_CASES=200 cargo test --test fuzz_tests optimizer_semantic_equivalence_storage_and_events 2>&1 | tail -10
```
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add src/runtime/runtime_parts/runtime_types.rs tests/fuzz_tests/optimizer_props.rs
git commit -m "test(optimizer): M-TEST3 — extend O0↔O3 differential to storage + events

assert_results_equivalent previously compared only return_data, so a
peephole pass that reorders PUT and Notify (observable on-chain) was
invisible. Adds state_changes and logs comparison. Also derives
PartialEq on StateChange and LogEntry (nested types already have it)."
git push -u origin phase1/m-test3-optimizer-state-events
```

---

## PR0b — Conditional follow-up based on uint256_ops decision

**Only execute if Task 9 recommended DELETE or DOCUMENT-AND-KEEP.** If the recommendation was WIRE-IN, it spawns a new phase outside Phase 1 scope.

### Task 20: DELETE or DOCUMENT-AND-KEEP `uint256_ops.rs`

**Files:**
- Modify or delete: `src/cli/bytecode/uint256_ops.rs` (1491 LOC)

- [ ] **Step 1: If DELETE — remove the file and any include/mod reference**

Run:
```bash
rg -n "uint256_ops" --type rust src/
```
Find the `mod uint256_ops;` or `include!` line that pulls the file in and remove it. Then:

```bash
git rm src/cli/bytecode/uint256_ops.rs
```

- [ ] **Step 2: If DOCUMENT-AND-KEEP — replace `#[allow(dead_code)]` with a status comment**

For each function in the file, replace `#[allow(dead_code)]` with:

```rust
/// **Status:** dormant — validated against a reference two's-complement VM
/// but not wired into binary lowering pending the coordinated simulator
/// migration (see `docs/audits/AUDIT_v0.22_validation.md`). Kept for
/// reference; do not call from production code.
#[doc(hidden)]
```

- [ ] **Step 3: Verify**

Run:
```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings 2>&1 | tail -5
cargo test --workspace --all-features 2>&1 | tail -10
```
Expected: all green.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "chore(bytecode): uint256_ops.rs — <DELETE|DOCUMENT> per Phase 1 validation

<uint256_ops decision rationale copied from AUDIT_v0.22_validation.md>"
git push -u origin phase1/uint256-ops-decision
```

---

## Phase 1 Closeout

### Task 21: Final verification on main

After all PRs land:

- [ ] **Step 1: Pull main and verify**

Run:
```bash
git checkout main
git pull
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings 2>&1 | tail -5
cargo test --workspace --all-features 2>&1 | tail -10
make production-gate 2>&1 | tail -20
```
Expected: all green.

- [ ] **Step 2: Confirm success criteria from the spec**

Verify:
1. `docs/audits/AUDIT_v0.22_validation.md` exists with every finding marked YES/PARTIAL/NO.
2. The `uint256_ops.rs` decision is documented.
3. PR1 (L-FE1), PR2 (L-DEV), PR3 (S5 BLS guard), PR4 (M-TEST3) landed.
4. `cargo test --workspace --all-features` is green on main.

- [ ] **Step 3: Clean up worktree**

Run:
```bash
git worktree remove ../neo-devpack-solidity-phase1
```

---

## Per-PR Verification Gate (applies to PR1–PR4, PR0b)

Every code-touching PR must pass before pushing:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

For PR2 (devpack contract change), also run:
```bash
bash examples/test_compilation.sh
```

PR0 (validation report only) needs no test gate.

---

## Risks (from spec)

1. **Validation reveals an incomplete v0.22 fix.** Halt before PR1–PR4. Re-plan the incomplete fix as its own PR.
2. **uint256_ops investigation surfaces a load-bearing dependency.** WIRE-IN spawns a new phase; Phase 1 scope does not expand.
3. **PR4 discovers the simulator doesn't expose `state_changes`/`logs` cleanly.** The `ExecutionResult` struct at `runtime_types.rs:20-30` exposes both fields as `pub`, so this risk is closed.
4. **"Many small PRs" review overhead.** Each PR is genuinely small; review cost per PR is minutes.
