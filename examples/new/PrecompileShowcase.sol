// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "../../devpack/libraries/Precompiles.sol";
import "../../devpack/contracts/Syscalls.sol";

/**
 * @title PrecompileShowcase
 * @notice Demonstrates all EVM precompiled contract equivalents on Neo N3.
 *         Tests: ecrecover, sha256, ripemd160, identity, modexp,
 *         BLS12-381 curve ops (ecAdd, ecMul, ecPairing), and signature verification.
 */
contract PrecompileShowcase {
    event HashComputed(string algorithm, bytes32 hash);
    event ModExpResult(uint256 result);
    event SignatureVerified(address signer, bool valid);
    event PairingResult(bool success);

    // ========== Hash Function Tests ==========

    function testSha256(bytes calldata data) public view returns (bytes32) {
        return Precompiles.sha256Hash(data);
    }

    function testRipemd160(bytes calldata data) public view returns (bytes20) {
        return Precompiles.ripemd160Hash(data);
    }

    function testKeccak256(bytes calldata data) public pure returns (bytes32) {
        return keccak256(data);
    }

    function testIdentity(bytes calldata data) public pure returns (bytes memory) {
        return Precompiles.identity(data);
    }

    // ========== Modular Exponentiation ==========

    function testModExp(uint256 base, uint256 exp, uint256 mod) public pure returns (uint256) {
        return Precompiles.modExp(base, exp, mod);
    }

    // RSA-style verification helper
    function testRSAModExp() public pure returns (uint256) {
        // 3^5 mod 13 = 243 mod 13 = 9
        return Precompiles.modExp(3, 5, 13);
    }

    // ========== Signature Recovery ==========

    function testEcRecover(
        bytes32 hash,
        uint8 v,
        bytes32 r,
        bytes32 s
    ) public view returns (address) {
        return Precompiles.ecRecover(hash, v, r, s);
    }

    // ========== BLS12-381 Curve Operations ==========

    function testBls12381Add(bytes calldata p1, bytes calldata p2) public view returns (bytes memory) {
        return Precompiles.ecAdd(p1, p2);
    }

    function testBls12381Mul(bytes calldata point, bytes calldata scalar) public view returns (bytes memory) {
        return Precompiles.ecMul(point, scalar);
    }

    function testBls12381Pairing(bytes calldata g1, bytes calldata g2) public view returns (bytes memory) {
        return Precompiles.ecPairing(g1, g2);
    }

    // ========== Ethereum-Compatible Signature Verification ==========

    function verifyEthSignature(
        bytes32 messageHash,
        bytes calldata publicKey,
        bytes calldata signature
    ) public view returns (bool) {
        return Precompiles.verifyEthSignature(messageHash, publicKey, signature);
    }

    // ========== Neo-Native Signature Verification ==========

    function verifyNeoSignature(
        bytes32 messageHash,
        bytes calldata publicKey,
        bytes calldata signature
    ) public view returns (bool) {
        return Precompiles.verifyNeoSignature(messageHash, publicKey, signature);
    }

    // ========== Multi-Algorithm Hash Comparison ==========

    function compareHashes(bytes calldata data) public view returns (
        bytes32 keccakHash,
        bytes32 sha256Hash,
        bytes20 ripemd160Hash
    ) {
        keccakHash = keccak256(data);
        sha256Hash = Precompiles.sha256Hash(data);
        ripemd160Hash = Precompiles.ripemd160Hash(data);
    }
}
