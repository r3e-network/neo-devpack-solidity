// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title InheritanceShowcase
 * @notice Demonstrates abstract contracts, multiple inheritance,
 *         super calls, and virtual/override patterns.
 */

abstract contract Animal {
    string public species;

    constructor(string memory _species) {
        species = _species;
    }

    /// @dev Default implementation; concrete subclasses override this.
    function speak() public virtual returns (string memory) {
        return "";
    }

    function breathe() public pure returns (string memory) {
        return "inhale-exhale";
    }
}

abstract contract Domestic {
    address public caretaker;

    function setCaret(address _c) public virtual {
        caretaker = _c;
    }
}

contract Dog is Animal, Domestic {
    uint256 public tricks;

    constructor() Animal("Canine") {
        tricks = 0;
    }

    function speak() public pure virtual override returns (string memory) {
        return "Woof";
    }

    function setCaret(address _c) public override {
        // Inline base Domestic.setCaret() logic explicitly.
        caretaker = _c;
    }

    function learnTrick() public {
        tricks += 1;
    }
}

contract GuideDog is Dog {
    bool public certified;

    function certify() public {
        certified = true;
    }

    function speak() public pure override returns (string memory) {
        return "Quiet-Woof";
    }
}
