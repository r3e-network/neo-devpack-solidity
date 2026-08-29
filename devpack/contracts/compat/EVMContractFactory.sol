// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "../NativeCalls.sol";

/**
 * @title EVMContractFactory
 * @notice Neo-native replacements for CREATE/CREATE2/update/selfdestruct-style flows.
 * @dev Neo deploys, updates, and destroys contracts through the ContractManagement
 *      native contract. CREATE2 deterministic address semantics do not exist on Neo;
 *      salts are accepted as application metadata rather than used to derive hashes.
 */
abstract contract EVMContractFactory {
    function _deployLikeCreate(bytes memory nef, bytes memory manifest)
        internal
        returns (address contractHash)
    {
        contractHash = NativeCalls.deployContract(nef, manifest);
    }

    function _deployLikeCreate(
        bytes memory nef,
        bytes memory manifest,
        bytes memory data,
        bytes32 salt,
        bytes32 labelHash
    ) internal returns (address contractHash) {
        salt;
        labelHash;
        contractHash = NativeCalls.deployContract(nef, manifest, data);
    }

    function _updateLikeProxyUpgrade(
        bytes memory nef,
        bytes memory manifest,
        bytes memory data,
        bytes32 labelHash
    ) internal {
        labelHash;
        NativeCalls.updateContract(nef, manifest, data);
    }

    function _destroyLikeSelfdestruct() internal {
        NativeCalls.destroyContract();
    }

    function _evmIsContract(address account) internal view returns (bool) {
        return NativeCalls.isContract(account);
    }
}
