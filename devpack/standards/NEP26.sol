// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title NEP-26 NEP-11 Receiver Callback
 * @dev Callback invoked when a contract receives NEP-11 tokens.
 * Spec: https://github.com/neo-project/proposals/blob/master/nep-26.mediawiki
 */
interface INEP26Receiver {
    // NEP-11 token IDs are ByteString (up to 64 bytes), so `tokenId` is a
    // dynamic `bytes`, matching INEP11Receiver in NEP11.sol and what the
    // devpack's NEP-11 base actually passes — NOT a fixed `bytes32`, which
    // would mis-encode any non-32-byte id.
    function onNEP11Payment(
        address from,
        uint256 amount,
        bytes calldata tokenId,
        bytes calldata data
    ) external;
}
