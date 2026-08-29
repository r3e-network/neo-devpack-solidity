// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/// @title Bank - A simple bank contract demonstrating storage, events, and access control
/// @notice Shows how to build a basic banking contract with deposits, withdrawals, and balance tracking
contract Bank {
    mapping(address => uint256) private balances;
    uint256 public totalDeposits;
    address public contractOwner;

    event Deposit(address indexed account, uint256 amount, uint256 newBalance);
    event Withdrawal(address indexed account, uint256 amount, uint256 newBalance);
    event Transfer(address indexed from, address indexed to, uint256 amount);

    /// @notice Constructor sets the owner
    constructor() {
        contractOwner = msg.sender;
    }

    /// @notice Deposit funds to the bank
    /// @param amount Amount to deposit (in smallest units)
    function deposit(uint256 amount) public {
        require(amount > 0, "Deposit amount must be positive");
        balances[msg.sender] += amount;
        totalDeposits += amount;
        emit Deposit(msg.sender, amount, balances[msg.sender]);
    }

    /// @notice Withdraw funds from the bank
    /// @param amount Amount to withdraw
    function withdraw(uint256 amount) public {
        require(amount > 0, "Withdrawal amount must be positive");
        require(balances[msg.sender] >= amount, "Insufficient balance");
        balances[msg.sender] -= amount;
        totalDeposits -= amount;
        emit Withdrawal(msg.sender, amount, balances[msg.sender]);
    }

    /// @notice Get the balance of an account
    /// @param account Address to check
    /// @return The account balance
    function getBalance(address account) public view returns (uint256) {
        return balances[account];
    }

    /// @notice Get the caller's balance
    /// @return The caller's balance
    function myBalance() public view returns (uint256) {
        return balances[msg.sender];
    }

    /// @notice Transfer funds to another account (NEP-17 style)
    /// @param from Sender address (must match witness)
    /// @param to Recipient address
    /// @param amount Amount to transfer
    /// @param data Arbitrary transfer payload for callback compatibility
    function transfer(address from, address to, uint256 amount, bytes memory data) public returns (bool) {
        data;
        require(from == msg.sender, "from must be caller");
        require(amount > 0, "Transfer amount must be positive");
        require(balances[from] >= amount, "Insufficient balance");
        require(to != address(0), "Cannot transfer to zero address");

        balances[from] -= amount;
        balances[to] += amount;
        emit Transfer(from, to, amount);
        return true;
    }

    /// @notice Get total deposits in the bank
    /// @return Total of all deposits
    function getTotalDeposits() public view returns (uint256) {
        return totalDeposits;
    }

    /// @notice Get contract owner
    /// @return The owner address
    function getOwner() public view returns (address) {
        return contractOwner;
    }

    /// @notice Emergency function to drain all funds (owner only)
    /// @param recipient Where to send the funds
    function emergencyDrain(address recipient) public {
        require(msg.sender == contractOwner, "Only owner can drain");
        require(recipient != address(0), "Cannot drain to zero address");
        uint256 amount = balances[address(this)];
        balances[address(this)] = 0;
        totalDeposits = 0;
        balances[recipient] += amount;
    }

    /// @notice Helper to check if caller is owner
    /// @return True if caller is owner
    function isOwner() public view returns (bool) {
        return msg.sender == contractOwner;
    }
}

/// Usage:
/// 1. Compile: neo-solc Bank.sol -I devpack -O2 -o build/Bank
/// 2. Deploy: neo-cli contract deploy build/Bank.nef build/Bank.manifest.json --network testnet
/// 3. Interact:
///    - deposit(1000): neo-cli contract invoke <hash> deposit '[1000]'
///    - myBalance(): neo-cli contract invoke <hash> myBalance
///    - transfer(0x123..., 500): neo-cli contract invoke <hash> transfer '["0x123...", 500]'
///    - getOwner(): neo-cli contract invoke <hash> getOwner
