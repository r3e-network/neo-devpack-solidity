// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title BatchMintNFT — ERC-2309 consecutive transfer event demo, compiled to Neo N3.
contract BatchMintNFT {
    string public buildTag = "batch-mint-v1";
    string public name = "Batch Mint NFT";
    string public symbol = "BMNFT";

    address public deployer;
    uint256 public nextId;
    mapping(uint256 => address) public ownerOf;

    function claimDeployer() public {
        require(deployer == address(0), "BMNFT: already claimed");
        deployer = msg.sender;
    }

    /// Mint a contiguous range [from, to-1] to a single owner.
    /// Emits a ConsecutiveTransfer-style event would happen here on Ethereum;
    /// on Neo we encode the same info via Storage and return the range so the
    /// caller can record it off-chain.
    function batchMint(address to, uint256 count) public returns (uint256 fromId, uint256 toId) {
        require(msg.sender == deployer, "BMNFT: deployer only");
        require(count > 0, "BMNFT: count > 0");
        fromId = nextId + 1;
        toId   = nextId + count;
        for (uint256 i = fromId; i <= toId; ++i) {
            ownerOf[i] = to;
        }
        nextId = toId;
    }

    function totalMinted() public view returns (uint256) { return nextId; }
}
