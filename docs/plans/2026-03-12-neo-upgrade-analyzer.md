# Neo Upgrade Analyzer Implementation Plan

**Goal:** Add a minimal Neo upgrade analyzer that reports EVM-to-Neo migration issues without changing the existing NEF/manifest compilation pipeline.

**Architecture:** Keep compilation centered on the current `compile_contracts` / `compile_metadata` flow. Add a small source-pattern analyzer for common EVM compatibility constructs, combine it with existing compiler diagnostics, and expose it through a CLI `--analyze` mode. Preserve artifact generation behavior when `--analyze` is not used.

**Tech Stack:** Rust, Clap, Serde JSON, existing neo-devpack-solidity CLI and Solidity diagnostic pipeline

## Task 1: Lock The Expected Behavior With Tests

**Files:**
- Create: `tests/analyzer_cli_tests.rs`

**Step 1: Write the failing analyzer CLI test**

Add a test that:
- writes a temporary Solidity file using `tx.origin`, `blockhash`, `selfdestruct`, and `msg.sig`
- runs `neo-solc --analyze <file>`
- expects JSON output with:
  - at least one `auto_compatible` finding
  - at least one `manual_migration` finding
  - a compile failure flag when `msg.sig` is present

**Step 2: Run test to verify it fails**

Run: `cargo test --test analyzer_cli_tests analyze_mode_reports_upgrade_findings -- --exact`
Historical expected result: FAIL at the start of this plan because `--analyze` did not exist yet. In the current compiler, `--analyze` is implemented and this targeted test should pass.

**Step 3: Write the failing diagnostic propagation test**

Add a test that:
- writes a temporary Solidity file with a library `external` function
- runs `neo-solc --json-errors <file>`
- expects stderr JSON to contain the original compiler suggestion for that validation error

**Step 4: Run test to verify it fails**

Run: `cargo test --test analyzer_cli_tests json_errors_preserve_validation_suggestions -- --exact`
Historical expected result: FAIL at the start of this plan because failing-path suggestions were not preserved yet.

## Task 2: Implement The Analyzer And Wire The CLI

**Files:**
- Modify: `src/cli/mod.rs`
- Modify: `src/cli/cli_parts/cli_run.rs`
- Modify: `src/cli/cli_parts/cli_run/args.rs`
- Modify: `src/cli/cli_parts/cli_run/single_file.rs`
- Modify: `src/cli/cli_parts/cli_run/compile.rs`
- Modify: `src/cli/cli_parts/cli_diagnostics.rs`
- Modify: `src/solidity.rs`
- Create: `src/solidity/upgrade.rs`
- Create: `src/cli/cli_parts/cli_analyze.rs`

**Step 1: Add the CLI flag**

Add `--analyze` to the single-file CLI surface with help text that makes it clear this emits a Neo upgrade report instead of writing `.nef` / `.manifest.json`.

**Step 2: Add a source-pattern analyzer**

Implement a small `src/solidity/upgrade.rs` module that scans Solidity source for common EVM compatibility patterns and returns structured findings with:
- severity
- category (`auto_compatible`, `manual_migration`, `manifest_review`)
- message
- suggestion

**Step 3: Add a CLI report builder**

Implement `src/cli/cli_parts/cli_analyze.rs` that:
- runs the source-pattern analyzer
- runs normal compilation in-memory
- converts success warnings or failure diagnostics into a single JSON report
- prints the report to stdout

**Step 4: Preserve original diagnostics on failure**

Update the CLI diagnostic helpers so validation / semantic / IR failures keep their original suggestion text, instead of discarding it when rendered to stderr.

**Step 5: Hook `--analyze` into single-file execution**

Branch in `run_single_file()` so analyze mode resolves imports, builds the report, prints JSON, and skips artifact emission.

## Task 3: Verify The Compiler Still Produces Correct Artifacts

**Files:**
- No new files unless a small assertion helper is needed

**Step 1: Run the new analyzer tests**

Run: `cargo test --test analyzer_cli_tests -- --test-threads=1`
Expected: PASS

**Step 2: Run focused CLI/unit regressions**

Run: `cargo test test_nef_file_structure --test e2e_compilation_tests -- --exact`
Expected: PASS

Run: `cargo test test_manifest_structure --test e2e_compilation_tests -- --exact`
Expected: PASS

Run: `cargo test test_evm_compat_blockhash_auto_mapped --test e2e_compilation_tests -- --exact`
Expected: PASS

Run: `cargo test test_evm_compat_msg_sig_error --test e2e_compilation_tests -- --exact`
Expected: PASS

**Step 3: Commit**

```bash
git add tests/analyzer_cli_tests.rs src/solidity/upgrade.rs src/cli/cli_parts/cli_analyze.rs src/cli/mod.rs src/cli/cli_parts/cli_run.rs src/cli/cli_parts/cli_run/args.rs src/cli/cli_parts/cli_run/single_file.rs src/cli/cli_parts/cli_run/compile.rs src/cli/cli_parts/cli_diagnostics.rs src/solidity.rs docs/plans/2026-03-12-neo-upgrade-analyzer.md
git commit -m "feat: add neo upgrade analyzer"
```
