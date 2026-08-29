// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "../NativeContracts.sol";
import "../Syscalls.sol";

/**
 * @title Native Ledger — Neo N3 Ledger native contract operations
 */

library NativeLedger {
    address constant LEDGER_CONTRACT = NativeContracts.LEDGER_CONTRACT;
    // ========== Ledger Native Contract ==========

    /**
     * @dev Get current block index
     */
    function currentIndex() internal view returns (uint256) {
        bytes memory result = Syscalls.contractCall(NativeContracts.LEDGER_CONTRACT, "currentIndex", "");
        return abi.decode(result, (uint256));
    }

    /**
     * @dev Get current block hash
     */
    function currentHash() internal view returns (bytes32) {
        bytes memory result = Syscalls.contractCall(NativeContracts.LEDGER_CONTRACT, "currentHash", "");
        return abi.decode(result, (bytes32));
    }

    /**
     * @dev Get block by index
     */
    function getBlock(uint256 index) internal view returns (Syscalls.Block memory) {
        bytes memory params = abi.encode(index);
        bytes memory result = Syscalls.contractCall(NativeContracts.LEDGER_CONTRACT, "getBlock", params);
        return abi.decode(result, (Syscalls.Block));
    }

    /**
     * @dev Get block by hash
     */
    function getBlock(bytes32 hash) internal view returns (Syscalls.Block memory) {
        bytes memory params = abi.encode(hash);
        bytes memory result = Syscalls.contractCall(NativeContracts.LEDGER_CONTRACT, "getBlock", params);
        return abi.decode(result, (Syscalls.Block));
    }

    /**
     * @dev Get transaction by hash
     */
    function getTransaction(bytes32 hash) internal view returns (Syscalls.Transaction memory) {
        bytes memory params = abi.encode(hash);
        bytes memory result = Syscalls.contractCall(NativeContracts.LEDGER_CONTRACT, "getTransaction", params);
        return abi.decode(result, (Syscalls.Transaction));
    }

    /**
     * @dev Get transaction height
     */
    function getTransactionHeight(bytes32 hash) internal view returns (int256) {
        bytes memory params = abi.encode(hash);
        bytes memory result = Syscalls.contractCall(NativeContracts.LEDGER_CONTRACT, "getTransactionHeight", params);
        return abi.decode(result, (int256));
    }

    /**
     * @dev Get transaction from block
     */
    function getTransactionFromBlock(uint256 blockIndex, uint256 txIndex)
        internal
        view
        returns (Syscalls.Transaction memory)
    {
        bytes memory params = abi.encode(blockIndex, txIndex);
        bytes memory result = Syscalls.contractCall(NativeContracts.LEDGER_CONTRACT, "getTransactionFromBlock", params);
        return abi.decode(result, (Syscalls.Transaction));
    }

    /**
     * @dev Get transaction from block by hash
     */
    function getTransactionFromBlock(bytes32 blockHash, uint256 txIndex)
        internal
        view
        returns (Syscalls.Transaction memory)
    {
        bytes memory params = abi.encode(blockHash, txIndex);
        bytes memory result = Syscalls.contractCall(NativeContracts.LEDGER_CONTRACT, "getTransactionFromBlock", params);
        return abi.decode(result, (Syscalls.Transaction));
    }

    /**
     * @dev Get transaction signers
     */
    function getTransactionSigners(bytes32 hash) internal view returns (Syscalls.Signer[] memory) {
        bytes memory params = abi.encode(hash);
        bytes memory result = Syscalls.contractCall(NativeContracts.LEDGER_CONTRACT, "getTransactionSigners", params);
        return abi.decode(result, (Syscalls.Signer[]));
    }

    /**
     * @dev Get transaction VM state
     */
    function getTransactionVMState(bytes32 hash) internal view returns (uint8) {
        bytes memory params = abi.encode(hash);
        bytes memory result = Syscalls.contractCall(NativeContracts.LEDGER_CONTRACT, "getTransactionVMState", params);
        return abi.decode(result, (uint8));
    }

}
