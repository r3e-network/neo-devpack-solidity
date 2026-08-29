// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title Bond — ERC-3525 semi-fungible reference, compiled to Neo N3 via neo-solc.
contract Bond {
    string public buildTag = "bond-v3-testnet-mirror";   // ensures distinct bytecode
    string public name = "Demo Solidity Bond";
    string public symbol = "DSOLBND";
    uint8  public valueDecimalsValue = 8;

    uint256 private _nextId;

    struct Token { address owner; uint256 slot; uint256 value; }
    mapping(uint256 => Token) public tokens;

    /// NOTE: events suppressed for testnet deployability — see DemoToken.sol notes.

    function valueDecimals() public view returns (uint8) { return valueDecimalsValue; }

    function balanceOfToken(uint256 tokenId) public view returns (uint256) {
        return tokens[tokenId].value;
    }

    function ownerOf(uint256 tokenId) public view returns (address) {
        address o = tokens[tokenId].owner;
        require(o != address(0), "Bond: nonexistent token");
        return o;
    }

    function slotOf(uint256 tokenId) public view returns (uint256) {
        require(tokens[tokenId].owner != address(0), "Bond: nonexistent token");
        return tokens[tokenId].slot;
    }

    /// Demo faucet — anyone can mint, demo only.
    function mint(address to, uint256 slot, uint256 value) public returns (uint256 tokenId) {
        tokenId = ++_nextId;
        tokens[tokenId] = Token(to, slot, value);
    }

    /// Move `value` from one tokenId to another within the same slot.
    function transferValueToToken(uint256 fromTokenId, uint256 toTokenId, uint256 value) public {
        Token storage src = tokens[fromTokenId];
        Token storage dst = tokens[toTokenId];
        require(msg.sender == src.owner, "Bond: not owner");
        require(src.slot == dst.slot,    "Bond: slot mismatch");
        require(src.value >= value,      "Bond: insufficient value");
        src.value -= value;
        dst.value += value;
    }

    /// Split: move `value` to a new token owned by `to`.
    function transferValueToAddress(uint256 fromTokenId, address to, uint256 value)
        public returns (uint256 newTokenId)
    {
        Token storage src = tokens[fromTokenId];
        require(msg.sender == src.owner, "Bond: not owner");
        require(src.value >= value,      "Bond: insufficient value");
        src.value -= value;
        newTokenId = ++_nextId;
        tokens[newTokenId] = Token(to, src.slot, value);
    }
}
