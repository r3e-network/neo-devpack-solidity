// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title SoulboundCert — ERC-5192 minimal soulbound NFT, compiled to Neo N3.
contract SoulboundCert {
    string public buildTag = "soulbound-v1";
    string public name = "Soulbound Cert";
    string public symbol = "SBC";

    address public issuer;
    uint256 private _next;

    mapping(uint256 => address) private _owner;
    mapping(uint256 => bool)    private _locked;

    function claimIssuer() public {
        require(issuer == address(0), "SBC: already claimed");
        issuer = msg.sender;
    }

    function locked(uint256 tokenId) public view returns (bool) {
        return _locked[tokenId];
    }

    function ownerOf(uint256 tokenId) public view returns (address) {
        address o = _owner[tokenId];
        require(o != address(0), "SBC: nonexistent");
        return o;
    }

    function issue(address to, bool soulbound) public returns (uint256 tokenId) {
        require(msg.sender == issuer, "SBC: issuer only");
        tokenId = ++_next;
        _owner[tokenId] = to;
        if (soulbound) _locked[tokenId] = true;
    }

    function burn(uint256 tokenId) public {
        require(msg.sender == _owner[tokenId] || msg.sender == issuer, "SBC: not authorized");
        delete _owner[tokenId];
        delete _locked[tokenId];
    }

    function unlock(uint256 tokenId) public {
        require(msg.sender == issuer, "SBC: issuer only");
        _locked[tokenId] = false;
    }
}
