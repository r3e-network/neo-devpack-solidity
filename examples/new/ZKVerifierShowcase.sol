// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "../../devpack/libraries/Precompiles.sol";
import "../../devpack/contracts/Syscalls.sol";

/**
 * @title ZKVerifierShowcase
 * @notice Demonstrates zero-knowledge proof verification on Neo N3 using
 *         BLS12-381 curve operations (CryptoLib native contract).
 *
 * This contract shows how ZK-SNARK verification works on Neo N3:
 *   - BLS12-381 pairing checks (Groth16-style)
 *   - Point arithmetic (add, mul, negate)
 *   - Proof element handling
 *   - Public input commitment computation
 *
 * @dev Neo N3 uses BLS12-381 (same as Ethereum 2.0 / Zcash Sapling).
 *      EVM uses BN254 (alt_bn128). Proof systems targeting Neo N3 must
 *      generate proofs for BLS12-381 instead of BN254.
 */
contract ZKVerifierShowcase {
    // Verification key (set by deployer, circuit-specific)
    bytes public vkAlpha;
    bytes public vkBeta;
    bytes public vkGamma;
    bytes public vkDelta;
    bytes[] public vkIC; // IC[0..n] for public input linearization

    address public owner;
    bool public initialized;

    // Proof verification results
    mapping(bytes32 => bool) public verifiedProofs;
    uint256 public totalVerified;

    event ProofVerified(bytes32 indexed proofHash, bool valid);
    event VerificationKeySet(address indexed setter);

    error NotOwner();
    error NotInitialized();
    error AlreadyInitialized();
    error InvalidProof();

    modifier onlyOwner() {
        if (msg.sender != owner) revert NotOwner();
        _;
    }

    constructor() {
        owner = msg.sender;
    }

    /**
     * @dev Set the verification key for a specific circuit.
     * @notice Must be called before any proofs can be verified.
     */
    function setVerificationKey(
        bytes calldata _vkAlpha,
        bytes calldata _vkBeta,
        bytes calldata _vkGamma,
        bytes calldata _vkDelta,
        bytes[] calldata _vkIC
    ) external onlyOwner {
        if (initialized) revert AlreadyInitialized();

        vkAlpha = _vkAlpha;
        vkBeta = _vkBeta;
        vkGamma = _vkGamma;
        vkDelta = _vkDelta;

        delete vkIC;
        for (uint256 i = 0; i < _vkIC.length; i++) {
            vkIC.push(_vkIC[i]);
        }

        initialized = true;
        emit VerificationKeySet(msg.sender);
    }

    /**
     * @dev Verify a Groth16-style ZK proof on BLS12-381.
     * @param proofA G1 element of the proof
     * @param proofB G2 element of the proof
     * @param proofC G1 element of the proof
     * @param publicInputs Array of public inputs (field elements as bytes)
     * @return valid True if the proof verifies correctly
     */
    function verifyProof(
        bytes calldata proofA,
        bytes calldata proofB,
        bytes calldata proofC,
        bytes[] calldata publicInputs
    ) external returns (bool valid) {
        if (!initialized) revert NotInitialized();
        require(publicInputs.length + 1 == vkIC.length, "ZK: wrong number of public inputs");

        // Compute public input commitment: vkIC[0] + sum(publicInputs[i] * vkIC[i+1])
        bytes memory commitment = vkIC[0];
        for (uint256 i = 0; i < publicInputs.length; i++) {
            bytes memory term = Precompiles.ecMul(vkIC[i + 1], publicInputs[i]);
            commitment = Precompiles.ecAdd(commitment, term);
        }

        // Verify pairing equation
        valid = Precompiles.verifyGroth16Proof(
            proofA, proofB, proofC,
            vkAlpha, vkBeta, vkGamma, vkDelta,
            commitment
        );

        // Record result
        bytes32 proofHash = keccak256(abi.encode(proofA, proofB, proofC));
        verifiedProofs[proofHash] = valid;
        if (valid) totalVerified++;

        emit ProofVerified(proofHash, valid);
    }

    /**
     * @dev Check if a proof has been previously verified.
     */
    function isProofVerified(bytes32 proofHash) external view returns (bool) {
        return verifiedProofs[proofHash];
    }

    // ========== BLS12-381 Primitive Tests ==========

    function testPointAdd(bytes calldata a, bytes calldata b) external view returns (bytes memory) {
        return Precompiles.ecAdd(a, b);
    }

    function testScalarMul(bytes calldata point, bytes calldata scalar) external view returns (bytes memory) {
        return Precompiles.ecMul(point, scalar);
    }

    function testPairing(bytes calldata g1, bytes calldata g2) external view returns (bytes memory) {
        return Precompiles.ecPairing(g1, g2);
    }

    function testPointNegate(bytes calldata point) external view returns (bytes memory) {
        return Precompiles.g1Negate(point);
    }

    function testPointsEqual(bytes calldata a, bytes calldata b) external view returns (bool) {
        return Precompiles.pointsEqual(a, b);
    }
}
