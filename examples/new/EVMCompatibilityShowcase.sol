// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "../../devpack/contracts/compat/EVMNativeAssetAdapter.sol";
import "../../devpack/contracts/compat/EVMFallbackDispatcher.sol";
import "../../devpack/contracts/compat/EVMContractFactory.sol";

contract EVMCompatibilityShowcase is
    EVMNativeAssetAdapter,
    EVMFallbackDispatcher,
    EVMContractFactory
{
    bytes4 public constant PING_SELECTOR = bytes4(hex"11111111");

    uint256 public paymentCount;
    uint256 public fallbackCount;
    uint256 public lastFallbackResult;

    function _onEVMValue(
        address token,
        address from,
        uint256 amount,
        bytes memory data
    ) internal override {
        token;
        from;
        data;
        require(amount > 0, "zero value");
        paymentCount += 1;
    }

    function _dispatch(bytes4 selector, bytes memory data)
        internal
        override
        returns (bytes memory)
    {
        require(selector == PING_SELECTOR, "unsupported selector");
        fallbackCount += 1;
        lastFallbackResult = paymentCount + data.length + fallbackCount;
        return "";
    }

    function routePing(bytes memory data) public returns (uint256) {
        _dispatch(PING_SELECTOR, data);
        return lastFallbackResult;
    }

    function isContractLikeEVM(address account) public view returns (bool) {
        return _evmIsContract(account);
    }
}
