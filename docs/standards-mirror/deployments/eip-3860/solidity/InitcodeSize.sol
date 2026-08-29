// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title InitcodeSize — EIP-3860 initcode size limit demo, compiled to Neo N3.
/// @notice EIP-3860 caps EVM initcode at 49152 bytes and charges 2 gas per
/// 32-byte word. Neo's NEF script is capped by ContractManagement /
/// PolicyContract — the limit is enforced by the deploy-time validator, not by
/// per-byte gas. This contract exposes the EVM constant so clients can sanity-
/// check their port.
contract InitcodeSize {
    string public buildTag = "initcode-size-v1";
    uint256 public constant EVM_MAX_INITCODE_SIZE = 49152;

    function maxInitcodeSize() public pure returns (uint256) { return EVM_MAX_INITCODE_SIZE; }
}
