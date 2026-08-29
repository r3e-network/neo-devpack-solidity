// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract SimpleNFT {
    string public name = "Neo Artifacts";
    string public symbol = "ART";
    // NEP-11 non-divisible tokens use 0 decimals.
    uint8 public decimals = 0;
    uint256 public totalSupply;

    mapping(uint256 => address) private owners;
    mapping(address => uint256) private balances;
    uint256[] private allTokens;

    event Transfer(address indexed from, address indexed to, uint256 indexed tokenId, bytes data);

    function ownerOf(uint256 tokenId) public view returns (address) {
        address owner = owners[tokenId];
        require(owner != address(0), "nonexistent token");
        return owner;
    }

    function balanceOf(address account) public view returns (uint256) {
        require(account != address(0), "zero address");
        return balances[account];
    }

    function tokensOf(address account) public view returns (uint256[] memory) {
        uint256[] memory result = new uint256[](balances[account]);
        uint256 cursor = 0;
        for (uint256 i = 0; i < allTokens.length; i++) {
            if (owners[allTokens[i]] == account) {
                result[cursor] = allTokens[i];
                cursor++;
            }
        }
        return result;
    }

    function mint(address to, uint256 tokenId) public {
        require(to != address(0), "zero address");
        require(owners[tokenId] == address(0), "already minted");

        owners[tokenId] = to;
        balances[to] += 1;
        totalSupply += 1;
        allTokens.push(tokenId);

        emit Transfer(address(0), to, tokenId, "");
    }

    function transfer(address to, uint256 tokenId, bytes memory data) public {
        require(to != address(0), "zero address");
        address from = ownerOf(tokenId);
        require(msg.sender == from, "only owner can transfer");

        owners[tokenId] = to;
        balances[from] -= 1;
        balances[to] += 1;
        emit Transfer(from, to, tokenId, data);
    }

    // Compatibility wrapper for familiar ERC-721 UX.
    function transferFrom(address from, address to, uint256 tokenId) public {
        require(from == ownerOf(tokenId), "not owner");
        transfer(to, tokenId, "");
    }
}
