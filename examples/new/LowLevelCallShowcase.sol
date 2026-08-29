// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

interface IGasToken {
    function balanceOf(address account) external view returns (uint256);

    function transfer(address from, address to, uint256 amount, bytes memory data)
        external
        returns (bool);
}

/**
 * @title LowLevelCallShowcase
 * @notice Demonstrates Neo-supported low-level call patterns with static targets.
 */
contract LowLevelCallShowcase {
    function readViaSignature(address account) external returns (bool ok, bytes memory data) {
        bytes memory payload = abi.encodeWithSignature("balanceOf(address)", account);
        (ok, data) = address(0xd2a4cff31913016155e38e474a2c06d08be276cf).staticcall(payload);
    }

    function readViaSelector(address account) external returns (bool ok, bytes memory data) {
        bytes memory payload = abi.encodeWithSelector(IGasToken.balanceOf.selector, account);
        (ok, data) = address(0xd2a4cff31913016155e38e474a2c06d08be276cf).staticcall(payload);
    }

    function writeViaSignature(address to, uint256 amount)
        external
        returns (bool ok, bytes memory data)
    {
        bytes memory payload = abi.encodeWithSignature(
            "transfer(address,address,uint256,bytes)",
            address(this),
            to,
            amount,
            ""
        );
        (ok, data) = address(0xd2a4cff31913016155e38e474a2c06d08be276cf).call(payload);
    }
}
