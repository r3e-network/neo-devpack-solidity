// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title NamespacedStorage — ERC-7201 namespaced-storage layout demo, compiled to Neo N3.
/// @notice ERC-7201 prescribes deriving storage slots from a namespace string so that
/// upgrade-friendly contracts cannot collide with each other. On Neo this is the
/// natural pattern: every storage key carries a namespace prefix byte, and Storage
/// queries already scope to the contract's hash.
contract NamespacedStorage {
    string public buildTag = "namespaced-v1";
    string public namespaceTag = "r3e.standards-mirror.demo.v1";

    address public deployer;
    mapping(bytes32 => uint256) private _slot;

    function claimDeployer() public {
        require(deployer == address(0), "NS: already claimed");
        deployer = msg.sender;
    }

    function setSlot(bytes32 key, uint256 value) public {
        require(msg.sender == deployer, "NS: deployer only");
        _slot[key] = value;
    }

    function getSlot(bytes32 key) public view returns (uint256) { return _slot[key]; }
}
