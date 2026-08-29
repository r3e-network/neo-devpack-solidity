// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title ModularAccount — ERC-7579 modular smart account, compiled to Neo N3.
/// @notice ERC-7579 standardises module install/uninstall/exec on smart accounts.
/// Module types: 1 = validator, 2 = executor, 3 = fallback, 4 = hook.
contract ModularAccount {
    string public buildTag = "modular-account-v1";

    address public owner;
    mapping(address => uint256) public moduleType; // 0 = not installed
    uint256 public moduleCount;

    function claimOwner() public {
        require(owner == address(0), "MA: already claimed");
        owner = msg.sender;
    }

    function installModule(uint256 moduleTypeId, address module) public {
        require(msg.sender == owner, "MA: owner only");
        require(moduleTypeId >= 1 && moduleTypeId <= 4, "MA: bad type");
        require(moduleType[module] == 0, "MA: already installed");
        moduleType[module] = moduleTypeId;
        moduleCount += 1;
    }

    function uninstallModule(address module) public {
        require(msg.sender == owner, "MA: owner only");
        require(moduleType[module] != 0, "MA: not installed");
        moduleType[module] = 0;
        moduleCount -= 1;
    }

    function isModuleInstalled(uint256 moduleTypeId, address module) public view returns (bool) {
        return moduleType[module] == moduleTypeId;
    }
}
