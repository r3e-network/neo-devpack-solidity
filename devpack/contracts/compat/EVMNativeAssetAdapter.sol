// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "../Syscalls.sol";

/**
 * @title EVMNativeAssetAdapter
 * @notice Bridges EVM-style payable/msg.value flows to Neo N3 NEP-17 payment callbacks.
 * @dev Neo does not send native value through arbitrary method calls. GAS and other
 *      NEP-17 assets are received through onNEP17Payment(token sender, amount, data).
 *      Inherit this contract and override _onEVMValue to route those payments through
 *      logic that originally expected receive(), payable functions, or msg.value.
 */
abstract contract EVMNativeAssetAdapter {
    address private _lastEVMValueToken;
    address private _lastEVMValueSender;
    uint256 private _lastEVMValueAmount;
    uint256 private _lastEVMValueDataLength;

    /**
     * @notice Neo NEP-17 receiver callback.
     * @dev The NEP-17 token contract is the calling script hash on Neo.
     */
    function onNEP17Payment(address from, uint256 amount, Any calldata data) external virtual {
        address token = Syscalls.getCallingScriptHash();
        bytes memory rawData = Any.unwrap(data);

        _lastEVMValueToken = token;
        _lastEVMValueSender = from;
        _lastEVMValueAmount = amount;
        _lastEVMValueDataLength = rawData.length;

        _onEVMValue(token, from, amount, rawData);
    }

    function lastEVMValueToken() public view returns (address) {
        return _lastEVMValueToken;
    }

    function lastEVMValueSender() public view returns (address) {
        return _lastEVMValueSender;
    }

    function lastEVMValueAmount() public view returns (uint256) {
        return _lastEVMValueAmount;
    }

    function lastEVMValueDataLength() public view returns (uint256) {
        return _lastEVMValueDataLength;
    }

    function _onEVMValue(
        address token,
        address from,
        uint256 amount,
        bytes memory data
    ) internal virtual {
        token;
        from;
        amount;
        data;
    }
}
