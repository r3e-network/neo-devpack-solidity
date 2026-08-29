// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title InterfaceRegistry — ERC-1820 reference, compiled to Neo N3.
contract InterfaceRegistry {
    string public buildTag = "iface-registry-v1";

    // (account, interfaceHash) -> implementer
    mapping(address => mapping(bytes32 => address)) private _implementer;
    mapping(address => address) private _manager;

    function getImplementer(address account, bytes32 interfaceHash) public view returns (address) {
        return _implementer[account][interfaceHash];
    }

    function getManager(address account) public view returns (address) {
        address m = _manager[account];
        return m == address(0) ? account : m;
    }

    function setImplementer(address account, bytes32 interfaceHash, address impl) public {
        require(getManager(account) == msg.sender, "Registry: not manager");
        _implementer[account][interfaceHash] = impl;
    }

    function setManager(address account, address newManager) public {
        require(getManager(account) == msg.sender, "Registry: not manager");
        _manager[account] = newManager == account ? address(0) : newManager;
    }
}
