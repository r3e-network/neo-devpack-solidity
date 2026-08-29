// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "../NativeContracts.sol";
import "../Syscalls.sol";

/**
 * @title Native GAS — Neo N3 GAS native contract operations
 */

library NativeGAS {
    address constant GAS_CONTRACT = NativeContracts.GAS_CONTRACT;
    // ========== GAS Token Native Contract ==========
    
    /**
     * @dev Get GAS total supply
     */
    function gasTotalSupply() internal view returns (uint256) {
        bytes memory result = Syscalls.contractCall(NativeContracts.GAS_CONTRACT, "totalSupply", "");
        return abi.decode(result, (uint256));
    }
    
    /**
     * @dev Get GAS balance of account
     */
    function gasBalanceOf(address account) internal view returns (uint256) {
        bytes memory params = abi.encode(account);
        bytes memory result = Syscalls.contractCall(NativeContracts.GAS_CONTRACT, "balanceOf", params);
        return abi.decode(result, (uint256));
    }
    
    /**
     * @dev Transfer GAS tokens
     */
    function gasTransfer(address from, address to, uint256 amount, bytes memory data) 
        internal 
        returns (bool) 
    {
        bytes memory params = abi.encode(from, to, amount, data);
        bytes memory result = Syscalls.contractCall(NativeContracts.GAS_CONTRACT, "transfer", params);
        return abi.decode(result, (bool));
    }
    
    /**
     * @dev Get GAS decimals
     */
    function gasDecimals() internal view returns (uint8) {
        bytes memory result = Syscalls.contractCall(NativeContracts.GAS_CONTRACT, "decimals", "");
        return abi.decode(result, (uint8));
    }
    
    /**
     * @dev Get GAS symbol
     */
    function gasSymbol() internal view returns (string memory) {
        bytes memory result = Syscalls.contractCall(NativeContracts.GAS_CONTRACT, "symbol", "");
        return abi.decode(result, (string));
    }
    
}
