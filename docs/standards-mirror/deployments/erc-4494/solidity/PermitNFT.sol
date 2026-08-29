// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title PermitNFT — ERC-4494 NFT-with-permit demo, compiled to Neo N3.
/// @notice ERC-4494 ports ERC-2612 to NFTs: signed permission to transfer one
/// specific tokenId. On Neo, the same witness-scope mechanism applies — the holder
/// signs the transferring transaction directly. This contract demos the
/// per-token-nonce pattern that an indexer might still want.
contract PermitNFT {
    string public buildTag = "permit-nft-v1";
    string public name = "Permit NFT";
    string public symbol = "PNFT";

    address public deployer;
    uint256 public nextId;
    mapping(uint256 => address) public ownerOf;
    mapping(uint256 => uint256) public tokenNonce;

    function claimDeployer() public {
        require(deployer == address(0), "PNFT: already claimed");
        deployer = msg.sender;
    }

    function mint(address to) public returns (uint256 id) {
        require(msg.sender == deployer, "PNFT: deployer only");
        id = ++nextId;
        ownerOf[id] = to;
    }

    /// Increments the per-token nonce. The holder authorizes by signing the
    /// transaction (witness scope), so an off-chain signature is unnecessary.
    function permit(uint256 tokenId) public {
        require(msg.sender == ownerOf[tokenId], "PNFT: not owner");
        tokenNonce[tokenId] += 1;
    }

    function nonceOf(uint256 tokenId) public view returns (uint256) { return tokenNonce[tokenId]; }
}
