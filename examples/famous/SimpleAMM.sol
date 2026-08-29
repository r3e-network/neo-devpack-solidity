// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/**
 * @title SimpleAMM — Constant-Product Automated Market Maker
 * @notice Neo N3 adaptation of Uniswap V2 (simplified).
 *
 * Two-token pool using storage-tracked balances.
 * Implements x*y=k invariant for swaps, LP share minting/burning.
 *
 * Compiler constraints respected:
 *   - No inline assembly — uses library math
 *   - All arithmetic in uint256
 *   - Import devpack via -I devpack
 */
contract SimpleAMM {
    address public tokenA;
    address public tokenB;

    uint256 public reserveA;
    uint256 public reserveB;

    uint256 public totalShares;
    mapping(address => uint256) public shares;

    /// @notice Swap fee: 0.3% (30 basis points).
    uint256 public constant SWAP_FEE = 30;
    uint256 private constant BASIS_POINTS = 10000;

    event LiquidityAdded(address indexed provider, uint256 amountA, uint256 amountB, uint256 sharesMinted);
    event LiquidityRemoved(address indexed provider, uint256 amountA, uint256 amountB, uint256 sharesBurned);
    event Swap(address indexed trader, address tokenIn, uint256 amountIn, uint256 amountOut);

    bool private initialized;

    constructor() {}

    /// @notice Initialize the pool with two token addresses. Can only be called once.
    function initialize(address _tokenA, address _tokenB) external {
        require(!initialized, "AMM: already initialized");
        require(_tokenA != address(0), "AMM: zero tokenA");
        require(_tokenB != address(0), "AMM: zero tokenB");
        require(_tokenA != _tokenB, "AMM: identical tokens");
        tokenA = _tokenA;
        tokenB = _tokenB;
        initialized = true;
    }

    /// @notice Add liquidity to the pool. First deposit sets the ratio.
    function addLiquidity(uint256 amountA, uint256 amountB) external returns (uint256 minted) {
        require(amountA > 0 && amountB > 0, "AMM: zero amounts");

        if (totalShares == 0) {
            // First deposit — use geometric mean for initial shares.
            minted = _sqrt(amountA * amountB);
            require(minted > 0, "AMM: insufficient initial liquidity");
        } else {
            // Proportional deposit.
            uint256 shareA = (amountA * totalShares) / reserveA;
            uint256 shareB = (amountB * totalShares) / reserveB;
            minted = shareA < shareB ? shareA : shareB;
        }

        reserveA += amountA;
        reserveB += amountB;
        shares[msg.sender] += minted;
        totalShares += minted;

        emit LiquidityAdded(msg.sender, amountA, amountB, minted);
    }

    /// @notice Remove liquidity by burning LP shares.
    function removeLiquidity(uint256 shareAmount) external returns (uint256 amountA, uint256 amountB) {
        require(shareAmount > 0, "AMM: zero shares");
        require(shares[msg.sender] >= shareAmount, "AMM: insufficient shares");

        amountA = (shareAmount * reserveA) / totalShares;
        amountB = (shareAmount * reserveB) / totalShares;
        require(amountA > 0 && amountB > 0, "AMM: insufficient liquidity burned");

        shares[msg.sender] -= shareAmount;
        totalShares -= shareAmount;
        reserveA -= amountA;
        reserveB -= amountB;

        emit LiquidityRemoved(msg.sender, amountA, amountB, shareAmount);
    }

    /// @notice Swap tokenIn for the other token using constant-product formula.
    function swap(address tokenIn, uint256 amountIn) external returns (uint256 amountOut) {
        require(amountIn > 0, "AMM: zero input");
        require(tokenIn == tokenA || tokenIn == tokenB, "AMM: invalid token");

        bool isA = (tokenIn == tokenA);
        uint256 resIn = isA ? reserveA : reserveB;
        uint256 resOut = isA ? reserveB : reserveA;

        // Apply fee: amountInWithFee = amountIn * (10000 - 30) / 10000
        uint256 amountInWithFee = (amountIn * (BASIS_POINTS - SWAP_FEE)) / BASIS_POINTS;

        // Constant product: amountOut = resOut * amountInWithFee / (resIn + amountInWithFee)
        amountOut = (resOut * amountInWithFee) / (resIn + amountInWithFee);
        require(amountOut > 0, "AMM: insufficient output");

        if (isA) {
            reserveA += amountIn;
            reserveB -= amountOut;
        } else {
            reserveB += amountIn;
            reserveA -= amountOut;
        }

        emit Swap(msg.sender, tokenIn, amountIn, amountOut);
    }

    /// @notice Get the spot price of a token in terms of the other.
    function getPrice(address token) external view returns (uint256) {
        require(token == tokenA || token == tokenB, "AMM: invalid token");
        if (token == tokenA) {
            require(reserveA > 0, "AMM: no reserveA");
            return (reserveB * 1e8) / reserveA;
        } else {
            require(reserveB > 0, "AMM: no reserveB");
            return (reserveA * 1e8) / reserveB;
        }
    }

    /// @notice Integer square root (Babylonian method).
    function _sqrt(uint256 x) internal pure returns (uint256 y) {
        if (x == 0) return 0;
        y = x;
        uint256 z = (x + 1) / 2;
        while (z < y) {
            y = z;
            z = (x / z + z) / 2;
        }
    }
}
