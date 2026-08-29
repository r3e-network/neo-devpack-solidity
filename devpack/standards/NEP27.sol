// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title NEP-27 NEP-17 Receiver Callback
 * @dev Callback invoked when a contract receives NEP-17 tokens.
 * Spec: https://github.com/neo-project/proposals/blob/master/nep-27.mediawiki
 */

import "../contracts/Types.sol";

interface INEP27Receiver {
    function onNEP17Payment(
        address from,
        uint256 amount,
        Any calldata data
    ) external;
}
