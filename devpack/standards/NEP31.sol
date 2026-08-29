// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title NEP-31 Contract Destroy Interface
 * @dev Standardized contract destruction entrypoint.
 * Spec: https://github.com/neo-project/proposals/blob/master/nep-31.mediawiki
 */
interface INEP31Destroyable {
    function destroy() external;
}
