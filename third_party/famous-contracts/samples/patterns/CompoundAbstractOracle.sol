// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title CompoundAbstractOracle — concrete contract holding an abstract-typed
/// field. Models Compound v2's CToken/Comptroller, which hold `PriceOracle` /
/// `InterestRateModel` (abstract contract) fields. A contract that merely
/// REFERENCES an abstract type must NOT be forced to implement its virtuals.
/// (neo-solc fix a335c87.)
abstract contract PriceOracle {
    function getUnderlyingPrice(address cToken) external view virtual returns (uint256);
}

abstract contract InterestRateModel {
    function getBorrowRate(uint256 cash, uint256 borrows, uint256 reserves)
        external view virtual returns (uint256);
}

contract Comptroller {
    PriceOracle public oracle;
    InterestRateModel public interestRateModel;

    function priceOf(address cToken) external view returns (uint256) {
        return oracle.getUnderlyingPrice(cToken);
    }

    function borrowRate(uint256 cash, uint256 borrows, uint256 reserves)
        external view returns (uint256)
    {
        return interestRateModel.getBorrowRate(cash, borrows, reserves);
    }
}
