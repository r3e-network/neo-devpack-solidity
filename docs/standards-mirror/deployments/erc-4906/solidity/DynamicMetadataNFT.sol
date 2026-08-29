// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title DynamicMetadataNFT — ERC-4906 metadata-update event demo, compiled to Neo N3.
contract DynamicMetadataNFT {
    string public buildTag = "dynamic-metadata-v1";
    string public name = "Dynamic Metadata NFT";
    string public symbol = "DMETA";

    address public deployer;
    uint256 public nextId;
    mapping(uint256 => string) private _uri;

    event MetadataUpdate(uint256 tokenId);
    event BatchMetadataUpdate(uint256 fromTokenId, uint256 toTokenId);

    function claimDeployer() public {
        require(deployer == address(0), "DMETA: already claimed");
        deployer = msg.sender;
    }

    function mint(string memory uri) public returns (uint256 id) {
        require(msg.sender == deployer, "DMETA: deployer only");
        id = ++nextId;
        _uri[id] = uri;
    }

    function tokenURI(uint256 id) public view returns (string memory) {
        require(bytes(_uri[id]).length > 0, "DMETA: nonexistent");
        return _uri[id];
    }

    function setTokenURI(uint256 id, string memory uri) public {
        require(msg.sender == deployer, "DMETA: deployer only");
        require(bytes(_uri[id]).length > 0, "DMETA: nonexistent");
        _uri[id] = uri;
        emit MetadataUpdate(id);
    }

    function setBatchTokenURI(uint256 fromId, uint256 toId, string memory uri) public {
        require(msg.sender == deployer, "DMETA: deployer only");
        require(fromId <= toId, "DMETA: bad range");
        for (uint256 i = fromId; i <= toId; ++i) {
            require(bytes(_uri[i]).length > 0, "DMETA: nonexistent in range");
            _uri[i] = uri;
        }
        emit BatchMetadataUpdate(fromId, toId);
    }
}
