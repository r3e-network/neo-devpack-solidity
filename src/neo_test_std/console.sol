// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/// @title neo-test console (hardhat/forge `console.log`-style debug output)
/// @notice `console.log(...)` emits via the NeoVM `System.Runtime.Log` syscall;
/// `neo-test` captures and prints those lines under each test (and always for a
/// failing test). On a real node these surface as application-log entries.
///
///   import "neo-std/console.sol";
///   console.log("here");
///   console.log("balance", balance);   // appends the value
library console {
    function log(string memory s) internal {
        Runtime.log(s);
    }

    function log(string memory label, uint256 v) internal {
        Runtime.log(string(abi.encodePacked(label, " = ", _u(v))));
    }

    function log(string memory label, int256 v) internal {
        if (v < 0) {
            Runtime.log(string(abi.encodePacked(label, " = -", _u(uint256(-v)))));
        } else {
            Runtime.log(string(abi.encodePacked(label, " = ", _u(uint256(v)))));
        }
    }

    function log(string memory label, bool v) internal {
        Runtime.log(string(abi.encodePacked(label, " = ", v ? "true" : "false")));
    }

    function log(string memory label, address v) internal {
        Runtime.log(string(abi.encodePacked(label, " = ", _hex(abi.encodePacked(v)))));
    }

    function logBytes(string memory label, bytes memory v) internal {
        Runtime.log(string(abi.encodePacked(label, " = 0x", _hex(v))));
    }

    // ---- helpers ----

    /// Decimal rendering of an unsigned integer.
    function _u(uint256 v) private pure returns (string memory) {
        if (v == 0) return "0";
        uint256 n = v;
        uint256 digits;
        while (n != 0) {
            digits++;
            n /= 10;
        }
        bytes memory buf = new bytes(digits);
        n = v;
        while (n != 0) {
            digits -= 1;
            buf[digits] = bytes1(uint8(48 + (n % 10)));
            n /= 10;
        }
        return string(buf);
    }

    /// Lowercase hex (no `0x` prefix) of a byte string.
    function _hex(bytes memory b) private pure returns (string memory) {
        bytes memory hexchars = "0123456789abcdef";
        bytes memory out = new bytes(b.length * 2);
        for (uint256 i = 0; i < b.length; i++) {
            out[2 * i] = hexchars[uint8(b[i]) >> 4];
            out[2 * i + 1] = hexchars[uint8(b[i]) & 0x0f];
        }
        return string(out);
    }
}
