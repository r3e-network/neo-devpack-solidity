// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

interface IDemoToken {
    function balanceOf(address) external view returns (uint256);
    function transfer(address to, uint256 amount) external returns (bool);
    function transferFrom(address from, address to, uint256 amount) external returns (bool);
}

/// @title YieldVault — ERC-4626 reference vault, compiled to Neo N3.
///        Wraps a NEP-17 underlying asset and mints share tokens 1:1 initially.
contract YieldVault {
    string public buildTag = "yield-vault-v1";
    string public name = "Yield Vault Share";
    string public symbol = "vDEMO";
    uint8  public decimals = 8;
    uint256 public totalSupply;
    address public asset;
    address public deployer;

    mapping(address => uint256) public balanceOf;

    function claimDeployer() public {
        require(deployer == address(0), "Vault: already claimed");
        deployer = msg.sender;
    }

    function setAsset(address newAsset) public {
        require(deployer != address(0), "Vault: unclaimed");
        require(msg.sender == deployer, "Vault: deployer only");
        require(asset == address(0), "Vault: already set");
        asset = newAsset;
    }

    function totalAssets() public view returns (uint256) {
        return IDemoToken(asset).balanceOf(address(this));
    }

    function convertToShares(uint256 assets) public view returns (uint256) {
        uint256 supply = totalSupply;
        // 1:1 exchange when vault is empty (or for first depositor). Subsequent
        // deposits compute pro-rata against current TVL.
        if (supply == 0) return assets;
        return (assets * supply) / totalAssets();
    }

    function convertToAssets(uint256 shares) public view returns (uint256) {
        uint256 supply = totalSupply;
        if (supply == 0) return shares;
        return (shares * totalAssets()) / supply;
    }

    function deposit(uint256 assets, address receiver) public returns (uint256 shares) {
        shares = convertToShares(assets);
        IDemoToken(asset).transferFrom(msg.sender, address(this), assets);
        balanceOf[receiver] += shares;
        totalSupply += shares;
    }

    function redeem(uint256 shares, address to, address from) public returns (uint256 assetsOut) {
        require(msg.sender == from, "Vault: not owner");
        require(balanceOf[from] >= shares, "Vault: insufficient shares");
        assetsOut = convertToAssets(shares);
        balanceOf[from] -= shares;
        totalSupply -= shares;
        IDemoToken(asset).transfer(to, assetsOut);
    }
}
