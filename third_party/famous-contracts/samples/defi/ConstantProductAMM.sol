// SPDX-License-Identifier: MIT
// A minimal constant-product (x*y=k) automated market maker, in the style of
// Uniswap V2's core swap/mint/burn mechanics (https://github.com/Uniswap/v2-core).
// Self-contained: inlines a minimal IERC20 interface and overflow-safe math.
// Educational sample — logic follows the canonical constant-product invariant
// with a 0.3% swap fee (997/1000), but is intentionally compact (no flash swaps,
// no price oracle, no LP-token permit).
pragma solidity =0.5.16;

// --- minimal ERC20 interface for the two reserve tokens ---
interface IERC20 {
    function balanceOf(address owner) external view returns (uint);
    function transfer(address to, uint value) external returns (bool);
    function transferFrom(address from, address to, uint value) external returns (bool);
}

// --- overflow-safe math (DappHub ds-math style) ---
library SafeMath {
    function add(uint x, uint y) internal pure returns (uint z) {
        require((z = x + y) >= x, 'ds-math-add-overflow');
    }
    function sub(uint x, uint y) internal pure returns (uint z) {
        require((z = x - y) <= x, 'ds-math-sub-underflow');
    }
    function mul(uint x, uint y) internal pure returns (uint z) {
        require(y == 0 || (z = x * y) / y == x, 'ds-math-mul-overflow');
    }
}

