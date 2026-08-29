// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title CompactSig — EIP-2098 compact signature format mirror, compiled to Neo N3.
/// @notice EIP-2098 packs (r, s, v) into a 64-byte form by encoding the y-parity
/// bit into the top bit of s. On Neo, secp256r1 signatures are already 64 bytes
/// (r || s) — no v / parity overhead. This contract exposes the comparable size
/// constants; the C# port contains the native 64-byte verification helper.
contract CompactSig {
    string public buildTag = "compact-sig-v1";
    uint256 public constant COMPACT_SIG_SIZE = 64;
    uint256 public constant LEGACY_SIG_SIZE  = 65;

    address public deployer;

    function claimDeployer() public {
        require(deployer == address(0), "CS: already claimed");
        deployer = msg.sender;
    }

    function compactSize() public pure returns (uint256) { return COMPACT_SIG_SIZE; }
    function legacySize() public pure returns (uint256) { return LEGACY_SIG_SIZE; }
    function getDeployer() public view returns (address) { return deployer; }
}
