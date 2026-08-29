// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title FunctionPolishShowcase
 * @notice Demonstrates payable, receive, fallback, modifiers with parameters,
 *         and function overloading (different arg counts) on Neo N3.
 *
 * Compilation produces the following expected warnings:
 *   W111 - payable modifier has no effect on Neo N3
 *   W105 - receive()/fallback() suggest onNEP17Payment
 */
contract FunctionPolishShowcase {
    address public owner;
    uint256 public balance;
    uint256 public callCount;
    bool public locked;

    event Deposited(address from, uint256 amount);
    event Withdrawn(address to, uint256 amount);

    constructor() {
        owner = msg.sender;
        locked = false;
    }

    // ── Modifiers ────────────────────────────────────────────────────

    modifier onlyOwner() {
        require(msg.sender == owner, "not owner");
        _;
    }

    modifier nonReentrant() {
        require(!locked, "reentrant call");
        locked = true;
        _;
        locked = false;
    }

    modifier minAmount(uint256 min) {
        require(msg.value >= min, "below minimum");
        _;
    }

    modifier counted() {
        callCount += 1;
        _;
    }

    // ── Payable function (W111 expected) ─────────────────────────────

    /// @notice Payable is a no-op on Neo; use onNEP17Payment instead.
    function deposit() public payable nonReentrant counted {
        balance += msg.value;
        emit Deposited(msg.sender, msg.value);
    }

    // ── receive() and fallback() (W105 expected) ─────────────────────

    receive() external payable {
        balance += msg.value;
    }

    fallback() external payable {
        balance += msg.value;
    }

    // ── Function overloading (different arg counts = OK) ─────────────

    function withdraw(uint256 amount) public onlyOwner nonReentrant {
        require(balance >= amount, "insufficient");
        balance -= amount;
        emit Withdrawn(msg.sender, amount);
    }

    function withdraw() public onlyOwner nonReentrant {
        uint256 amount = balance;
        balance = 0;
        emit Withdrawn(msg.sender, amount);
    }

    // ── Modifier with parameter + chaining ───────────────────────────

    function guardedDeposit() public payable minAmount(100) onlyOwner counted {
        balance += msg.value;
        emit Deposited(msg.sender, msg.value);
    }

    // ── View / pure helpers ──────────────────────────────────────────

    function getBalance() public view returns (uint256) {
        return balance;
    }

    function add(uint256 a, uint256 b) public pure returns (uint256) {
        return a + b;
    }
}
