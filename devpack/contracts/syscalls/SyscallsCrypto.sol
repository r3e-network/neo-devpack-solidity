// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./SyscallsBase.sol";

/**
 * @title Syscalls Crypto — Neo N3 Crypto operations
 */

library SyscallsCrypto {
    // ========== Cryptographic System Calls ==========

    /**
     * @dev Check signature against current script container
     */
    function checkSig(bytes memory publicKey, bytes memory signature) internal view returns (bool) {
        bytes memory data = abi.encode(publicKey, signature);
        return SyscallsBase._syscall("System.Crypto.CheckSig", data) != 0;
    }

    /**
     * @dev Check multi-signature against current script container
     */
    function checkMultisig(bytes[] memory publicKeys, bytes[] memory signatures) internal view returns (bool) {
        bytes memory data = abi.encode(publicKeys, signatures);
        return SyscallsBase._syscall("System.Crypto.CheckMultisig", data) != 0;
    }
    
    /**
     * @dev SHA256 hash
     */
    function sha256(bytes memory data) internal view returns (bytes32) {
        bytes memory params = abi.encode(data);
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.CRYPTO_LIB, "sha256", params);
        return abi.decode(result, (bytes32));
    }
    
    /**
     * @dev RIPEMD160 hash
     */
    function ripemd160(bytes memory data) internal view returns (bytes20) {
        bytes memory params = abi.encode(data);
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.CRYPTO_LIB, "ripemd160", params);
        return abi.decode(result, (bytes20));
    }

    /**
     * @dev SHA1 hash
     */
    function sha1(bytes memory data) internal view returns (bytes20) {
        bytes memory params = abi.encode(data);
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.CRYPTO_LIB, "sha1", params);
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
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.CRYPTO_LIB, "keccak256", params);
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
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.CRYPTO_LIB, "verifyWithECDsa", data);
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
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.CRYPTO_LIB, "verifyWithECDsa", data);
        return abi.decode(result, (bool));
    }
    
    /**
     * @dev Murmur32 hash
     */
    function murmur32(bytes memory data, uint32 seed) internal view returns (bytes4) {
        bytes memory params = abi.encode(data, seed);
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.CRYPTO_LIB, "murmur32", params);
        return abi.decode(result, (bytes4));
    }

    /**
     * @dev Recover secp256k1 public key from signature
     */
    function recoverSecp256K1(bytes memory messageHash, bytes memory signature) internal view returns (bytes memory) {
        bytes memory data = abi.encode(messageHash, signature);
        return SyscallsBase.contractCall(SyscallsBase.CRYPTO_LIB, "recoverSecp256K1", data);
    }

    /**
     * @dev Verify Ed25519 signature
     */
    function verifyWithEd25519(bytes memory message, bytes memory publicKey, bytes memory signature) internal view returns (bool) {
        bytes memory data = abi.encode(message, publicKey, signature);
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.CRYPTO_LIB, "verifyWithEd25519", data);
        return abi.decode(result, (bool));
    }

    /**
     * @dev Serialize BLS12-381 point (opaque handle)
     */
    function bls12381Serialize(bytes memory point) internal view returns (bytes memory) {
        bytes memory data = abi.encode(point);
        return SyscallsBase.contractCall(SyscallsBase.CRYPTO_LIB, "bls12381Serialize", data);
    }

    /**
     * @dev Deserialize BLS12-381 point (returns opaque handle)
     */
    function bls12381Deserialize(bytes memory data) internal view returns (bytes memory) {
        bytes memory params = abi.encode(data);
        return SyscallsBase.contractCall(SyscallsBase.CRYPTO_LIB, "bls12381Deserialize", params);
    }

    /**
     * @dev Compare BLS12-381 points
     */
    function bls12381Equal(bytes memory x, bytes memory y) internal view returns (bool) {
        bytes memory data = abi.encode(x, y);
        bytes memory result = SyscallsBase.contractCall(SyscallsBase.CRYPTO_LIB, "bls12381Equal", data);
        return abi.decode(result, (bool));
    }

    /**
     * @dev Add BLS12-381 points
     */
    function bls12381Add(bytes memory x, bytes memory y) internal view returns (bytes memory) {
        bytes memory data = abi.encode(x, y);
        return SyscallsBase.contractCall(SyscallsBase.CRYPTO_LIB, "bls12381Add", data);
    }

    /**
     * @dev Multiply BLS12-381 point by scalar
     */
    function bls12381Mul(bytes memory x, bytes memory mul, bool neg) internal view returns (bytes memory) {
        bytes memory data = abi.encode(x, mul, neg);
        return SyscallsBase.contractCall(SyscallsBase.CRYPTO_LIB, "bls12381Mul", data);
    }

    /**
     * @dev Pairing operation for BLS12-381
     */
    function bls12381Pairing(bytes memory g1, bytes memory g2) internal view returns (bytes memory) {
        bytes memory data = abi.encode(g1, g2);
        return SyscallsBase.contractCall(SyscallsBase.CRYPTO_LIB, "bls12381Pairing", data);
    }

    // NOTE: The former `bls12381G1Add` / `bls12381G1Mul` / `bls12381G1Neg` /
    // `bls12381G2Add` / `bls12381G2Mul` / `bls12381G2Neg` wrappers were
    // removed. The on-chain CryptoLib exposes exactly six BLS methods
    // (bls12381Serialize, bls12381Deserialize, bls12381Equal, bls12381Add,
    // bls12381Mul, bls12381Pairing — verified against
    // `CryptoLib.BLS12_381.cs`); G1 vs G2 is inferred on-chain from the
    // deserialized point length (48/96 bytes), not from the method name. The
    // invented names faulted with "Method not found" on a real node.

}
