// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title BitwiseShowcase
 * @notice Demonstrates bitwise AND/OR/XOR/NOT/shifts,
 *         bit packing, and flags pattern.
 */
contract BitwiseShowcase {
    uint8 public constant FLAG_READ    = 1;  // 0b0001
    uint8 public constant FLAG_WRITE   = 2;  // 0b0010
    uint8 public constant FLAG_EXECUTE = 4;  // 0b0100
    uint8 public constant FLAG_ADMIN   = 8;  // 0b1000

    mapping(address => uint8) public permissions;

    function grantFlag(address user, uint8 flag) public {
        permissions[user] = permissions[user] | flag;
    }

    function revokeFlag(address user, uint8 flag) public {
        permissions[user] = permissions[user] & ~flag;
    }

    function hasFlag(address user, uint8 flag) public view returns (bool) {
        return (permissions[user] & flag) == flag;
    }

    function toggleFlag(address user, uint8 flag) public {
        permissions[user] = permissions[user] ^ flag;
    }

    function shiftLeft(uint256 val, uint8 bits) public pure returns (uint256) {
        return val << bits;
    }

    function shiftRight(uint256 val, uint8 bits) public pure returns (uint256) {
        return val >> bits;
    }

    /// @notice Pack two uint128 values into one uint256
    function pack(uint128 high, uint128 low) public pure returns (uint256) {
        return (uint256(high) << 128) | uint256(low);
    }

    /// @notice Unpack a uint256 into two uint128 values
    function unpack(uint256 packed) public pure returns (uint128 high, uint128 low) {
        high = uint128(packed >> 128);
        low = uint128(packed);
    }
}
