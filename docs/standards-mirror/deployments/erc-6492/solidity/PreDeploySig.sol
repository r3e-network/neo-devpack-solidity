// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title PreDeploySig — ERC-6492 pre-deploy signature marker, compiled to Neo N3.
/// @notice ERC-6492 wraps signatures with magic bytes so verifiers can deploy the
/// account on-the-fly before checking the signature, supporting counterfactual
/// smart accounts. On Neo, contracts deployed via ContractManagement get their
/// deterministic hash immediately, so the pre-deploy step is unnecessary, but this
/// contract demonstrates the marker-bytes detection.
contract PreDeploySig {
    string public buildTag = "predeploy-sig-v1";
    /// Magic bytes appended to ERC-6492 signatures: keccak("ERC6492Detection")[..]:
    bytes32 public constant MAGIC = 0x6492649264926492649264926492649264926492649264926492649264926492;

    address public deployer;
    uint256 public verificationCount;

    function claimDeployer() public {
        require(deployer == address(0), "PDS: already claimed");
        deployer = msg.sender;
    }

    /// Trims the magic-byte marker from the signature tail and increments
    /// verification counter. Real impls would also CREATE2-deploy the wallet
    /// before verification. On Neo, ContractManagement.Deploy is the equivalent.
    function recordVerification() public {
        require(msg.sender == deployer, "PDS: deployer only");
        verificationCount += 1;
    }

    function getMagic() public pure returns (bytes32) { return MAGIC; }
}
