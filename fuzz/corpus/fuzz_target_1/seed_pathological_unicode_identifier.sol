// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract UnicodeIdent {
    // Non-ASCII identifiers should be rejected gracefully.
    uint256 public 名前;
    uint256 public переменная;
    uint256 public متغير;
    uint256 public 变量;
    uint256 public αβγ;

    function μέθοδος() external pure returns (uint256) {
        return 1;
    }

    function emoji() external pure returns (string memory) {
        return unicode"こんにちは 🌍";
    }

    function mixed_α_ident() external pure returns (uint256) { return 2; }
}
