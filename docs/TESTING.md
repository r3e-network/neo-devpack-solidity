# Testing Solidity on Neo with `neo-test`

`neo-test` is a Foundry-style test runner for Solidity contracts compiled to
NeoVM. You write test contracts in Solidity exactly like you would for Ethereum
(`function testFoo()`, `setUp()`, `assertEq`, `console.log`); `neo-test`
compiles each one with `neo-solc` and executes every test function on the
in-tree NeoVM — the same VM the compiler's own test suite trusts — reporting
PASS/FAIL, gas, decoded revert reasons, and log output.

## Quick start

```bash
cargo build --release --bin neo-test     # or: make build
```

Write a test file `test/Counter.t.sol`:

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "neo-std/Test.sol";       // assertions (bundled, no setup)
import "neo-std/console.sol";    // console.log debug output (bundled)

contract Counter {
    uint256 public count;
    function inc() public { count += 1; }
    function dec() public { require(count > 0, "underflow"); count -= 1; }
}

contract CounterTest is Test {
    Counter c;

    function setUp() public {
        c = new Counter();          // runs fresh before EACH test
    }

    function testIncrements() public {
        c.inc();
        console.log("count", c.count());
        assertEq(c.count(), 1);
    }

    function testFailUnderflow() public {
        c.dec();                    // count is 0 -> reverts -> this test PASSES
    }
}
```

Run it:

```bash
neo-test test/Counter.t.sol --gas
# or just `neo-test` to scan ./test (then .) for *.t.sol
```

```
Running 2 test(s) for test/Counter.t.sol:CounterTest
  [PASS] testFailUnderflow (gas: 160)
  [PASS] testIncrements (gas: 4001431)
    log: count = 1

Test result: ok. 2 passed; 0 failed; finished in 2.84ms
```

## Conventions (Foundry-compatible)

| Convention      | Meaning |
|-----------------|---------|
| `function test*()`     | A test. **Passes when it does not revert.** |
| `function testFail*()` | **Passes when it reverts** (negative test). |
| `function setUp()`     | Runs before **each** test, against fresh state (per-test isolation). |
| `require` / `assert` / `revert` | The assertion mechanism — a revert fails the test and its reason is shown. |

Each test runs in a fresh VM instance, so storage written by one test never
leaks into another (exactly like Foundry). Constructor + state-variable
initializers + `setUp()` run before every test.

## The bundled `neo-std` library

`import "neo-std/Test.sol"` and `import "neo-std/console.sol"` resolve
automatically (no remapping or install needed):

- **`Test`** — inherit it for `assertTrue` / `assertFalse` / `assertEq` (uint,
  int, bool, address, bytes32, string, bytes) / `assertNotEq` / `assertGt` /
  `assertGe` / `assertLt` / `assertLe` / `fail`. Each takes an optional message:
  `assertEq(a, b, "balances diverged")`.
- **`console`** — `console.log("here")`, `console.log("balance", amount)`,
  and typed overloads for `uint256`/`int256`/`bool`/`address`/`bytes`. Output is
  printed under each test (always for a failing test, and with `-v` for all).
  Under the hood it emits `System.Runtime.Log`, so the same calls surface as
  application logs on a real node.

## Cheatcodes (`vm.*`)

Inherit `Test` and you get the Foundry-style cheatcode handle `vm` (no import
beyond `neo-std/Test.sol`). Calls to `vm.*` are serviced by the runtime — the
same HEVM cheatcode-address mechanism Foundry uses — so no contract is deployed
and no manifest permission is needed:

```solidity
contract VaultTest is Test {
    Vault v;
    address alice = address(0x1111111111111111111111111111111111111111);

    function setUp() public { v = new Vault(); }

    function testDepositCreditsCaller() public {
        vm.prank(alice);          // the NEXT call's msg.sender is alice
        v.deposit();              // bal[alice] += 1
        assertEq(v.balanceOf(alice), 1);
    }
}
```

| Cheatcode | Effect |
|-----------|--------|
| `vm.prank(addr)` | Sets `msg.sender` for the **next** call only. `tx.origin` unaffected. |
| `vm.startPrank(addr)` / `vm.stopPrank()` | Sets `msg.sender` for **all** calls until stopped. |
| `vm.warp(secs)` | Sets `block.timestamp` (seconds). |
| `vm.roll(n)` | Sets `block.number`. |
| `vm.deal(addr, amount)` | Sets the account's GAS balance (read via `addr.balance`). |
| `vm.expectRevert()` | Expects the **next** call to revert; the revert is swallowed and execution continues. Fails the test if the call does not revert. |
| `vm.label(addr, name)` | Cosmetic; accepted and ignored. |
| `vm.assume(cond)` | Rejects the current input when `cond` is false. |

```solidity
function testWithdrawTooMuchReverts() public {
    vm.expectRevert();
    vault.withdraw(1 ether);   // reverts (empty vault) -> caught, test passes
    assertEq(vault.balanceOf(address(this)), 0);  // execution continues
}
```

## Decoded failure reasons

`neo-test` decodes the on-chain revert payload, so failures read clearly:

- `revert("msg")` / `require(cond, "msg")` → `revert: msg`
- a Solidity `Panic` → `panic: 0x12 (division or modulo by zero)`, `0x11`
  (overflow/underflow), `0x32` (array out of bounds), etc.
- custom errors → the error name.

## Options

```
neo-test [PATH ...] [OPTIONS]

