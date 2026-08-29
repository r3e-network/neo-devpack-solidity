// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title SuperShowcase
 * @notice Demonstrates the internal-helper pattern as an alternative to
 *         `super` calls when you want explicit shared logic reuse.
 *
 * Workaround: extract shared logic into a named internal function that both
 * base and derived contracts can call directly.
 */

contract Base {
    uint256 public value;

    function greet() public pure virtual returns (string memory) {
        return "Base";
    }

    /// @dev Shared logic extracted into an internal helper so derived
    ///      contracts can reuse it without needing `super`.
    function _baseSetValue(uint256 v) internal {
        value = v;
    }

    function setValue(uint256 v) public virtual {
        _baseSetValue(v);
    }
}

contract Child is Base {
    function greet() public pure virtual override returns (string memory) {
        return "Child";
    }

    /// @dev Explicitly call the extracted helper directly.
    function setValue(uint256 v) public virtual override {
        _baseSetValue(v);
        // Child-specific logic can go here
    }

    function childOnly() public pure returns (string memory) {
        return "child-only";
    }
}

contract GrandChild is Child {
    function greet() public pure override returns (string memory) {
        return "GrandChild";
    }

    /// @dev Reuses the same internal helper pattern.
    function setValue(uint256 v) public override {
        _baseSetValue(v);
        // GrandChild-specific logic can go here
    }
}
