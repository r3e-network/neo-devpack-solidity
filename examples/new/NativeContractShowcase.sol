// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "../../devpack/contracts/NativeCalls.sol";
import "../../devpack/contracts/Syscalls.sol";

/**
 * @title NativeContractShowcase
 * @notice Integration test contract exercising all 6 newly-implemented native
 *         contracts: Policy, Oracle, RoleManagement, Ledger, Notary, Treasury.
 *
 * Each function wraps a single native contract call so the compiler must
 * lower every NativeCalls helper to a correct System.Contract.Call opcode
 * sequence with the right contract hash, method name, and parameter encoding.
 */
contract NativeContractShowcase {

    // ── Policy ──────────────────────────────────────────────────────────

    function policyFeePerByte() external view returns (uint256) {
        return NativeCalls.getFeePerByte();
    }

    function policyExecFeeFactor() external view returns (uint32) {
        return NativeCalls.getExecFeeFactor();
    }

    function policyStoragePrice() external view returns (uint256) {
        return NativeCalls.getStoragePrice();
    }

    function policyIsBlocked(address account) external view returns (bool) {
        return NativeCalls.isBlocked(account);
    }

    // ── Oracle ──────────────────────────────────────────────────────────

    function oraclePrice() external view returns (uint256) {
        return NativeCalls.getOraclePrice();
    }

    function oracleRequestData(
        string calldata url,
        string calldata filter,
        string calldata callback,
        uint256 gas
    ) external {
        NativeCalls.requestOracleData(url, filter, callback, "", gas);
    }

    // ── RoleManagement ──────────────────────────────────────────────────

    function getDesignated(bytes1 role, uint256 index)
        external
        view
        returns (bytes[] memory)
    {
        return NativeCalls.getDesignatedByRole(role, index);
    }

    // ── Ledger ──────────────────────────────────────────────────────────

    function ledgerCurrentIndex() external view returns (uint256) {
        return NativeCalls.currentIndex();
    }

    function ledgerCurrentHash() external view returns (bytes32) {
        return NativeCalls.currentHash();
    }

    function ledgerTxVMState(bytes32 hash) external view returns (uint8) {
        return NativeCalls.getTransactionVMState(hash);
    }

    // ── Notary ─────────────────────────────────────────────────────────

    function notaryBalance(address account) external view returns (uint256) {
        return NativeCalls.notaryBalanceOf(account);
    }

    function notaryMaxDelta() external view returns (uint256) {
        return NativeCalls.notaryGetMaxNotValidBeforeDelta();
    }

    // ── Treasury ────────────────────────────────────────────────────────

    function treasuryIsValid() external view returns (bool) {
        return NativeCalls.treasuryVerify();
    }

    // ── Cross-contract utility ──────────────────────────────────────────

    function isNative(address addr) external pure returns (bool) {
        return NativeCalls.isNativeContract(addr);
    }

    function nativeName(address addr) external pure returns (string memory) {
        return NativeCalls.getNativeContractName(addr);
    }
}
