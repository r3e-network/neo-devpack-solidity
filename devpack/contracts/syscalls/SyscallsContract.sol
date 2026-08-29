// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./SyscallsTypes.sol";
import "./SyscallsBase.sol";

/**
 * @title Syscalls Contract — Neo N3 Contract operations
 */

library SyscallsContract {
    // ========== Contract System Calls ==========
    
    /**
     * @dev Call another contract
     */
    function SyscallsBase.contractCall(
        address scriptHash,
        string memory method,
        bytes memory params
    ) internal returns (bytes memory) {
        bytes memory data = abi.encode(scriptHash, method, params);
        return SyscallsBase._syscallBytes("System.Contract.Call", data);
    }
    
    /**
     * @dev Call contract with flags
     * @notice Neo N3 does not have System.Contract.CallEx. The flags parameter is
     *         accepted for API compatibility but currently has no effect.
     * @param scriptHash The target contract script hash
     * @param method The method name to call
     * @param params The encoded parameters
     * @param flags Call flags (currently ignored - reserved for future use)
     * @return The result of the contract call
     */
    function SyscallsBase.contractCallWithFlags(
        address scriptHash,
        string memory method,
        bytes memory params,
        uint8 flags
    ) internal returns (bytes memory) {
        // Flags parameter is reserved for future use when Neo N3 adds CallEx support
        // Currently ignored - passed in data for forward compatibility
        bytes memory data = abi.encode(scriptHash, method, params, flags);
        return SyscallsBase._syscallBytes("System.Contract.Call", data);
    }

    /**
     * @dev Get current call flags
     */
    function getCallFlags() internal view returns (uint8) {
        return uint8(SyscallsBase._syscall("System.Contract.GetCallFlags", ""));
    }

    /**
     * @dev Create a standard signature account script hash from an ECPoint public key.
     *
     * Syscall: System.Contract.CreateStandardAccount(pubkey: ByteString) -> UInt160
     */
    function createStandardAccount(bytes memory publicKey) internal view returns (address) {
        bytes memory data = abi.encode(publicKey);
        bytes memory result = SyscallsBase._syscallBytes("System.Contract.CreateStandardAccount", data);
        return abi.decode(result, (address));
    }

    /**
     * @dev Create a multisig account script hash from ECPoint public keys.
     *
     * Syscall: System.Contract.CreateMultisigAccount(m: int, pubkeys: Array) -> UInt160
     */
    function createMultisigAccount(uint256 m, bytes[] memory publicKeys) internal view returns (address) {
        bytes memory data = abi.encode(m, publicKeys);
        bytes memory result = SyscallsBase._syscallBytes("System.Contract.CreateMultisigAccount", data);
        return abi.decode(result, (address));
    }
    
    /**
     * @dev Create new contract
     */
    function contractCreate(bytes memory nef, bytes memory manifest) internal returns (address) {
        bytes memory data = abi.encode(nef, manifest);
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.CONTRACT_MANAGEMENT, "deploy", data);
        ContractStateNative memory state = abi.decode(result, (ContractStateNative));
        return state.hash;
    }

    /**
     * @dev Create new contract and pass deployment data to `_deploy(data, false)`
     */
    function contractCreate(bytes memory nef, bytes memory manifest, bytes memory deployData)
        internal
        returns (address)
    {
        bytes memory data = abi.encode(nef, manifest, deployData);
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.CONTRACT_MANAGEMENT, "deploy", data);
        ContractStateNative memory state = abi.decode(result, (ContractStateNative));
        return state.hash;
    }
    
    /**
     * @dev Update contract
     */
    function contractUpdate(bytes memory nef, bytes memory manifest) internal {
        bytes memory data = abi.encode(nef, manifest);
        SyscallsBase.contractCall(SyscallsBase.CONTRACT_MANAGEMENT, "update", data);
    }

    /**
     * @dev Update contract and pass update data to `_deploy(data, true)`
     */
    function contractUpdate(bytes memory nef, bytes memory manifest, bytes memory updateData) internal {
        bytes memory data = abi.encode(nef, manifest, updateData);
        SyscallsBase.contractCall(SyscallsBase.CONTRACT_MANAGEMENT, "update", data);
    }
    
    /**
     * @dev Destroy contract
     */
    function contractDestroy() internal {
        SyscallsBase.contractCall(SyscallsBase.CONTRACT_MANAGEMENT, "destroy", "");
    }
    
    /**
     * @dev Get executing script hash
     */
    function getExecutingScriptHash() internal view returns (address) {
        bytes memory result = SyscallsBase._syscallBytes("System.Runtime.GetExecutingScriptHash", "");
        return abi.decode(result, (address));
    }
    
    /**
     * @dev Get calling script hash
     */
    function getCallingScriptHash() internal view returns (address) {
        bytes memory result = SyscallsBase._syscallBytes("System.Runtime.GetCallingScriptHash", "");
        return abi.decode(result, (address));
    }
    
    /**
     * @dev Get entry script hash
     */
    function getEntryScriptHash() internal view returns (address) {
        bytes memory result = SyscallsBase._syscallBytes("System.Runtime.GetEntryScriptHash", "");
        return abi.decode(result, (address));
    }

    /**
     * @dev Get script container
     */
    function getScriptContainer() internal view returns (Transaction memory) {
        bytes memory result = SyscallsBase._syscallBytes("System.Runtime.GetScriptContainer", "");
        return abi.decode(result, (Transaction));
    }

    /**
     * @dev Load script with arguments
     */
    function loadScript(bytes memory script, uint8 callFlags, bytes[] memory args) internal {
        bytes memory data = abi.encode(script, callFlags, args);
        SyscallsBase._syscallVoid("System.Runtime.LoadScript", data);
    }
    
}
