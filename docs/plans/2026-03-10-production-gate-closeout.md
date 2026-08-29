# Production Gate Closeout Implementation Plan

**Goal:** Re-verify the full production gate on the current tree, reduce the remaining tooling advisory surface with the lowest-risk dependency changes available, and leave an explicit record of any residual risk that still blocks a fully clean dependency posture.

**Architecture:** Treat the remaining work as two mostly independent tracks. First, run the full repository quality gate exactly as documented so the current compiler/runtime/tooling tree has a fresh, captured pass/fail result. In parallel, refresh `npm audit` evidence for the tooling workspace, map the remaining advisories to direct or transitive sources, and only apply dependency changes that are demonstrably low-risk and can be verified with the existing tooling test/lint gates.

**Tech Stack:** Rust/cargo workspace, Makefile quality gates, Node/npm workspace under `tooling/`, TypeScript packages, npm audit, Markdown documentation.

## Task 1: Reproduce the current production gate result

**Files:**
- Verify: `Makefile`
- Verify: `docs/workflows/production.md`
- Verify: `docs/workflows/test.md`

**Step 1: Run the documented production gate**

Run: `make production-gate`
Expected: PASS with all Rust, runtime, tooling, and Neo Express smoke checks completing on the latest tree.

**Step 2: Capture the actual result**

If the gate fails, record the first failing command and stop broad changes until the root cause is understood.

**Step 3: Confirm gate/docs alignment**

Run: `rg -n "production-gate|tooling-test|tooling-lint|runtime-test" Makefile docs/workflows/production.md docs/workflows/test.md`
Expected: documentation matches the real gate steps.

## Task 2: Refresh dependency-risk evidence before changing manifests

**Files:**
- Verify: `tooling/package.json`
- Verify: `tooling/package-lock.json`
- Verify: `tooling/packages/*/package.json`

**Step 1: Capture a fresh audit snapshot**

Run: `cd tooling && npm audit --json`
Expected: a current advisory report that reflects the already-applied dependency upgrades.

**Step 2: Separate direct vs transitive risk**

Run: `cd tooling && npm ls hardhat @cityofzion/neon-js @cityofzion/neon-core elliptic mocha serialize-javascript tmp undici`
Expected: clear ownership for the remaining advisories, especially the `hardhat` 2.x and Neon dependency chains.

**Step 3: Identify the lowest-risk remediation candidates**

Prefer, in order:
- direct package bumps already compatible with existing semver ranges,
- `overrides` for clearly safe transitive fixes,
- documentation of residual advisories when the only path is a major-version migration with ecosystem risk.

## Task 3: Apply only targeted tooling dependency fixes

**Files:**
- Modify: `tooling/package.json`
- Modify: `tooling/package-lock.json`
- Optionally modify: `tooling/packages/*/package.json`

**Step 1: Add or update one remediation at a time**

Each dependency change must be isolated enough that the resulting audit delta can be attributed to that change.

**Step 2: Re-install lockfile state**

Run: `cd tooling && npm install`
Expected: lockfile updates cleanly with no broken workspace resolution.

**Step 3: Re-run focused tooling verification**

Run:
- `make tooling-test`
- `make tooling-lint`

Expected: PASS after each dependency remediation round.

## Task 4: Re-verify the end state and record residual risk honestly

**Files:**
- Verify: `tooling/package.json`
- Verify: `tooling/package-lock.json`
- Verify: touched docs if any were needed

**Step 1: Re-run the full production gate**

Run: `make production-gate`
Expected: PASS.

**Step 2: Re-run audit after the final dependency set**

Run: `cd tooling && npm audit --json`
Expected: lower advisory count than the current baseline, or an unchanged count with a clear explanation of why the remaining issues require high-risk ecosystem moves.

**Step 3: Summarize residual blockers**

Document:
- which advisories remain,
- which package trees own them,
- whether they affect dev-only or production paths,
- whether removing them requires a major migration such as `hardhat` 3 or a Neon stack replacement.
