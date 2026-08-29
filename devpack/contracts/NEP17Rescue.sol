// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title NEP17Rescue
 * @dev Optional extension for NEP-17 tokens to recover accidentally sent native tokens.
 *
 * IMPORTANT: In strict-manifest mode this rescue is limited to the two native
 * Neo N3 tokens (GAS and NEO) because recovering arbitrary NEP-17 tokens would
 * require a dynamic `Syscalls.contractCall` to an unknown contract hash, which
 * forces wildcard contract permissions in the manifest.
 *
 * If you need to rescue arbitrary NEP-17 tokens, use `Syscalls.contractCall`
 * directly and accept the wildcard-contract permission trade-off (or supply
 * explicit `--manifest-permissions` overrides at compile time).
 *
 * If you don't need token rescue functionality, inherit `NEP17` directly.
 */

import "../standards/NEP17.sol";
import "./NativeCalls.sol";

abstract contract NEP17Rescue is NEP17 {
    function emergencyTokenRecovery(
        address token,
        address to,
        uint256 amount,
        bytes calldata data
    ) external onlyOwner {
        require(token != address(this), "NEP17: cannot recover own tokens");
        require(to != address(0), "NEP17: cannot recover to zero address");

        bool ok;
        if (token == NativeCalls.GAS_CONTRACT) {
            ok = NativeCalls.gasTransfer(address(this), to, amount, data);
        } else if (token == NativeCalls.NEO_CONTRACT) {
            ok = NativeCalls.neoTransfer(address(this), to, amount, data);
        } else {
            revert("NEP17: strict rescue supports NEO/GAS only");
        }

        require(ok, "NEP17: token recovery failed");
    }
}

