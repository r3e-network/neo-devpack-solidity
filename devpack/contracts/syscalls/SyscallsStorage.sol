// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./SyscallsTypes.sol";
import "./SyscallsBase.sol";

/**
 * @title Syscalls Storage — Neo N3 Storage operations
 */

library SyscallsStorage {
    // ========== Storage System Calls ==========
    
    /**
     * @dev Get storage context
     */
    function getStorageContext() internal view returns (StorageContext memory) {
        bytes memory result = SyscallsBase._syscallBytes("System.Storage.GetContext", "");
        return abi.decode(result, (StorageContext));
    }
    
    /**
     * @dev Get read-only storage context
     */
    function getReadOnlyStorageContext() internal view returns (StorageContext memory) {
        bytes memory result = SyscallsBase._syscallBytes("System.Storage.GetReadOnlyContext", "");
        return abi.decode(result, (StorageContext));
    }

    /**
     * @dev Convert storage context to read-only
     */
    function storageAsReadOnly(StorageContext memory context) internal view returns (StorageContext memory) {
        bytes memory data = abi.encode(context);
        bytes memory result = SyscallsBase._syscallBytes("System.Storage.AsReadOnly", data);
        return abi.decode(result, (StorageContext));
    }
    
    /**
     * @dev Storage get
     */
    function storageGet(StorageContext memory context, bytes memory key) 
        internal 
        view 
        returns (bytes memory) 
    {
        bytes memory data = abi.encode(context, key);
        return SyscallsBase._syscallBytes("System.Storage.Get", data);
    }
    
    /**
     * @dev Storage put
     */
    function storagePut(StorageContext memory context, bytes memory key, bytes memory value) internal {
        bytes memory data = abi.encode(context, key, value);
        SyscallsBase._syscallVoid("System.Storage.Put", data);
    }
    
    /**
     * @dev Storage delete
     */
    function storageDelete(StorageContext memory context, bytes memory key) internal {
        bytes memory data = abi.encode(context, key);
        SyscallsBase._syscallVoid("System.Storage.Delete", data);
    }
    
    /**
     * @dev Storage find
     */
    function storageFind(StorageContext memory context, bytes memory prefix) 
        internal 
        view 
        returns (Iterator memory) 
    {
        // Neo N3 signature: Storage.Find(context, prefix, options)
        bytes memory data = abi.encode(context, prefix, uint8(0));
        bytes memory result = SyscallsBase._syscallBytes("System.Storage.Find", data);
        return abi.decode(result, (Iterator));
    }

    /**
     * @dev Storage find with options
     */
    function storageFind(
        StorageContext memory context,
        bytes memory prefix,
        uint8 options
    ) internal view returns (Iterator memory) {
        bytes memory data = abi.encode(context, prefix, options);
        bytes memory result = SyscallsBase._syscallBytes("System.Storage.Find", data);
        return abi.decode(result, (Iterator));
    }

    // Historical: `storageGetLocal`/`storagePutLocal` wrappers were removed (v0.27).
    // They targeted nonexistent Neo N3 syscalls. Use context-based `storage*` helpers.
    // See CHANGELOG for migration guidance.

}
