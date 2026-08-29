// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title FiatTokenInheritedEvents — a factory-deployed token whose method
/// emits an event INHERITED from an interface. Models USDC FiatToken / FRAX /
/// SushiSwap MasterChef: `contract Token is IERC20` emits IERC20's `Transfer`,
/// and a factory does `new Token()`. The sibling-merge must carry events the
/// deployed contract inherits (through its full base/interface closure), not
/// just those it declares directly. (neo-solc fix 6bd58cf.)
interface IERC20Events {
    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);
}

contract FiatToken is IERC20Events {
    mapping(address => uint256) public balanceOf;

    function mint(address to, uint256 amount) external {
        balanceOf[to] += amount;
        emit Transfer(address(0), to, amount);
    }
}

contract FiatTokenFactory {
    FiatToken public token;

    constructor() {
        token = new FiatToken();
    }

    function mintTo(address to, uint256 amount) external {
        token.mint(to, amount);
    }
}
