// SPDX-License-Identifier: MIT
pragma solidity ^0.8.10;

/// @title AaveLibraryEvents — library-declared event, inlined into the host.
/// @notice Models Aave v3's `ReserveLogic` pattern: a library declares its own
/// event (a copy of the pool interface's) and emits it unqualified. neo-solc
/// inlines library helpers into the consuming contract; this sample verifies
/// the library's event declaration is carried into the host's manifest so the
/// notification validates on Neo N3 (>= 3.6). (neo-solc fix 8fe413b.)
library ReserveLogic {
    event ReserveDataUpdated(address indexed reserve, uint256 liquidityRate, uint256 variableBorrowRate);

    function updateInterestRates(address reserve, uint256 liq, uint256 borrow) internal {
        emit ReserveDataUpdated(reserve, liq, borrow);
    }
}

contract Pool {
    using ReserveLogic for address;

    mapping(address => uint256) public liquidityRate;

    function setRate(address reserve, uint256 liq, uint256 borrow) external {
        liquidityRate[reserve] = liq;
        ReserveLogic.updateInterestRates(reserve, liq, borrow);
    }
}
