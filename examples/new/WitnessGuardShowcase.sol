// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title WitnessGuardShowcase
 * @notice Demonstrates witness-based authorization for owner/guardian workflows.
 *
 * Ownership is bootstrapped after deployment through `bootstrapOwner()` so the
 * initial controller is a real signer account rather than ContractManagement.
 */
contract WitnessGuardShowcase {
    address public owner;
    bool private ownerInitialized;

    mapping(address => bool) public isGuardian;
    mapping(address => uint256) public lockedUntil;

    event GuardianUpdated(address indexed guardian, bool enabled);
    event AccountLocked(address indexed account, uint256 untilTimestamp);
    event AccountUnlocked(address indexed account);
    event PrivilegedAction(address indexed actor, uint256 value, bytes32 memoHash);

    error AlreadyInitialized();
    error Unauthorized();
    error InvalidOwner(address account);
    error InvalidWitness(address account);
    error AccountLockedError(address account, uint256 untilTimestamp);

    modifier onlyOwnerWithWitness() {
        if (!Runtime.checkWitness(owner)) revert InvalidWitness(owner);
        _;
    }

    modifier onlyGuardianWithWitness(address guardian) {
        if (!isGuardian[guardian]) revert Unauthorized();
        if (!Runtime.checkWitness(guardian)) revert InvalidWitness(guardian);
        _;
    }

    constructor() {}

    function bootstrapOwner(address initialOwner) external {
        if (ownerInitialized) revert AlreadyInitialized();
        if (!Runtime.checkWitness(initialOwner)) revert InvalidWitness(initialOwner);

        owner = initialOwner;
        ownerInitialized = true;
    }

    function setGuardian(address guardian, bool enabled) external onlyOwnerWithWitness {
        isGuardian[guardian] = enabled;
    }

    function lockAccount(address guardian, address account, uint256 durationSeconds)
        external
        onlyGuardianWithWitness(guardian)
    {
        durationSeconds;
        lockedUntil[account] = 1;
    }

    function unlockAccount(address account) external onlyOwnerWithWitness {
        lockedUntil[account] = 0;
    }

    function privilegedAction(
        address actor,
        uint256 value,
        string calldata memo
    ) external {
        if (!Runtime.checkWitness(actor)) revert InvalidWitness(actor);

        if (lockedUntil[actor] != 0) {
            revert AccountLockedError(actor, lockedUntil[actor]);
        }
    }

    function isLocked(address account) external view returns (bool) {
        return lockedUntil[account] != 0;
    }

}
