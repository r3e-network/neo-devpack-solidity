// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/**
 * @title FlashLoan — Aave-style Flash Loan Pool
 * @notice Neo N3 adaptation of Aave V2 flash loans.
 *
 * Pool holds GAS deposits from liquidity providers.
 * flashLoan() transfers GAS to borrower, calls onFlashLoan() callback,
 * then verifies repayment + 0.09% fee.
 *
 * Compiler constraints respected:
 *   - No {value: ...} — uses NativeCalls.gasTransfer()
 *   - No receive()/fallback() — uses onNEP17Payment()
 *   - Import devpack via -I devpack
 */
contract FlashLoan {
    address private constant GAS_TOKEN = NativeCalls.GAS_CONTRACT;

    /// @notice Fee in basis points (9 = 0.09%).
    uint256 public constant FLASH_LOAN_FEE = 9;
    uint256 private constant BASIS_POINTS = 10000;

    /// @notice Liquidity provider balances.
    mapping(address => uint256) public deposits;

    /// @notice Total GAS deposited in the pool.
    uint256 public totalDeposits;

    /// @notice Accumulated fees available for distribution.
    uint256 public totalFees;

    event LiquidityDeposited(address indexed provider, uint256 amount);
    event LiquidityWithdrawn(address indexed provider, uint256 amount);
    event FlashLoanExecuted(address indexed borrower, uint256 amount, uint256 fee);

    /// @notice NEP-17 callback — receives GAS deposits from liquidity providers.
    function onNEP17Payment(address from, uint256 amount, Any calldata /*data*/) external {
        address caller = Syscalls.getCallingScriptHash();
        require(caller == GAS_TOKEN, "FlashLoan: only GAS accepted");
        require(amount > 0, "FlashLoan: zero deposit");

        deposits[from] += amount;
        totalDeposits += amount;

        emit LiquidityDeposited(from, amount);
    }

    /// @notice Withdraw deposited GAS from the pool.
    function withdraw(uint256 amount) external {
        require(deposits[msg.sender] >= amount, "FlashLoan: insufficient deposit");

        deposits[msg.sender] -= amount;
        totalDeposits -= amount;

        bool ok = NativeCalls.gasTransfer(address(this), msg.sender, amount, "");
        require(ok, "FlashLoan: transfer failed");

        emit LiquidityWithdrawn(msg.sender, amount);
    }

    /// @notice Execute a flash loan.
    /// @param borrower Contract that implements onFlashLoan(address,uint256,uint256,bytes).
    /// @param amount GAS amount to borrow.
    function flashLoan(address borrower, uint256 amount) external {
        require(amount > 0, "FlashLoan: zero amount");

        uint256 poolBalance = NativeCalls.gasBalanceOf(address(this));
        require(poolBalance >= amount, "FlashLoan: insufficient liquidity");

        uint256 fee = (amount * FLASH_LOAN_FEE) / BASIS_POINTS;

        // Transfer GAS to borrower.
        bool sent = NativeCalls.gasTransfer(address(this), borrower, amount, "");
        require(sent, "FlashLoan: loan transfer failed");

        // Call borrower's onFlashLoan callback.
        (bool success, ) = borrower.call(
            abi.encodeWithSignature(
                "onFlashLoan(address,uint256,uint256,bytes)",
                msg.sender,
                amount,
                fee,
                ""
            )
        );
        require(success, "FlashLoan: callback failed");

        // Verify repayment: pool must have original balance + fee.
        uint256 newBalance = NativeCalls.gasBalanceOf(address(this));
        require(newBalance >= poolBalance + fee, "FlashLoan: repayment insufficient");

        totalFees += fee;

        emit FlashLoanExecuted(borrower, amount, fee);
    }

    /// @notice View the current pool balance (GAS held by contract).
    function poolBalance() external view returns (uint256) {
        return NativeCalls.gasBalanceOf(address(this));
    }
}
