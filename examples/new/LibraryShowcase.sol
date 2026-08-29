// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

library MathLib {
    function add(uint256 a, uint256 b) internal pure returns (uint256) {
        return a + b;
    }

    function mul(uint256 a, uint256 b) internal pure returns (uint256) {
        return a * b;
    }
}

contract Calculator {
    function compute(uint256 x, uint256 y) public pure returns (uint256) {
        return MathLib.add(x, MathLib.mul(x, y));
    }
}
