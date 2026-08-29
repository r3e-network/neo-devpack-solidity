// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title Neo N3 Runtime Library
 * @dev Runtime services and utilities for Neo N3 blockchain
 * @author Jimmy <jimmy@r3e.network>
 *
 * `Runtime` is a compiler intrinsic: the `neo-devpack-solidity` compiler
 * lowers the members below directly to Neo N3 syscalls. The Solidity bodies
 * in this file exist for editor tooling (signatures/docs) and are never
 * compiled. Only the members listed here are supported; calling anything
 * else fails compilation with a diagnostic.
 *
 * Supported members:
 * - Notifications: notify, notifyIndexed
 * - Witnesses: checkWitness (address or pubkey), requireWitness,
 *   checkAnyWitness, checkAllWitnesses, checkMultiSigWitness
 * - Gas: gasLeft, burnGas
 * - Context: getTime, getTrigger, getInvocationCounter, getCurrentSigners,
 *   getCallFlags, getScriptContainer, loadScript, getNetwork, getPlatform,
 *   getAddressVersion, getRandom, getExecutingScriptHash,
 *   getCallingScriptHash, getEntryScriptHash
 * - Misc: log, initializeServices
 */

import "../contracts/Syscalls.sol";

library Runtime {
    using Syscalls for *;

    // ========== Event and Notification System ==========

    /**
     * @dev Emit notification (maps to System.Runtime.Notify)
     */
    function notify(string memory eventName, bytes memory data) internal {
        bytes memory notificationData = abi.encode(eventName, data);
        Syscalls.notify(notificationData);
    }

    /**
     * @dev Emit indexed notification with topics
     *      (lowered to notify(eventName, abi.encode(topics, data)))
     */
    function notifyIndexed(
        string memory eventName,
        bytes32[] memory topics,
        bytes memory data
    ) internal {
        bytes memory indexedData = abi.encode(topics, data);
        notify(eventName, indexedData);
    }

    // ========== Witness and Authorization ==========

    /**
     * @dev Check witness for account
     */
    function checkWitness(address account) internal view returns (bool) {
        return Syscalls.checkWitness(account);
    }

    /**
     * @dev Check witness by public key
     */
    function checkWitness(bytes memory publicKey) internal view returns (bool) {
        return Syscalls.checkWitness(publicKey);
    }

    /**
     * @dev Require witness or revert
     */
    function requireWitness(address account) internal view {
        require(checkWitness(account), "Runtime: invalid witness");
    }

    /**
     * @dev Check multiple witnesses (OR logic)
     */
    function checkAnyWitness(address[] memory accounts) internal view returns (bool) {
        for (uint256 i = 0; i < accounts.length; i++) {
            if (checkWitness(accounts[i])) {
                return true;
            }
        }
        return false;
    }

    /**
     * @dev Check multiple witnesses (AND logic)
     */
    function checkAllWitnesses(address[] memory accounts) internal view returns (bool) {
        for (uint256 i = 0; i < accounts.length; i++) {
            if (!checkWitness(accounts[i])) {
                return false;
            }
        }
        return true;
    }

    /**
     * @dev Multi-signature witness verification (threshold of distinct signers)
     */
    function checkMultiSigWitness(
        address[] memory signers,
        uint256 threshold
    ) internal view returns (bool) {
        require(threshold > 0, "Runtime: threshold must be positive");
        require(threshold <= signers.length, "Runtime: threshold exceeds signers");

        uint256 validWitnesses = 0;

        for (uint256 i = 0; i < signers.length; i++) {
            address signer = signers[i];

            // Prevent duplicate signers from satisfying quorum multiple times.
            for (uint256 j = 0; j < i; j++) {
                require(signers[j] != signer, "Runtime: duplicate signer");
            }

            if (checkWitness(signer)) {
                validWitnesses++;
                if (validWitnesses >= threshold) {
                    return true;
                }
            }
        }

        return false;
    }

    // ========== Gas Management ==========

    /**
     * @dev Get remaining gas
     */
    function gasLeft() internal view returns (uint256) {
        return Syscalls.gasLeft();
    }

    /**
     * @dev Burn gas (anti-spam)
     */
    function burnGas(uint256 amount) internal {
        Syscalls.burnGas(amount);
    }

    // ========== Execution Context ==========

    /**
     * @dev Get current block timestamp (System.Runtime.GetTime)
     */
    function getTime() internal view returns (uint256) {
        return Syscalls.getTime();
    }

    /**
     * @dev Get invocation trigger type (System.Runtime.GetTrigger)
     */
    function getTrigger() internal view returns (uint8) {
        return Syscalls.getTrigger();
    }

    /**
     * @dev Get invocation counter for the executing contract
     */
    function getInvocationCounter() internal view returns (uint256) {
        return Syscalls.getInvocationCounter();
    }

    // NOTE: there is intentionally no `hasRole` helper. Neo N3's native
    // RoleManagement contract only exposes getDesignatedByRole(role, index)
    // (the ECPoint node list designated at a block height) and
    // designateAsRole; neither real Neo N3 nor the official C# devpack
    // provides a generic role-membership check. A previous `hasRole`
    // helper silently ignored its `role` argument and fell back to
    // checkWitness(account) — a misleading surface for an access-control
    // primitive (and uncallable anyway: Runtime is a compiler-intrinsic
    // library and `hasRole` had no intrinsic lowering). For node-role
    // checks use NativeCalls.getDesignatedByRole() and scan the returned
    // list; for account authorization use checkWitness/requireWitness.

    /**
     * @dev Get current transaction signers
     */
    function getCurrentSigners() internal view returns (Syscalls.Signer[] memory) {
        return Syscalls.getCurrentSigners();
    }

    /**
     * @dev Get current call flags
     */
    function getCallFlags() internal view returns (uint8) {
        return Syscalls.getCallFlags();
    }

    /**
     * @dev Get current script container (transaction)
     */
    function getScriptContainer() internal view returns (Syscalls.Transaction memory) {
        return Syscalls.getScriptContainer();
    }

    /**
     * @dev Load script with arguments
     */
    function loadScript(bytes memory script, uint8 callFlags, bytes[] memory args) internal {
        Syscalls.loadScript(script, callFlags, args);
    }

    /**
     * @dev Get network magic number
     */
    function getNetwork() internal view returns (uint32) {
        return Syscalls.getNetwork();
    }

    /**
     * @dev Get platform name ("NEO")
     */
    function getPlatform() internal view returns (string memory) {
        return Syscalls.getPlatform();
    }

    /**
     * @dev Get address version byte
     */
    function getAddressVersion() internal view returns (uint8) {
        return Syscalls.getAddressVersion();
    }

    /**
     * @dev Get next pseudo-random number (System.Runtime.GetRandom)
     */
    function getRandom() internal view returns (uint256) {
        return Syscalls.getCurrentRandom();
    }

    /**
     * @dev Get executing contract script hash
     */
    function getExecutingScriptHash() internal view returns (address) {
        return Syscalls.getExecutingScriptHash();
    }

    /**
     * @dev Get calling contract script hash
     */
    function getCallingScriptHash() internal view returns (address) {
        return Syscalls.getCallingScriptHash();
    }

    /**
     * @dev Get entry script hash
     */
    function getEntryScriptHash() internal view returns (address) {
        return Syscalls.getEntryScriptHash();
    }

    // ========== Logging and Debugging ==========

    /**
     * @dev Log message to Neo logs (System.Runtime.Log)
     */
    function log(string memory message) internal {
        Syscalls.log(message);
    }

    /**
     * @dev Devpack compatibility initializer. The compiler lowers this to a
     *      no-op (Neo N3 runtime services need no explicit initialization).
     */
    function initializeServices() internal pure returns (bool) {
        return true;
    }
}
