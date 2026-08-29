// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

library BadLib {
    uint256 counter;

    function increment() internal returns (uint256) {
        counter += 1;
        return counter;
    }
}

contract User {
    function use_lib() public returns (uint256) {
        return BadLib.increment();
    }
}
