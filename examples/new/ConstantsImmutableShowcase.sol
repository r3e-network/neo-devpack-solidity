// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title ConstantsImmutableShowcase
 * @notice Demonstrates constant, immutable, and compile-time evaluation.
 */
contract ConstantsImmutableShowcase {
    uint256 public constant MAX_SUPPLY = 100_000_000;
    uint8 public constant DECIMALS = 8;
    string public constant TOKEN_NAME = "DemoToken";
    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN");

    address public immutable deployer;
    uint256 public immutable deployBlock;
    uint256 public immutable initialRate;

    uint256 public minted;

    constructor(uint256 _rate) {
        deployer = msg.sender;
        deployBlock = block.number;
        initialRate = _rate;
    }

    function mint(uint256 amount) public {
        require(minted + amount <= MAX_SUPPLY, "exceeds max supply");
        minted += amount;
    }

    function getInfo()
        public
        view
        returns (
            string memory name,
            uint8 decimals,
            address dep,
            uint256 rate
        )
    {
        return (TOKEN_NAME, DECIMALS, deployer, initialRate);
    }
}
