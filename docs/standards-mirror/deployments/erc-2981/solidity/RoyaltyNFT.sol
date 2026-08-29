// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title RoyaltyNFT — ERC-721 + ERC-2981 reference, compiled to Neo N3 via neo-solc.
contract RoyaltyNFT {
    string public buildTag = "royaltyNFT-v3-testnet-mirror";  // ensures distinct bytecode
    string public name = "Demo Solidity Royalty NFT";
    string public symbol = "DSOLROY";

    uint256 private _nextId;

    struct RoyaltyData { address receiver; uint96 basisPoints; }

    mapping(uint256 => address)     private _owner;
    mapping(address => uint256)     private _balance;
    mapping(uint256 => RoyaltyData) public  tokenRoyalty;
    RoyaltyData                     public  defaultRoyalty;

    /// NOTE: events suppressed for testnet deployability — see DemoToken.sol notes.

    function balanceOf(address owner) public view returns (uint256) {
        require(owner != address(0), "ERC721: zero owner");
        return _balance[owner];
    }

    function ownerOf(uint256 tokenId) public view returns (address) {
        address o = _owner[tokenId];
        require(o != address(0), "ERC721: nonexistent token");
        return o;
    }

    /// Demo faucet — anyone can mint, demo only.
    function mint(address to) public returns (uint256 tokenId) {
        tokenId = ++_nextId;
        _owner[tokenId] = to;
        _balance[to] += 1;
    }

    function setDefaultRoyalty(address receiver, uint96 bps) public {
        require(bps <= 10_000, "RoyaltyNFT: bps > 100%");
        defaultRoyalty = RoyaltyData(receiver, bps);
    }

    function setTokenRoyalty(uint256 tokenId, address receiver, uint96 bps) public {
        require(_owner[tokenId] != address(0), "RoyaltyNFT: nonexistent token");
        require(bps <= 10_000,                  "RoyaltyNFT: bps > 100%");
        tokenRoyalty[tokenId] = RoyaltyData(receiver, bps);
    }

    /// ERC-2981: single-recipient royalty info.
    function royaltyInfo(uint256 tokenId, uint256 salePrice)
        public view returns (address receiver, uint256 royaltyAmount)
    {
        RoyaltyData memory r = tokenRoyalty[tokenId].receiver != address(0)
            ? tokenRoyalty[tokenId]
            : defaultRoyalty;
        return (r.receiver, (salePrice * r.basisPoints) / 10_000);
    }

    function royaltyAmount(uint256 tokenId, uint256 salePrice) public view returns (uint256) {
        (, uint256 amount) = royaltyInfo(tokenId, salePrice);
        return amount;
    }
}
