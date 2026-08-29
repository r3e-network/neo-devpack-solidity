// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title DomainExposer — ERC-5267 EIP-712 domain retrieval, compiled to Neo N3.
contract DomainExposer {
    string public buildTag = "domain-exposer-v1";
    string public NAME = "MyDApp";
    string public VERSION = "1";

    function getName() public view returns (string memory) { return NAME; }
    function getVersion() public view returns (string memory) { return VERSION; }
}
