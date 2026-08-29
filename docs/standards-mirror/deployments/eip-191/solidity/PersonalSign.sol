// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title PersonalSign — EIP-191 personal_sign prefix mirror, compiled to Neo N3.
/// @notice EIP-191 prefixes messages with "\x19Ethereum Signed Message:\n" + length
/// before signing. On Neo, off-chain signatures are usually verified via
/// CryptoLib.VerifyWithECDsa (secp256r1). This Solidity-side demo stores the
/// signer identity and exposes the EIP-191 prefix; the C# port contains the native
/// signature verification helper.
contract PersonalSign {
    string public buildTag = "personal-sign-v1";
    bytes19 public constant EIP191_PREFIX = bytes19(0x19457468657265756d205369676e6564204d65);

    address public deployer;
    address public registeredSigner;

    function claimDeployer() public {
        require(deployer == address(0), "PS: already claimed");
        deployer = msg.sender;
    }

    function setRegisteredSigner(address signer) public {
        require(msg.sender == deployer, "PS: deployer only");
        registeredSigner = signer;
    }

    function getRegisteredSigner() public view returns (address) { return registeredSigner; }
    function prefixLength() public pure returns (uint256) { return 19; }
}
