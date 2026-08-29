// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title DemoToken — ERC-20 reference, compiled to Neo N3 via neo-solc.
///        Lives on Neo testnet as part of the standards-mirror deployment kit.
contract DemoToken {
    string public buildTag = "demoToken-v3-testnet-mirror";   // ensures distinct bytecode
    string public name = "Demo Solidity Token";
    string public symbol = "DEMOSOL";
    uint8  public decimals = 8;
    uint256 public totalSupply;

    mapping(address => uint256)                     public balanceOf;
    mapping(address => mapping(address => uint256)) public allowance;

    /// NOTE: events removed for testnet deployability. neo-solc 0.18 emits the
    /// keccak256 of the ERC-20 event signature as a topic, and that hash starts
    /// with byte 0xDD which the runtime treats as a non-UTF-8 string and faults.
    /// The state changes work; we just don't emit Transfer/Approval here.
    /// In real deployments the corresponding NEP-17 contract emits a NEP-17
    /// `Transfer` notification (different signature, no keccak required).

    /// @notice Demo-only faucet: anyone can mint to anyone.
    function faucet(address to, uint256 amount) public {
        balanceOf[to] += amount;
        totalSupply += amount;
    }

    function transfer(address to, uint256 amount) public returns (bool) {
        _transfer(msg.sender, to, amount);
        return true;
    }

    function approve(address spender, uint256 amount) public returns (bool) {
        allowance[msg.sender][spender] = amount;
        return true;
    }

    function transferFrom(address from, address to, uint256 amount) public returns (bool) {
        uint256 allowed = allowance[from][msg.sender];
        require(allowed >= amount, "ERC20: insufficient allowance");
        if (allowed != type(uint256).max) {
            allowance[from][msg.sender] = allowed - amount;
        }
        _transfer(from, to, amount);
        return true;
    }

    function _transfer(address from, address to, uint256 amount) internal {
        require(to != address(0),          "ERC20: zero recipient");
        require(balanceOf[from] >= amount, "ERC20: insufficient balance");
        unchecked {
            balanceOf[from] -= amount;
            balanceOf[to]   += amount;
        }
    }
}
