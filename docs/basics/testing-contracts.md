---
title: "Testing Contracts"
description: "Testing Contracts section index."
---

# Testing Contracts

## Testing your own contracts with `neo-test`

For day-to-day contract development, use **`neo-test`** — a native, Foundry-style Solidity test runner that compiles your test contracts with `neo-solc` and executes `test*` / `testFail*` / `setUp()` functions directly on the in-tree NeoVM. You write tests in Solidity exactly as you would for Ethereum:

```solidity
import "neo-std/Test.sol";       // assertions (bundled)
import "neo-std/console.sol";    // console.log (bundled)

contract CounterTest is Test {
    Counter c;
    function setUp() public { c = new Counter(); }      // fresh state per test
    function testIncrements() public {
        c.inc();
        assertEq(c.count(), 1);
    }
    function testFailUnderflow() public { c.dec(); }    // passes iff it reverts
}
```

```bash
neo-test test/Counter.t.sol --gas    # or `neo-test` to scan ./test for *.t.sol
```

It gives you per-test state isolation, cross-contract `new`, decoded revert/`Panic` reasons, `console.log` / `console.logBytes`, gas reporting, value-rich assertions (`assertEq failed: 3 != 5`), and Foundry cheatcodes via the HEVM address (`vm.prank` / `startPrank` / `stopPrank` / `warp` / `roll` / `deal` / `label` / `assume` / `expectRevert`). `neo-forge test` delegates to it. See [**Testing on Neo with `neo-test`**](/TESTING) for the full guide.

## Testing the compiler itself

The Neo DevPack for Solidity project uses a layered testing strategy that validates the compiler at every level: unit tests for individual components, integration tests for the pipeline, E2E compilation tests for real contracts, conformance tests against reference implementations, and Neo-Express smoke tests for on-chain deployment behavior.

## Sections

| Section |
| --- |
| [Testing Philosophy](/basics/testing-contracts/testing-philosophy) |
| [Quick Validation](/basics/testing-contracts/quick-validation) |
| [Unit Tests](/basics/testing-contracts/unit-tests) |
| [E2E Compilation Tests](/basics/testing-contracts/e2e-compilation-tests) |
| [Conformance Tests](/basics/testing-contracts/conformance-tests) |
| [Strict Compatibility Compilation Sweep](/basics/testing-contracts/strict-compatibility-compilation-sweep) |
| [Neo-Express Smoke Tests](/basics/testing-contracts/neo-express-smoke-tests) |
| [Workspace-Wide Test Suites](/basics/testing-contracts/workspace-wide-test-suites) |
| [Code Coverage](/basics/testing-contracts/code-coverage) |
| [CI/CD Pipeline Setup](/basics/testing-contracts/ci-cd-pipeline-setup) |
| [Writing Your Own Tests](/basics/testing-contracts/writing-your-own-tests) |
| [Test Summary](/basics/testing-contracts/test-summary) |
| [Related Pages](/basics/testing-contracts/related-pages) |
