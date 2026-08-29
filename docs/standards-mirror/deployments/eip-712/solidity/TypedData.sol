// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title TypedData — EIP-712 domain-separator mirror, compiled to Neo N3.
/// @notice EIP-712 hashes a domain separator + struct hash, then verifies a
/// secp256k1 signature over the digest. On Neo, the canonical form is
/// CryptoLib.VerifyWithECDsa over an arbitrary digest. This Solidity-side demo
/// exposes the fixed domain hash; the C# port contains the native signature
/// verification helper.
contract TypedData {
    string public buildTag = "typed-data-v1";
    bytes32 public constant DOMAIN_SEPARATOR =
        0x47e79534a245952e8b16893a336b85a3d9ea9fa8c573f3d803afb92a79469218;

    address public deployer;

    function claimDeployer() public {
        require(deployer == address(0), "TD: already claimed");
        deployer = msg.sender;
    }

    function getDomainSeparator() public pure returns (bytes32) { return DOMAIN_SEPARATOR; }
}
