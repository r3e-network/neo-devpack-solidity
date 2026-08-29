// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title DIDRegistry — ERC-1056 lightweight identity, compiled to Neo N3.
contract DIDRegistry {
    string public buildTag = "did-registry-v1";

    mapping(address => address) public owners;     // identity -> owner override
    mapping(address => mapping(bytes32 => mapping(address => uint256))) public delegates;
    mapping(address => uint256) public changed;

    function identityOwner(address identity) public view returns (address) {
        address o = owners[identity];
        return o == address(0) ? identity : o;
    }

    function changeOwner(address identity, address newOwner) public {
        require(msg.sender == identityOwner(identity), "DID: not owner");
        owners[identity] = newOwner;
        changed[identity] = block.number;
    }

    function addDelegate(address identity, bytes32 delegateType, address delegate, uint256 validity)
        public
    {
        require(msg.sender == identityOwner(identity), "DID: not owner");
        delegates[identity][delegateType][delegate] = block.timestamp + validity;
        changed[identity] = block.number;
    }

    function validDelegate(address identity, bytes32 delegateType, address delegate)
        public view returns (bool)
    {
        return delegates[identity][delegateType][delegate] > block.timestamp;
    }
}
