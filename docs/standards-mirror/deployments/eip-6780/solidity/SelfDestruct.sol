// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title SelfDestruct — EIP-6780 SELFDESTRUCT change demo, compiled to Neo N3.
/// @notice EIP-6780 nerfed SELFDESTRUCT: post-Cancun it only clears storage
/// (and zeros code) when called in the same transaction the contract was created
/// in. Neo never had this confusion — `ContractManagement.Destroy()` is a
/// deliberate, owner-authorized operation. This contract demonstrates the
/// authorize-and-destroy pattern.
contract SelfDestruct {
    string public buildTag = "self-destruct-v1";

    address public deployer;
    bool public destructEnabled;

    function claimDeployer() public {
        require(deployer == address(0), "SD: already claimed");
        deployer = msg.sender;
    }

    function armDestruct() public {
        require(msg.sender == deployer, "SD: deployer only");
        destructEnabled = true;
    }

    function isArmed() public view returns (bool) { return destructEnabled; }
}
