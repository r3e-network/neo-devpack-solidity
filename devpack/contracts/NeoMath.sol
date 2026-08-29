// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title NeoMath
 * @dev Math helpers for NeoVM-compatible contracts.
 */
abstract contract NeoMath {
    /**
     * @dev Ceiling division with denominator validation.
     */
    function nmCeilingDiv(uint256 numerator, uint256 denominator) internal pure returns (uint256) {
        require(denominator > 0, "invalid denominator");
        if (numerator == 0) {
            return 0;
        }
        return (numerator + denominator - 1) / denominator;
    }

    /**
     * @dev Multiply and clamp to NeoVM Integer max (2^255-1).
     */
    function nmMulClampMax(uint256 a, uint256 b) internal pure returns (uint256) {
        uint256 nmNeoIntMax = 0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff;
        if (a == 0 || b == 0) {
            return 0;
        }
        if (a > nmNeoIntMax / b) {
            return nmNeoIntMax;
        }
        return a * b;
    }
}
