// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract EvmCompatBlockErrors {
    function getCoinbase() public view returns (address) {
        return block.coinbase;
    }

    function getDifficulty() public view returns (uint256) {
        return block.difficulty;
    }

    function getGasLimit() public view returns (uint256) {
        return block.gaslimit;
    }

    function getBaseFee() public view returns (uint256) {
        return block.basefee;
    }
}
