// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title MultiToken — ERC-1155 reference, compiled to Neo N3 via neo-solc.
contract MultiToken {
    string public buildTag = "multi-token-v1";
    string public name = "Demo Solidity Multi-Token";

    address public deployer;
    mapping(uint256 => mapping(address => uint256)) public balances;
    mapping(uint256 => uint256) public totalSupplyOf;

    function claimDeployer() public {
        require(deployer == address(0), "MultiToken: already claimed");
        deployer = msg.sender;
    }

    function balanceOf(address account, uint256 id) public view returns (uint256) {
        return balances[id][account];
    }

    function balanceOfBatch(address[] calldata accounts, uint256[] calldata ids)
        public view returns (uint256[] memory)
    {
        require(accounts.length == ids.length, "MultiToken: length mismatch");
        uint256[] memory out = new uint256[](accounts.length);
        for (uint i; i < accounts.length; ++i) {
            out[i] = balances[ids[i]][accounts[i]];
        }
        return out;
    }

    function mint(address to, uint256 id, uint256 amount) public {
        require(msg.sender == deployer, "MultiToken: only deployer");
        balances[id][to] += amount;
        totalSupplyOf[id] += amount;
    }

    function safeTransferFrom(address from, address to, uint256 id, uint256 amount, bytes calldata)
        public
    {
        require(from == msg.sender, "MultiToken: not authorized");
        require(balances[id][from] >= amount, "MultiToken: insufficient");
        balances[id][from] -= amount;
        balances[id][to]   += amount;
    }
}
