// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title InterfaceDetector — ERC-165 supportsInterface compatibility shim, compiled to Neo N3.
/// @notice On Neo, the manifest's `supportedstandards` field already advertises which
/// standards a contract implements. This contract demonstrates an explicit C# /
/// Solidity supportsInterface(bytes4) — useful as a compatibility shim for cross-chain
/// SDK clients that expect ERC-165 dispatch.
contract InterfaceDetector {
    string public buildTag = "interface-detector-v1";

    // ERC-165 itself
    bytes4 public constant ID_ERC165       = 0x01ffc9a7;
    // ERC-721 NFT
    bytes4 public constant ID_ERC721       = 0x80ac58cd;
    // ERC-1155 Multi-Token
    bytes4 public constant ID_ERC1155      = 0xd9b67a26;
    // ERC-20 (note: ERC-20 doesn't actually have a 165 ID; this is illustrative)
    bytes4 public constant ID_ERC20_LIKE   = 0x36372b07;

    function supportsInterface(bytes4 id) public pure returns (bool) {
        if (id == 0x01ffc9a7) return true; // ERC-165
        if (id == 0x80ac58cd) return true; // ERC-721
        if (id == 0xd9b67a26) return true; // ERC-1155
        if (id == 0x36372b07) return true; // ERC-20-like
        return false;
    }
}
