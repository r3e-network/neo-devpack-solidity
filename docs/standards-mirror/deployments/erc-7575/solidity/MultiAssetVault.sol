// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title MultiAssetVault — ERC-7575 multi-asset share vault, compiled to Neo N3.
/// @notice ERC-7575 lets a single share token represent claims on multiple
/// underlying assets. The contract registers (asset → shareToken) pairs via the
/// share() method.
contract MultiAssetVault {
    string public buildTag = "multi-asset-vault-v1";
    string public name = "Multi-Asset Vault";

    address public deployer;
    address public shareToken;
    mapping(address => bool) public assetRegistered;
    uint256 public assetCount;

    function claimDeployer() public {
        require(deployer == address(0), "MAV: already claimed");
        deployer = msg.sender;
    }

    function setShare(address share) public {
        require(msg.sender == deployer, "MAV: deployer only");
        shareToken = share;
    }

    function registerAsset(address asset) public {
        require(msg.sender == deployer, "MAV: deployer only");
        require(!assetRegistered[asset], "MAV: dup");
        assetRegistered[asset] = true;
        assetCount += 1;
    }

    function share() public view returns (address) { return shareToken; }
    function isRegistered(address asset) public view returns (bool) { return assetRegistered[asset]; }
}
