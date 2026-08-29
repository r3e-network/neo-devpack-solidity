// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title AsyncVault — ERC-7540 async deposit/redeem vault, compiled to Neo N3.
/// @notice ERC-7540 extends ERC-4626 with two-phase async operations:
/// requestDeposit() → ... time/gate ... → deposit().
/// This demo records pending requests; the actual settlement is left to off-chain
/// keepers per spec.
contract AsyncVault {
    string public buildTag = "async-vault-v1";
    string public name = "Async Vault Share";
    string public symbol = "AVS";

    address public deployer;
    uint256 public pendingDepositCount;
    uint256 public pendingRedeemCount;
    uint256 public claimedDepositCount;
    mapping(uint256 => address) public depositOwner;
    mapping(uint256 => uint256) public depositAssets;

    function claimDeployer() public {
        require(deployer == address(0), "AV: already claimed");
        deployer = msg.sender;
    }

    function requestDeposit(uint256 assets) public returns (uint256 requestId) {
        requestId = ++pendingDepositCount;
        depositOwner[requestId] = msg.sender;
        depositAssets[requestId] = assets;
    }

    function requestRedeem(uint256 /*shares*/) public returns (uint256 requestId) {
        requestId = ++pendingRedeemCount;
    }

    function pendingDepositRequest(uint256 requestId) public view returns (uint256) {
        return depositAssets[requestId];
    }

    function claimDeposit(uint256 requestId) public {
        require(depositOwner[requestId] == msg.sender, "AV: not request owner");
        require(depositAssets[requestId] != 0, "AV: no pending deposit");
        delete depositAssets[requestId];
        claimedDepositCount += 1;
    }
}
