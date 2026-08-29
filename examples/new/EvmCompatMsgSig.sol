// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract EvmCompatMsgSig {
    function getSelector() public pure returns (bytes4) {
        return msg.sig;
    }
}
