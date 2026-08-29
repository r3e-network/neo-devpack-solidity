// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title VirtualOverrideShowcase
 * @notice Demonstrates correct usage of virtual/override keywords
 *         in inheritance hierarchies for the Neo N3 Solidity compiler.
 */

abstract contract Base {
    string public name;

    constructor(string memory _name) {
        name = _name;
    }

    /// A virtual function that derived contracts can override.
    function greet() public virtual returns (string memory) {
        return "Hello from Base";
    }

    /// A non-virtual function that should NOT be overridden.
    function version() public pure returns (uint256) {
        return 1;
    }
}

interface IGreeter {
    /// Interface functions are implicitly virtual.
    function farewell() external returns (string memory);
}

contract Child is Base, IGreeter {
    constructor() Base("Child") {}

    /// Correctly overrides Base.greet() with override keyword.
    function greet() public pure virtual override returns (string memory) {
        return "Hello from Child";
    }

    /// Implements IGreeter.farewell() -- interface impl requires override.
    function farewell() external pure override returns (string memory) {
        return "Goodbye from Child";
    }
}

contract GrandChild is Child {
    /// Overrides Child.greet() which is itself an override.
    function greet() public pure override returns (string memory) {
        return "Hello from GrandChild";
    }
}
