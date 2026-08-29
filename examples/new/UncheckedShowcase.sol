// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/// @title UncheckedShowcase
/// @notice Demonstrates that `unchecked { }` blocks compile correctly.
/// On NeoVM (BigInteger arithmetic, no overflow), unchecked blocks are
/// semantically identical to normal blocks.
contract UncheckedShowcase {
    function uncheckedAdd(uint256 a, uint256 b) public pure returns (uint256) {
        unchecked {
            return a + b;
        }
    }

    function uncheckedLoop() public pure returns (uint256) {
        uint256 sum = 0;
        unchecked {
            for (uint256 i = 0; i < 10; i++) {
                sum += i;
            }
        }
        return sum;
    }
}
