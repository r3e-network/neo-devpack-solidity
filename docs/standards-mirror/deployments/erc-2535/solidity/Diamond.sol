// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title Diamond — ERC-2535 facet-router reference, compiled to Neo N3.
///        Demonstrates method-name routing to facet contracts. The Solidity
///        version uses string method names (rather than 4-byte selectors)
///        because neo-solc cannot emit `delegatecall` and routes invocations
///        via `Contract.Call(target, methodName, ...)` which expects names.
contract Diamond {
    string public buildTag = "diamond-v1";
    address public ownerAddress;

    mapping(string => address) public facetAddress;
    string[] public methodList;

    function claimOwner() public {
        require(ownerAddress == address(0), "Diamond: already claimed");
        ownerAddress = msg.sender;
    }

    function addFacet(string memory method, address facet) public {
        require(msg.sender == ownerAddress, "Diamond: not owner");
        if (facetAddress[method] == address(0)) {
            methodList.push(method);
        }
        facetAddress[method] = facet;
    }

    function removeFacet(string memory method) public {
        require(msg.sender == ownerAddress, "Diamond: not owner");
        delete facetAddress[method];
    }

    function getFacet(string memory method) public view returns (address) {
        return facetAddress[method];
    }

    function methodCount() public view returns (uint256) {
        return methodList.length;
    }
}
