// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./SyscallsTypes.sol";
import "./SyscallsBase.sol";

/**
 * @title Syscalls Runtime — Neo N3 Runtime operations
 */

library SyscallsRuntime {
    // ========== Runtime System Calls ==========
    
    /**
     * @dev Check witness
     */
    function checkWitness(address hash) internal view returns (bool) {
        bytes memory data = abi.encode(hash);
        return SyscallsBase._syscall("System.Runtime.CheckWitness", data) != 0;
    }

    /**
     * @dev Check witness (public key)
     */
    function checkWitness(bytes memory publicKey) internal view returns (bool) {
        bytes memory data = abi.encode(publicKey);
        return SyscallsBase._syscall("System.Runtime.CheckWitness", data) != 0;
    }
    
    /**
     * @dev Get time (block timestamp)
     */
    function getTime() internal view returns (uint256) {
        return SyscallsBase._syscall("System.Runtime.GetTime", "");
    }
    
    /**
     * @dev Get gas left
     */
    function gasLeft() internal view returns (uint256) {
        return SyscallsBase._syscall("System.Runtime.GasLeft", "");
    }
    
    /**
     * @dev Get platform information
     */
    function getPlatform() internal view returns (string memory) {
        bytes memory result = SyscallsBase._syscallBytes("System.Runtime.Platform", "");
        return abi.decode(result, (string));
    }
    
    /**
     * @dev Get trigger type
     */
    function getTrigger() internal view returns (uint8) {
        return uint8(SyscallsBase._syscall("System.Runtime.GetTrigger", ""));
    }
    
    /**
     * @dev Emit notification
     */
    function notify(bytes memory data) internal {
        bytes memory params = abi.encode(data);
        SyscallsBase._syscallVoid("System.Runtime.Notify", params);
    }
    
    /**
     * @dev Get notifications
     */
    function getNotifications(address hash) internal view returns (Notification[] memory) {
        bytes memory data = abi.encode(hash);
        bytes memory result = SyscallsBase._syscallBytes("System.Runtime.GetNotifications", data);
        return abi.decode(result, (Notification[]));
    }

    /**
     * @dev Get all notifications
     */
    function getNotifications() internal view returns (Notification[] memory) {
        bytes memory result = SyscallsBase._syscallBytes("System.Runtime.GetNotifications", "");
        return abi.decode(result, (Notification[]));
    }
    
    /**
     * @dev Log message
     */
    function log(string memory message) internal {
        bytes memory data = abi.encode(message);
        SyscallsBase._syscallVoid("System.Runtime.Log", data);
    }

    /**
     * @dev Get current transaction signers
     */
    function getCurrentSigners() internal view returns (Signer[] memory) {
        bytes memory result = SyscallsBase._syscallBytes("System.Runtime.CurrentSigners", "");
        return abi.decode(result, (Signer[]));
    }

    /**
     * @dev Get network magic number
     */
    function getNetwork() internal view returns (uint256) {
        return SyscallsBase._syscall("System.Runtime.GetNetwork", "");
    }

    /**
     * @dev Get address version (Neo N3 address version byte)
     */
    function getAddressVersion() internal view returns (uint8) {
        return uint8(SyscallsBase._syscall("System.Runtime.GetAddressVersion", ""));
    }

    /**
     * @dev Get invocation counter (number of times current contract has been invoked)
     */
    function getInvocationCounter() internal view returns (uint256) {
        return SyscallsBase._syscall("System.Runtime.GetInvocationCounter", "");
    }

    /**
     * @dev Get random number (derived from block context)
     */
    function getRandom() internal view returns (uint256) {
        return SyscallsBase._syscall("System.Runtime.GetRandom", "");
    }

    /**
     * @dev Burn GAS (consumes GAS from the current execution context)
     */
    function burnGas(uint256 amount) internal {
        bytes memory data = abi.encode(amount);
        SyscallsBase._syscallVoid("System.Runtime.BurnGas", data);
    }

    /**
     * @dev Get message value (msg.value equivalent — the value attached to the current invocation)
     */
    function getMsgValue() internal view returns (uint256) {
        return SyscallsBase._syscall("System.Runtime.GetMsgValue", "");
    }
    
}
