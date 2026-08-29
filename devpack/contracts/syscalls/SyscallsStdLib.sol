// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./SyscallsTypes.sol";
import "./SyscallsBase.sol";

/**
 * @title Syscalls StdLib — Neo N3 StdLib operations
 */

library SyscallsStdLib {
    // ========== StdLib System Calls ==========

    /**
     * @dev Serialize stack item
     */
    function serialize(bytes memory data) internal view returns (bytes memory) {
        bytes memory params = abi.encode(data);
        return SyscallsBase.contractCall(SyscallsBase.STD_LIB, "serialize", params);
    }

    /**
     * @dev Deserialize stack item
     */
    function deserialize(bytes memory data) internal view returns (bytes memory) {
        bytes memory params = abi.encode(data);
        return SyscallsBase.contractCall(SyscallsBase.STD_LIB, "deserialize", params);
    }

    /**
     * @dev Integer to string (base 10)
     */
    function itoa(int256 value) internal view returns (string memory) {
        bytes memory params = abi.encode(value);
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.STD_LIB, "itoa", params);
        return abi.decode(result, (string));
    }

    /**
     * @dev Integer to string with base (10 or 16)
     */
    function itoa(int256 value, uint8 base) internal view returns (string memory) {
        bytes memory params = abi.encode(value, base);
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.STD_LIB, "itoa", params);
        return abi.decode(result, (string));
    }

    /**
     * @dev String to integer (base 10)
     */
    function atoi(string memory value) internal view returns (int256) {
        bytes memory params = abi.encode(value);
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.STD_LIB, "atoi", params);
        return abi.decode(result, (int256));
    }

    /**
     * @dev String to integer with base (10 or 16)
     */
    function atoi(string memory value, uint8 base) internal view returns (int256) {
        bytes memory params = abi.encode(value, base);
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.STD_LIB, "atoi", params);
        return abi.decode(result, (int256));
    }

    // ========== JSON System Calls ==========
    
    /**
     * @dev Serialize to JSON
     */
    function jsonSerialize(bytes memory data) internal view returns (bytes memory) {
        bytes memory params = abi.encode(data);
        return SyscallsBase.contractCall(SyscallsBase.STD_LIB, "jsonSerialize", params);
    }
    
    /**
     * @dev Deserialize from JSON
     */
    function jsonDeserialize(bytes memory json) internal view returns (bytes memory) {
        bytes memory params = abi.encode(json);
        return SyscallsBase.contractCall(SyscallsBase.STD_LIB, "jsonDeserialize", params);
    }
    
    // ========== Base64 System Calls ==========
    
    /**
     * @dev Base64 encode
     */
    function base64Encode(bytes memory data) internal view returns (string memory) {
        bytes memory params = abi.encode(data);
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.STD_LIB, "base64Encode", params);
        return abi.decode(result, (string));
    }
    
    /**
     * @dev Base64 decode
     */
    function base64Decode(string memory data) internal view returns (bytes memory) {
        bytes memory params = abi.encode(data);
        return SyscallsBase.contractCall(SyscallsBase.STD_LIB, "base64Decode", params);
    }

    /**
     * @dev Base64Url encode
     */
    function base64UrlEncode(string memory data) internal view returns (string memory) {
        bytes memory params = abi.encode(data);
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.STD_LIB, "base64UrlEncode", params);
        return abi.decode(result, (string));
    }

    /**
     * @dev Base64Url decode
     */
    function base64UrlDecode(string memory data) internal view returns (string memory) {
        bytes memory params = abi.encode(data);
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.STD_LIB, "base64UrlDecode", params);
        return abi.decode(result, (string));
    }

    // ========== Base58 System Calls ==========

    /**
     * @dev Base58 encode
     */
    function base58Encode(bytes memory data) internal view returns (string memory) {
        bytes memory params = abi.encode(data);
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.STD_LIB, "base58Encode", params);
        return abi.decode(result, (string));
    }

    /**
     * @dev Base58 decode
     */
    function base58Decode(string memory data) internal view returns (bytes memory) {
        bytes memory params = abi.encode(data);
        return SyscallsBase.contractCall(SyscallsBase.STD_LIB, "base58Decode", params);
    }

    /**
     * @dev Base58Check encode
     */
    function base58CheckEncode(bytes memory data) internal view returns (string memory) {
        bytes memory params = abi.encode(data);
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.STD_LIB, "base58CheckEncode", params);
        return abi.decode(result, (string));
    }

    /**
     * @dev Base58Check decode
     */
    function base58CheckDecode(string memory data) internal view returns (bytes memory) {
        bytes memory params = abi.encode(data);
        return SyscallsBase.contractCall(SyscallsBase.STD_LIB, "base58CheckDecode", params);
    }

    /**
     * @dev Hex encode a byte array (lowercase hex)
     *
     * Real StdLib method — `StdLib.cs` declares `HexEncode(byte[])` with a
     * `[ContractMethod]` attribute (the v0.27 removal note claiming otherwise
     * was factually wrong). Lowercase hex, matching C# `ToHexString`... note
     * that `ToHexString` emits UPPERCASE hex; adjust case at the call site if
     * lowercase is required.
     */
    function hexEncode(bytes memory data) internal view returns (string memory) {
        bytes memory params = abi.encode(data);
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.STD_LIB, "hexEncode", params);
        return abi.decode(result, (string));
    }

    /**
     * @dev Decode a hex string back to bytes
     *
     * Real StdLib method — `StdLib.cs` declares `HexDecode(string)`.
     */
    function hexDecode(string memory data) internal view returns (bytes memory) {
        bytes memory params = abi.encode(data);
        return SyscallsBase.contractCall(SyscallsBase.STD_LIB, "hexDecode", params);
    }

    // ========== Memory and String Utilities ==========

    /**
     * @dev Compare two byte arrays
     */
    function memoryCompare(bytes memory left, bytes memory right) internal view returns (int256) {
        bytes memory params = abi.encode(left, right);
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.STD_LIB, "memoryCompare", params);
        return abi.decode(result, (int256));
    }

    /**
     * @dev Search for a value in memory (start at 0)
     */
    function memorySearch(bytes memory mem, bytes memory value) internal view returns (int256) {
        bytes memory params = abi.encode(mem, value);
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.STD_LIB, "memorySearch", params);
        return abi.decode(result, (int256));
    }

    /**
     * @dev Search for a value in memory (start at offset)
     */
    function memorySearch(bytes memory mem, bytes memory value, int256 start) internal view returns (int256) {
        bytes memory params = abi.encode(mem, value, start);
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.STD_LIB, "memorySearch", params);
        return abi.decode(result, (int256));
    }

    /**
     * @dev Search for a value in memory (start at offset, optionally backward)
     */
    function memorySearch(bytes memory mem, bytes memory value, int256 start, bool backward)
        internal
        view
        returns (int256)
    {
        bytes memory params = abi.encode(mem, value, start, backward);
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.STD_LIB, "memorySearch", params);
        return abi.decode(result, (int256));
    }

    /**
     * @dev Split a string by separator
     */
    function stringSplit(string memory value, string memory separator) internal view returns (string[] memory) {
        bytes memory params = abi.encode(value, separator);
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.STD_LIB, "stringSplit", params);
        return abi.decode(result, (string[]));
    }

    /**
     * @dev Split a string by separator with optional empty removal
     */
    function stringSplit(
        string memory value,
        string memory separator,
        bool removeEmptyEntries
    ) internal view returns (string[] memory) {
        bytes memory params = abi.encode(value, separator, removeEmptyEntries);
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.STD_LIB, "stringSplit", params);
        return abi.decode(result, (string[]));
    }

    /**
     * @dev Get string length in text elements
     */
    function strLen(string memory value) internal view returns (uint256) {
        bytes memory params = abi.encode(value);
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.STD_LIB, "strLen", params);
        return abi.decode(result, (uint256));
    }
    
    // ========== Iterator System Calls ==========
    
    /**
     * @dev Get next iterator value
     */
    function iteratorNext(Iterator memory iterator) internal returns (bool) {
        bytes memory data = abi.encode(iterator);
        return SyscallsBase._syscall("System.Iterator.Next", data) != 0;
    }
    
    /**
     * @dev Get iterator value
     */
    function iteratorValue(Iterator memory iterator) internal view returns (bytes memory) {
        bytes memory data = abi.encode(iterator);
        return SyscallsBase._syscallBytes("System.Iterator.Value", data);
    }
    
}
