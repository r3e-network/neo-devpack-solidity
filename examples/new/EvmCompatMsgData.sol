// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract EvmCompatMsgData {
    function getData(uint256 value) public pure returns (bytes memory) {
        value;
        return msg.data;
    }
}
