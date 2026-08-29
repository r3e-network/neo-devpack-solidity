// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "../NativeContracts.sol";
import "../Syscalls.sol";

/**
 * @title Native Policy — Neo N3 Policy native contract operations
 */

library NativePolicy {
    address constant POLICY_CONTRACT = NativeContracts.POLICY_CONTRACT;
    // ========== Policy Native Contract ==========
    
    /**
     * @dev Get fee per byte
     */
    function getFeePerByte() internal view returns (uint256) {
        bytes memory result = Syscalls.contractCall(NativeContracts.POLICY_CONTRACT, "getFeePerByte", "");
        return abi.decode(result, (uint256));
    }
    
    /**
     * @dev Set fee per byte
     */
    function setFeePerByte(uint256 value) internal {
        bytes memory params = abi.encode(value);
        Syscalls.contractCall(NativeContracts.POLICY_CONTRACT, "setFeePerByte", params);
    }
    
    /**
     * @dev Get execution fee factor
     */
    function getExecFeeFactor() internal view returns (uint32) {
        bytes memory result = Syscalls.contractCall(NativeContracts.POLICY_CONTRACT, "getExecFeeFactor", "");
        return abi.decode(result, (uint32));
    }

    /**
     * @dev Get execution fee factor in picoGAS
     */
    function getExecPicoFeeFactor() internal view returns (uint256) {
        bytes memory result = Syscalls.contractCall(NativeContracts.POLICY_CONTRACT, "getExecPicoFeeFactor", "");
        return abi.decode(result, (uint256));
    }
    
    /**
     * @dev Set execution fee factor
     *
     * C# signature: `SetExecFeeFactor(ulong value)` — uint64, not uint32.
     */
    function setExecFeeFactor(uint64 value) internal {
        bytes memory params = abi.encode(value);
        Syscalls.contractCall(NativeContracts.POLICY_CONTRACT, "setExecFeeFactor", params);
    }
    
    /**
     * @dev Get storage price
     */
    function getStoragePrice() internal view returns (uint256) {
        bytes memory result = Syscalls.contractCall(NativeContracts.POLICY_CONTRACT, "getStoragePrice", "");
        return abi.decode(result, (uint256));
    }

    /**
     * @dev Get milliseconds per block
     */
    function getMillisecondsPerBlock() internal view returns (uint256) {
        bytes memory result = Syscalls.contractCall(NativeContracts.POLICY_CONTRACT, "getMillisecondsPerBlock", "");
        return abi.decode(result, (uint256));
    }

    /**
     * @dev Set milliseconds per block
     */
    function setMillisecondsPerBlock(uint256 value) internal {
        bytes memory params = abi.encode(value);
        Syscalls.contractCall(NativeContracts.POLICY_CONTRACT, "setMillisecondsPerBlock", params);
    }

    /**
     * @dev Get max valid-until-block increment
     */
    function getMaxValidUntilBlockIncrement() internal view returns (uint256) {
        bytes memory result = Syscalls.contractCall(NativeContracts.POLICY_CONTRACT, "getMaxValidUntilBlockIncrement", "");
        return abi.decode(result, (uint256));
    }

    /**
     * @dev Set max valid-until-block increment
     */
    function setMaxValidUntilBlockIncrement(uint256 value) internal {
        bytes memory params = abi.encode(value);
        Syscalls.contractCall(NativeContracts.POLICY_CONTRACT, "setMaxValidUntilBlockIncrement", params);
    }

    /**
     * @dev Get max traceable blocks
     */
    function getMaxTraceableBlocks() internal view returns (uint256) {
        bytes memory result = Syscalls.contractCall(NativeContracts.POLICY_CONTRACT, "getMaxTraceableBlocks", "");
        return abi.decode(result, (uint256));
    }

    /**
     * @dev Set max traceable blocks
     */
    function setMaxTraceableBlocks(uint256 value) internal {
        bytes memory params = abi.encode(value);
        Syscalls.contractCall(NativeContracts.POLICY_CONTRACT, "setMaxTraceableBlocks", params);
    }

    /**
     * @dev Get attribute fee
     */
    function getAttributeFee(uint8 attributeType) internal view returns (uint256) {
        bytes memory params = abi.encode(attributeType);
        bytes memory result = Syscalls.contractCall(NativeContracts.POLICY_CONTRACT, "getAttributeFee", params);
        return abi.decode(result, (uint256));
    }

    /**
     * @dev Set attribute fee
     */
    function setAttributeFee(uint8 attributeType, uint256 value) internal {
        bytes memory params = abi.encode(attributeType, value);
        Syscalls.contractCall(NativeContracts.POLICY_CONTRACT, "setAttributeFee", params);
    }
    
    /**
     * @dev Set storage price
     */
    function setStoragePrice(uint256 value) internal {
        bytes memory params = abi.encode(value);
        Syscalls.contractCall(NativeContracts.POLICY_CONTRACT, "setStoragePrice", params);
    }
    
    /**
     * @dev Block account
     *
     * C# `BlockAccount` returns bool (false when the account was already
     * blocked or the committee check failed).
     */
    function blockAccount(address account) internal returns (bool) {
        bytes memory params = abi.encode(account);
        bytes memory result = Syscalls.contractCall(NativeContracts.POLICY_CONTRACT, "blockAccount", params);
        return abi.decode(result, (bool));
    }

    /**
     * @dev Unblock account
     *
     * C# `UnblockAccount` returns bool (false when the account was not
     * blocked).
     */
    function unblockAccount(address account) internal returns (bool) {
        bytes memory params = abi.encode(account);
        bytes memory result = Syscalls.contractCall(NativeContracts.POLICY_CONTRACT, "unblockAccount", params);
        return abi.decode(result, (bool));
    }
    
    /**
     * @dev Check if account is blocked
     */
    function isBlocked(address account) internal view returns (bool) {
        bytes memory params = abi.encode(account);
        bytes memory result = Syscalls.contractCall(NativeContracts.POLICY_CONTRACT, "isBlocked", params);
        return abi.decode(result, (bool));
    }

    /**
     * @dev Get blocked accounts iterator
     */
    function getBlockedAccounts() internal view returns (Syscalls.Iterator memory) {
        bytes memory result = Syscalls.contractCall(NativeContracts.POLICY_CONTRACT, "getBlockedAccounts", "");
        return abi.decode(result, (Syscalls.Iterator));
    }

    /**
     * @dev Recover blocked funds (committee only)
     */
    function recoverFund(address account, address token) internal returns (bool) {
        bytes memory params = abi.encode(account, token);
        bytes memory result = Syscalls.contractCall(NativeContracts.POLICY_CONTRACT, "recoverFund", params);
        return abi.decode(result, (bool));
    }

    /**
     * @dev Set whitelist fee contract (committee only)
     */
    function setWhitelistFeeContract(
        address contractHash,
        string memory method,
        uint8 argCount,
        uint256 fixedFee
    ) internal {
        bytes memory params = abi.encode(contractHash, method, argCount, fixedFee);
        Syscalls.contractCall(NativeContracts.POLICY_CONTRACT, "setWhitelistFeeContract", params);
    }

    /**
     * @dev Remove whitelist fee contract (committee only)
     */
    function removeWhitelistFeeContract(
        address contractHash,
        string memory method,
        uint8 argCount
    ) internal {
        bytes memory params = abi.encode(contractHash, method, argCount);
        Syscalls.contractCall(NativeContracts.POLICY_CONTRACT, "removeWhitelistFeeContract", params);
    }

    /**
     * @dev Get whitelisted fee contracts iterator
     */
    function getWhitelistFeeContracts() internal view returns (Syscalls.Iterator memory) {
        bytes memory result = Syscalls.contractCall(NativeContracts.POLICY_CONTRACT, "getWhitelistFeeContracts", "");
        return abi.decode(result, (Syscalls.Iterator));
    }
    
}
