// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract Vault {
    address public immutable token;
    address public owner;

    mapping(address => uint256) public balances;

    event Deposit(address indexed from, uint256 amount);
    event Withdraw(address indexed to, uint256 amount);

    constructor() {
        token = NativeCalls.GAS_CONTRACT;
        owner = msg.sender;
    }

    function deposit(uint256 amount) external {
        require(NativeCalls.gasTransfer(msg.sender, address(this), amount, ""), "transfer failed");
    }

    function onNEP17Payment(address from, uint256 amount, bytes memory) external {
        require(msg.sender == NativeCalls.GAS_CONTRACT, "unsupported token");
        balances[from] += amount;
        emit Deposit(from, amount);
    }

    function withdraw(uint256 amount) external {
        uint256 bal = balances[msg.sender];
        require(bal >= amount, "insufficient balance");
        balances[msg.sender] = bal - amount;
        require(NativeCalls.gasTransfer(address(this), msg.sender, amount, ""), "transfer failed");
        emit Withdraw(msg.sender, amount);
    }
}
