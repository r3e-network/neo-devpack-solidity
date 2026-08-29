// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract FixedPointError {
    fixed128x18 public price;

    function setPrice(fixed128x18 _price) public {
        price = _price;
    }

    function getPrice() public view returns (fixed128x18) {
        return price;
    }
}
