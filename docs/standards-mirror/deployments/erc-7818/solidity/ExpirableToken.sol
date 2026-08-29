// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title ExpirableToken — ERC-7818 reference, compiled to Neo N3 via neo-solc.
contract ExpirableToken {
    string public buildTag = "expirable-v1";
    string public name = "Demo Expirable";
    string public symbol = "DEXP";
    uint8 public decimals = 0;

    uint256 public epochDuration;   // seconds
    uint256 public retentionEpochs; // expire after N epochs

    address public deployer;
    mapping(address => mapping(uint256 => uint256)) public balanceAtEpoch;

    function claimDeployer() public {
        require(deployer == address(0), "Expirable: already claimed");
        deployer = msg.sender;
    }

    function setup(uint256 dur, uint256 retention) public {
        require(deployer != address(0), "Expirable: unclaimed");
        require(msg.sender == deployer, "Expirable: deployer only");
        epochDuration = dur;
        retentionEpochs = retention;
    }

    function currentEpoch() public view returns (uint256) {
        return block.timestamp / epochDuration;
    }

    function balanceOf(address account) public view returns (uint256 total) {
        uint256 cur = currentEpoch();
        uint256 from = cur >= retentionEpochs ? cur - retentionEpochs + 1 : 0;
        for (uint256 i = from; i <= cur; ++i) {
            total += balanceAtEpoch[account][i];
        }
    }

    function mint(address to, uint256 amount) public {
        require(msg.sender == deployer, "Expirable: deployer only");
        balanceAtEpoch[to][currentEpoch()] += amount;
    }
}
