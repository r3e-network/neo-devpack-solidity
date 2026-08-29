// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title TypeNameShowcase
 * @notice Demonstrates `type(T).name` expressions for contracts and
 *         primitive types. On NeoVM these resolve to compile-time string
 *         constants pushed onto the evaluation stack.
 */
contract TypeNameShowcase {
    function contractName() public pure returns (string memory) {
        return type(TypeNameShowcase).name;
    }

    function uintName() public pure returns (string memory) {
        return type(uint256).name;
    }

    function intName() public pure returns (string memory) {
        return type(int128).name;
    }
}
