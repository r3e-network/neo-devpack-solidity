// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title SmartAccount — ERC-4337 user-operation smart account, compiled to Neo N3.
/// @notice ERC-4337 introduces a separate mempool for user operations validated
/// via validateUserOp(). On Neo, smart-contract accounts are first-class via
/// NEP-30 (verify trigger), and the equivalent of validateUserOp is the contract's
/// `verify` method.
contract SmartAccount {
    string public buildTag = "smart-account-v1";

    address public owner;
    uint256 public nonce;

    function claimOwner() public {
        require(owner == address(0), "SA: already claimed");
        owner = msg.sender;
    }

    /// EVM equivalent: validateUserOp(userOp, hash, missingFunds) returns (uint256).
    /// We accept opaque userOpHash + signature placeholder; real impl would
    /// recover signer and check vs owner. Returns 0 on success per spec.
    function validateUserOp(bytes32 userOpHash, bytes calldata /*signature*/) public returns (uint256) {
        require(msg.sender == owner, "SA: bundler must be owner-witness");
        nonce += 1;
        userOpHash; // silence unused
        return 0;
    }

    function getNonce() public view returns (uint256) { return nonce; }
}
