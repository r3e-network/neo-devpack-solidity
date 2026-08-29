// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract EvmCompatAddressCodehash {
    function getCodeHash(address addr) public view returns (bytes32) {
        return addr.codehash;
    }
}
