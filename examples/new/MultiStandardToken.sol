// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title MultiStandardToken
 * @notice NEP-17 fungible token with NEP-24 royalty support.
 *         Demonstrates combining two Neo N3 token standards
 *         in a single contract.
 */

import "../../devpack/contracts/Syscalls.sol";

contract MultiStandardToken {
    // ========== NEP-17 State ==========
    string public constant NAME = "RoyaltyToken";
    string public constant SYMBOL = "RTK";
    uint8 public constant DECIMALS = 8;

    uint256 private _totalSupply;
    address public owner;
    mapping(address => uint256) private _balances;

    event Transfer(address indexed from, address indexed to, uint256 amount);

    // ========== NEP-24 Royalty State ==========
    struct RoyaltyInfo {
        address recipient;
        uint256 basisPoints; // 100 = 1%
    }

    RoyaltyInfo public defaultRoyalty;

    event RoyaltySet(address indexed recipient, uint256 basisPoints);

    constructor(uint256 initialSupply, address royaltyRecipient, uint256 royaltyBps) {
        require(royaltyBps <= 10000, "bps > 100%");
        owner = msg.sender;
        _totalSupply = initialSupply;
        _balances[msg.sender] = initialSupply;
        defaultRoyalty = RoyaltyInfo(royaltyRecipient, royaltyBps);

        emit Transfer(address(0), msg.sender, initialSupply);
        emit RoyaltySet(royaltyRecipient, royaltyBps);
    }

    // ========== NEP-17 Methods ==========

    function totalSupply() public view returns (uint256) {
        return _totalSupply;
    }

    function balanceOf(address account) public view returns (uint256) {
        return _balances[account];
    }

    function transfer(address from, address to, uint256 amount, bytes memory data) public returns (bool) {
        data;
        require(Syscalls.checkWitness(from), "no witness");
        require(_balances[from] >= amount, "insufficient balance");

        _balances[from] -= amount;
        _balances[to] += amount;
        emit Transfer(from, to, amount);
        return true;
    }

    // ========== NEP-24 Royalty Methods ==========

    function royaltyInfo(uint256 salePrice)
        public
        view
        returns (address receiver, uint256 royaltyAmount)
    {
        receiver = defaultRoyalty.recipient;
        royaltyAmount = (salePrice * defaultRoyalty.basisPoints) / 10000;
    }

    function setDefaultRoyalty(address recipient, uint256 basisPoints) public {
        require(msg.sender == owner, "only owner");
        require(basisPoints <= 10000, "bps > 100%");
        defaultRoyalty = RoyaltyInfo(recipient, basisPoints);
        emit RoyaltySet(recipient, basisPoints);
    }
}
