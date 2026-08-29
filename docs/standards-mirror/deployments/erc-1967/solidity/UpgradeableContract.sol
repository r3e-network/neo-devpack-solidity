// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title UpgradeableContract — ERC-1967 in-place upgrade pattern, compiled to Neo N3.
///        Demonstrates the NEP-22 update mechanism — Neo upgrades the contract
///        bytecode in place rather than via a proxy.
contract UpgradeableContract {
    string public buildTag = "upgradeable-v1";
    address public ownerAddress;
    uint256 public version;
    string public greeting;

    function claimOwner() public {
        require(ownerAddress == address(0), "Upgradeable: already claimed");
        ownerAddress = msg.sender;
        version = 1;
        greeting = "Hello from v1";
    }

    function setGreeting(string memory g) public {
        require(msg.sender == ownerAddress, "Upgradeable: not owner");
        greeting = g;
    }

    function getVersion() public view returns (uint256) { return version; }
    function getGreeting() public view returns (string memory) { return greeting; }
}
