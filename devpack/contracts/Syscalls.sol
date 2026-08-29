// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./NativeContracts.sol";

/**
 * @title Neo N3 Syscall and Native-call Helpers
 * @dev Provides access to N3 syscalls and native contract wrappers supported by the embedded runtime.
 * @author Jimmy <jimmy@r3e.network>
 *
 * This library provides Solidity helpers for the syscall and native contract surfaces that
 * neo-devpack-solidity lowers today. The embedded runtime does not implement every Neo N3 host syscall.
 *
 * NOTE: Neo N3 exposes many features via *native contracts* (Ledger, Policy,
 * Oracle, ContractManagement, etc.) rather than syscalls. The neo-devpack-solidity
 * compiler lowers these helpers to `System.Contract.Call` as needed.
 *
 * DOMAIN-SPECIFIC LIBRARIES: For better code organization, this library has been
 * split into focused domain modules under `syscalls/`:
 *
 *   syscalls/SyscallsLedger.sol    — Blockchain & Ledger operations
 *   syscalls/SyscallsContract.sol  — Contract management & script hashes
 *   syscalls/SyscallsStorage.sol   — Storage operations
 *   syscalls/SyscallsRuntime.sol   — Runtime environment queries
 *   syscalls/SyscallsCrypto.sol    — Cryptographic operations
 *   syscalls/SyscallsStdLib.sol    — StdLib, encoding, memory utilities
 *   syscalls/SyscallsPolicy.sol    — Network policy parameters
 *   syscalls/SyscallsOracle.sol    — Oracle operations
 *   syscalls/SyscallsRole.sol      — Role management
 *   syscalls/SyscallsBase.sol      — Shared internal helpers & constants
 *   syscalls/SyscallsTypes.sol     — Data structure definitions
 *
 * Import domain libraries directly for finer-grained dependencies:
 *   import "./syscalls/SyscallsLedger.sol";
 */

