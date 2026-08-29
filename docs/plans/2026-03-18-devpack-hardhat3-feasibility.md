# Devpack Hardhat 3 Feasibility Notes

**Goal:** Record the current, evidence-backed blockers to migrating `devpack` from Hardhat `2.28.6` to Hardhat `3.x`, which is the only remaining path to remove the final `devpack` dev-only audit residue.

**Architecture:** Treat this as a migration feasibility snapshot, not a completed implementation. The active repository remains on the verified Hardhat 2-based `devpack` flow. These notes capture what happened in an isolated probe after upgrading only `devpack` to `hardhat@3.1.12` and applying the smallest possible ESM packaging changes.

**Tech Stack:** npm, Hardhat 3, TypeScript plugin APIs, Neo-specific Hardhat plugins, ESM config/runtime loading

## Probe Setup

**Files:**
- Review: `devpack/package.json`
- Review: `devpack/hardhat.config.js`
- Review: `tooling/packages/hardhat-solc-neo/src/**/*.ts`
- Review: `tooling/packages/hardhat-neo-deployer/src/**/*.ts`

**Step 1: Isolated copy**

Create a scratch copy of `devpack/` and `tooling/` outside the repo worktree.

**Step 2: Upgrade only the probe**

Run: `npm --prefix <probe>/devpack install hardhat@3.1.12 --save-dev`

Observed result:
- `npm audit --json` in the probe dropped to `0` vulnerabilities.
- This confirms the final `devpack` audit residue is indeed rooted in Hardhat 2.

## Confirmed Hardhat 3 Blockers

**Files:**
- Review: `devpack/package.json`
- Review: `devpack/hardhat.config.js`
- Review: `tooling/packages/hardhat-solc-neo/src/tasks/*.ts`
- Review: `tooling/packages/hardhat-neo-deployer/src/tasks/*.ts`
- Review: `tooling/packages/hardhat-neo-deployer/src/index.ts`

**Step 1: Hardhat 3 requires ESM package mode**

Running `npm --prefix <probe>/devpack test` immediately failed with:

- `Hardhat only supports ESM projects.`

Minimal probe fix:
- add `"type": "module"` to `devpack/package.json`
- convert `devpack/hardhat.config.js` to ESM syntax (`createRequire`, `export default`)

**Step 2: Plugin runtime API incompatibility**

After the minimal ESM conversion, `npm --prefix <probe>/devpack test` failed again with:

- `optional plugin '@neo-devpack-solidity/hardhat-solc-neo' not loaded: Cannot read properties of undefined (reading 'boolean')`
- `optional plugin '@neo-devpack-solidity/hardhat-neo-deployer' not loaded: (0 , config_1.task)(...).addParam is not a function`

This shows the current plugin task code is still written against the Hardhat 2 task builder surface (`addParam`, `addOptionalParam`, `types.boolean`, etc.).

**Step 3: Hardhat 3 network config schema mismatch**

The same probe run also failed with:

- `Invalid config ... networks.neo_*.type: Invalid discriminator value. Expected 'http' | 'edr-simulated'`

This means `devpack`’s current network config shape is not accepted by Hardhat 3 without a schema-aware rewrite.

### Scope Assessment

**Files:**
- Review: `tooling/packages/hardhat-solc-neo/src/tasks/*.ts`
- Review: `tooling/packages/hardhat-neo-deployer/src/tasks/*.ts`
- Review: `tooling/packages/hardhat-neo-deployer/src/index.ts`

**Step 1: Migration is no longer a package.json-only cleanup**

The probe showed three distinct migration fronts:

- ESM packaging changes in `devpack`
- Hardhat 3 task API rewrites in both Neo Hardhat plugins
- Hardhat 3 network config/schema updates in `devpack` and probably plugin config extensions

**Step 2: Safe stopping point**

Do not merge speculative Hardhat 3 edits into the verified repository until the migration is treated as its own project with:

- task API adaptation
- config schema adaptation
- end-to-end re-verification of `devpack` compile/deploy/verify flows

## Recommended Next Steps

**Files:**
- Modify: `devpack/package.json`
- Modify: `devpack/hardhat.config.js`
- Modify: `tooling/packages/hardhat-solc-neo/src/tasks/*.ts`
- Modify: `tooling/packages/hardhat-neo-deployer/src/tasks/*.ts`
- Modify: `tooling/packages/hardhat-neo-deployer/src/index.ts`

**Step 1: Start with task API migration**

Map Hardhat 2 task-builder calls (`addParam`, `addOptionalParam`, `types.*`) to Hardhat 3 task definitions using the `addOption`/`addPositionalArgument`/`addFlag` surface.

**Step 2: Update network config shape**

Adapt `devpack/hardhat.config.js` and any plugin config extensions to the Hardhat 3 network discriminator model (`type: "http"` for real RPC networks).

**Step 3: Re-run the real gates**

Required commands after any real migration attempt:

- `npm --prefix devpack test`
- `npm --prefix devpack run test:integration`
- `cd devpack && npm pack --dry-run`
- `npm --prefix devpack audit --json`
- `make production-gate`
