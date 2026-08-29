// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title VotingToken — ERC-5805 voting token (block-clock mode), compiled to Neo N3.
contract VotingToken {
    string public buildTag = "voting-v1";
    string public name = "Demo Vote";
    string public symbol = "DVOTE";
    uint8 public decimals = 0;

    uint256 public totalSupply;
    address public deployer;

    mapping(address => uint256) public balanceOf;
    mapping(address => address) public delegates;
    mapping(address => uint256) public votingPower;

    function claimDeployer() public {
        require(deployer == address(0), "Vote: already claimed");
        deployer = msg.sender;
    }

    function mint(address to, uint256 amount) public {
        require(msg.sender == deployer, "Vote: deployer only");
        balanceOf[to] += amount;
        totalSupply += amount;
        // If the recipient had delegated, increase that delegate's power.
        address d = delegates[to];
        if (d != address(0)) votingPower[d] += amount;
    }

    function delegate(address delegatee) public {
        address current = delegates[msg.sender];
        if (current != address(0)) votingPower[current] -= balanceOf[msg.sender];
        delegates[msg.sender] = delegatee;
        votingPower[delegatee] += balanceOf[msg.sender];
    }

    function getVotes(address account) public view returns (uint256) {
        return votingPower[account];
    }

    function clock() public view returns (uint48) {
        return uint48(block.number);
    }

    function CLOCK_MODE() public pure returns (string memory) {
        return "mode=blocknumber&from=default";
    }
}
