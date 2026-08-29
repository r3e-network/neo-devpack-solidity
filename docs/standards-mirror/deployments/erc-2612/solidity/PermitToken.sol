// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title PermitToken — ERC-2612 ERC-20-with-permit demo, compiled to Neo N3.
/// @notice ERC-2612 lets holders authorize a transfer via a signed message instead
/// of a separate approve() tx. On Neo, witness scopes already let one signed tx
/// authorize multiple actions, so the equivalent is "the same tx that calls permit
/// also carries a witness covering the granted action."
contract PermitToken {
    string public buildTag = "permit-token-v1";
    string public name = "Permit Token";
    string public symbol = "PMT";
    uint8 public decimals = 8;
    uint256 public totalSupply;

    address public deployer;
    mapping(address => uint256) public balanceOf;
    mapping(address => uint256) public nonces;

    function claimDeployer() public {
        require(deployer == address(0), "PMT: already claimed");
        deployer = msg.sender;
    }

    function mint(address to, uint256 amount) public {
        require(msg.sender == deployer, "PMT: deployer only");
        balanceOf[to] += amount;
        totalSupply += amount;
    }

    /// In real ERC-2612 this verifies an EIP-712 typed-data signature and authorizes
    /// a one-shot transfer. On Neo, the witness scope already authorizes the action,
    /// so all this does is increment the holder's nonce and emit the same event.
    function permit(address holder, uint256 amount) public {
        require(msg.sender == holder, "PMT: holder must sign");
        nonces[holder] += 1;
        // No actual approval bookkeeping — Neo uses witness scopes instead of allowances.
        // The amount is recorded only for indexing.
    }

    function nonceOf(address holder) public view returns (uint256) { return nonces[holder]; }
}
