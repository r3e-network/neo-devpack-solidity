// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title ClockGov — ERC-6372 contract clock reference, compiled to Neo N3.
contract ClockGov {
    string public buildTag = "clock-v1";

    function clock() external view returns (uint48) {
        return uint48(block.number);
    }

    function CLOCK_MODE() external pure returns (string memory) {
        return "mode=blocknumber&from=default";
    }
}
