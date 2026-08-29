# Neon Utility Extraction Implementation Plan

**Goal:** Reduce the remaining `@cityofzion/neon-js` coupling by replacing the simple address and encoding helper usage with repo-owned utilities in `@neo-devpack-solidity/types`, without touching the deployer's transaction/signing stack yet.

**Architecture:** Keep this slice intentionally narrow. Extract only the helper functions that are already used in read-only and formatting paths: Neo base58 address validation/conversion, Hash160 endianness conversion, and hex/base64 helpers. Move them into the shared types package with focused tests, then switch `abi-router`, `cli-tools`, and the Hardhat verify path to use the shared implementation.

**Tech Stack:** npm workspaces, TypeScript, Vitest, Node `crypto`, existing tooling packages.

## Task 1: Capture the current helper surface

**Files:**
- Review: `tooling/packages/abi-router/src/neo-utils.ts`
- Review: `tooling/packages/cli-tools/src/compiler-cli.ts`
- Review: `tooling/packages/hardhat-solc-neo/src/tasks/verify-contract.ts`
- Review: `tooling/packages/neo-foundry/src/cast.ts`

**Step 1: Identify the helper subset**

Confirm the first slice only needs:
- base58check decode / address validation,
- address <-> script hash conversion,
- Hash160 little-endian / big-endian conversion,
- hex <-> base64 conversion used by verification.

**Step 2: Exclude deployer-specific primitives**

Do not include transaction building, signing, `wallet.Account`, or `sc`/`tx` abstractions in this pass.

## Task 2: Write failing tests first in the shared package

**Files:**
- Create: `tooling/packages/types/test/neo.test.ts`
- Verify: `tooling/packages/abi-router/test/neo-utils.test.ts`

**Step 1: Add shared utility tests**

Cover:
- `NepwUjd9GhqgNkrfXaxj9mmsFhFzGoFuWM` <-> `0xd2a4cff31913016155e38e474a2c06d08be276cf`
- EVM address <-> Neo Hash160 conversion
- invalid base58 checksum rejection
- `hex -> base64 -> hex` roundtrip

**Step 2: Run the new tests to watch red**

Run: `cd tooling/packages/types && npm test -- --run test/neo.test.ts`
Historical expected result: FAIL at the start of this plan because the shared helper module did not exist yet.

## Task 3: Implement shared helpers in `@neo-devpack-solidity/types`

**Files:**
- Create: `tooling/packages/types/src/neo.ts`
- Modify: `tooling/packages/types/src/index.ts`

**Step 1: Implement minimal helpers**

Use Node primitives only:
- `crypto` for checksum hashing,
- base58 alphabet logic from the existing `neo-foundry` decoder,
- Buffer transforms for endian conversion and base64/hex conversion.

**Step 2: Export only the shared helper surface**

Keep the module focused on stateless helpers. Do not add account or RPC classes here.

**Step 3: Verify green in the types package**

Run: `cd tooling/packages/types && npm test -- --run test/neo.test.ts`
Expected: PASS.

## Task 4: Migrate non-deployer packages to the shared helpers

**Files:**
- Modify: `tooling/packages/abi-router/src/neo-utils.ts`
- Modify: `tooling/packages/abi-router/src/rpc-adapter.ts`
- Modify: `tooling/packages/cli-tools/src/compiler-cli.ts`
- Modify: `tooling/packages/hardhat-solc-neo/src/tasks/verify-contract.ts`
- Modify: `tooling/packages/abi-router/test/neo-utils.test.ts`
- Modify package manifests only if dependency cleanup is needed for these packages

**Step 1: Replace direct Neon helper usage**

Switch imports from `@cityofzion/neon-js` to `@neo-devpack-solidity/types` for the helper subset only.

**Step 2: Keep behavior unchanged**

Preserve:
- accepted address formats,
- error messages where they are user-facing,
- existing endian conventions.

**Step 3: Re-run the touched tests**

Run:
- `cd tooling/packages/abi-router && npm test -- --run test/neo-utils.test.ts`
- `make tooling-test`
- `make tooling-lint`

Expected: PASS.

## Task 5: Reassess the remaining Neon dependency surface

**Files:**
- Verify: `tooling/package.json`
- Verify: `tooling/package-lock.json`

**Step 1: Inspect remaining Neon consumers**

Run: `cd tooling && rg -n "@cityofzion/neon-js|neon-js" packages`
Expected: usage is reduced and now concentrated in deployer/account/signing paths.

**Step 2: Decide whether the next pass is still bounded**

If the remaining usage is mostly `wallet.Account`, `sc`, and `tx`, stop and document that the next step is a dedicated deployer/runtime replacement, not another small helper extraction.
