// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title DemoNFT — ERC-721 reference, compiled to Neo N3 via neo-solc.
contract DemoNFT {
    string public buildTag = "demoNFT-v3-testnet-mirror";   // ensures distinct bytecode
    string public name = "Demo Solidity NFT";
    string public symbol = "DSOLNFT";

    mapping(uint256 => address) private _owner;
    mapping(address => uint256) private _balance;
    mapping(uint256 => address) private _approved;
    mapping(address => mapping(address => bool)) private _operator;

    uint256 private _nextId;

    /// NOTE: ERC-721 events suppressed for testnet deployability — see DemoToken.sol notes.

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

    function approve(address to, uint256 tokenId) public {
        address owner = ownerOf(tokenId);
        require(msg.sender == owner || _operator[owner][msg.sender], "ERC721: not authorized");
        _approved[tokenId] = to;
    }

    function setApprovalForAll(address operator, bool approved) public {
        _operator[msg.sender][operator] = approved;
    }

    function transferFrom(address from, address to, uint256 tokenId) public {
        require(_isAuthorized(msg.sender, tokenId), "ERC721: not authorized");
        require(ownerOf(tokenId) == from,           "ERC721: wrong from");
        require(to != address(0),                   "ERC721: zero to");
        delete _approved[tokenId];
        _balance[from] -= 1;
        _balance[to]   += 1;
        _owner[tokenId] = to;
    }

    function _isAuthorized(address spender, uint256 tokenId) internal view returns (bool) {
        address owner = ownerOf(tokenId);
        return spender == owner
            || _approved[tokenId] == spender
            || _operator[owner][spender];
    }
}
