# Devpack Final Audit Sweep Implementation Plan

**Goal:** Finish the remaining `devpack` review surface by characterizing the full audit output, removing any low-risk unused or misleading package metadata, and re-running the `devpack` verification commands until only justified legacy dev-toolchain risk remains.

**Architecture:** Treat `npm --prefix devpack audit --json` as the characterization gate. If it reports issues, trace them back to concrete `devpack` dependencies, remove only dependencies or metadata that are clearly unnecessary, and keep the local Hardhat 2 workflow intact. Re-run the targeted `devpack` tests, integration flow, packing check, and audit commands after each cleanup.

**Tech Stack:** npm, Hardhat 2, Ethers 5, local linked packages, Markdown docs

## Task 1: Capture The Remaining Devpack Audit Surface

**Files:**
- Modify: `none`
- Test: `npm --prefix devpack audit --json`
- Review: `devpack/package.json`
- Review: `devpack/package-lock.json`

**Step 1: Run the full devpack audit**

Run: `npm --prefix devpack audit --json`
Expected: Either PASS with `0` vulnerabilities or FAIL with legacy dev-only findings from the Hardhat 2 toolchain.

**Step 2: Map the findings to direct dependencies**

Run: `npm --prefix devpack ls --depth=2`
Expected: Enough dependency context to determine whether any finding comes from a removable direct dependency.

## Task 2: Remove Any Safe, Low-Risk Devpack Bloat

**Files:**
- Modify: `devpack/package.json`
- Modify: `devpack/package-lock.json`
- Modify: `devpack/README.md`
- Modify: `devpack/DEVPACK_GUIDE.md`

**Step 1: Remove one unnecessary direct dependency or misleading package field at a time**

Only change metadata or dependencies that are clearly unused or incorrect for the published source package.

**Step 2: Keep the local workflow intact**

Preserve the current `NODE_PRESERVE_SYMLINKS=1` Hardhat scripts, peer dependency expectations, and linked local plugin resolution.

## Task 3: Re-Verify The Devpack Surface

**Files:**
- Test: `npm --prefix devpack test`
- Test: `npm --prefix devpack run test:integration`
- Test: `npm --prefix devpack pack --dry-run`
- Test: `npm --prefix devpack audit --omit=dev --json`
- Test: `npm --prefix devpack audit --json`

**Step 1: Re-run the targeted devpack tests**

Run: `npm --prefix devpack test`
Expected: PASS.

Run: `npm --prefix devpack run test:integration`
Expected: PASS.

**Step 2: Re-check package and runtime surfaces**

Run: `npm --prefix devpack pack --dry-run`
Expected: PASS with the expected source-oriented tarball.

Run: `npm --prefix devpack audit --omit=dev --json`
Expected: PASS with `0` vulnerabilities.

**Step 3: Re-check the full audit**

Run: `npm --prefix devpack audit --json`
Expected: Either PASS with `0` vulnerabilities or only documented dev-only findings tied to the legacy Hardhat 2 stack.
