// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title Achievement — ERC-5114 soulbound badge bound to a parent NFT, compiled to Neo N3.
contract Achievement {
    string public buildTag = "achievement-v1";
    string public name = "Demo Achievement";
    string public symbol = "ACHV";

    address public deployer;
    uint256 private _next;

    struct Parent { address nft; uint256 tokenId; }
    mapping(uint256 => Parent) public parents;

    function claimDeployer() public {
        require(deployer == address(0), "ACHV: already claimed");
        deployer = msg.sender;
    }

    function attach(address parentNft, uint256 parentTokenId) public returns (uint256 badgeId) {
        require(msg.sender == deployer, "ACHV: deployer only");
        badgeId = ++_next;
        parents[badgeId] = Parent(parentNft, parentTokenId);
    }

    function badgeCount() public view returns (uint256) { return _next; }

    function parentOf(uint256 badgeId) public view returns (address, uint256) {
        Parent memory p = parents[badgeId];
        require(p.nft != address(0), "ACHV: nonexistent");
        return (p.nft, p.tokenId);
    }
}
