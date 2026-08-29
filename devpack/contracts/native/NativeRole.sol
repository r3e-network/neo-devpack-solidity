// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "../NativeContracts.sol";
import "../Syscalls.sol";

/**
 * @title Native Role — Neo N3 Role native contract operations
 */

library NativeRole {
    address constant ROLE_CONTRACT = NativeContracts.ROLE_CONTRACT;
    // ========== Role Management Native Contract ==========
    
    /**
     * @dev Designate as role
     */
    function designateAsRole(bytes1 role, bytes[] memory publicKeys) internal {
        bytes memory params = abi.encode(role, publicKeys);
        Syscalls.contractCall(NativeContracts.ROLE_MANAGEMENT, "designateAsRole", params);
    }
    
    /**
     * @dev Get designated by role
     */
    function getDesignatedByRole(bytes1 role, uint256 index) internal view returns (bytes[] memory) {
        bytes memory params = abi.encode(role, index);
        bytes memory result = Syscalls.contractCall(NativeContracts.ROLE_MANAGEMENT, "getDesignatedByRole", params);
        return abi.decode(result, (bytes[]));
    }

}
