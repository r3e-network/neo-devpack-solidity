// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title DeterministicFactory — ERC-1014 CREATE2-style deploy demo, compiled to Neo N3.
/// @notice CREATE2 lets you predict a contract's address from
/// (deployer, salt, init_code_hash) before deploying. On Neo, contract hashes are
/// already deterministic: Hash160(deployer || nef.script || manifest.name). This
/// factory just records deploy events on behalf of the deployer.
contract DeterministicFactory {
    string public buildTag = "deterministic-factory-v1";

    address public deployer;
    uint256 public deployCount;
    mapping(uint256 => bytes32) public lastSalt;

    function claimDeployer() public {
        require(deployer == address(0), "DF: already claimed");
        deployer = msg.sender;
    }

    /// In Solidity this is normally:
    ///   return Create2.deploy(0, salt, type(MyContract).creationCode);
    /// Neo's equivalent on-chain is ContractManagement.Deploy(nef, manifest, data),
    /// which hashes (sender || script || name) instead of (sender || salt || code).
    /// This demo just records the salt for transparency.
    function recordDeploy(bytes32 salt) public {
        require(msg.sender == deployer, "DF: deployer only");
        deployCount += 1;
        lastSalt[deployCount] = salt;
    }
}
