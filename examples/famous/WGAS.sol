// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/**
 * @title WGAS — Wrapped GAS (NEP-17)
 * @notice Neo N3 adaptation of WETH9, the most deployed contract on Ethereum.
 *
 * Deposits GAS via onNEP17Payment and mints 1:1 WGAS tokens.
 * Withdrawals burn WGAS and return GAS via NativeCalls.gasTransfer().
 *
 * Compiler constraints respected:
 *   - No inline assembly
 *   - No {value: ...} — uses NativeCalls.gasTransfer()
 *   - No receive()/fallback() — uses onNEP17Payment() callback
 *   - Import devpack via -I devpack
 *
 * Current limitation in this repository:
 *   - Multi-holder balance mapping is not enabled in Neo-Express smoke environment
 *     because `mapping`/array storage paths still depend on `keccak256` runtime support.
 *   - This WGAS sample therefore uses a single-holder ledger model to keep deployment
 *     and end-to-end tests deterministic.
 */
contract WGAS {
    address private constant GAS_TOKEN = NativeCalls.GAS_CONTRACT;

    string public name;
    string public symbol;
    uint8 public decimals;

    uint256 public totalSupply;
    address private holder;
    uint256 private holderBalance;

    address private allowanceOwner;
    address private allowanceSpender;
    uint256 private allowanceAmount;

    event Deposit(address indexed from, uint256 amount);
    event Withdrawal(address indexed to, uint256 amount);
    event Transfer(address indexed from, address indexed to, uint256 amount);
    event Approval(address indexed owner, address indexed spender, uint256 amount);

    constructor() {
        name = "Wrapped GAS";
        symbol = "WGAS";
        decimals = 8;
    }

    /// @notice NEP-17 callback — receives GAS deposits and mints WGAS 1:1.
    function onNEP17Payment(address from, uint256 amount, Any calldata /*data*/) external {
        address caller = Syscalls.getCallingScriptHash();
        require(caller == GAS_TOKEN, "WGAS: only GAS accepted");
        require(amount > 0, "WGAS: zero deposit");

        if (holder == address(0)) {
            holder = from;
        } else {
            require(holder == from, "WGAS: multi-holder unsupported");
        }
        holderBalance += amount;
        totalSupply += amount;

        emit Deposit(from, amount);
        emit Transfer(address(0), from, amount);
    }

    /// @notice Burn WGAS and withdraw GAS back to caller.
    function withdraw(uint256 amount) external {
        require(msg.sender == holder, "WGAS: unsupported holder");
        require(holderBalance >= amount, "WGAS: insufficient balance");

        holderBalance -= amount;
        if (holderBalance == 0) {
            holder = address(0);
        }
        totalSupply -= amount;

        bool ok = NativeCalls.gasTransfer(address(this), msg.sender, amount, "");
        require(ok, "WGAS: GAS transfer failed");

        emit Withdrawal(msg.sender, amount);
        emit Transfer(msg.sender, address(0), amount);
    }

    /// @notice NEP-17 standard transfer (4-parameter signature).
    function transfer(address from, address to, uint256 amount, Any calldata data) external returns (bool) {
        data;
        if (!Runtime.checkWitness(from)) {
            return false;
        }
        if (from == address(0) || to == address(0)) {
            return false;
        }

        if (amount == 0 || from == to) {
            emit Transfer(from, to, amount);
            return true;
        }
        if (from != holder || holderBalance < amount) {
            return false;
        }
        if (amount != holderBalance) {
            return false;
        }

        holder = to;
        emit Transfer(from, to, amount);
        return true;
    }

    function balanceOf(address account) public view returns (uint256) {
        if (account == holder) {
            return holderBalance;
        }
        return 0;
    }

    /// @notice Approve spender to transfer WGAS on behalf of caller.
    function approve(address spender, uint256 amount) external returns (bool) {
        allowanceOwner = msg.sender;
        allowanceSpender = spender;
        allowanceAmount = amount;
        emit Approval(msg.sender, spender, amount);
        return true;
    }

    function allowance(address owner, address spender) public view returns (uint256) {
        if (owner == allowanceOwner && spender == allowanceSpender) {
            return allowanceAmount;
        }
        return 0;
    }

    /// @notice Transfer WGAS from one address to another (requires allowance).
    function transferFrom(address from, address to, uint256 amount) external returns (bool) {
        uint256 allowed = allowance(from, msg.sender);
        require(allowed >= amount, "WGAS: allowance exceeded");
        require(from != address(0), "WGAS: from zero address");
        require(to != address(0), "WGAS: to zero address");
        require(from == holder, "WGAS: unsupported holder");
        require(holderBalance >= amount, "WGAS: insufficient balance");

        if (amount == 0 || from == to) {
            emit Transfer(from, to, amount);
            return true;
        }
        require(amount == holderBalance, "WGAS: partial transfer unsupported");

        allowanceAmount = allowed - amount;
        holder = to;

        emit Transfer(from, to, amount);
        return true;
    }
}
