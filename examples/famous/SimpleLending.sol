// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/**
 * @title SimpleLending — Compound-style Lending Pool
 * @notice Neo N3 adaptation of Compound/Aave lending (simplified).
 *
 * Single-asset GAS lending pool with 150% collateral ratio,
 * simple per-block interest accrual, and liquidation.
 *
 * Compiler constraints respected:
 *   - No {value: ...} — uses NativeCalls.gasTransfer()
 *   - No receive()/fallback() — uses onNEP17Payment()
 *   - Import devpack via -I devpack
 */
contract SimpleLending {
    address private constant GAS_TOKEN = NativeCalls.GAS_CONTRACT;

    /// @notice Collateral ratio: 150% (scaled by 100).
    uint256 public constant COLLATERAL_RATIO = 150;

    /// @notice Liquidation bonus: 5% (scaled by 100).
    uint256 public constant LIQUIDATION_BONUS = 5;

    /// @notice Interest rate per block in basis points (1 = 0.01%).
    uint256 public constant INTEREST_RATE_PER_BLOCK = 1;
    uint256 private constant BASIS_POINTS = 10000;

    struct UserAccount {
        uint256 deposited;
        uint256 borrowed;
        uint256 lastBlock;
    }

    mapping(address => UserAccount) public accounts;
    uint256 public totalDeposited;
    uint256 public totalBorrowed;

    event Deposited(address indexed user, uint256 amount);
    event Withdrawn(address indexed user, uint256 amount);
    event Borrowed(address indexed user, uint256 amount);
    event Repaid(address indexed user, uint256 amount);
    event Liquidated(address indexed borrower, address indexed liquidator, uint256 debtRepaid, uint256 collateralSeized);

    /// @notice Tracks addresses that have signalled intent to repay.
    mapping(address => bool) private pendingRepay;

    /// @notice Signal that the next GAS transfer is a repayment (call before GAS.transfer).
    function prepareRepay() external {
        require(accounts[msg.sender].borrowed > 0, "Lending: no debt");
        pendingRepay[msg.sender] = true;
    }

    /// @notice NEP-17 callback — receives GAS deposits or repayments.
    /// Call prepareRepay() before transferring GAS to trigger repayment.
    function onNEP17Payment(address from, uint256 amount, Any calldata /*data*/) external {
        address caller = Syscalls.getCallingScriptHash();
        require(caller == GAS_TOKEN, "Lending: only GAS accepted");
        require(amount > 0, "Lending: zero amount");

        if (pendingRepay[from]) {
            pendingRepay[from] = false;
            _repay(from, amount);
        } else {
            _deposit(from, amount);
        }
    }

    function _deposit(address user, uint256 amount) internal {
        _accrueInterest(user);
        accounts[user].deposited += amount;
        totalDeposited += amount;
        emit Deposited(user, amount);
    }

    /// @notice Withdraw deposited GAS.
    function withdraw(uint256 amount) external {
        _accrueInterest(msg.sender);
        UserAccount storage acct = accounts[msg.sender];

        require(acct.deposited >= amount, "Lending: insufficient deposit");

        // Ensure remaining collateral covers debt.
        uint256 remaining = acct.deposited - amount;
        require(
            remaining * 100 >= acct.borrowed * COLLATERAL_RATIO,
            "Lending: undercollateralized"
        );

        acct.deposited -= amount;
        totalDeposited -= amount;

        bool ok = NativeCalls.gasTransfer(address(this), msg.sender, amount, "");
        require(ok, "Lending: transfer failed");

        emit Withdrawn(msg.sender, amount);
    }

    /// @notice Borrow GAS against deposited collateral.
    function borrow(uint256 amount) external {
        require(amount > 0, "Lending: zero borrow");
        _accrueInterest(msg.sender);
        UserAccount storage acct = accounts[msg.sender];

        uint256 newDebt = acct.borrowed + amount;
        require(
            acct.deposited * 100 >= newDebt * COLLATERAL_RATIO,
            "Lending: insufficient collateral"
        );

        acct.borrowed = newDebt;
        totalBorrowed += amount;

        bool ok = NativeCalls.gasTransfer(address(this), msg.sender, amount, "");
        require(ok, "Lending: borrow transfer failed");

        emit Borrowed(msg.sender, amount);
    }

    function _repay(address user, uint256 amount) internal {
        _accrueInterest(user);
        UserAccount storage acct = accounts[user];

        uint256 repayAmount = amount > acct.borrowed ? acct.borrowed : amount;
        acct.borrowed -= repayAmount;
        totalBorrowed -= repayAmount;

        emit Repaid(user, repayAmount);
    }

    /// @notice Liquidate an undercollateralized borrower.
    function liquidate(address borrower) external {
        _accrueInterest(borrower);
        UserAccount storage acct = accounts[borrower];

        require(acct.borrowed > 0, "Lending: no debt");
        require(
            acct.deposited * 100 < acct.borrowed * COLLATERAL_RATIO,
            "Lending: not undercollateralized"
        );

        uint256 debt = acct.borrowed;
        uint256 collateralSeized = debt + (debt * LIQUIDATION_BONUS) / 100;
        if (collateralSeized > acct.deposited) {
            collateralSeized = acct.deposited;
        }

        acct.borrowed = 0;
        acct.deposited -= collateralSeized;
        totalBorrowed -= debt;
        totalDeposited -= collateralSeized;

        // Transfer seized collateral to liquidator.
        bool ok = NativeCalls.gasTransfer(address(this), msg.sender, collateralSeized, "");
        require(ok, "Lending: liquidation transfer failed");

        emit Liquidated(borrower, msg.sender, debt, collateralSeized);
    }

    /// @notice Accrue simple per-block interest on a user's debt.
    function _accrueInterest(address user) internal {
        UserAccount storage acct = accounts[user];
        if (acct.borrowed > 0 && acct.lastBlock < block.number) {
            uint256 blocks = block.number - acct.lastBlock;
            uint256 interest = (acct.borrowed * INTEREST_RATE_PER_BLOCK * blocks) / BASIS_POINTS;
            acct.borrowed += interest;
            totalBorrowed += interest;
        }
        acct.lastBlock = block.number;
    }


    /// @notice View a user's current debt including accrued interest.
    function getDebt(address user) external view returns (uint256) {
        UserAccount storage acct = accounts[user];
        if (acct.borrowed == 0) return 0;
        uint256 blocks = block.number > acct.lastBlock ? block.number - acct.lastBlock : 0;
        uint256 interest = (acct.borrowed * INTEREST_RATE_PER_BLOCK * blocks) / BASIS_POINTS;
        return acct.borrowed + interest;
    }

    /// @notice View a user's collateral ratio (scaled by 100).
    function getCollateralRatio(address user) external view returns (uint256) {
        UserAccount storage acct = accounts[user];
        if (acct.borrowed == 0) return type(uint256).max;
        return (acct.deposited * 100) / acct.borrowed;
    }
}
