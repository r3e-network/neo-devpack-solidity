// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

interface IDemoToken {
    function balanceOf(address) external view returns (uint256);
    function transfer(address to, uint256 amount) external returns (bool);
    function transferFrom(address from, address to, uint256 amount) external returns (bool);
}

interface IFlashBorrower {
    function onFlashLoan(address initiator, address token, uint256 amount, uint256 fee, bytes calldata data)
        external returns (bytes32);
}

/// @title FlashLender — ERC-3156 reference flash-loan provider, compiled to Neo N3.
contract FlashLender {
    string public buildTag = "flash-lender-v1";
    bytes32 private constant CALLBACK_SUCCESS = keccak256("ERC3156FlashBorrower.onFlashLoan");

    address public token;
    uint96  public feeBps;   // basis points, e.g. 9 = 0.09%
    address public owner;

    function claimOwner() public {
        require(owner == address(0), "Lender: already claimed");
        owner = msg.sender;
    }

    function setup(address token_, uint96 fee_) public {
        require(owner != address(0), "Lender: unclaimed");
        require(msg.sender == owner, "Lender: owner only");
        token = token_;
        feeBps = fee_;
    }

    function maxFlashLoan(address t) public view returns (uint256) {
        if (t != token) return 0;
        return IDemoToken(token).balanceOf(address(this));
    }

    function flashFee(address t, uint256 amount) public view returns (uint256) {
        require(t == token, "Lender: wrong token");
        return amount * feeBps / 10000;
    }

    function flashLoan(IFlashBorrower receiver, address t, uint256 amount, bytes calldata data)
        public returns (bool)
    {
        require(t == token, "Lender: wrong token");
        uint256 balanceBefore = maxFlashLoan(t);
        uint256 fee = flashFee(t, amount);

        // Send principal
        IDemoToken(token).transfer(address(receiver), amount);

        // Borrower runs strategy and must return CALLBACK_SUCCESS
        bytes32 ret = receiver.onFlashLoan(msg.sender, token, amount, fee, data);
        require(ret == CALLBACK_SUCCESS, "Lender: callback failed");

        // Verify repaid principal + fee
        uint256 balanceAfter = maxFlashLoan(t);
        require(balanceAfter >= balanceBefore + fee, "Lender: not repaid");
        return true;
    }
}
