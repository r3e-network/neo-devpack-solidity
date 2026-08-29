// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title NEP-29 Deployment Callback
 * @dev Standardized Neo contract deployment/update callback signature.
 * Spec: https://github.com/neo-project/proposals/blob/master/nep-29.mediawiki
 */
interface INEP29Deployable {
    function _deploy(bytes calldata data, bool update) external;
}
