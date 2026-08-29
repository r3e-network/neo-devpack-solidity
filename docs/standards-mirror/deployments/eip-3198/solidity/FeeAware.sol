// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title FeeAware — EIP-3198 fee introspection demo, compiled to Neo N3.
/// @notice EIP-3198 added BASEFEE so contracts can read the current basefee.
/// On Neo, fees are governed by the Policy native contract; the fee model is
/// per-byte and per-system-call rather than EIP-1559's gas auction. This contract
/// exposes a tag and lets clients read the contract-stored "last seen fee" hint
/// that the deployer can update for testing.
contract FeeAware {
    string public buildTag = "fee-aware-v1";

    address public deployer;
    uint256 public lastSeenFeePerByte;

    function claimDeployer() public {
        require(deployer == address(0), "FeeAware: already claimed");
        deployer = msg.sender;
    }

    function recordFee(uint256 fee) public {
        require(msg.sender == deployer, "FeeAware: deployer only");
        lastSeenFeePerByte = fee;
    }

    function getLastSeenFee() public view returns (uint256) { return lastSeenFeePerByte; }
}
