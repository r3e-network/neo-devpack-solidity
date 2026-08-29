// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title TrustedForwarder — ERC-2771 reference forwarder, compiled to Neo N3.
/// Demo intentionally avoids dynamic-method `target.call(opaque_bytes)` —
/// neo-solc requires statically known method names. Production forwarders on
/// Neo use `Contract.Call(target, methodName, callFlags, args)` directly,
/// which is what the C# version demonstrates.
contract TrustedForwarder {
    string public buildTag = "forwarder-v1";

    mapping(address => uint256) public nonces;

    event Forwarded(address indexed signer, address indexed target, uint256 nonce);

    /// Demo: bumps the signer's nonce. The C# port has the full forwarding
    /// pattern with named-method invocation via Contract.Call.
    function bumpNonce(address signer) external {
        nonces[signer]++;
        emit Forwarded(signer, address(0), nonces[signer]);
    }

    function getNonce(address signer) external view returns (uint256) {
        return nonces[signer];
    }
}
