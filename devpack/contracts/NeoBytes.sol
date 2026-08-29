// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title NeoBytes
 * @dev Byte parsing helpers for NeoVM-oriented ABI/data bridge handling.
 */
abstract contract NeoBytes {
    /**
     * @dev Check whether every item is exactly 1 byte.
     */
    function nbAllSingleByteItems(bytes[] calldata items) internal pure returns (bool) {
        for (uint256 i = 0; i < items.length; i++) {
            if (items[i].length != 1) {
                return false;
            }
        }
        return true;
    }

    /**
     * @dev Flatten an array of byte slices into one contiguous buffer.
     */
    function nbFlattenItems(bytes[] calldata items) internal pure returns (bytes memory out) {
        uint256 total = 0;
        for (uint256 i = 0; i < items.length; i++) {
            total += items[i].length;
        }

        out = new bytes(total);
        uint256 offset = 0;
        for (uint256 i = 0; i < items.length; i++) {
            bytes calldata item = items[i];
            for (uint256 j = 0; j < item.length; j++) {
                out[offset++] = item[j];
            }
        }
    }

    /**
     * @dev Read a 32-byte unsigned integer from big-endian memory bytes at `start`.
     */
    function nbBytesToUintBE(bytes memory raw, uint256 start) internal pure returns (uint256 value) {
        if (start + 32 > raw.length) {
            return 0;
        }

        for (uint256 i = 0; i < 32; i++) {
            value = (value << 8) | uint256(uint8(raw[start + i]));
        }
    }

    /**
     * @dev Decode UTF-8 string slice from bytes range.
     */
    function nbBytesToString(bytes memory raw, uint256 start, uint256 len) internal pure returns (string memory) {
        if (start + len > raw.length) {
            return "";
        }

        bytes memory out = new bytes(len);
        for (uint256 i = 0; i < len; i++) {
            out[i] = raw[start + i];
        }
        return string(out);
    }

    /**
     * @dev Decode little-endian uint from memory bytes (up to 32 bytes).
     */
    function nbBytesToUintLE(bytes memory raw) internal pure returns (uint256 value) {
        uint256 len = raw.length;
        if (len > 32) {
            len = 32;
        }

        for (uint256 i = 0; i < len; i++) {
            value |= uint256(uint8(raw[i])) << (8 * i);
        }
    }

    /**
     * @dev Decode little-endian uint from calldata bytes (up to 32 bytes).
     */
    function nbBytesToUintLE(bytes calldata raw) internal pure returns (uint256 value) {
        uint256 len = raw.length;
        if (len > 32) {
            len = 32;
        }

        for (uint256 i = 0; i < len; i++) {
            value |= uint256(uint8(raw[i])) << (8 * i);
        }
    }
}
