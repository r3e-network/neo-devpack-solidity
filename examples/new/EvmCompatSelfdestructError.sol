// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract EvmCompatSelfdestructError {
    function destroy(address payable recipient) public {
        selfdestruct(recipient);
    }
}
