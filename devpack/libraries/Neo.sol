// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title Neo N3 Blockchain Utilities
 * @dev High-level access to Neo N3 blockchain features
 * @author Jimmy <jimmy@r3e.network>
 *
 * `Neo` is a compiler intrinsic: the `neo-devpack-solidity` compiler lowers
 * the members below directly to Neo N3 syscalls and native contract calls.
 * The Solidity bodies in this file exist for editor tooling
 * (signatures/docs) and are never compiled. Only the members listed here are
 * supported; calling anything else fails compilation with a diagnostic.
 *
 * Supported members:
 * - Ledger: getCurrentBlock, getBlockByIndex, getBlockHeight, getBlockTime,
 *   getTransaction, getTransactionHeight, transactionExists
 * - Balances/transfers: getNeoBalance, getGasBalance, transferNeo, transferGas
 * - Crypto: verifyWithWitness, verifySignature, sha256Hash, ripemd160Hash,
 *   getRandom
 * - Contracts: callContract, deployContract
 * - Network/policy: getNetworkMagic, getGasPrice, getStoragePrice
 * - Governance: isCommittee, getCommittee, getValidators, isValidator
 */

import "../contracts/Syscalls.sol";
import "../contracts/NativeCalls.sol";

library Neo {
    using Syscalls for *;
    using NativeCalls for *;

    // ========== Block Information ==========

    /**
     * @dev Get current block index.
     * @notice Lowered to `Ledger.currentIndex` (same as `getBlockHeight`).
     */
    function getCurrentBlock() internal view returns (uint256) {
        return Syscalls.getCurrentIndex();
    }

    /**
     * @dev Get block by index (lowered to `Ledger.getBlock`)
     */
    function getBlockByIndex(uint256 index) internal view returns (Syscalls.Block memory) {
        return Syscalls.getBlock(index);
    }

    /**
     * @dev Get current block height
     */
    function getBlockHeight() internal view returns (uint256) {
        return Syscalls.getCurrentIndex();
    }

    /**
     * @dev Get block timestamp (lowered to `System.Runtime.GetTime`)
     */
    function getBlockTime() internal view returns (uint256) {
        return Syscalls.getTime();
    }

    // ========== Transaction Information ==========

    /**
     * @dev Get transaction information (lowered to `Ledger.getTransaction`)
     */
    function getTransaction(bytes32 txHash) internal view returns (Syscalls.Transaction memory) {
        return Syscalls.getTransaction(txHash);
    }

    /**
     * @dev Get transaction height
     */
    function getTransactionHeight(bytes32 txHash) internal view returns (int256) {
        return Syscalls.getTransactionHeight(txHash);
    }

    /**
     * @dev Check if transaction exists (lowered to `Ledger.getTransactionHeight`;
     *      -1 indicates a missing/unknown transaction on Neo N3)
     *
     * NOTE: Relies on signed integer semantics — Ledger.getTransactionHeight
     * returns int256(-1) for unknown transactions. The explicit comparison
     * against -1 is robust against changes in return encoding behavior.
     */
    function transactionExists(bytes32 txHash) internal view returns (bool) {
        int256 height = Syscalls.getTransactionHeight(txHash);
        return height >= 0;
    }

    // ========== Account and Balance Management ==========

    /**
     * @dev Get NEO balance of account
     */
    function getNeoBalance(address account) internal view returns (uint256) {
        return NativeCalls.neoBalanceOf(account);
    }

    /**
     * @dev Get GAS balance of account
     */
    function getGasBalance(address account) internal view returns (uint256) {
        return NativeCalls.gasBalanceOf(account);
    }

    /**
     * @dev Transfer NEO tokens (lowered to NEO native `transfer` with empty data)
     */
    function transferNeo(address from, address to, uint256 amount) internal returns (bool) {
        return NativeCalls.neoTransfer(from, to, amount, "");
    }

    /**
     * @dev Transfer GAS tokens (lowered to GAS native `transfer` with empty data)
     */
    function transferGas(address from, address to, uint256 amount) internal returns (bool) {
        return NativeCalls.gasTransfer(from, to, amount, "");
    }

    // ========== Cryptographic Operations ==========

    /**
     * @dev Verify witness for account (lowered to `System.Runtime.CheckWitness`)
     */
    function verifyWithWitness(address account) internal view returns (bool) {
        return Syscalls.checkWitness(account);
    }

    /**
     * @dev Verify ECDSA signature (secp256r1, Neo's default curve).
     */
    function verifySignature(
        bytes32 hash,
        bytes memory publicKey,
        bytes memory signature
    ) internal view returns (bool) {
        return Syscalls.verifyWithECDsa(hash, publicKey, signature, 23); // secp256r1
    }

    /**
     * @dev SHA256 hash (lowered to `CryptoLib.sha256`)
     */
    function sha256Hash(bytes memory data) internal view returns (bytes32) {
        return Syscalls.sha256(data);
    }

    /**
     * @dev RIPEMD160 hash (lowered to `CryptoLib.ripemd160`)
     */
    function ripemd160Hash(bytes memory data) internal view returns (bytes20) {
        return Syscalls.ripemd160(data);
    }

    /**
     * @dev Get next pseudo-random number (lowered to `System.Runtime.GetRandom`)
     */
    function getRandom() internal view returns (uint256) {
        return Syscalls.getCurrentRandom();
    }

    // ========== Contract Management ==========

    /**
     * @dev Call another contract (lowered to `System.Contract.Call`)
     */
    function callContract(
        address contractHash,
        string memory method,
        bytes memory params
    ) internal returns (bytes memory) {
        return Syscalls.contractCall(contractHash, method, params);
    }

    /**
     * @dev Deploy new contract (lowered to `ContractManagement.deploy`)
     */
    function deployContract(bytes memory nef, bytes memory manifest) internal returns (address) {
        return NativeCalls.deployContract(nef, manifest);
    }

    // ========== Network Information ==========

    /**
     * @dev Get network magic number (lowered to `System.Runtime.GetNetwork`)
     */
    function getNetworkMagic() internal view returns (uint32) {
        return Syscalls.getNetwork();
    }

    /**
     * @dev Get current network fee per byte (lowered to `Policy.getFeePerByte`)
     *
     * NOTE: This was previously named getGasPrice(), which was misleading
     * because Neo N3's "gas price" is determined by getExecFeeFactor() and
     * getExecPicoFeeFactor(), not the fee-per-byte. The new name accurately
     * reflects that it returns Policy.getFeePerByte().
     */
    function getFeePerByte() internal view returns (uint256) {
        return NativeCalls.getFeePerByte();
    }

    /**
     * @dev Deprecated: use getFeePerByte() instead.
     */
    function getGasPrice() internal view returns (uint256) {
        return getFeePerByte();
    }

    /**
     * @dev Get storage price per byte (lowered to `Policy.getStoragePrice`)
     */
    function getStoragePrice() internal view returns (uint256) {
        return NativeCalls.getStoragePrice();
    }

    // ========== Governance Functions ==========

    /**
     * @dev Check if account is committee member
     */
    function isCommittee(address account) internal view returns (bool) {
        return NativeCalls.isCommittee(account);
    }

    /**
     * @dev Get committee members (public keys)
     */
    function getCommittee() internal view returns (bytes[] memory) {
        return NativeCalls.getCommittee();
    }

    /**
     * @dev Get next block validators
     */
    function getValidators() internal view returns (address[] memory) {
        return NativeCalls.getNextBlockValidators();
    }

    /**
     * @dev Check if account is validator
     */
    function isValidator(address account) internal view returns (bool) {
        return NativeCalls.isValidator(account);
    }
}
