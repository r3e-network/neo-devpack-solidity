// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/// @title Using-For Dispatch Showcase
/// @notice Demonstrates `using X for Y` library member-call syntax on Neo N3.

library SafeMath {
    function add(uint256 a, uint256 b) internal pure returns (uint256) {
        return a + b;
    }

    function sub(uint256 a, uint256 b) internal pure returns (uint256) {
        require(a >= b, "SafeMath: underflow");
        return a - b;
    }

    function mul(uint256 a, uint256 b) internal pure returns (uint256) {
        return a * b;
    }
}

contract UsingForShowcase {
    using SafeMath for uint256;

    uint256 private _total;

    /// @notice Add a value using library member-call syntax (x.add(y))
    function addToTotal(uint256 value) public {
        _total = _total.add(value);
    }

    /// @notice Subtract using library member-call syntax
    function subFromTotal(uint256 value) public {
        _total = _total.sub(value);
    }

    /// @notice Multiply using library member-call syntax
    function mulTotal(uint256 value) public {
        _total = _total.mul(value);
    }

    /// @notice Direct library call syntax (always works)
    function addDirect(uint256 a, uint256 b) public pure returns (uint256) {
        return SafeMath.add(a, b);
    }

    function getTotal() public view returns (uint256) {
        return _total;
    }
}
