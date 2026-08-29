# Neon Deployer Replacement Implementation Plan

**Goal:** Remove the final `@cityofzion/neon-js` dependency by replacing the remaining Hardhat deployer usage with repo-owned account, transaction, and contract-parameter primitives.

**Architecture:** The remaining Neon surface is now isolated to `tooling/packages/hardhat-neo-deployer`. Split the work into three layers: address/account primitives, transaction/signing primitives, and contract-parameter encoding. Replace one layer at a time with tests and keep the RPC/client/build behavior unchanged.

**Tech Stack:** npm workspaces, TypeScript, Node `crypto`, Vitest, Neo N3 serialization/signing rules, Makefile production gate.

## Task 1: Lock down the remaining Neon surface with tests

**Files:**
- Modify: `tooling/packages/hardhat-neo-deployer/test/neo-primitives.test.ts`
- Create: `tooling/packages/hardhat-neo-deployer/test/account-primitives.test.ts`
- Create: `tooling/packages/hardhat-neo-deployer/test/transaction-primitives.test.ts`

**Step 1: Add account/address tests**

Cover:
- private key / WIF -> address derivation
- address version handling
- signer contract script derivation

**Step 2: Add transaction serialization tests**

Cover:
- `validUntilBlock`, fee, nonce, signers, and witness serialization
- deterministic tx hex against one current Neon-backed golden case

**Step 3: Add contract param encoding tests**

Cover:
- string, bytes, Hash160, arrays, and constructor args currently encoded through `u.HexString` / `sc.ContractParam`

## Task 2: Replace account primitives

**Files:**
- Create: `tooling/packages/hardhat-neo-deployer/src/account-primitives.ts`
- Modify: `tooling/packages/hardhat-neo-deployer/src/account-manager.ts`
- Modify: `tooling/packages/hardhat-neo-deployer/src/tasks/account.ts`
- Modify: `tooling/packages/hardhat-neo-deployer/src/deployer.ts`

**Step 1: Implement repo-owned account helpers**

Implement:
- WIF decode / encode,
- secp256r1 private key -> public key derivation,
- Neo signature contract script,
- address derivation using the shared Neo address helpers.

**Step 2: Remove `wallet.Account` / `wallet.generatePrivateKey` usage**

Keep the externally visible account config and CLI behavior unchanged.

## Task 3: Replace transaction/signing primitives

**Files:**
- Create: `tooling/packages/hardhat-neo-deployer/src/transaction-primitives.ts`
- Modify: `tooling/packages/hardhat-neo-deployer/src/deployer.ts`

**Step 1: Implement minimal transaction model**

Support only what the deployer currently needs:
- invocation transactions,
- signer scopes currently used,
- witness attachment,
- transaction hashing/signing,
- serialization to raw tx hex.

**Step 2: Match current deploy behavior exactly**

No feature expansion. Preserve the same RPC calls, receipt polling, and artifact output.

## Task 4: Replace remaining contract-parameter encoding helpers

**Files:**
- Create: `tooling/packages/hardhat-neo-deployer/src/contract-params.ts`
- Modify: `tooling/packages/hardhat-neo-deployer/src/deployer.ts`

**Step 1: Recreate the narrow `sc.ContractParam` subset in use**

Only implement the parameter kinds already used by deployment and invocation paths.

**Step 2: Remove final `u.HexString` usage**

Use repo-owned hex/base64/endian helpers instead.

## Task 5: Verify and delete the dependency

**Files:**
- Modify: `tooling/packages/hardhat-neo-deployer/package.json`
- Modify: `tooling/package-lock.json`

**Step 1: Remove `@cityofzion/neon-js`**

Run: `cd tooling && npm install`
Expected: install succeeds without Neon packages in the deployer tree.

**Step 2: Re-run verification**

Run:
- `make tooling-test`
- `make tooling-lint`
- `make production-gate`
- `cd tooling && npm audit --json`

Expected: the final four low-severity Neon/elliptic advisories disappear.
