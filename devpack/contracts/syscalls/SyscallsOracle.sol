// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./SyscallsBase.sol";

/**
 * @title Syscalls Oracle — Neo N3 Oracle operations
 */

library SyscallsOracle {
    // ========== Oracle System Calls ==========
    
    /**
     * @dev Make oracle request
     */
    function oracleRequest(
        string memory url,
        string memory filter,
        string memory callback,
        bytes memory userData,
        uint256 gasForResponse
    ) internal {
        bytes memory data = abi.encode(url, filter, callback, userData, gasForResponse);
        SyscallsBase.contractCall(SyscallsBase.ORACLE_CONTRACT, "request", data);
    }
    
    /**
     * @dev Get oracle price
     */
    function getOraclePrice() internal view returns (uint256) {
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.ORACLE_CONTRACT, "getPrice", "");
        return abi.decode(result, (uint256));
    }

    // NOTE: The Oracle native contract exposes exactly five ABI methods on
    // Neo N3 (request, getPrice, setPrice, finish, verify). The former
    // `getOracleNodes` / `getOracleRequests` wrappers called methods that do
    // NOT exist on any Neo N3 node and were removed (they would fault with
    // "Method not found" on-chain).

}
