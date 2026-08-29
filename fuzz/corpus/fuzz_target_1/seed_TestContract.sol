// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract TestContract {
    uint256 private value;
    
    event ValueChanged(uint256 newValue);
    
    constructor(uint256 _initialValue) {
        value = _initialValue;
        emit ValueChanged(_initialValue);
    }
    
    function setValue(uint256 _value) public {
        value = _value;
        emit ValueChanged(_value);
    }
    
    function getValue() public view returns (uint256) {
        return value;
    }
}
