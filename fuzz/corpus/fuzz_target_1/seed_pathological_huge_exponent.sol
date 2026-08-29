// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract HugeExponent {
    uint256 constant A = 1e999;
    uint256 constant B = 1e1023;
    uint256 constant C = 1e1024;
    uint256 constant D = 1e1025;
    uint256 constant E = 1e4294967295;
    uint256 constant F = 2e500;
    uint256 constant G = 9e1023;
    int256 constant H = 1e308;

    function getLarge() external pure returns (uint256) {
        return 1e1000;
    }

    function borderline() external pure returns (uint256) {
        return 1e1024 / 1e1023;
    }
}
