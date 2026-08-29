// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "../NativeContracts.sol";
import "../Syscalls.sol";

/**
 * @title Native Treasury — Neo N3 Treasury native contract operations
 */

library NativeTreasury {
    address constant TREASURY_CONTRACT = NativeContracts.TREASURY_CONTRACT;
    // ========== Treasury Native Contract ==========

    /**
     * @dev Verify treasury transaction
     */
    function treasuryVerify() internal view returns (bool) {
        bytes memory result = Syscalls.contractCall(NativeContracts.TREASURY_CONTRACT, "verify", "");
        return abi.decode(result, (bool));
    }

    /**
     * @dev Handle NEP-17 payment
     */
    function treasuryOnNEP17Payment(address from, uint256 amount, bytes memory data) internal {
        bytes memory params = abi.encode(from, amount, data);
        Syscalls.contractCall(NativeContracts.TREASURY_CONTRACT, "onNEP17Payment", params);
    }

    /**
     * @dev Handle NEP-11 payment
     */
    function treasuryOnNEP11Payment(
        address from,
        uint256 amount,
        bytes memory tokenId,
        bytes memory data
    ) internal {
        bytes memory params = abi.encode(from, amount, tokenId, data);
        Syscalls.contractCall(NativeContracts.TREASURY_CONTRACT, "onNEP11Payment", params);
    }

}
