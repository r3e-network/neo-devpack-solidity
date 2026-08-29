// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "neo-std/Vm.sol";

/// @title neo-test standard test base (forge-std-style)
/// @notice Inherit `Test` in your `*.t.sol` contracts to get assertions and the
/// cheatcode handle `vm` (from `CheatVm`). A test PASSES when it does not
/// revert; an assertion that fails calls `revert(reason)`, which `neo-test`
/// reports as `[FAIL] name (revert: reason)`.
///
///   import "neo-std/Test.sol";
///   contract MyTest is Test {
///       function setUp() public { /* ... */ }
///       function testThing() public { vm.warp(1000); assertEq(1 + 1, 2); }
///   }
abstract contract Test is CheatVm {
    // ---- boolean ----
    function assertTrue(bool condition) internal pure {
        if (!condition) revert("assertTrue: expected true");
    }
    function assertTrue(bool condition, string memory err) internal pure {
        if (!condition) revert(err);
    }
    function assertFalse(bool condition) internal pure {
        if (condition) revert("assertFalse: expected false");
    }
    function assertFalse(bool condition, string memory err) internal pure {
        if (condition) revert(err);
    }

    // ---- equality ----
    // Failure messages embed the actual operands (Foundry-style) so a failing
    // assertion reads e.g. `assertEq failed: 3 != 5` instead of a bare label.
    // The message is only built on the failure path, so passing asserts stay
    // cheap and `pure`.
    function assertEq(uint256 a, uint256 b) internal pure {
        if (a != b) revert(_neq("assertEq", _u(a), _u(b)));
    }
    function assertEq(uint256 a, uint256 b, string memory err) internal pure {
        if (a != b) revert(_with(err, _u(a), _u(b), " != "));
    }
    function assertEq(int256 a, int256 b) internal pure {
        if (a != b) revert(_neq("assertEq", _i(a), _i(b)));
    }
    function assertEq(int256 a, int256 b, string memory err) internal pure {
        if (a != b) revert(_with(err, _i(a), _i(b), " != "));
    }
    function assertEq(bool a, bool b) internal pure {
        if (a != b) revert(_neq("assertEq", _b(a), _b(b)));
    }
    function assertEq(bool a, bool b, string memory err) internal pure {
        if (a != b) revert(_with(err, _b(a), _b(b), " != "));
    }
    function assertEq(address a, address b) internal pure {
        if (a != b) revert("assertEq(address): a != b");
    }
    function assertEq(address a, address b, string memory err) internal pure {
        if (a != b) revert(err);
    }
    function assertEq(bytes32 a, bytes32 b) internal pure {
        if (a != b) revert("assertEq(bytes32): a != b");
    }
    function assertEq(bytes32 a, bytes32 b, string memory err) internal pure {
        if (a != b) revert(err);
    }
    function assertEq(string memory a, string memory b) internal pure {
        if (keccak256(bytes(a)) != keccak256(bytes(b))) revert("assertEq(string): a != b");
    }
    function assertEq(string memory a, string memory b, string memory err) internal pure {
        if (keccak256(bytes(a)) != keccak256(bytes(b))) revert(err);
    }
    function assertEq(bytes memory a, bytes memory b) internal pure {
        if (keccak256(a) != keccak256(b)) revert("assertEq(bytes): a != b");
    }
    function assertEq(bytes memory a, bytes memory b, string memory err) internal pure {
        if (keccak256(a) != keccak256(b)) revert(err);
    }

    // ---- inequality ----
    function assertNotEq(uint256 a, uint256 b) internal pure {
        if (a == b) revert(_neq("assertNotEq", _u(a), _u(b)));
    }
    function assertNotEq(address a, address b) internal pure {
        if (a == b) revert("assertNotEq(address): a == b");
    }
    function assertNotEq(bytes32 a, bytes32 b) internal pure {
        if (a == b) revert("assertNotEq(bytes32): a == b");
    }

    // ---- ordering (unsigned) ----
    function assertGt(uint256 a, uint256 b) internal pure {
        if (!(a > b)) revert(_cmp("assertGt", _u(a), _u(b), " <= "));
    }
    function assertGt(uint256 a, uint256 b, string memory err) internal pure {
        if (!(a > b)) revert(_with(err, _u(a), _u(b), " <= "));
    }
    function assertGe(uint256 a, uint256 b) internal pure {
        if (!(a >= b)) revert(_cmp("assertGe", _u(a), _u(b), " < "));
    }
    function assertLt(uint256 a, uint256 b) internal pure {
        if (!(a < b)) revert(_cmp("assertLt", _u(a), _u(b), " >= "));
    }
    function assertLe(uint256 a, uint256 b) internal pure {
        if (!(a <= b)) revert(_cmp("assertLe", _u(a), _u(b), " > "));
    }

    // ---- explicit failure ----
    function fail() internal pure {
        revert("fail()");
    }
    function fail(string memory err) internal pure {
        revert(err);
    }

    // ---- message helpers (only invoked on the failure path) ----
    function _neq(string memory tag, string memory a, string memory b)
        private pure returns (string memory)
    {
        return string(abi.encodePacked(tag, " failed: ", a, " != ", b));
    }
    function _cmp(string memory tag, string memory a, string memory b, string memory op)
        private pure returns (string memory)
    {
        return string(abi.encodePacked(tag, " failed: ", a, op, b));
    }
    function _with(string memory err, string memory a, string memory b, string memory op)
        private pure returns (string memory)
    {
        return string(abi.encodePacked(err, ": ", a, op, b));
    }
    function _b(bool v) private pure returns (string memory) {
        return v ? "true" : "false";
    }
    /// Decimal rendering of an unsigned integer.
    function _u(uint256 v) private pure returns (string memory) {
        if (v == 0) return "0";
        uint256 n = v;
        uint256 digits;
        while (n != 0) { digits++; n /= 10; }
        bytes memory buf = new bytes(digits);
        n = v;
        while (n != 0) { digits -= 1; buf[digits] = bytes1(uint8(48 + (n % 10))); n /= 10; }
        return string(buf);
    }
    /// Decimal rendering of a signed integer.
    function _i(int256 v) private pure returns (string memory) {
        if (v < 0) return string(abi.encodePacked("-", _u(uint256(-v))));
        return _u(uint256(v));
    }
}
