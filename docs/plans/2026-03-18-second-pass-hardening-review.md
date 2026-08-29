# Second Pass Hardening Review Implementation Plan

**Goal:** Expand the review beyond the Rust default workspace checks by exercising the remaining repo verification surfaces and inspecting high-risk production code paths for additional defects.

**Architecture:** Treat this as repeated review cycles. First run the broader automated checks that were not part of the previous pass, then inspect non-test panic paths and large compiler/runtime hotspots for manual findings. If a concrete defect appears, add a failing test first, implement the minimal fix, and re-run the affected verification commands plus the broader suite.

**Tech Stack:** Rust, Cargo, npm workspaces, TypeScript, .NET 8, existing test suites

## Task 1: Run The Broader Verification Surfaces

**Files:**
- Modify: `none`
- Test: `cargo test --release`
- Test: `npm --prefix tooling test`
- Test: `npm --prefix tooling run lint`
- Test: `npm --prefix tooling run typecheck`
- Test: `dotnet test tests/Neo.Sol.Runtime.Tests/Neo.Sol.Runtime.Tests.csproj --configuration Release`

**Step 1: Run the release Rust suite**

Run: `cargo test --release`
Expected: PASS.

**Step 2: Run tooling validation**

Run: `npm --prefix tooling test`
Expected: PASS.

Run: `npm --prefix tooling run lint`
Expected: PASS.

Run: `npm --prefix tooling run typecheck`
Expected: PASS.

**Step 3: Run the runtime .NET suite**

Run: `dotnet test tests/Neo.Sol.Runtime.Tests/Neo.Sol.Runtime.Tests.csproj --configuration Release`
Expected: PASS.

## Task 2: Review High-Risk Production Paths

**Files:**
- Modify: `none`
- Review: `src/**/*.rs`
- Review: `tooling/packages/**/*.{ts,tsx,js}`
- Review: `src/Neo.Sol.Runtime/**/*.cs`

**Step 1: Search production panic points**

Run: `rg -n "unwrap\\(|expect\\(|panic!\\(|todo!|unimplemented!" src tooling/packages --glob '!**/tests/**'`
Expected: manageable list of candidates for manual review.

**Step 2: Inspect the largest or riskiest files**

Focus on compiler lowering, runtime interop, and tooling entrypoints where a panic or unchecked assumption can escape to users.

## Task 3: TDD Any New Defect

**Files:**
- Modify: exact production file discovered in Task 2
- Test: exact test file covering the defect

**Step 1: Write the failing test**

Add the smallest test that demonstrates the defect.

**Step 2: Run the targeted test to verify RED**

Run the smallest command that exercises the new test.
Expected: FAIL for the intended reason.

**Step 3: Implement the minimal fix**

Change only the production path needed to satisfy the failing test.

**Step 4: Re-run targeted and broad verification**

Run the targeted test, then the impacted broader verification commands from Task 1.
Expected: PASS.
