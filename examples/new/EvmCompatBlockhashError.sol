// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract EvmCompatBlockhashError {
    function getBlockHash(uint256 n) public view returns (bytes32) {
        return blockhash(n);
    }
}
