// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

type Price is uint256;

contract UserDefinedTypeError {
    Price public currentPrice;

    function setPrice(uint256 _price) public {
        currentPrice = Price.wrap(_price);
    }
}
