// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title SetCode — EIP-7702 set-code-for-EOAs demo, compiled to Neo N3.
/// @notice EIP-7702 lets an EOA delegate execution to contract code via a signed
/// authorization, blurring the EOA / contract distinction. On Neo, accounts can
/// already be smart contracts via NEP-30 (the verify trigger). Anyone deploying
/// a contract with a verify() method has produced a "smart account" — no
/// retroactive upgrade required.
contract SetCode {
    string public buildTag = "set-code-v1";

    address public delegateTarget;
    uint256 public delegationCount;
    address public deployer;

    function claimDeployer() public {
        require(deployer == address(0), "SC: already claimed");
        deployer = msg.sender;
    }

    function setDelegate(address target) public {
        require(msg.sender == deployer, "SC: deployer only");
        delegateTarget = target;
        delegationCount += 1;
    }

    function getDelegate() public view returns (address) { return delegateTarget; }
}