// The pair holds two ERC20 tokens and mints/burns internal LP shares.
contract ConstantProductAMM {
    using SafeMath for uint;

    string public constant name = 'Minimal LP Token';
    string public constant symbol = 'MIN-LP';
    uint8 public constant decimals = 18;

    uint public constant MINIMUM_LIQUIDITY = 10**3;

    address public token0;
    address public token1;

    uint112 private reserve0;
    uint112 private reserve1;

    // --- LP token accounting ---
    uint public totalSupply;
    mapping(address => uint) public balanceOf;
    mapping(address => mapping(address => uint)) public allowance;

    event Transfer(address indexed from, address indexed to, uint value);
    event Approval(address indexed owner, address indexed spender, uint value);
    event Mint(address indexed sender, uint amount0, uint amount1);
    event Burn(address indexed sender, uint amount0, uint amount1, address indexed to);
    event Swap(
        address indexed sender,
        uint amount0In,
        uint amount1In,
        uint amount0Out,
        uint amount1Out,
        address indexed to
    );
    event Sync(uint112 reserve0, uint112 reserve1);

    constructor(address _token0, address _token1) public {
        require(_token0 != _token1, 'AMM: IDENTICAL_ADDRESSES');
        require(_token0 != address(0) && _token1 != address(0), 'AMM: ZERO_ADDRESS');
        token0 = _token0;
        token1 = _token1;
    }

    function getReserves() public view returns (uint112 _reserve0, uint112 _reserve1) {
        _reserve0 = reserve0;
        _reserve1 = reserve1;
    }

    // --- LP token ERC20 logic ---
    function _mintLP(address to, uint value) private {
        totalSupply = totalSupply.add(value);
        balanceOf[to] = balanceOf[to].add(value);
        emit Transfer(address(0), to, value);
    }

    function _burnLP(address from, uint value) private {
        balanceOf[from] = balanceOf[from].sub(value);
        totalSupply = totalSupply.sub(value);
        emit Transfer(from, address(0), value);
    }

    function approve(address spender, uint value) external returns (bool) {
        allowance[msg.sender][spender] = value;
        emit Approval(msg.sender, spender, value);
        return true;
    }

    function transfer(address to, uint value) external returns (bool) {
        balanceOf[msg.sender] = balanceOf[msg.sender].sub(value);
        balanceOf[to] = balanceOf[to].add(value);
        emit Transfer(msg.sender, to, value);
        return true;
    }

    function transferFrom(address from, address to, uint value) external returns (bool) {
        if (allowance[from][msg.sender] != uint(-1)) {
            allowance[from][msg.sender] = allowance[from][msg.sender].sub(value);
        }
        balanceOf[from] = balanceOf[from].sub(value);
        balanceOf[to] = balanceOf[to].add(value);
        emit Transfer(from, to, value);
        return true;
    }

    function _update(uint balance0, uint balance1) private {
        require(balance0 <= uint112(-1) && balance1 <= uint112(-1), 'AMM: OVERFLOW');
        reserve0 = uint112(balance0);
        reserve1 = uint112(balance1);
        emit Sync(reserve0, reserve1);
    }

    // Babylonian sqrt (from Uniswap v2 Math library).
    function sqrt(uint y) internal pure returns (uint z) {
        if (y > 3) {
            z = y;
            uint x = y / 2 + 1;
            while (x < z) {
                z = x;
                x = (y / x + x) / 2;
            }
        } else if (y != 0) {
            z = 1;
        }
    }

    function min(uint x, uint y) internal pure returns (uint z) {
        z = x < y ? x : y;
    }

    // Deposit both tokens (transfer them in first), receive LP shares.
    function mint(address to) external returns (uint liquidity) {
        (uint112 _reserve0, uint112 _reserve1) = getReserves();
        uint balance0 = IERC20(token0).balanceOf(address(this));
        uint balance1 = IERC20(token1).balanceOf(address(this));
        uint amount0 = balance0.sub(_reserve0);
        uint amount1 = balance1.sub(_reserve1);

        uint _totalSupply = totalSupply;
        if (_totalSupply == 0) {
            liquidity = sqrt(amount0.mul(amount1)).sub(MINIMUM_LIQUIDITY);
            _mintLP(address(0), MINIMUM_LIQUIDITY); // permanently lock minimum liquidity
        } else {
            liquidity = min(
                amount0.mul(_totalSupply) / _reserve0,
                amount1.mul(_totalSupply) / _reserve1
            );
        }
        require(liquidity > 0, 'AMM: INSUFFICIENT_LIQUIDITY_MINTED');
        _mintLP(to, liquidity);

        _update(balance0, balance1);
        emit Mint(msg.sender, amount0, amount1);
    }

    // Burn LP shares (transfer them to this contract first), receive both tokens.
    function burn(address to) external returns (uint amount0, uint amount1) {
        uint balance0 = IERC20(token0).balanceOf(address(this));
        uint balance1 = IERC20(token1).balanceOf(address(this));
        uint liquidity = balanceOf[address(this)];

        uint _totalSupply = totalSupply;
        amount0 = liquidity.mul(balance0) / _totalSupply;
        amount1 = liquidity.mul(balance1) / _totalSupply;
        require(amount0 > 0 && amount1 > 0, 'AMM: INSUFFICIENT_LIQUIDITY_BURNED');
        _burnLP(address(this), liquidity);
        require(IERC20(token0).transfer(to, amount0), 'AMM: TRANSFER_FAILED');
        require(IERC20(token1).transfer(to, amount1), 'AMM: TRANSFER_FAILED');

        balance0 = IERC20(token0).balanceOf(address(this));
        balance1 = IERC20(token1).balanceOf(address(this));

        _update(balance0, balance1);
        emit Burn(msg.sender, amount0, amount1, to);
    }

    // Swap: caller must have transferred the input token in first. Enforces the
    // constant-product invariant with a 0.3% fee (balances scaled by 1000, in by 3).
    function swap(uint amount0Out, uint amount1Out, address to) external {
        require(amount0Out > 0 || amount1Out > 0, 'AMM: INSUFFICIENT_OUTPUT_AMOUNT');
        (uint112 _reserve0, uint112 _reserve1) = getReserves();
        require(amount0Out < _reserve0 && amount1Out < _reserve1, 'AMM: INSUFFICIENT_LIQUIDITY');

        require(to != token0 && to != token1, 'AMM: INVALID_TO');
        if (amount0Out > 0) require(IERC20(token0).transfer(to, amount0Out), 'AMM: TRANSFER_FAILED');
        if (amount1Out > 0) require(IERC20(token1).transfer(to, amount1Out), 'AMM: TRANSFER_FAILED');

        uint balance0 = IERC20(token0).balanceOf(address(this));
        uint balance1 = IERC20(token1).balanceOf(address(this));

        uint amount0In = balance0 > _reserve0 - amount0Out ? balance0 - (_reserve0 - amount0Out) : 0;
        uint amount1In = balance1 > _reserve1 - amount1Out ? balance1 - (_reserve1 - amount1Out) : 0;
        require(amount0In > 0 || amount1In > 0, 'AMM: INSUFFICIENT_INPUT_AMOUNT');

        uint balance0Adjusted = balance0.mul(1000).sub(amount0In.mul(3));
        uint balance1Adjusted = balance1.mul(1000).sub(amount1In.mul(3));
        require(
            balance0Adjusted.mul(balance1Adjusted) >= uint(_reserve0).mul(_reserve1).mul(1000**2),
            'AMM: K'
        );

        _update(balance0, balance1);
        emit Swap(msg.sender, amount0In, amount1In, amount0Out, amount1Out, to);
    }
}
