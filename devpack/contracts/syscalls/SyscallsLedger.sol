// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./SyscallsTypes.sol";
import "./SyscallsBase.sol";

/**
 * @title Syscalls Ledger — Neo N3 Ledger operations
 */

library SyscallsLedger {
    // ========== Blockchain System Calls ==========
    
    /**
     * @dev Get current block index
     */
    function getCurrentIndex() internal view returns (uint256) {
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.LEDGER_CONTRACT, "currentIndex", "");
        return abi.decode(result, (uint256));
    }

    /**
     * @dev Get current block hash
     */
    function getCurrentHash() internal view returns (bytes32) {
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.LEDGER_CONTRACT, "currentHash", "");
        return abi.decode(result, (bytes32));
    }
    
    /**
     * @dev Get block by index
     */
    function getBlock(uint256 index) internal view returns (Block memory) {
        bytes memory data = abi.encode(index);
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.LEDGER_CONTRACT, "getBlock", data);
        return abi.decode(result, (Block));
    }

    /**
     * @dev Get block by hash
     */
    function getBlock(bytes32 hash) internal view returns (Block memory) {
        bytes memory data = abi.encode(hash);
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.LEDGER_CONTRACT, "getBlock", data);
        return abi.decode(result, (Block));
    }
    
    /**
     * @dev Get transaction by hash
     */
    function getTransaction(bytes32 hash) internal view returns (Transaction memory) {
        bytes memory data = abi.encode(hash);
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.LEDGER_CONTRACT, "getTransaction", data);
        return abi.decode(result, (Transaction));
    }
    
    /**
     * @dev Get transaction height
     */
    function getTransactionHeight(bytes32 hash) internal view returns (int256) {
        bytes memory data = abi.encode(hash);
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.LEDGER_CONTRACT, "getTransactionHeight", data);
        return abi.decode(result, (int256));
    }
    
    /**
     * @dev Get transaction from block
     */
    function getTransactionFromBlock(uint256 blockIndex, uint256 txIndex) 
        internal 
        view 
        returns (Transaction memory) 
    {
        bytes memory data = abi.encode(blockIndex, txIndex);
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.LEDGER_CONTRACT, "getTransactionFromBlock", data);
        return abi.decode(result, (Transaction));
    }

    /**
     * @dev Get transaction from block by hash
     */
    function getTransactionFromBlock(bytes32 blockHash, uint256 txIndex)
        internal
        view
        returns (Transaction memory)
    {
        bytes memory data = abi.encode(blockHash, txIndex);
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.LEDGER_CONTRACT, "getTransactionFromBlock", data);
        return abi.decode(result, (Transaction));
    }

    /**
     * @dev Get transaction signers
     */
    function getTransactionSigners(bytes32 hash) internal view returns (Signer[] memory) {
        bytes memory data = abi.encode(hash);
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.LEDGER_CONTRACT, "getTransactionSigners", data);
        return abi.decode(result, (Signer[]));
    }

    /**
     * @dev Get transaction VM state
     */
    function getTransactionVMState(bytes32 hash) internal view returns (uint8) {
        bytes memory data = abi.encode(hash);
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.LEDGER_CONTRACT, "getTransactionVMState", data);
        return abi.decode(result, (uint8));
    }
    
}
