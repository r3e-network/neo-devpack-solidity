// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title NEP-22 Contract Update Interface
 * @dev Standardized update entrypoint for upgradeable Neo N3 contracts.
 * Spec: https://github.com/neo-project/proposals/blob/master/nep-22.mediawiki
 */
interface INEP22 {
    /// @notice Replace the current contract script/manifest while preserving storage.
    function update(
        bytes calldata nefFile,
        string calldata manifest,
        bytes calldata data
    ) external;
}

/**
 * @title NEP-22 Upgradeable Base
 * @dev Lightweight reusable base that routes the NEP-22 method to an internal hook.
 */
abstract contract NEP22Upgradeable is INEP22 {
    function update(
        bytes calldata nefFile,
        string calldata manifest,
        bytes calldata data
    ) public virtual override {
        _update(nefFile, manifest, data);
    }

    function _update(
        bytes calldata nefFile,
        string calldata manifest,
        bytes calldata data
    ) internal virtual;
}
