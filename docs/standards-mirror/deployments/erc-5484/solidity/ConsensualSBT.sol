// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title ConsensualSBT — ERC-5484 with explicit burn-auth, compiled to Neo N3.
contract ConsensualSBT {
    string public buildTag = "consensual-sbt-v1";
    string public name = "Consensual SBT";
    string public symbol = "CSBT";

    // BurnAuth: 0=IssuerOnly, 1=OwnerOnly, 2=Both, 3=Neither
    address public issuer;
    uint256 private _next;
    mapping(uint256 => address) public ownerOf;
    mapping(uint256 => uint8)   public burnAuth;

    function claimIssuer() public {
        require(issuer == address(0), "CSBT: already claimed");
        issuer = msg.sender;
    }

    function issue(address to, uint8 auth) public returns (uint256 tokenId) {
        require(msg.sender == issuer, "CSBT: issuer only");
        require(auth <= 3, "CSBT: bad auth");
        tokenId = ++_next;
        ownerOf[tokenId] = to;
        burnAuth[tokenId] = auth;
    }

    function burn(uint256 tokenId) public {
        uint8 auth = burnAuth[tokenId];
        address owner = ownerOf[tokenId];
        bool can =
            (auth == 0 && msg.sender == issuer) ||
            (auth == 1 && msg.sender == owner) ||
            (auth == 2 && (msg.sender == issuer || msg.sender == owner));
        require(can, "CSBT: not authorized");
        delete ownerOf[tokenId];
        delete burnAuth[tokenId];
    }

    function tokenCount() public view returns (uint256) { return _next; }
}
