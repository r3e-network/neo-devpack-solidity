// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

interface IFoo {
    function bar(uint256 x) external returns (bool);
}

contract EvmCompatEncodeCalldata {
    function withSignature(uint256 value) public pure returns (bytes memory) {
        return abi.encodeWithSignature("bar(uint256)", value);
    }

    function withSelector(uint256 value) public pure returns (bytes memory) {
        return abi.encodeWithSelector(IFoo.bar.selector, value);
    }
}
