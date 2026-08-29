// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./SyscallsBase.sol";

/**
 * @title Syscalls Role — Neo N3 Role operations
 */

library SyscallsRole {
    // ========== Role Management System Calls ==========
    
    /**
     * @dev Get designated by role
     */
    function getDesignatedByRole(bytes1 role, uint256 index) internal view returns (bytes[] memory) {
        bytes memory data = abi.encode(role, index);
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.ROLE_MANAGEMENT, "getDesignatedByRole", data);
        return abi.decode(result, (bytes[]));
    }
    
}
