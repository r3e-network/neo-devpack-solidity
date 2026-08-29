// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract DecimalUnderflow {
    // Huge negative exponents — should be handled cleanly, not panic.
    uint256 constant A = 1e-2000000000;
    uint256 constant B = 1e-1024;
    uint256 constant C = 1e-1025;
    uint256 constant D = 5e-999;
    uint256 constant E = 0e-0;
    uint256 constant F = 1e-4294967296;
    uint256 constant G = 1.5e-500;

    // Fixed point literals at extremes
    function val() external pure returns (uint256) {
        return 1e-1000;
    }
}
