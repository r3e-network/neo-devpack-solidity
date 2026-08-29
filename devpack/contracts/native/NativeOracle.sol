// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "../NativeContracts.sol";
import "../Syscalls.sol";

/**
 * @title Native Oracle — Neo N3 Oracle native contract operations
 */

library NativeOracle {
    address constant ORACLE_CONTRACT = NativeContracts.ORACLE_CONTRACT;
    // ========== Oracle Native Contract ==========
    
    /**
     * @dev Request oracle data
     */
    function requestOracleData(
        string memory url,
        string memory filter,
        string memory callback,
        bytes memory userData,
        uint256 gasForResponse
    ) internal {
        bytes memory params = abi.encode(url, filter, callback, userData, gasForResponse);
        Syscalls.contractCall(NativeContracts.ORACLE_CONTRACT, "request", params);
    }
    
    /**
     * @dev Get oracle price
     */
    function getOraclePrice() internal view returns (uint256) {
        bytes memory result = Syscalls.contractCall(NativeContracts.ORACLE_CONTRACT, "getPrice", "");
        return abi.decode(result, (uint256));
    }
    
    /**
     * @dev Set oracle price
     */
    function setOraclePrice(uint256 price) internal {
        bytes memory params = abi.encode(price);
        Syscalls.contractCall(NativeContracts.ORACLE_CONTRACT, "setPrice", params);
    }

    /**
     * @dev Finish oracle response (oracle nodes)
     */
    function oracleFinish() internal {
        Syscalls.contractCall(NativeContracts.ORACLE_CONTRACT, "finish", "");
    }

    /**
     * @dev Verify oracle response transaction
     */
    function oracleVerify() internal view returns (bool) {
        bytes memory result = Syscalls.contractCall(NativeContracts.ORACLE_CONTRACT, "verify", "");
        return abi.decode(result, (bool));
    }

    // NOTE: The Oracle native contract exposes exactly five ABI methods on
    // Neo N3 (request, getPrice, setPrice, finish, verify). Earlier versions
    // of this library exposed `getOracleNodes` / `getOracleRequests` /
    // `getOracleRequest`, which do NOT exist on any Neo N3 node (verified
    // against neo-project/neo v3.9.x and master) — calling them faults with
    // "Method not found". They were removed; inspect oracle activity via
    // `System.Runtime.GetNotifications` in the oracle callback instead.

}
