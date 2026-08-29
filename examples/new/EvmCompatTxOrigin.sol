// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract EvmCompatTxOrigin {
    function getOrigin() public view returns (address) {
        return tx.origin;
    }
}
