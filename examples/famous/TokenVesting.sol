// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/**
 * @title TokenVesting — Linear Vesting with Cliff
 * @notice Neo N3 adaptation of OpenZeppelin VestingWallet / Sablier streaming.
 *
 * Beneficiary receives GAS linearly over a duration after a cliff period.
 * release() computes vested amount based on block.timestamp and transfers GAS.
 *
 * Compiler constraints respected:
 *   - No {value: ...} — uses NativeCalls.gasTransfer()
 *   - No receive()/fallback() — uses onNEP17Payment()
 *   - Import devpack via -I devpack
 */
contract TokenVesting {
    address private constant GAS_TOKEN = NativeCalls.GAS_CONTRACT;

    address public beneficiary;
    address public owner;

    uint256 public startTime;
    uint256 public cliffDuration;
    uint256 public vestingDuration;
    uint256 public totalAllocation;
    uint256 public released;

    bool public revoked;

    event TokensDeposited(address indexed from, uint256 amount);
    event TokensReleased(address indexed beneficiary, uint256 amount);
    event VestingRevoked(address indexed owner, uint256 unreleased);

    modifier onlyOwner() {
        require(msg.sender == owner, "Vesting: not owner");
        _;
    }

    modifier onlyBeneficiary() {
        require(msg.sender == beneficiary, "Vesting: not beneficiary");
        _;
    }

    constructor() {
        owner = msg.sender;
        beneficiary = msg.sender;
        startTime = block.timestamp;
        cliffDuration = 100;
        vestingDuration = 1000;
    }

    /// @notice Re-initialize vesting parameters. Only owner, only before any release.
    function initialize(
        address _beneficiary,
        uint256 _startTime,
        uint256 _cliffDuration,
        uint256 _vestingDuration
    ) external {
        require(msg.sender == owner, "Vesting: not owner");
        require(released == 0, "Vesting: already releasing");
        require(_beneficiary != address(0), "Vesting: zero beneficiary");
        require(_vestingDuration > 0, "Vesting: zero duration");
        require(_cliffDuration <= _vestingDuration, "Vesting: cliff > duration");

        beneficiary = _beneficiary;
        startTime = _startTime;
        cliffDuration = _cliffDuration;
        vestingDuration = _vestingDuration;
    }

    /// @notice NEP-17 callback — receives GAS funding for the vesting schedule.
    function onNEP17Payment(address from, uint256 amount, Any calldata /*data*/) external {
        address caller = Syscalls.getCallingScriptHash();
        require(caller == GAS_TOKEN, "Vesting: only GAS accepted");
        require(amount > 0, "Vesting: zero deposit");

        totalAllocation += amount;
        emit TokensDeposited(from, amount);
    }

    /// @notice Release vested tokens to the beneficiary.
    function release() external onlyBeneficiary {
        require(!revoked, "Vesting: revoked");

        uint256 releasable = releasableAmount();
        require(releasable > 0, "Vesting: nothing to release");

        released += releasable;

        bool ok = NativeCalls.gasTransfer(address(this), beneficiary, releasable, "");
        require(ok, "Vesting: transfer failed");

        emit TokensReleased(beneficiary, releasable);
    }

    /// @notice Revoke vesting — return unvested GAS to owner.
    function revoke() external onlyOwner {
        require(!revoked, "Vesting: already revoked");

        uint256 vested = vestedAmount();
        uint256 unreleased = totalAllocation - vested;

        revoked = true;

        if (unreleased > 0) {
            bool ok = NativeCalls.gasTransfer(address(this), owner, unreleased, "");
            require(ok, "Vesting: revoke transfer failed");
        }

        emit VestingRevoked(owner, unreleased);
    }

    /// @notice Compute the total vested amount at current time.
    function vestedAmount() public view returns (uint256) {
        uint256 currentTime = block.timestamp;

        if (currentTime < startTime + cliffDuration) {
            return 0;
        }

        if (currentTime >= startTime + vestingDuration) {
            return totalAllocation;
        }

        uint256 elapsed = currentTime - startTime;
        return (totalAllocation * elapsed) / vestingDuration;
    }

    /// @notice Compute the releasable amount (vested minus already released).
    function releasableAmount() public view returns (uint256) {
        return vestedAmount() - released;
    }
}
