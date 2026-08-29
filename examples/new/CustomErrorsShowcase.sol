// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title CustomErrorsShowcase
 * @notice Demonstrates custom errors, revert with parameters, and error selectors.
 */
contract CustomErrorsShowcase {
    error Unauthorized(address caller, address required);
    error InsufficientBalance(uint256 available, uint256 requested);
    error InvalidInput(string reason);
    error ZeroAddress();

    address public owner;
    mapping(address => uint256) public balances;

    constructor() {
        owner = msg.sender;
    }

    function deposit() public {
        balances[msg.sender] += 1;
    }

    function withdraw(uint256 amount) public {
        if (amount == 0) {
            revert InvalidInput("amount must be non-zero");
        }
        uint256 bal = balances[msg.sender];
        if (bal < amount) {
            revert InsufficientBalance(bal, amount);
        }
        balances[msg.sender] = bal - amount;
    }

    function adminReset(address account) public {
        if (msg.sender != owner) {
            revert Unauthorized(msg.sender, owner);
        }
        if (account == address(0)) {
            revert ZeroAddress();
        }
        balances[account] = 0;
    }
}
