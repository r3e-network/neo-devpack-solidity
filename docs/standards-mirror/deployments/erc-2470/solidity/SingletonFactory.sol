// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title SingletonFactory — ERC-2470 deploy demo, compiled to Neo N3.
///        Demonstrates that on Neo, "the singleton factory" is essentially every
///        deployer: ContractManagement.Deploy yields a deterministic script hash
///        from (caller, nef, manifestName), so multi-chain deploys produce the
///        same address without requiring a special factory contract.
contract SingletonFactory {
    string public buildTag = "singleton-factory-v1";
    address public deployer;
    uint256 public deployCount;

    function claimDeployer() public {
        require(deployer == address(0), "Factory: already claimed");
        deployer = msg.sender;
    }

    /// Demo: increments a counter. A real Neo factory delegates to
    /// ContractManagement.Deploy directly — see the C# port for that.
    function recordDeploy() public {
        deployCount++;
    }
}