library Syscalls {

    // Native contract script hashes — imported from NativeContracts.sol (single source of truth).
    // These aliases exist for backward compatibility. New code should use
    // `NativeContracts.<NAME>` directly.
    address constant CONTRACT_MANAGEMENT = NativeContracts.CONTRACT_MANAGEMENT;
    address constant POLICY_CONTRACT = NativeContracts.POLICY_CONTRACT;
    address constant ORACLE_CONTRACT = NativeContracts.ORACLE_CONTRACT;
    address constant ROLE_MANAGEMENT = NativeContracts.ROLE_MANAGEMENT;
    address constant LEDGER_CONTRACT = NativeContracts.LEDGER_CONTRACT;
    address constant CRYPTO_LIB = NativeContracts.CRYPTO_LIB;
    address constant STD_LIB = NativeContracts.STD_LIB;
    
    // ========== Blockchain System Calls ==========
    
    /**
     * @dev Get current block index
     */
    function getCurrentIndex() internal view returns (uint256) {
        bytes memory result = contractCall(LEDGER_CONTRACT, "currentIndex", "");
        return abi.decode(result, (uint256));
    }

    /**
     * @dev Get current block hash
     */
    function getCurrentHash() internal view returns (bytes32) {
        bytes memory result = contractCall(LEDGER_CONTRACT, "currentHash", "");
        return abi.decode(result, (bytes32));
    }
    
    /**
     * @dev Get block by index
     */
    function getBlock(uint256 index) internal view returns (Block memory) {
        bytes memory data = abi.encode(index);
        bytes memory result = contractCall(LEDGER_CONTRACT, "getBlock", data);
        return abi.decode(result, (Block));
    }

    /**
     * @dev Get block by hash
     */
    function getBlock(bytes32 hash) internal view returns (Block memory) {
        bytes memory data = abi.encode(hash);
        bytes memory result = contractCall(LEDGER_CONTRACT, "getBlock", data);
        return abi.decode(result, (Block));
    }
    
    /**
     * @dev Get transaction by hash
     */
    function getTransaction(bytes32 hash) internal view returns (Transaction memory) {
        bytes memory data = abi.encode(hash);
        bytes memory result = contractCall(LEDGER_CONTRACT, "getTransaction", data);
        return abi.decode(result, (Transaction));
    }
    
    /**
     * @dev Get transaction height
     */
    function getTransactionHeight(bytes32 hash) internal view returns (int256) {
        bytes memory data = abi.encode(hash);
        bytes memory result = contractCall(LEDGER_CONTRACT, "getTransactionHeight", data);
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
        bytes memory result = contractCall(LEDGER_CONTRACT, "getTransactionFromBlock", data);
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
        bytes memory result = contractCall(LEDGER_CONTRACT, "getTransactionFromBlock", data);
        return abi.decode(result, (Transaction));
    }

    /**
     * @dev Get transaction signers
     */
    function getTransactionSigners(bytes32 hash) internal view returns (Signer[] memory) {
        bytes memory data = abi.encode(hash);
        bytes memory result = contractCall(LEDGER_CONTRACT, "getTransactionSigners", data);
        return abi.decode(result, (Signer[]));
    }

    /**
     * @dev Get transaction VM state
     */
    function getTransactionVMState(bytes32 hash) internal view returns (uint8) {
        bytes memory data = abi.encode(hash);
        bytes memory result = contractCall(LEDGER_CONTRACT, "getTransactionVMState", data);
        return abi.decode(result, (uint8));
    }
    
    // ========== Contract System Calls ==========
    
    /**
     * @dev Call another contract
     */
    function contractCall(
        address scriptHash,
        string memory method,
        bytes memory params
    ) internal returns (bytes memory) {
        bytes memory data = abi.encode(scriptHash, method, params);
        return _syscallBytes("System.Contract.Call", data);
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
    function contractCallWithFlags(
        address scriptHash,
        string memory method,
        bytes memory params,
        uint8 flags
    ) internal returns (bytes memory) {
        // Flags parameter is reserved for future use when Neo N3 adds CallEx support
        // Currently ignored - passed in data for forward compatibility
        bytes memory data = abi.encode(scriptHash, method, params, flags);
        return _syscallBytes("System.Contract.Call", data);
    }

    /**
     * @dev Get current call flags
     */
    function getCallFlags() internal view returns (uint8) {
        return uint8(_syscall("System.Contract.GetCallFlags", ""));
    }

    /**
     * @dev Create a standard signature account script hash from an ECPoint public key.
     *
     * Syscall: System.Contract.CreateStandardAccount(pubkey: ByteString) -> UInt160
     */
    function createStandardAccount(bytes memory publicKey) internal view returns (address) {
        bytes memory data = abi.encode(publicKey);
        bytes memory result = _syscallBytes("System.Contract.CreateStandardAccount", data);
        return abi.decode(result, (address));
    }

    /**
     * @dev Create a multisig account script hash from ECPoint public keys.
     *
     * Syscall: System.Contract.CreateMultisigAccount(m: int, pubkeys: Array) -> UInt160
     */
    function createMultisigAccount(uint256 m, bytes[] memory publicKeys) internal view returns (address) {
        bytes memory data = abi.encode(m, publicKeys);
        bytes memory result = _syscallBytes("System.Contract.CreateMultisigAccount", data);
        return abi.decode(result, (address));
    }
    
    /**
     * @dev Create new contract
     */
    function contractCreate(bytes memory nef, bytes memory manifest) internal returns (address) {
        bytes memory data = abi.encode(nef, manifest);
        bytes memory result = contractCall(CONTRACT_MANAGEMENT, "deploy", data);
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
        bytes memory result = contractCall(CONTRACT_MANAGEMENT, "deploy", data);
        ContractStateNative memory state = abi.decode(result, (ContractStateNative));
        return state.hash;
    }
    
    /**
     * @dev Update contract
     */
    function contractUpdate(bytes memory nef, bytes memory manifest) internal {
        bytes memory data = abi.encode(nef, manifest);
        contractCall(CONTRACT_MANAGEMENT, "update", data);
    }

    /**
     * @dev Update contract and pass update data to `_deploy(data, true)`
     */
    function contractUpdate(bytes memory nef, bytes memory manifest, bytes memory updateData) internal {
        bytes memory data = abi.encode(nef, manifest, updateData);
        contractCall(CONTRACT_MANAGEMENT, "update", data);
    }
    
    /**
     * @dev Destroy contract
     */
    function contractDestroy() internal {
        contractCall(CONTRACT_MANAGEMENT, "destroy", "");
    }
    
    /**
     * @dev Get executing script hash
     */
    function getExecutingScriptHash() internal view returns (address) {
        bytes memory result = _syscallBytes("System.Runtime.GetExecutingScriptHash", "");
        return abi.decode(result, (address));
    }
    
    /**
     * @dev Get calling script hash
     */
    function getCallingScriptHash() internal view returns (address) {
        bytes memory result = _syscallBytes("System.Runtime.GetCallingScriptHash", "");
        return abi.decode(result, (address));
    }
    
    /**
     * @dev Get entry script hash
     */
    function getEntryScriptHash() internal view returns (address) {
        bytes memory result = _syscallBytes("System.Runtime.GetEntryScriptHash", "");
        return abi.decode(result, (address));
    }

    /**
     * @dev Get script container
     */
    function getScriptContainer() internal view returns (Transaction memory) {
        bytes memory result = _syscallBytes("System.Runtime.GetScriptContainer", "");
        return abi.decode(result, (Transaction));
    }

    /**
     * @dev Load script with arguments
     */
    function loadScript(bytes memory script, uint8 callFlags, bytes[] memory args) internal {
        bytes memory data = abi.encode(script, callFlags, args);
        _syscallVoid("System.Runtime.LoadScript", data);
    }
    
    // ========== Storage System Calls ==========
    
    /**
     * @dev Get storage context
     */
    function getStorageContext() internal view returns (StorageContext memory) {
        bytes memory result = _syscallBytes("System.Storage.GetContext", "");
        return abi.decode(result, (StorageContext));
    }
    
    /**
     * @dev Get read-only storage context
     */
    function getReadOnlyStorageContext() internal view returns (StorageContext memory) {
        bytes memory result = _syscallBytes("System.Storage.GetReadOnlyContext", "");
        return abi.decode(result, (StorageContext));
    }

    /**
     * @dev Convert storage context to read-only
     */
    function storageAsReadOnly(StorageContext memory context) internal view returns (StorageContext memory) {
        bytes memory data = abi.encode(context);
        bytes memory result = _syscallBytes("System.Storage.AsReadOnly", data);
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
        return _syscallBytes("System.Storage.Get", data);
    }
    
    /**
     * @dev Storage put
     */
    function storagePut(StorageContext memory context, bytes memory key, bytes memory value) internal {
        bytes memory data = abi.encode(context, key, value);
        _syscallVoid("System.Storage.Put", data);
    }
    
    /**
     * @dev Storage delete
     */
    function storageDelete(StorageContext memory context, bytes memory key) internal {
        bytes memory data = abi.encode(context, key);
        _syscallVoid("System.Storage.Delete", data);
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
        bytes memory result = _syscallBytes("System.Storage.Find", data);
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
        bytes memory result = _syscallBytes("System.Storage.Find", data);
        return abi.decode(result, (Iterator));
    }

    // NOTE: The former `storageGetLocal`/`storagePutLocal`/`storageDeleteLocal`/
    // `storageFindLocal` wrappers were removed. They mapped to fictional
    // `System.Storage.Local.*` syscalls that do not exist in Neo N3's interop
    // table, so any contract using them faulted the moment it ran on a real
    // node. Use the context-based `storageGet/storagePut/storageDelete/
    // storageFind` wrappers instead — Neo N3 storage contexts are always
    // private to the owning contract.

    // ========== Runtime System Calls ==========
    
    /**
     * @dev Check witness
     */
    function checkWitness(address hash) internal view returns (bool) {
        bytes memory data = abi.encode(hash);
        return _syscall("System.Runtime.CheckWitness", data) != 0;
    }

    /**
     * @dev Check witness (public key)
     */
    function checkWitness(bytes memory publicKey) internal view returns (bool) {
        bytes memory data = abi.encode(publicKey);
        return _syscall("System.Runtime.CheckWitness", data) != 0;
    }
    
    /**
     * @dev Get time (block timestamp)
     */
    function getTime() internal view returns (uint256) {
        return _syscall("System.Runtime.GetTime", "");
    }
    
    /**
     * @dev Get gas left
     */
    function gasLeft() internal view returns (uint256) {
        return _syscall("System.Runtime.GasLeft", "");
    }
    
    /**
     * @dev Get platform information
     */
    function getPlatform() internal view returns (string memory) {
        bytes memory result = _syscallBytes("System.Runtime.Platform", "");
        return abi.decode(result, (string));
    }
    
    /**
     * @dev Get trigger type
     */
    function getTrigger() internal view returns (uint8) {
        return uint8(_syscall("System.Runtime.GetTrigger", ""));
    }
    
    /**
     * @dev Emit notification
     */
    function notify(bytes memory data) internal {
        bytes memory params = abi.encode(data);
        _syscallVoid("System.Runtime.Notify", params);
    }
    
    /**
     * @dev Get notifications
     */
    function getNotifications(address hash) internal view returns (Notification[] memory) {
        bytes memory data = abi.encode(hash);
        bytes memory result = _syscallBytes("System.Runtime.GetNotifications", data);
        return abi.decode(result, (Notification[]));
    }

    /**
     * @dev Get all notifications
     */
    function getNotifications() internal view returns (Notification[] memory) {
        bytes memory result = _syscallBytes("System.Runtime.GetNotifications", "");
        return abi.decode(result, (Notification[]));
    }
    
    /**
     * @dev Log message
     */
    function log(string memory message) internal {
        bytes memory data = abi.encode(message);
        _syscallVoid("System.Runtime.Log", data);
    }

    /**
     * @dev Get current transaction signers
     */
    function getCurrentSigners() internal view returns (Signer[] memory) {
        bytes memory result = _syscallBytes("System.Runtime.CurrentSigners", "");
        return abi.decode(result, (Signer[]));
    }
    
    // ========== Cryptographic System Calls ==========

    /**
     * @dev Check signature against current script container
     */
    function checkSig(bytes memory publicKey, bytes memory signature) internal view returns (bool) {
        bytes memory data = abi.encode(publicKey, signature);
        return _syscall("System.Crypto.CheckSig", data) != 0;
    }

    /**
     * @dev Check multi-signature against current script container
     */
    function checkMultisig(bytes[] memory publicKeys, bytes[] memory signatures) internal view returns (bool) {
        bytes memory data = abi.encode(publicKeys, signatures);
        return _syscall("System.Crypto.CheckMultisig", data) != 0;
    }
    
    /**
     * @dev SHA256 hash
     */
    function sha256(bytes memory data) internal view returns (bytes32) {
        bytes memory params = abi.encode(data);
        bytes memory result = contractCall(CRYPTO_LIB, "sha256", params);
        return abi.decode(result, (bytes32));
    }
    
    /**
     * @dev RIPEMD160 hash
     */
    function ripemd160(bytes memory data) internal view returns (bytes20) {
        bytes memory params = abi.encode(data);
        bytes memory result = contractCall(CRYPTO_LIB, "ripemd160", params);
        return abi.decode(result, (bytes20));
    }

    /**
     * @dev SHA1 hash
     */
    function sha1(bytes memory data) internal view returns (bytes20) {
        bytes memory params = abi.encode(data);
        bytes memory result = contractCall(CRYPTO_LIB, "sha1", params);
        return abi.decode(result, (bytes20));
    }

    /**
     * @dev Keccak-256 hash (CryptoLib native call, added at Neo N3 Cockatrice hardfork)
     *
     * NOTE: Solidity's built-in `keccak256()` is also lowered to this native call
     * by the neo-devpack-solidity compiler. This explicit wrapper is provided for
     * discoverability when calling through the Syscalls namespace.
     */
    function neoKeccak256(bytes memory data) internal view returns (bytes32) {
        bytes memory params = abi.encode(data);
        bytes memory result = contractCall(CRYPTO_LIB, "keccak256", params);
        return abi.decode(result, (bytes32));
    }

    // NamedCurveHash values (Neo.SmartContract.Native.NamedCurveHash)
    uint8 constant SECP256K1_SHA256 = 22;
    uint8 constant SECP256R1_SHA256 = 23;
    uint8 constant SECP256K1_KECCAK256 = 122;
    uint8 constant SECP256R1_KECCAK256 = 123;
    
    /**
     * @dev Verify ECDSA signature
     */
    function verifyWithECDsa(
        bytes32 hash,
        bytes memory publicKey,
        bytes memory signature,
        uint8 curve
    ) internal view returns (bool) {
        bytes memory data = abi.encode(hash, publicKey, signature, curve);
        bytes memory result = contractCall(CRYPTO_LIB, "verifyWithECDsa", data);
        return abi.decode(result, (bool));
    }

    /**
     * @dev Verify ECDSA signature (message bytes)
     */
    function verifyWithECDsa(
        bytes memory message,
        bytes memory publicKey,
        bytes memory signature,
        uint8 curve
    ) internal view returns (bool) {
        bytes memory data = abi.encode(message, publicKey, signature, curve);
        bytes memory result = contractCall(CRYPTO_LIB, "verifyWithECDsa", data);
        return abi.decode(result, (bool));
    }
    
    /**
     * @dev Murmur32 hash
     */
    function murmur32(bytes memory data, uint32 seed) internal view returns (bytes4) {
        bytes memory params = abi.encode(data, seed);
        bytes memory result = contractCall(CRYPTO_LIB, "murmur32", params);
        return abi.decode(result, (bytes4));
    }

    /**
     * @dev Recover secp256k1 public key from signature
     */
    function recoverSecp256K1(bytes memory messageHash, bytes memory signature) internal view returns (bytes memory) {
        bytes memory data = abi.encode(messageHash, signature);
        return contractCall(CRYPTO_LIB, "recoverSecp256K1", data);
    }

    /**
     * @dev Verify Ed25519 signature
     */
    function verifyWithEd25519(bytes memory message, bytes memory publicKey, bytes memory signature) internal view returns (bool) {
        bytes memory data = abi.encode(message, publicKey, signature);
        bytes memory result = contractCall(CRYPTO_LIB, "verifyWithEd25519", data);
        return abi.decode(result, (bool));
    }

    /**
     * @dev Serialize BLS12-381 point (opaque handle)
     */
    function bls12381Serialize(bytes memory point) internal view returns (bytes memory) {
        bytes memory data = abi.encode(point);
        return contractCall(CRYPTO_LIB, "bls12381Serialize", data);
    }

    /**
     * @dev Deserialize BLS12-381 point (returns opaque handle)
     */
    function bls12381Deserialize(bytes memory data) internal view returns (bytes memory) {
        bytes memory params = abi.encode(data);
        return contractCall(CRYPTO_LIB, "bls12381Deserialize", params);
    }

    /**
     * @dev Compare BLS12-381 points
     */
    function bls12381Equal(bytes memory x, bytes memory y) internal view returns (bool) {
        bytes memory data = abi.encode(x, y);
        bytes memory result = contractCall(CRYPTO_LIB, "bls12381Equal", data);
        return abi.decode(result, (bool));
    }

    /**
     * @dev Add BLS12-381 points
     */
    function bls12381Add(bytes memory x, bytes memory y) internal view returns (bytes memory) {
        bytes memory data = abi.encode(x, y);
        return contractCall(CRYPTO_LIB, "bls12381Add", data);
    }

    /**
     * @dev Multiply BLS12-381 point by scalar
     */
    function bls12381Mul(bytes memory x, bytes memory mul, bool neg) internal view returns (bytes memory) {
        bytes memory data = abi.encode(x, mul, neg);
        return contractCall(CRYPTO_LIB, "bls12381Mul", data);
    }

    /**
     * @dev Pairing operation for BLS12-381
     */
    function bls12381Pairing(bytes memory g1, bytes memory g2) internal view returns (bytes memory) {
        bytes memory data = abi.encode(g1, g2);
        return contractCall(CRYPTO_LIB, "bls12381Pairing", data);
    }

    // NOTE: The former `bls12381G1Add` / `bls12381G1Mul` / `bls12381G1Neg` /
    // `bls12381G2Add` / `bls12381G2Mul` / `bls12381G2Neg` wrappers were
    // removed (kept in sync with syscalls/SyscallsCrypto.sol). The on-chain
    // CryptoLib exposes exactly six BLS methods (bls12381Serialize,
    // bls12381Deserialize, bls12381Equal, bls12381Add, bls12381Mul,
    // bls12381Pairing — verified against `CryptoLib.BLS12_381.cs`); G1 vs G2 is
    // inferred on-chain from the deserialized point length (48/96 bytes), not
    // from the method name. The invented names faulted with "Method not found"
    // on a real node, and the compiler's builtin whitelist (resolve.rs) rejects
    // them with an "unsupported builtin library call" diagnostic.
    
    // ========== StdLib System Calls ==========

    /**
     * @dev Serialize stack item
     */
    function serialize(bytes memory data) internal view returns (bytes memory) {
        bytes memory params = abi.encode(data);
        return contractCall(STD_LIB, "serialize", params);
    }

    /**
     * @dev Deserialize stack item
     */
    function deserialize(bytes memory data) internal view returns (bytes memory) {
        bytes memory params = abi.encode(data);
        return contractCall(STD_LIB, "deserialize", params);
    }

    /**
     * @dev Integer to string (base 10)
     */
    function itoa(int256 value) internal view returns (string memory) {
        bytes memory params = abi.encode(value);
        bytes memory result = contractCall(STD_LIB, "itoa", params);
        return abi.decode(result, (string));
    }

    /**
     * @dev Integer to string with base (10 or 16)
     */
    function itoa(int256 value, uint8 base) internal view returns (string memory) {
        bytes memory params = abi.encode(value, base);
        bytes memory result = contractCall(STD_LIB, "itoa", params);
        return abi.decode(result, (string));
    }

    /**
     * @dev String to integer (base 10)
     */
    function atoi(string memory value) internal view returns (int256) {
        bytes memory params = abi.encode(value);
        bytes memory result = contractCall(STD_LIB, "atoi", params);
        return abi.decode(result, (int256));
    }

    /**
     * @dev String to integer with base (10 or 16)
     */
    function atoi(string memory value, uint8 base) internal view returns (int256) {
        bytes memory params = abi.encode(value, base);
        bytes memory result = contractCall(STD_LIB, "atoi", params);
        return abi.decode(result, (int256));
    }

    // ========== JSON System Calls ==========
    
    /**
     * @dev Serialize to JSON
     */
    function jsonSerialize(bytes memory data) internal view returns (bytes memory) {
        bytes memory params = abi.encode(data);
        return contractCall(STD_LIB, "jsonSerialize", params);
    }
    
    /**
     * @dev Deserialize from JSON
     */
    function jsonDeserialize(bytes memory json) internal view returns (bytes memory) {
        bytes memory params = abi.encode(json);
        return contractCall(STD_LIB, "jsonDeserialize", params);
    }
    
    // ========== Base64 System Calls ==========
    
    /**
     * @dev Base64 encode
     */
    function base64Encode(bytes memory data) internal view returns (string memory) {
        bytes memory params = abi.encode(data);
        bytes memory result = contractCall(STD_LIB, "base64Encode", params);
        return abi.decode(result, (string));
    }
    
    /**
     * @dev Base64 decode
     */
    function base64Decode(string memory data) internal view returns (bytes memory) {
        bytes memory params = abi.encode(data);
        return contractCall(STD_LIB, "base64Decode", params);
    }

    /**
     * @dev Base64Url encode
     */
    function base64UrlEncode(string memory data) internal view returns (string memory) {
        bytes memory params = abi.encode(data);
        bytes memory result = contractCall(STD_LIB, "base64UrlEncode", params);
        return abi.decode(result, (string));
    }

    /**
     * @dev Base64Url decode
     */
    function base64UrlDecode(string memory data) internal view returns (string memory) {
        bytes memory params = abi.encode(data);
        bytes memory result = contractCall(STD_LIB, "base64UrlDecode", params);
        return abi.decode(result, (string));
    }

    // ========== Base58 System Calls ==========

    /**
     * @dev Base58 encode
     */
    function base58Encode(bytes memory data) internal view returns (string memory) {
        bytes memory params = abi.encode(data);
        bytes memory result = contractCall(STD_LIB, "base58Encode", params);
        return abi.decode(result, (string));
    }

    /**
     * @dev Base58 decode
     */
    function base58Decode(string memory data) internal view returns (bytes memory) {
        bytes memory params = abi.encode(data);
        return contractCall(STD_LIB, "base58Decode", params);
    }

    /**
     * @dev Base58Check encode
     */
    function base58CheckEncode(bytes memory data) internal view returns (string memory) {
        bytes memory params = abi.encode(data);
        bytes memory result = contractCall(STD_LIB, "base58CheckEncode", params);
        return abi.decode(result, (string));
    }

    /**
     * @dev Base58Check decode
     */
    function base58CheckDecode(string memory data) internal view returns (bytes memory) {
        bytes memory params = abi.encode(data);
        return contractCall(STD_LIB, "base58CheckDecode", params);
    }

    /**
     * @dev Hex encode a byte array
     *
     * Real StdLib method — `StdLib.cs` declares `HexEncode(byte[])` with a
     * `[ContractMethod]` attribute (the earlier removal note claiming the
     * method doesn't exist was factually wrong). C# `ToHexString` emits
     * UPPERCASE hex; adjust case at the call site if lowercase is required.
     */
    function hexEncode(bytes memory data) internal view returns (string memory) {
        bytes memory params = abi.encode(data);
        bytes memory result = contractCall(STD_LIB, "hexEncode", params);
        return abi.decode(result, (string));
    }

    /**
     * @dev Decode a hex string back to bytes
     *
     * Real StdLib method — `StdLib.cs` declares `HexDecode(string)`.
     */
    function hexDecode(string memory data) internal view returns (bytes memory) {
        bytes memory params = abi.encode(data);
        return contractCall(STD_LIB, "hexDecode", params);
    }

    // ========== Memory and String Utilities ==========

    /**
     * @dev Compare two byte arrays
     */
    function memoryCompare(bytes memory left, bytes memory right) internal view returns (int256) {
        bytes memory params = abi.encode(left, right);
        bytes memory result = contractCall(STD_LIB, "memoryCompare", params);
        return abi.decode(result, (int256));
    }

    /**
     * @dev Search for a value in memory (start at 0)
     */
    function memorySearch(bytes memory mem, bytes memory value) internal view returns (int256) {
        bytes memory params = abi.encode(mem, value);
        bytes memory result = contractCall(STD_LIB, "memorySearch", params);
        return abi.decode(result, (int256));
    }

    /**
     * @dev Search for a value in memory (start at offset)
     */
    function memorySearch(bytes memory mem, bytes memory value, int256 start) internal view returns (int256) {
        bytes memory params = abi.encode(mem, value, start);
        bytes memory result = contractCall(STD_LIB, "memorySearch", params);
        return abi.decode(result, (int256));
    }

    /**
     * @dev Search for a value in memory (start at offset, optionally backward)
     */
    function memorySearch(bytes memory mem, bytes memory value, int256 start, bool backward)
        internal
        view
        returns (int256)
    {
        bytes memory params = abi.encode(mem, value, start, backward);
        bytes memory result = contractCall(STD_LIB, "memorySearch", params);
        return abi.decode(result, (int256));
    }

    /**
     * @dev Split a string by separator
     */
    function stringSplit(string memory value, string memory separator) internal view returns (string[] memory) {
        bytes memory params = abi.encode(value, separator);
        bytes memory result = contractCall(STD_LIB, "stringSplit", params);
        return abi.decode(result, (string[]));
    }

    /**
     * @dev Split a string by separator with optional empty removal
     */
    function stringSplit(
        string memory value,
        string memory separator,
        bool removeEmptyEntries
    ) internal view returns (string[] memory) {
        bytes memory params = abi.encode(value, separator, removeEmptyEntries);
        bytes memory result = contractCall(STD_LIB, "stringSplit", params);
        return abi.decode(result, (string[]));
    }

    /**
     * @dev Get string length in text elements
     */
    function strLen(string memory value) internal view returns (uint256) {
        bytes memory params = abi.encode(value);
        bytes memory result = contractCall(STD_LIB, "strLen", params);
        return abi.decode(result, (uint256));
    }
    
    // ========== Iterator System Calls ==========
    
    /**
     * @dev Get next iterator value
     */
    function iteratorNext(Iterator memory iterator) internal returns (bool) {
        bytes memory data = abi.encode(iterator);
        return _syscall("System.Iterator.Next", data) != 0;
    }
    
    /**
     * @dev Get iterator value
     */
    function iteratorValue(Iterator memory iterator) internal view returns (bytes memory) {
        bytes memory data = abi.encode(iterator);
        return _syscallBytes("System.Iterator.Value", data);
    }
    
    // ========== Internal Syscall Implementation ==========
    
    /**
     * @dev Internal syscall that returns uint256
     */
    function _syscall(string memory method, bytes memory params) private view returns (uint256) {
        // Production syscall implementation using Neo VM native interface
        bytes memory callData = abi.encodeWithSignature("neoSyscall(string,bytes)", method, params);
        
        (bool success, bytes memory result) = address(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF).staticcall(callData);
        
        if (!success || result.length < 32) {
            // Fallback to method-specific implementations
            return _handleSyscallFallback(method, params);
        }
        
        return abi.decode(result, (uint256));
    }
    
    /**
     * @dev Fallback syscall implementations for specific methods
     */
    function _handleSyscallFallback(string memory method, bytes memory params) private view returns (uint256) {
        bytes32 methodHash = keccak256(bytes(method));
        
        if (methodHash == keccak256("System.Blockchain.GetHeight")) {
            return block.number;
        } else if (methodHash == keccak256("System.Runtime.GetTime")) {
            return block.timestamp;
        } else if (methodHash == keccak256("System.Runtime.GasLeft")) {
            // Fallback runtime does not expose gas accounting.
            return 0;
        } else if (methodHash == keccak256("System.Runtime.CheckWitness")) {
            address account = abi.decode(params, (address));
            return account == msg.sender ? 1 : 0;
        }
        
        return 0;
    }
    
    /**
     * @dev Internal syscall that returns bytes
     */
    function _syscallBytes(string memory method, bytes memory params) private view returns (bytes memory) {
        // Production syscall implementation for bytes return values
        bytes memory callData = abi.encodeWithSignature("neoSyscallBytes(string,bytes)", method, params);
        
        (bool success, bytes memory result) = address(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF).staticcall(callData);
        
        if (!success) {
            // Fallback to method-specific implementations
            return _handleBytesSyscallFallback(method, params);
        }
        
        return result;
    }
    
    /**
     * @dev Fallback syscall implementations for bytes methods
     */
    function _handleBytesSyscallFallback(string memory method, bytes memory params) private view returns (bytes memory) {
        bytes32 methodHash = keccak256(bytes(method));
        
        if (methodHash == keccak256("System.Runtime.GetExecutingScriptHash")) {
            return abi.encode(address(this));
        } else if (methodHash == keccak256("System.Runtime.GetCallingScriptHash")) {
            return abi.encode(msg.sender);
        }
        
        return "";
    }

    /**
     * @dev Internal syscall that returns void
     */
    function _syscallVoid(string memory method, bytes memory params) private {
        // Production syscall implementation for void return methods
        bytes memory callData = abi.encodeWithSignature("neoSyscallVoid(string,bytes)", method, params);
        
        (bool success, ) = address(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF).call(callData);
        
        if (!success) {
            // Fallback to method-specific implementations
            _handleVoidSyscallFallback(method, params);
        }
    }
    
    /**
     * @dev Fallback syscall implementations for void methods
     */
    function _handleVoidSyscallFallback(string memory method, bytes memory params) private {
        bytes32 methodHash = keccak256(bytes(method));
        
        if (methodHash == keccak256("System.Storage.Put")) {
            return;
        } else if (methodHash == keccak256("System.Storage.Delete")) {
            return;
        } else if (methodHash == keccak256("System.Runtime.Notify")) {
            return;
        }
    }
    
    // ========== Data Structures ==========
    
    // Ledger.getBlock returns a TrimmedBlock stack item, not the full block payload.
    struct Block {
        bytes32 hash;
        uint256 version;
        bytes32 previousHash;
        bytes32 merkleRoot;
        uint256 timestamp;
        uint256 nonce;
        uint256 index;
        uint256 primaryIndex;
        address nextConsensus;
        uint256 txCount;
    }
    
    // Ledger.getTransaction returns the base transaction fields (no signers/witnesses).
    struct Transaction {
        bytes32 hash;
        uint256 version;
        uint256 nonce;
        address sender;
        uint256 systemFee;
        uint256 networkFee;
        uint256 validUntilBlock;
        bytes script;
    }
    
    struct Witness {
        bytes invocationScript;
        bytes verificationScript;
    }

    uint8 constant WITNESS_RULE_DENY = 0x00;
    uint8 constant WITNESS_RULE_ALLOW = 0x01;

    uint8 constant WITNESS_CONDITION_BOOLEAN = 0x00;
    uint8 constant WITNESS_CONDITION_NOT = 0x01;
    uint8 constant WITNESS_CONDITION_AND = 0x02;
    uint8 constant WITNESS_CONDITION_OR = 0x03;
    uint8 constant WITNESS_CONDITION_SCRIPT_HASH = 0x18;
    uint8 constant WITNESS_CONDITION_GROUP = 0x19;
    uint8 constant WITNESS_CONDITION_CALLED_BY_ENTRY = 0x20;
    uint8 constant WITNESS_CONDITION_CALLED_BY_CONTRACT = 0x28;
    uint8 constant WITNESS_CONDITION_CALLED_BY_GROUP = 0x29;

    // WitnessCondition is represented as a NeoVM Array:
    // [type, ...condition-specific data]. Use StdLib.serialize/deserialize
    // if you need to inspect it in Solidity.
    struct WitnessRule {
        uint8 action;
        bytes condition;
    }
    
    // WitnessScope bit flags (see Neo WitnessScope enum).
    uint8 constant WITNESS_SCOPE_NONE = 0x00;
    uint8 constant WITNESS_SCOPE_CALLED_BY_ENTRY = 0x01;
    uint8 constant WITNESS_SCOPE_CUSTOM_CONTRACTS = 0x10;
    uint8 constant WITNESS_SCOPE_CUSTOM_GROUPS = 0x20;
    uint8 constant WITNESS_SCOPE_WITNESS_RULES = 0x40;
    uint8 constant WITNESS_SCOPE_GLOBAL = 0x80;

    struct Signer {
        address account;
        uint8 scopes;
        address[] allowedContracts;
        bytes[] allowedGroups;
        // Witness rules (only present when scopes includes WITNESS_SCOPE_WITNESS_RULES).
        WitnessRule[] rules;
    }
    
    struct StorageContext {
        int256 id;
        bool isReadOnly;
    }
    
    struct Iterator {
        uint256 id;
        bool hasNext;
        bytes currentKey;
        bytes currentValue;
    }
    
    struct Notification {
        address scriptHash;
        string eventName;
        // State array passed to Runtime.notify(...)
        bytes[] state;
    }

    // ContractManagement.getContract returns:
    // [id, updateCounter, hash, nef, manifestStruct]
    struct ContractStateNative {
        int256 id;
        uint256 updateCounter;
        address hash;
        bytes nef;
        bytes manifest;
    }
    
    // TriggerType values (Neo.SmartContract.TriggerType)
    uint8 constant TRIGGER_ON_PERSIST = 0x01;
    uint8 constant TRIGGER_POST_PERSIST = 0x02;
    uint8 constant TRIGGER_VERIFICATION = 0x20;
    uint8 constant TRIGGER_APPLICATION = 0x40;
    uint8 constant TRIGGER_SYSTEM = TRIGGER_ON_PERSIST | TRIGGER_POST_PERSIST;
    uint8 constant TRIGGER_ALL = TRIGGER_SYSTEM | TRIGGER_VERIFICATION | TRIGGER_APPLICATION;
    
    // ========== Advanced Syscalls ==========
    
    /**
     * @dev Get current random number
     */
    function getCurrentRandom() internal view returns (uint256) {
        return _syscall("System.Runtime.GetRandom", "");
    }
    
    /**
     * @dev Get network magic number
     */
    function getNetwork() internal view returns (uint32) {
        return uint32(_syscall("System.Runtime.GetNetwork", ""));
    }
    
    /**
     * @dev Get address version
     */
    function getAddressVersion() internal view returns (uint8) {
        return uint8(_syscall("System.Runtime.GetAddressVersion", ""));
    }
    
    /**
     * @dev Burn GAS
     */
    function burnGas(uint256 amount) internal {
        bytes memory data = abi.encode(amount);
        _syscallVoid("System.Runtime.BurnGas", data);
    }
    
    /**
     * @dev Get invocation counter
     */
    function getInvocationCounter() internal view returns (uint256) {
        return _syscall("System.Runtime.GetInvocationCounter", "");
    }

    /**
     * @dev Get message value (msg.value equivalent — the value attached to the current invocation)
     */
    function getMsgValue() internal view returns (uint256) {
        return _syscall("System.Runtime.GetMsgValue", "");
    }
    
    // ========== Policy System Calls ==========
    
    /**
     * @dev Get fee per byte
     */
    function getFeePerByte() internal view returns (uint256) {
        bytes memory result = contractCall(POLICY_CONTRACT, "getFeePerByte", "");
        return abi.decode(result, (uint256));
    }
    
    /**
     * @dev Get exec fee factor
     */
    function getExecFeeFactor() internal view returns (uint32) {
        bytes memory result = contractCall(POLICY_CONTRACT, "getExecFeeFactor", "");
        return abi.decode(result, (uint32));
    }

    /**
     * @dev Get exec fee factor in picoGAS units
     */
    function getExecPicoFeeFactor() internal view returns (uint256) {
        bytes memory result = contractCall(POLICY_CONTRACT, "getExecPicoFeeFactor", "");
        return abi.decode(result, (uint256));
    }

    /**
     * @dev Get storage price
     */
    function getStoragePrice() internal view returns (uint256) {
        bytes memory result = contractCall(POLICY_CONTRACT, "getStoragePrice", "");
        return abi.decode(result, (uint256));
    }

    /**
     * @dev Get block milliseconds
     */
    function getMillisecondsPerBlock() internal view returns (uint256) {
        bytes memory result = contractCall(POLICY_CONTRACT, "getMillisecondsPerBlock", "");
        return abi.decode(result, (uint256));
    }

    /**
     * @dev Get max valid-until-block increment
     */
    function getMaxValidUntilBlockIncrement() internal view returns (uint256) {
        bytes memory result = contractCall(POLICY_CONTRACT, "getMaxValidUntilBlockIncrement", "");
        return abi.decode(result, (uint256));
    }

    /**
     * @dev Get max traceable blocks
     */
    function getMaxTraceableBlocks() internal view returns (uint256) {
        bytes memory result = contractCall(POLICY_CONTRACT, "getMaxTraceableBlocks", "");
        return abi.decode(result, (uint256));
    }

    /**
     * @dev Get attribute fee
     */
    function getAttributeFee(uint8 attributeType) internal view returns (uint256) {
        bytes memory data = abi.encode(attributeType);
        bytes memory result = contractCall(POLICY_CONTRACT, "getAttributeFee", data);
        return abi.decode(result, (uint256));
    }

    /**
     * @dev Check if account is blocked
     */
    function isBlocked(address account) internal view returns (bool) {
        bytes memory data = abi.encode(account);
        bytes memory result = contractCall(POLICY_CONTRACT, "isBlocked", data);
        return abi.decode(result, (bool));
    }
    
    // ========== Oracle System Calls ==========
    
    /**
     * @dev Make oracle request
     */
    function oracleRequest(
        string memory url,
        string memory filter,
        string memory callback,
        bytes memory userData,
        uint256 gasForResponse
    ) internal {
        bytes memory data = abi.encode(url, filter, callback, userData, gasForResponse);
        contractCall(ORACLE_CONTRACT, "request", data);
    }
    
    /**
     * @dev Get oracle price
     */
    function getOraclePrice() internal view returns (uint256) {
        bytes memory result = contractCall(ORACLE_CONTRACT, "getPrice", "");
        return abi.decode(result, (uint256));
    }
    
    // ========== Role Management System Calls ==========
    
    /**
     * @dev Get designated by role
     */
    function getDesignatedByRole(bytes1 role, uint256 index) internal view returns (bytes[] memory) {
        bytes memory data = abi.encode(role, index);
        bytes memory result = contractCall(ROLE_MANAGEMENT, "getDesignatedByRole", data);
        return abi.decode(result, (bytes[]));
    }
    
    // ========== Utility Functions ==========
    
    /**
     * @dev Convert script hash to address
     */
    function scriptHashToAddress(bytes20 scriptHash) internal pure returns (address) {
        // A Neo script hash IS a 160-bit value; an `address` holds it directly.
        // The previous `uint160(uint256(bytes32(scriptHash)))` LEFT-aligned the
        // 20 bytes into a bytes32 (high bytes), then `uint160(...)` kept the LOW
        // 160 bits — discarding the top 12 bytes of the hash. Convert the 20
        // bytes straight to uint160 to preserve every byte.
        return address(uint160(scriptHash));
    }
    
    /**
     * @dev Convert address to script hash
     */
    function addressToScriptHash(address addr) internal pure returns (bytes20) {
        return bytes20(uint160(addr));
    }
    
    /**
     * @dev Validate Neo address format
     */
    function isValidAddress(address addr) internal pure returns (bool) {
        return addr != address(0) && uint160(addr) != 0;
    }
    
    /**
     * @dev Get contract NEF (script container)
     *
     * NOTE: The neo-devpack-solidity compiler treats this as an intrinsic and lowers it
     * to `ContractManagement.getContract(contractHash).nef`.
     */
    function getContractScript(address contractHash) internal view returns (bytes memory) {
        bytes memory data = abi.encode(contractHash);
        bytes memory result = contractCall(CONTRACT_MANAGEMENT, "getContract", data);
        if (result.length == 0) {
            return "";
        }
        ContractStateNative memory state = abi.decode(result, (ContractStateNative));
        return state.nef;
    }
    
    /**
     * @dev Check if contract exists
     *
     * NOTE: The compiler lowers this helper to `ContractManagement.isContract(contractHash)`
     * on Neo N3 for correctness and efficiency.
     */
    function contractExists(address contractHash) internal view returns (bool) {
        bytes memory data = abi.encode(contractHash);
        bytes memory result = contractCall(CONTRACT_MANAGEMENT, "isContract", data);
        return abi.decode(result, (bool));
    }
}
