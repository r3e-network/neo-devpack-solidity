// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title Push0 — EIP-3855 PUSH0 opcode demo, compiled to Neo N3.
/// @notice EIP-3855 added the PUSH0 opcode (0x5f) to EVM. NeoVM has had PUSH0
/// (0x10) since day one. This contract demonstrates the optimization: returning
/// a literal zero compiles to a single opcode on both targets.
contract Push0 {
    string public buildTag = "push0-v1";

    function zero() public pure returns (uint256) { return 0; }
    function falseBool() public pure returns (bool) { return false; }
}
