// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "../contracts/Syscalls.sol";

/**
 * @title Precompiles
 * @dev EVM precompiled contract compatibility layer for Neo N3.
 *
 * Maps Ethereum-style precompile helper addresses (0x01-0x09) to Neo N3
 * native contract operations where possible. These helpers are not a full
 * byte-for-byte EVM precompile emulation layer; functions with no Neo N3
 * equivalent revert with a clear message.
 *
 * Supported:
 *   0x01 ecrecover   → CryptoLib.recoverSecp256K1
 *   0x02 sha256      → CryptoLib.sha256
 *   0x03 ripemd160   → CryptoLib.ripemd160
 *   0x04 identity    → pass-through (no-op)
 *   0x05 modexp      → uint256 helper using NeoVM BigInteger arithmetic
 *                       (not full EIP-198 byte-payload parity)
 *   0x06 ecAdd       → Neo BLS12-381 adaptation, not Ethereum BN256
 *   0x07 ecMul       → Neo BLS12-381 adaptation, not Ethereum BN256
 *   0x08 ecPairing   → Neo BLS12-381 adaptation, not Ethereum BN256
 *   0x09 blake2f     → not available on Neo N3
 *
 * @notice ZK operations use BLS12-381 (Neo N3 native) instead of Ethereum
 * BN256/alt_bn128. Proof systems and proof encodings must be adapted before
 * using these helpers.
 */
