// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title GuardedNFT — ERC-6147 NFT with guard delegate, compiled to Neo N3.
/// @notice Each token can register a `guard` address. While the guard is set,
/// only the guard can transfer the token; once cleared, control returns to the
/// owner. This single-required-signer model is what's expressible in both
/// Solidity (one msg.sender per call) and Neo C# (one Runtime.CheckWitness call),
/// keeping the two implementations behaviorally aligned.
contract GuardedNFT {
    string public buildTag = "guarded-v2";
    string public name = "Guarded NFT";
    string public symbol = "GNFT";

    address public deployer;
    uint256 private _next;
    mapping(uint256 => address) public ownerOf;
    mapping(uint256 => address) public guardOf;

    function claimDeployer() public {
        require(deployer == address(0), "GNFT: already claimed");
        deployer = msg.sender;
    }

    function mint(address to) public returns (uint256 tokenId) {
        require(msg.sender == deployer, "GNFT: deployer only");
        tokenId = ++_next;
        ownerOf[tokenId] = to;
    }

    function setGuard(uint256 tokenId, address guard) public {
        require(msg.sender == ownerOf[tokenId], "GNFT: not owner");
        guardOf[tokenId] = guard;
    }

    /// Transfer is gated by whichever address is currently in control:
    /// the guard if set, otherwise the owner.
    function transfer(address to, uint256 tokenId) public returns (bool) {
        address owner = ownerOf[tokenId];
        require(owner != address(0), "GNFT: nonexistent");
        address guard = guardOf[tokenId];
        address required = guard == address(0) ? owner : guard;
        require(msg.sender == required, "GNFT: not authorized");
        ownerOf[tokenId] = to;
        return true;
    }

    function tokenCount() public view returns (uint256) { return _next; }
}
