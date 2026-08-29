// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title NEP-30 Verification Callback
 * @dev Standardized contract-level verification entrypoint.
 * Spec: https://github.com/neo-project/proposals/blob/master/nep-30.mediawiki
 */
interface INEP30Verifiable {
    function verify() external returns (bool);
}
