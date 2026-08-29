// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title AccessList — EIP-2930 access-list demo, compiled to Neo N3.
/// @notice EIP-2930 lets a transaction declare upfront which accounts and
/// storage keys it will touch, in exchange for cheaper gas. Neo's witness scopes
/// are the equivalent — `Global`, `CalledByEntry`, `CustomContracts`,
/// `CustomGroups`, `WitnessRules` — but they grant authorization, not gas pricing.
/// This contract exposes the witness-scope constants for clients to inspect.
contract AccessList {
    string public buildTag = "access-list-v1";

    function scopeNone()             public pure returns (uint8) { return 0x00; }
    function scopeCalledByEntry()    public pure returns (uint8) { return 0x01; }
    function scopeCustomContracts()  public pure returns (uint8) { return 0x10; }
    function scopeCustomGroups()     public pure returns (uint8) { return 0x20; }
    function scopeWitnessRules()     public pure returns (uint8) { return 0x40; }
    function scopeGlobal()           public pure returns (uint8) { return 0x80; }
}
