// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

error InsufficientBalance(uint256 available, uint256 required);
error Unauthorized();
error InvalidAmount();

/**
 * @title RequireCustomErrorShowcase
 * @notice Demonstrates `require(condition, CustomError(...))` syntax
 *         (Solidity 0.8.26+). On NeoVM the error name and arg count are
 *         preserved in the THROW message for diagnostics.
 */
contract RequireCustomErrorShowcase {
    mapping(address => uint256) private balances;
    address private owner;

    constructor() {
        owner = msg.sender;
    }

    function deposit(uint256 amount) public {
        require(amount > 0, InvalidAmount());
        balances[msg.sender] += amount;
    }

    function withdraw(uint256 amount) public {
        require(amount > 0, InvalidAmount());
        require(
            balances[msg.sender] >= amount,
            InsufficientBalance(balances[msg.sender], amount)
        );
        balances[msg.sender] -= amount;
    }

    function adminReset(address account) public {
        require(msg.sender == owner, Unauthorized());
        balances[account] = 0;
    }

    function getBalance(address account) public view returns (uint256) {
        return balances[account];
    }
}
