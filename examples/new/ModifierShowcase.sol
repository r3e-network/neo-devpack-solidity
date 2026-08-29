// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title ModifierShowcase
 * @notice Demonstrates function modifiers, modifier chaining,
 *         modifiers with arguments, and the _ placeholder.
 *
 * Neo N3 adaptation: replaces EVM `payable` / `msg.value` patterns with
 * explicit amount parameters, since Neo has no "attached value" concept.
 */
contract ModifierShowcase {
    address public owner;
    bool public paused;
    uint256 public value;
    uint256 public callCount;

    constructor() {
        owner = msg.sender;
        paused = false;
    }

    modifier onlyOwner() {
        require(msg.sender == owner, "not owner");
        _;
    }

    modifier whenNotPaused() {
        require(!paused, "paused");
        _;
    }

    modifier costs(uint256 amount, uint256 minValue) {
        require(amount >= minValue, "insufficient amount");
        _;
    }

    modifier counted() {
        callCount += 1;
        _;
    }

    function pause() public onlyOwner {
        paused = true;
    }

    function unpause() public onlyOwner {
        paused = false;
    }

    /// @notice Chained modifiers: onlyOwner + whenNotPaused + counted
    function setValue(uint256 _v) public onlyOwner whenNotPaused counted {
        value = _v;
    }

    /// @notice Modifier with argument — Neo N3 uses explicit amount parameter
    function deposit(uint256 amount) public costs(amount, 1) whenNotPaused counted {
        value += amount;
    }

    function getCallCount() public view returns (uint256) {
        return callCount;
    }
}
