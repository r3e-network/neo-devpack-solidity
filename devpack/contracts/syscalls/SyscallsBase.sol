// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "../NativeContracts.sol";

/**
 * @title Syscalls Base — Shared helpers for domain syscall libraries
 * @dev Internal helpers used by all Syscalls* domain libraries.
 *      Contract hash constants imported from NativeContracts.sol (single source of truth).
 */
library SyscallsBase {
    // Native contract script hashes — imported from NativeContracts.sol
    address constant CONTRACT_MANAGEMENT = NativeContracts.CONTRACT_MANAGEMENT;
    address constant POLICY_CONTRACT = NativeContracts.POLICY_CONTRACT;
    address constant ORACLE_CONTRACT = NativeContracts.ORACLE_CONTRACT;
    address constant ROLE_MANAGEMENT = NativeContracts.ROLE_MANAGEMENT;
    address constant LEDGER_CONTRACT = NativeContracts.LEDGER_CONTRACT;
    address constant CRYPTO_LIB = NativeContracts.CRYPTO_LIB;
    address constant STD_LIB = NativeContracts.STD_LIB;

    // ========== Core Syscall Helpers ==========

    function contractCall(
        address scriptHash,
        string memory method,
        bytes memory params
    ) internal returns (bytes memory) {
        bytes memory data = abi.encode(scriptHash, method, params);
        return _syscallBytes("System.Contract.Call", data);
    }

    function contractCallWithFlags(
        address scriptHash,
        string memory method,
        bytes memory params,
        uint8 flags
    ) internal returns (bytes memory) {
        bytes memory data = abi.encode(scriptHash, method, params, flags);
        return _syscallBytes("System.Contract.Call", data);
    }

    function _syscall(string memory method, bytes memory params) internal view returns (uint256) {
        bytes memory callData = abi.encodeWithSignature("neoSyscall(string,bytes)", method, params);
        (bool success, bytes memory result) = address(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF).staticcall(callData);
        if (!success || result.length < 32) {
            return _handleSyscallFallback(method, params);
        }
        return abi.decode(result, (uint256));
    }

    function _handleSyscallFallback(string memory method, bytes memory params) internal view returns (uint256) {
        bytes32 methodHash = keccak256(bytes(method));
        if (methodHash == keccak256("System.Blockchain.GetHeight")) {
            return block.number;
        } else if (methodHash == keccak256("System.Runtime.GetTime")) {
            return block.timestamp;
        } else if (methodHash == keccak256("System.Runtime.GasLeft")) {
            return 0;
        } else if (methodHash == keccak256("System.Runtime.CheckWitness")) {
            address account = abi.decode(params, (address));
            return account == msg.sender ? 1 : 0;
        }
        return 0;
    }

    function _syscallBytes(string memory method, bytes memory params) internal view returns (bytes memory) {
        bytes memory callData = abi.encodeWithSignature("neoSyscallBytes(string,bytes)", method, params);
        (bool success, bytes memory result) = address(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF).staticcall(callData);
        if (!success) {
            return _handleBytesSyscallFallback(method, params);
        }
        return result;
    }

    function _handleBytesSyscallFallback(string memory method, bytes memory params) internal view returns (bytes memory) {
        bytes32 methodHash = keccak256(bytes(method));
        if (methodHash == keccak256("System.Runtime.GetExecutingScriptHash")) {
            return abi.encode(address(this));
        } else if (methodHash == keccak256("System.Runtime.GetCallingScriptHash")) {
            return abi.encode(msg.sender);
        }
        return "";
    }

    function _syscallVoid(string memory method, bytes memory params) internal {
        bytes memory callData = abi.encodeWithSignature("neoSyscallVoid(string,bytes)", method, params);
        (bool success, ) = address(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF).call(callData);
        if (!success) {
            _handleVoidSyscallFallback(method, params);
        }
    }

    function _handleVoidSyscallFallback(string memory method, bytes memory params) internal pure {
        bytes32 methodHash = keccak256(bytes(method));
        if (methodHash == keccak256("System.Storage.Put")) {
            return;
        } else if (methodHash == keccak256("System.Storage.Delete")) {
            return;
        } else if (methodHash == keccak256("System.Runtime.Notify")) {
            return;
        }
    }
}
