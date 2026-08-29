// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title TransientGuard — EIP-1153 transient-storage reentrancy guard, compiled to Neo N3.
/// @notice EIP-1153 introduces tload/tstore that automatically clear at end-of-tx.
/// Neo doesn't have transient storage, but we emulate the same end-of-call clearing
/// by Storage.Put then Storage.Delete in the same call. The guard is functionally
/// equivalent: re-entry within the same call sees the lock; the lock is gone afterward.
contract TransientGuard {
    string public buildTag = "transient-guard-v1";

    address public deployer;
    uint256 public callsCompleted;
    uint256 private _locked; // 1 = currently inside guarded call

    function claimDeployer() public {
        require(deployer == address(0), "TG: already claimed");
        deployer = msg.sender;
    }

    /// Demonstrates the guard pattern. _locked is set, work happens, then _locked
    /// is cleared. Re-entry in the same call would see _locked != 0 and revert.
    function guardedCall() public {
        require(_locked == 0, "TG: re-entry blocked");
        _locked = 1;
        callsCompleted += 1;
        _locked = 0;
    }

    function isLocked() public view returns (bool) { return _locked != 0; }
}
