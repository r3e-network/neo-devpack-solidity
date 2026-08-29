// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title HookedToken — ERC-777 reference (without ERC-1820 registry), compiled to Neo N3.
contract HookedToken {
    string public buildTag = "hooked-v1";
    string public name = "Demo Hooked Token";
    string public symbol = "DHOOK";
    uint8 public decimals = 8;
    uint256 public totalSupply;

    address public deployer;
    mapping(address => uint256) public balanceOf;

    function claimDeployer() public {
        require(deployer == address(0), "Hooked: already claimed");
        deployer = msg.sender;
    }

    function mint(address to, uint256 amount) public {
        require(msg.sender == deployer, "Hooked: deployer only");
        balanceOf[to] += amount;
        totalSupply += amount;
    }

    /// @notice ERC-777 send semantics — recipient hook fires AFTER state updates
    /// (re-entrancy safe by construction, vs. the original ERC-777 ordering).
    function send(address to, uint256 amount, bytes calldata) public returns (bool) {
        require(balanceOf[msg.sender] >= amount, "Hooked: insufficient");
        balanceOf[msg.sender] -= amount;
        balanceOf[to] += amount;
        // In a real impl, the recipient hook would be invoked here via a static
        // method name — neo-solc requires named-method invocation, so we elide
        // the dynamic dispatch in this demo.
        return true;
    }
}