library Precompiles {

    // ========== 0x01: ECRECOVER ==========

    /**
     * @dev Recover signer address from ECDSA signature (secp256k1).
     * @param hash The message hash (32 bytes)
     * @param v Recovery id (27 or 28)
     * @param r Signature r component (32 bytes)
     * @param s Signature s component (32 bytes)
     * @return signer The recovered address, or address(0) on failure
     */
    function ecRecover(
        bytes32 hash,
        uint8 v,
        bytes32 r,
        bytes32 s
    ) internal view returns (address signer) {
        // Use Solidity's built-in ecrecover. The compiler lowers this to
        // CryptoLib.recoverSecp256K1 (yields uncompressed 65-byte pubkey),
        // followed by CryptoLib.keccak256 + RIGHT 20 to produce the
        // Ethereum-spec 20-byte address: keccak256(pubkey[1..])[12..32].
        return ecrecover(hash, v, r, s);
    }

    // ========== 0x02: SHA-256 ==========

    /**
     * @dev Compute SHA-256 hash.
     * @param data Input data
     * @return digest 32-byte hash
     */
    function sha256Hash(bytes memory data) internal view returns (bytes32 digest) {
        return sha256(data);
    }

    // ========== 0x03: RIPEMD-160 ==========

    /**
     * @dev Compute RIPEMD-160 hash.
     * @param data Input data
     * @return digest 20-byte hash
     */
    function ripemd160Hash(bytes memory data) internal view returns (bytes20 digest) {
        return ripemd160(data);
    }

    // ========== 0x04: IDENTITY ==========

    /**
     * @dev Identity function (data pass-through).
     * @param data Input data
     * @return The same data, unchanged
     */
    function identity(bytes memory data) internal pure returns (bytes memory) {
        return data;
    }

    // ========== 0x05: MODEXP ==========

    /**
     * @notice NOT EIP-198 compliant — only handles uint256 values, not variable-length byte arrays
     * @dev Modular exponentiation: base^exp % mod.
     * @notice NeoVM uses arbitrary-precision BigInteger natively, so this
     * operation is performed using standard arithmetic operators. This helper
     * accepts typed uint256 values and does NOT implement EIP-198's raw input
     * byte layout, variable-length operands, or gas schedule. Porting EVM
     * cryptography code that uses the EIP-198 byte-payload format will produce
     * incorrect results unless adapted.
     * @param base The base value
     * @param exp The exponent
     * @param mod The modulus (must be non-zero)
     * @return result base^exp % mod
     */
    function modExp(
        uint256 base,
        uint256 exp,
        uint256 mod
    ) internal pure returns (uint256 result) {
        require(mod != 0, "Precompiles: modulus is zero");

        if (mod == 1) return 0;
        if (exp == 0) return 1;

        // Square-and-multiply algorithm for modular exponentiation
        result = 1;
        base = base % mod;

        while (exp > 0) {
            if (exp % 2 == 1) {
                result = mulmod(result, base, mod);
            }
            exp = exp / 2;
            base = mulmod(base, base, mod);
        }
    }

    // ========== 0x06: BLS12-381 Point Addition ==========

    /**
     * @dev Neo BLS12-381 G1 point addition adaptation.
     * @notice This is not Ethereum's BN256/alt_bn128 ecAdd precompile.
     * Contracts using Ethereum BN256 proofs must migrate curves and encoding.
     * @param p1 First G1 point (serialized)
     * @param p2 Second G1 point (serialized)
     * @return result Sum point p1 + p2 (serialized)
     */
    function ecAdd(bytes memory p1, bytes memory p2) internal view returns (bytes memory result) {
        return Syscalls.bls12381Add(p1, p2);
    }

    // ========== 0x07: BLS12-381 Scalar Multiplication ==========

    /**
     * @dev Neo BLS12-381 G1 scalar multiplication adaptation.
     * @notice This is not Ethereum's BN256/alt_bn128 ecMul precompile.
     * @param point G1 point (serialized)
     * @param scalar Scalar multiplier (serialized as big-endian bytes)
     * @return result point * scalar (serialized)
     */
    function ecMul(bytes memory point, bytes memory scalar) internal view returns (bytes memory result) {
        return Syscalls.bls12381Mul(point, scalar, false);
    }

    // ========== 0x08: BLS12-381 Pairing Check ==========

    /**
     * @dev Neo BLS12-381 pairing adaptation.
     * @notice This is not Ethereum's BN256/alt_bn128 pairing precompile.
     * Proof systems must use Neo-compatible BLS12-381 point encodings.
     * @param g1 Serialized G1 point(s)
     * @param g2 Serialized G2 point(s)
     * @return result Pairing computation result
     */
    function ecPairing(bytes memory g1, bytes memory g2) internal view returns (bytes memory result) {
        return Syscalls.bls12381Pairing(g1, g2);
    }

    // ========== 0x09: BLAKE2f ==========

    /**
     * @dev BLAKE2f compression function.
     * @notice Not available on Neo N3. Use SHA-256 or Keccak-256 instead.
     */
    function blake2f(bytes memory) internal pure returns (bytes memory) {
        revert("Precompiles: blake2f not available on Neo N3; use sha256 or keccak256");
    }

    // ========== ZK Proof Helpers (BLS12-381) ==========

    /**
     * @dev Verify a ZK proof using BLS12-381 pairing.
     * @notice This is a simplified Groth16-style verification skeleton.
     * Actual proof verification requires properly encoded proof elements
     * and a verification key specific to your circuit. Ethereum BN256
     * verifier contracts are not plug-compatible with this helper.
     * @param proofA G1 proof element A
     * @param proofB G2 proof element B
     * @param proofC G1 proof element C
     * @param vkAlpha G1 verification key alpha
     * @param vkBeta G2 verification key beta
     * @param vkGamma G2 verification key gamma
     * @param vkDelta G2 verification key delta
     * @param publicInputsCommitment G1 commitment of public inputs
     * @return valid True if the proof is valid
     */
    function verifyGroth16Proof(
        bytes memory proofA,
        bytes memory proofB,
        bytes memory proofC,
        bytes memory vkAlpha,
        bytes memory vkBeta,
        bytes memory vkGamma,
        bytes memory vkDelta,
        bytes memory publicInputsCommitment
    ) internal view returns (bool valid) {
        // Groth16 verification equation:
        // e(A, B) = e(alpha, beta) * e(publicInputsCommitment, gamma) * e(C, delta)
        //
        // Equivalently: e(A, B) * e(-alpha, beta) * e(-publicInputsCommitment, gamma) * e(-C, delta) == 1

        // Compute pairings
        bytes memory pairingAB = Syscalls.bls12381Pairing(proofA, proofB);
        bytes memory pairingAlphaBeta = Syscalls.bls12381Pairing(vkAlpha, vkBeta);
        bytes memory pairingInputGamma = Syscalls.bls12381Pairing(publicInputsCommitment, vkGamma);
        bytes memory pairingCDelta = Syscalls.bls12381Pairing(proofC, vkDelta);

        // The verification holds if the product of all pairings is the identity
        // In practice, the pairing library handles the multi-pairing check
        // For BLS12-381, we verify: e(A,B) == e(alpha,beta) * e(inputs,gamma) * e(C,delta)
        // Using the equality check on the pairing results
        return Syscalls.bls12381Equal(
            pairingAB,
            Syscalls.bls12381Add(
                pairingAlphaBeta,
                Syscalls.bls12381Add(pairingInputGamma, pairingCDelta)
            )
        );
    }

    /**
     * @dev Negate a BLS12-381 G1 point.
     * @param point G1 point to negate
     * @return The negated point
     */
    function g1Negate(bytes memory point) internal view returns (bytes memory) {
        // Multiply by scalar 1 with negation flag
        return Syscalls.bls12381Mul(point, abi.encodePacked(uint256(1)), true);
    }

    /**
     * @dev Check if two BLS12-381 points are equal.
     */
    function pointsEqual(bytes memory a, bytes memory b) internal view returns (bool) {
        return Syscalls.bls12381Equal(a, b);
    }

    /**
     * @dev Verify an ECDSA signature using secp256k1 + Keccak256 (Ethereum-compatible).
     * @param messageHash The Keccak256 hash of the message
     * @param publicKey The signer's public key (33 or 65 bytes)
     * @param signature The ECDSA signature (64 bytes, r||s)
     * @return valid True if the signature is valid
     */
    function verifyEthSignature(
        bytes32 messageHash,
        bytes memory publicKey,
        bytes memory signature
    ) internal view returns (bool valid) {
        return Syscalls.verifyWithECDsa(
            messageHash,
            publicKey,
            signature,
            Syscalls.SECP256K1_KECCAK256
        );
    }

    /**
     * @dev Verify an ECDSA signature using secp256r1 + SHA256 (Neo N3 native).
     */
    function verifyNeoSignature(
        bytes32 messageHash,
        bytes memory publicKey,
        bytes memory signature
    ) internal view returns (bool valid) {
        return Syscalls.verifyWithECDsa(
            messageHash,
            publicKey,
            signature,
            Syscalls.SECP256R1_SHA256
        );
    }
}
