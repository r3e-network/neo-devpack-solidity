// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract Counter {
    uint256 private value;

    event Incremented(address indexed caller, uint256 newValue);
    event Decremented(address indexed caller, uint256 newValue);

    constructor(uint256 initial) {
        value = initial;
    }

    function get() public view returns (uint256) {
        return value;
    }

    function inc() public {
        value += 1;
        emit Incremented(msg.sender, value);
    }

    function dec() public {
        require(value > 0, "underflow");
        value -= 1;
        emit Decremented(msg.sender, value);
    }
}
