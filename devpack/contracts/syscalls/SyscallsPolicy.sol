// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./SyscallsBase.sol";

/**
 * @title Syscalls Policy — Neo N3 Policy operations
 */

library SyscallsPolicy {
    // ========== Policy System Calls ==========
    
    /**
     * @dev Get fee per byte
     */
    function getFeePerByte() internal view returns (uint256) {
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.POLICY_CONTRACT, "getFeePerByte", "");
        return abi.decode(result, (uint256));
    }
    
    /**
     * @dev Get exec fee factor
     */
    function getExecFeeFactor() internal view returns (uint32) {
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.POLICY_CONTRACT, "getExecFeeFactor", "");
        return abi.decode(result, (uint32));
    }

    /**
     * @dev Get exec fee factor in picoGAS units
     */
    function getExecPicoFeeFactor() internal view returns (uint256) {
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.POLICY_CONTRACT, "getExecPicoFeeFactor", "");
        return abi.decode(result, (uint256));
    }

    /**
     * @dev Get storage price
     */
    function getStoragePrice() internal view returns (uint256) {
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.POLICY_CONTRACT, "getStoragePrice", "");
        return abi.decode(result, (uint256));
    }

    /**
     * @dev Get block milliseconds
     */
    function getMillisecondsPerBlock() internal view returns (uint256) {
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.POLICY_CONTRACT, "getMillisecondsPerBlock", "");
        return abi.decode(result, (uint256));
    }

    /**
     * @dev Get max valid-until-block increment
     */
    function getMaxValidUntilBlockIncrement() internal view returns (uint256) {
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.POLICY_CONTRACT, "getMaxValidUntilBlockIncrement", "");
        return abi.decode(result, (uint256));
    }

    /**
     * @dev Get max traceable blocks
     */
    function getMaxTraceableBlocks() internal view returns (uint256) {
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.POLICY_CONTRACT, "getMaxTraceableBlocks", "");
        return abi.decode(result, (uint256));
    }

    /**
     * @dev Get attribute fee
     */
    function getAttributeFee(uint8 attributeType) internal view returns (uint256) {
        bytes memory data = abi.encode(attributeType);
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.POLICY_CONTRACT, "getAttributeFee", data);
        return abi.decode(result, (uint256));
    }

    /**
     * @dev Check if account is blocked
     */
    function isBlocked(address account) internal view returns (bool) {
        bytes memory data = abi.encode(account);
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.POLICY_CONTRACT, "isBlocked", data);
        return abi.decode(result, (bool));
    }
    
}
