// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title EVMFallbackDispatcher
 * @notice Explicit fallback routing for contracts ported from EVM semantics.
 * @dev Neo N3 does not route unknown method selectors to an implicit fallback entrypoint.
 *      Expose dispatch(selector, data) and let callers or adapters pass the selector
 *      intentionally. This keeps the ABI manifest explicit and auditable.
 */
abstract contract EVMFallbackDispatcher {
    function dispatch(bytes4 selector, bytes calldata data)
        external
        virtual
        returns (bytes memory result)
    {
        result = _dispatch(selector, data);
    }

    function _dispatch(bytes4 selector, bytes memory data)
        internal
        virtual
        returns (bytes memory result)
    {
        selector;
        data;
        revert("EVM fallback selector unsupported");
    }
}
