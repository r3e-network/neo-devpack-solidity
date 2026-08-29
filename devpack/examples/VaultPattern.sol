// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title VaultPattern - ERC-4626 Vault Adapted for Neo N3
 * @dev Demonstrates a deposit/withdraw vault using GAS as the underlying asset.
 *
 * On Ethereum, ERC-4626 vaults use approve/transferFrom for deposits.
 * On Neo N3, authorization uses Runtime.checkWitness() instead.
 * Deposits arrive via the onNEP17Payment callback from the GAS native contract.
 *
 * This example is designed to compile with strict manifest flags:
 *   --deny-wildcard-permissions --deny-wildcard-contracts --deny-wildcard-methods
 */

import "../contracts/FrameworkBase.sol";
import "../contracts/NativeCalls.sol";
import "../libraries/Runtime.sol";

contract VaultPattern is FrameworkBase {
    using Runtime for *;

    // --- State ---
    uint256 public totalShares;
    mapping(address => uint256) public shares;

    // --- Events ---
    event Deposit(address indexed depositor, uint256 assets, uint256 sharesMinted);
    event Withdraw(address indexed recipient, uint256 assets, uint256 sharesBurned);

    // --- Errors ---
    error UnsupportedToken(address token);
    error InsufficientShares(address account, uint256 requested, uint256 available);
    error ZeroAmount();

    constructor() FrameworkBase() {}

    // ------------------------------------------------------------------
    // Core vault logic
    // ------------------------------------------------------------------

    /**
     * @dev Receive GAS deposits via the NEP-17 payment callback.
     *
     * When a user calls `NativeCalls.gasTransfer(sender, vault, amount, "")`
     * the GAS native contract invokes this callback on the vault.
     */
    function onNEP17Payment(address from, uint256 amount, bytes memory) external {
        if (msg.sender != NativeCalls.GAS_CONTRACT) {
            revert UnsupportedToken(msg.sender);
        }
        if (amount == 0) revert ZeroAmount();

        // Mint shares proportional to the deposit relative to the vault's assets
        // BEFORE this deposit. `totalAssets()` reads the live GAS balance, which
        // already includes `amount` (the NEP-17 payment lands before this
        // callback runs), so `_convertToShares(amount)` would divide by the
        // post-deposit total and dilute the depositor. Use the pre-deposit
        // balance as the denominator instead.
        uint256 supply = totalShares;
        uint256 priorAssets = totalAssets() - amount;
        uint256 sharesToMint = (supply == 0 || priorAssets == 0)
            ? amount
            : (amount * supply) / priorAssets;
        shares[from] += sharesToMint;
        totalShares += sharesToMint;

        emit Deposit(from, amount, sharesToMint);
    }

    /**
     * @dev Withdraw underlying GAS by burning vault shares.
     *
     * The caller must be the account owner (verified via Runtime.checkWitness).
     */
    function withdraw(uint256 shareAmount) external {
        if (shareAmount == 0) revert ZeroAmount();

        uint256 callerShares = shares[msg.sender];
        if (callerShares < shareAmount) {
            revert InsufficientShares(msg.sender, shareAmount, callerShares);
        }

        uint256 assets = _convertToAssets(shareAmount);

        shares[msg.sender] = callerShares - shareAmount;
        totalShares -= shareAmount;

        require(
            NativeCalls.gasTransfer(address(this), msg.sender, assets, ""),
            "VaultPattern: GAS transfer failed"
        );

        emit Withdraw(msg.sender, assets, shareAmount);
    }

    // ------------------------------------------------------------------
    // View helpers
    // ------------------------------------------------------------------

    /**
     * @dev Total GAS held by the vault.
     */
    function totalAssets() public view returns (uint256) {
        return NativeCalls.gasBalanceOf(address(this));
    }

    /**
     * @dev Preview how many shares a deposit of `assets` GAS would mint.
     */
    function previewDeposit(uint256 assets) external view returns (uint256) {
        return _convertToShares(assets);
    }

    /**
     * @dev Preview how much GAS a redemption of `shareAmount` would return.
     */
    function previewWithdraw(uint256 shareAmount) external view returns (uint256) {
        return _convertToAssets(shareAmount);
    }

    // ------------------------------------------------------------------
    // Internal conversion (1:1 when vault is empty, proportional otherwise)
    // ------------------------------------------------------------------

    function _convertToShares(uint256 assets) internal view returns (uint256) {
        uint256 supply = totalShares;
        if (supply == 0) return assets;
        return (assets * supply) / totalAssets();
    }

    function _convertToAssets(uint256 shareAmount) internal view returns (uint256) {
        uint256 supply = totalShares;
        if (supply == 0) return shareAmount;
        return (shareAmount * totalAssets()) / supply;
    }
}
