// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title ExpressionShowcase
 * @notice Demonstrates expression features: ternary, compound assignment,
 *         pre/post increment/decrement, and bitwise operations.
 */
contract ExpressionShowcase {
    uint256 public counter;
    int256 public signed_counter;

    function ternaryOp(uint256 x) public pure returns (uint256) {
        return x > 10 ? x * 2 : x + 1;
    }

    function compoundAssignment(uint256 a) public returns (uint256) {
        counter += a;
        counter -= 1;
        counter *= 2;
        counter /= 3;
        counter %= 7;
        return counter;
    }

    function preIncrement() public returns (uint256) {
        return ++counter;
    }

    function postIncrement() public returns (uint256) {
        return counter++;
    }

    function preDecrement() public returns (int256) {
        return --signed_counter;
    }

    function postDecrement() public returns (int256) {
        return signed_counter--;
    }

    function bitwiseOps(uint256 a, uint256 b) public pure returns (uint256) {
        uint256 r = a & b;
        r = r | (a ^ b);
        r = r << 2;
        r = r >> 1;
        r = ~r;
        return r;
    }

    function comparisonChain(uint256 x) public pure returns (bool) {
        return x >= 5 && x <= 100 && x != 42;
    }
}
