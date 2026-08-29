// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "../NativeContracts.sol";
import "../Syscalls.sol";

/**
 * @title Native Notary — Neo N3 Notary native contract operations
 */

library NativeNotary {
    address constant NOTARY_CONTRACT = NativeContracts.NOTARY_CONTRACT;
    // ========== Notary Native Contract ==========

    /**
     * @dev Verify notary-assisted transaction
     */
    function notaryVerify(bytes memory signature) internal view returns (bool) {
        bytes memory params = abi.encode(signature);
        bytes memory result = Syscalls.contractCall(NativeContracts.NOTARY_CONTRACT, "verify", params);
        return abi.decode(result, (bool));
    }

    /**
     * @dev Get notary deposit balance
     */
    function notaryBalanceOf(address account) internal view returns (uint256) {
        bytes memory params = abi.encode(account);
        bytes memory result = Syscalls.contractCall(NativeContracts.NOTARY_CONTRACT, "balanceOf", params);
        return abi.decode(result, (uint256));
    }

    /**
     * @dev Get notary deposit expiration height
     */
    function notaryExpirationOf(address account) internal view returns (uint256) {
        bytes memory params = abi.encode(account);
        bytes memory result = Syscalls.contractCall(NativeContracts.NOTARY_CONTRACT, "expirationOf", params);
        return abi.decode(result, (uint256));
    }

    /**
     * @dev Lock notary deposit until a specific block height
     */
    function notaryLockDepositUntil(address account, uint256 till) internal returns (bool) {
        bytes memory params = abi.encode(account, till);
        bytes memory result = Syscalls.contractCall(NativeContracts.NOTARY_CONTRACT, "lockDepositUntil", params);
        return abi.decode(result, (bool));
    }

    /**
     * @dev Withdraw notary deposit
     */
    function notaryWithdraw(address from, address to) internal returns (bool) {
        bytes memory params = abi.encode(from, to);
        bytes memory result = Syscalls.contractCall(NativeContracts.NOTARY_CONTRACT, "withdraw", params);
        return abi.decode(result, (bool));
    }

    /**
     * @dev Get maximum NotValidBefore delta
     */
    function notaryGetMaxNotValidBeforeDelta() internal view returns (uint256) {
        bytes memory result = Syscalls.contractCall(NativeContracts.NOTARY_CONTRACT, "getMaxNotValidBeforeDelta", "");
        return abi.decode(result, (uint256));
    }

    /**
     * @dev Set maximum NotValidBefore delta
     */
    function notarySetMaxNotValidBeforeDelta(uint256 value) internal {
        bytes memory params = abi.encode(value);
        Syscalls.contractCall(NativeContracts.NOTARY_CONTRACT, "setMaxNotValidBeforeDelta", params);
    }

    /**
     * @dev Handle NEP-17 payment (notary deposit)
     */
    function notaryOnNEP17Payment(address from, uint256 amount, bytes memory data) internal {
        bytes memory params = abi.encode(from, amount, data);
        Syscalls.contractCall(NativeContracts.NOTARY_CONTRACT, "onNEP17Payment", params);
    }

}
