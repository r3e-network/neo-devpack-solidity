---
title: "Testing Contracts: Writing Your Own Tests"
description: "Writing Your Own Tests from Testing Contracts."
---

# Writing Your Own Tests

[Back to Testing Contracts](/basics/testing-contracts)

## Testing your contract with `neo-test` (Foundry-style)

To test a contract *you* are writing (as opposed to adding a test to the compiler's own suite), reach for **`neo-test`** — the native Foundry-style runner. Write a Solidity test file and run it directly on the in-tree NeoVM:

```solidity
// test/Counter.t.sol
import "neo-std/Test.sol";       // assertions (bundled, no setup)
import "neo-std/console.sol";    // console.log (bundled)

contract CounterTest is Test {
    Counter c;

    function setUp() public { c = new Counter(); }   // fresh state before EACH test

    function testIncrements() public {
        c.inc();
        console.log("count", c.count());
        assertEq(c.count(), 1);
    }

    function testFailUnderflow() public { c.dec(); } // passes iff the call reverts
}
```

```bash
neo-test test/Counter.t.sol --gas    # or `neo-test` to scan ./test for *.t.sol
neo-forge test                       # Neo Foundry delegates to the same runner
```

`test*` passes when it does not revert; `testFail*` passes when it does; `setUp()` runs before each test against fresh state. You also get decoded revert/`Panic` reasons, `console.log`, gas reporting, and Foundry cheatcodes (`vm.prank` / `startPrank` / `stopPrank` / `warp` / `roll` / `deal` / `label` / `assume` / `expectRevert`). See [**Testing on Neo with `neo-test`**](/TESTING) for the full reference.

## Adding a Compilation Test

1. Create a new Solidity file in `examples/` or `examples/new/`.
2. The E2E test suite automatically picks up files in these directories.
3. Run `cargo test --test e2e_compilation_tests` to verify.

## Adding a Smoke Test

1. Create a new script in `examples/` following the pattern of existing scripts.
2. The script should:
    - Create a temporary directory and clean up on exit
    - Resolve `neo-solc` and `neoxp` binaries
    - Write a test contract inline
    - Compile, deploy, invoke, and validate
3. Add a Make target in the `Makefile`.
4. Add the target to `test-deploy-smoke-full` dependencies.

## Adding a Unit Test

Add `#[test]` functions in the relevant module or create a new test file in `tests/`:

```rust
#[test]
fn test_my_feature() {
    let source = r#"
        // SPDX-License-Identifier: MIT
        pragma solidity ^0.8.20;
        contract Test {
            function foo() public pure returns (uint256) {
                return 42;
            }
        }
    "#;
    // Use the compiler API to compile and validate
    // ...
}
```
