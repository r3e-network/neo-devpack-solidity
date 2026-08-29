// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title TypeCastingShowcase
 * @notice Demonstrates explicit casts (uint8→uint256, address→uint160),
 *         bytes conversions, and abi.encode/decode patterns.
 */
contract TypeCastingShowcase {
    function widenUint(uint8 small) public pure returns (uint256) {
        return uint256(small);
    }

    function narrowUint(uint256 big) public pure returns (uint8) {
        require(big <= 255, "overflow");
        return uint8(big);
    }

    function addressToUint(address addr) public pure returns (uint160) {
        return uint160(addr);
    }

    function uintToAddress(uint160 val) public pure returns (address) {
        return address(val);
    }

    function bytesToUint(bytes32 data) public pure returns (uint256) {
        return uint256(data);
    }

    function uintToBytes(uint256 val) public pure returns (bytes32) {
        return bytes32(val);
    }

    function encodeValues(
        uint256 a,
        address b,
        string memory c
    ) public pure returns (bytes memory) {
        return abi.encode(a, b, c);
    }

    function decodeValues(bytes memory data)
        public
        pure
        returns (uint256, address, string memory)
    {
        (uint256 a, address b, string memory c) = abi.decode(data, (uint256, address, string));
        return (a, b, c);
    }

    function signedCast(int256 signed) public pure returns (uint256) {
        require(signed >= 0, "negative");
        return uint256(signed);
    }
}
