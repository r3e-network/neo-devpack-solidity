# Hardhat 3 Migration Spike Implementation Plan

> Status update: this spike was superseded by
> `docs/plans/2026-03-18-devpack-hardhat3-feasibility.md`. Current verified packages stay on
> Hardhat `>=2.28.6 <3`; do not widen peer ranges or move workspace dependencies to Hardhat 3
> without completing the dedicated API/config migration described in the feasibility notes.

**Goal:** Determine whether the Neo Hardhat tooling packages can be migrated from Hardhat 2.28.6 to Hardhat 3.x without breaking the verified production gate, and reduce the remaining audit surface rooted in the legacy Hardhat 2 dependency stack.

**Architecture:** Treat this as a migration spike with strict verification. First move the manifests and peer ranges to a Hardhat 3-compatible shape, then install and let TypeScript/tests reveal the concrete API breaks. Fix only the specific compatibility issues required to restore the tooling gates, and only then re-run the full repository gate.

**Tech Stack:** npm workspaces, TypeScript, Hardhat plugin APIs, Vitest, ESLint, Makefile production gate.

## Task 1: Update the manifest contract for Hardhat 3

**Files:**
- Modify: `tooling/package.json`
- Modify: `tooling/packages/hardhat-solc-neo/package.json`
- Modify: `tooling/packages/hardhat-neo-deployer/package.json`

**Step 1: Capture the baseline**

Run: `cd tooling && npm audit --json`
Expected: remaining advisories are concentrated in `hardhat`, `mocha`, `serialize-javascript`, `undici`, and the Neon/ethers low-severity trees.

**Step 2: Probe direct and peer Hardhat ranges in isolation**

In a scratch copy only, move the root and workspace Hardhat dependencies to a Hardhat 3 release and
widen peer dependencies only if dual support still appears realistic after TypeScript, plugin-load,
network-config, devpack, and Neo-Express checks pass.

**Step 3: Refresh the workspace lockfile**

Run: `cd tooling && npm install`
Expected: install succeeds and the dependency graph drops the legacy Hardhat 2 transitive stack.

## Task 2: Let the compiler surface actual Hardhat 3 compatibility breaks

**Files:**
- Verify: `tooling/packages/hardhat-solc-neo/src/**/*.ts`
- Verify: `tooling/packages/hardhat-neo-deployer/src/**/*.ts`

**Step 1: Run the tooling gates unchanged**

Run:
- `make tooling-test`
- `make tooling-lint`

Expected: FAIL or PASS with concrete evidence about Hardhat 3 compatibility.

**Step 2: Fix only real compatibility breaks**

If failures appear, patch only the specific Hardhat API changes revealed by TypeScript/tests. Do not mix unrelated refactors into the migration.

## Task 3: Re-verify the security posture and full repo gate

**Files:**
- Verify: `tooling/package-lock.json`
- Verify: touched Hardhat package sources if any

**Step 1: Re-run audit**

Run:
- `cd tooling && npm audit --json`
- `cd tooling && npm audit --omit=dev --json`

Expected: the Hardhat-rooted high/moderate advisories are materially reduced or eliminated.

**Step 2: Re-run the full repository gate**

Run: `make production-gate`
Expected: PASS on the migrated tree.

**Step 3: Stop if the migration surface expands**

If the migration requires broad architectural changes or breaks the published plugin API substantially, stop and document the blockers instead of continuing speculative edits.