PATH                       A .sol file or a directory (scanned for *.t.sol, else *.sol).
-m, --match-test <S>       Only run test fns whose name contains <S>
-c, --match-contract <S>   Only run contracts whose name contains <S>
-I, --include <DIR>        Extra import root (repeatable; e.g. node_modules)
-v, --verbose              Show logs and revert reasons for every test
--gas                      Show per-test gas usage
--no-color                 Disable ANSI color
```

Imports are resolved from disk, so multi-file projects, relative imports, a
`remappings.txt`, and a co-located `node_modules` (`@openzeppelin/...`) all work
just as with `neo-solc`.

## Exit code

`neo-test` exits `0` when every test passes and `1` on any failure or
compile/import error — drop it straight into CI.

## `msg.sender`, `tx.origin`, and `address(this)`

These resolve the way you expect from Ethereum:

- Inside a test method (the entry frame), `msg.sender == tx.origin` — both are
  the transaction signer.
- A contract you `new` and then call **does** observe the calling contract's
  `address(this)` as its `msg.sender`. So the canonical pattern works:

  ```solidity
  Vault v = new Vault();
  v.deposit();                          // deposit(): bal[msg.sender] += 1
  assertEq(v.balanceOf(address(this)), 1);   // ✓ credited to the test contract
  ```

- In any nested call, `msg.sender` (the direct caller) differs from `tx.origin`
  (the entry signer), so "only the direct caller can authorize X" guards behave
  correctly.

One simulator-model caveat: all contracts compiled into a single test bundle
execute under the *same* executing script hash, so `address(this)` is identical
across the test contract and the contracts it `new`s. The load-bearing
invariants above (callee `msg.sender` == caller `address(this)`, and
`msg.sender != tx.origin` when nested) still hold; what is *not* modeled is a
distinct per-instance address for each `new`-deployed contract.

## `neo-forge test`

If you drive your project through Neo Foundry, `neo-forge test` delegates to
`neo-test` — the same runner, conventions, cheatcodes, and output described
above — so `test/*.t.sol` behaves identically whether you invoke `neo-test`
directly or through `neo-forge`.

## Known limitations

- **No distinct per-instance address** for each `new`-deployed contract in a
  single test bundle (see the `address(this)` note above). The load-bearing
  `msg.sender` / `tx.origin` invariants still hold.
