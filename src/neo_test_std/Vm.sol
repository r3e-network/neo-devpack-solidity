// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/// @title neo-test cheatcodes (`vm.*`)
/// @notice Foundry-compatible cheatcodes for manipulating test state. Calls to
/// the well-known HEVM cheatcode address are intercepted and serviced by the
/// neo-test runtime — no contract is deployed at that address. Inherit `Test`
/// (which extends `CheatVm`) and call e.g. `vm.prank(alice)` / `vm.warp(1000)`.
///
/// Supported today: warp, roll, prank, startPrank, stopPrank, deal, label,
/// assume. (Unknown cheats are accepted as no-ops for forward compatibility.)
interface Vm {
    /// Sets `block.timestamp`.
    function warp(uint256 newTimestamp) external;

    /// Sets `block.number` (Ledger.currentIndex on Neo).
    function roll(uint256 newHeight) external;

    /// Sets `msg.sender` for the NEXT call only. `tx.origin` is unaffected.
    function prank(address sender) external;

    /// Sets `msg.sender` for ALL subsequent calls until `stopPrank()`.
    function startPrank(address sender) external;

    /// Stops an active `prank` / `startPrank`.
    function stopPrank() external;

    /// Sets the GAS balance of `who` to `newBalance`.
    function deal(address who, uint256 newBalance) external;

    /// Labels an address for trace output (cosmetic; accepted and ignored).
    function label(address account, string calldata newLabel) external;

    /// Rejects the current input when `condition` is false (fuzzing helper).
    function assume(bool condition) external;

    /// Expects the NEXT call to revert (for any reason). The test passes if
    /// that call reverts and continues; if it does NOT revert, the test fails
    /// with "expectRevert: call did not revert".
    function expectRevert() external;
}

/// Base that exposes the cheatcode handle `vm`. `Test` inherits this, so any
/// test contract that `is Test` can call `vm.*` directly.
abstract contract CheatVm {
    Vm internal constant vm = Vm(0x7109709ECfa91a80626fF3989D68f67F5b1DD12D);
}
