// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/// @title Staking - Token staking with rewards for Neo N3
/// @author Neo DevPack for Solidity Team
/// @notice Stake tokens to earn rewards over time
/// @dev Implements a simple staking mechanism with configurable APY
contract Staking {
    // Staker information
    struct StakerInfo {
        uint256 stakedAmount;
        uint256 rewardDebt;
        uint256 lastStakeTime;
        uint256 totalClaimed;
    }

    // Pool information
    struct PoolInfo {
        uint256 totalStaked;
        uint256 rewardPerSecond;
        uint256 accRewardPerShare;
        uint256 lastRewardTime;
        uint256 minStakeAmount;
        uint256 lockPeriod;
        bool active;
    }

    // State variables
    PoolInfo public pool;
    mapping(address => StakerInfo) public stakers;
    address[] public stakerList;
    mapping(address => bool) public isStaker;

    address public owner;
    uint256 public totalRewardsDistributed;
    uint256 public rewardTokenBalance;

    // Precision for reward calculations
    uint256 private constant PRECISION = 1e18;

    // Events
    event Staked(address indexed user, uint256 amount, uint256 timestamp);
    event Unstaked(address indexed user, uint256 amount, uint256 timestamp);
    event RewardClaimed(address indexed user, uint256 amount, uint256 timestamp);
    event RewardRateUpdated(uint256 oldRate, uint256 newRate);
    event PoolUpdated(uint256 accRewardPerShare, uint256 timestamp);
    event RewardsDeposited(uint256 amount);
    event EmergencyWithdraw(address indexed user, uint256 amount);

    /// @notice Contract constructor
    /// @param _rewardPerSecond Initial reward rate per second
    /// @param _minStakeAmount Minimum amount that can be staked
    /// @param _lockPeriod Lock period in seconds
    constructor(
        uint256 _rewardPerSecond,
        uint256 _minStakeAmount,
        uint256 _lockPeriod
    ) {
        owner = msg.sender;

        pool = PoolInfo({
            totalStaked: 0,
            rewardPerSecond: _rewardPerSecond,
            accRewardPerShare: 0,
            lastRewardTime: block.timestamp,
            minStakeAmount: _minStakeAmount,
            lockPeriod: _lockPeriod,
            active: true
        });

        totalRewardsDistributed = 0;
        rewardTokenBalance = 0;
    }

    modifier onlyOwner() {
        require(msg.sender == owner, "Staking: caller is not owner");
        _;
    }

    modifier poolActive() {
        require(pool.active, "Staking: pool is not active");
        _;
    }

    /// @notice Update pool reward variables
    function updatePool() public {
        if (block.timestamp <= pool.lastRewardTime) {
            return;
        }

        if (pool.totalStaked == 0) {
            pool.lastRewardTime = block.timestamp;
            return;
        }

        uint256 timePassed = block.timestamp - pool.lastRewardTime;
        uint256 rewards = timePassed * pool.rewardPerSecond;

        pool.accRewardPerShare += (rewards * PRECISION) / pool.totalStaked;
        pool.lastRewardTime = block.timestamp;

        emit PoolUpdated(pool.accRewardPerShare, block.timestamp);
    }

    /// @notice Calculate pending rewards for a staker
    /// @param user The staker address
    /// @return Pending reward amount
    function pendingRewards(address user) public view returns (uint256) {
        StakerInfo storage staker = stakers[user];
        if (staker.stakedAmount == 0) {
            return 0;
        }

        uint256 accRewardPerShare = pool.accRewardPerShare;

        if (block.timestamp > pool.lastRewardTime && pool.totalStaked > 0) {
            uint256 timePassed = block.timestamp - pool.lastRewardTime;
            uint256 rewards = timePassed * pool.rewardPerSecond;
            accRewardPerShare += (rewards * PRECISION) / pool.totalStaked;
        }

        return (staker.stakedAmount * accRewardPerShare / PRECISION) - staker.rewardDebt;
    }

    /// @notice Stake tokens
    /// @param amount Amount to stake
    function stake(uint256 amount) public poolActive {
        require(amount >= pool.minStakeAmount, "Staking: amount below minimum");

        updatePool();

        StakerInfo storage staker = stakers[msg.sender];

        // Claim any pending rewards first
        if (staker.stakedAmount > 0) {
            uint256 pending = (staker.stakedAmount * pool.accRewardPerShare / PRECISION) - staker.rewardDebt;
            if (pending > 0) {
                _safeRewardTransfer(msg.sender, pending);
            }
        } else {
            // New staker
            if (!isStaker[msg.sender]) {
                stakerList.push(msg.sender);
                isStaker[msg.sender] = true;
            }
        }

        staker.stakedAmount += amount;
        staker.lastStakeTime = block.timestamp;
        staker.rewardDebt = staker.stakedAmount * pool.accRewardPerShare / PRECISION;

        pool.totalStaked += amount;

        emit Staked(msg.sender, amount, block.timestamp);
    }

    /// @notice Unstake tokens
    /// @param amount Amount to unstake
    function unstake(uint256 amount) public {
        StakerInfo storage staker = stakers[msg.sender];
        require(staker.stakedAmount >= amount, "Staking: insufficient staked amount");
        require(
            block.timestamp >= staker.lastStakeTime + pool.lockPeriod,
            "Staking: still in lock period"
        );

        updatePool();

        // Claim pending rewards
        uint256 pending = (staker.stakedAmount * pool.accRewardPerShare / PRECISION) - staker.rewardDebt;
        if (pending > 0) {
            _safeRewardTransfer(msg.sender, pending);
        }

        staker.stakedAmount -= amount;
        staker.rewardDebt = staker.stakedAmount * pool.accRewardPerShare / PRECISION;

        pool.totalStaked -= amount;

        emit Unstaked(msg.sender, amount, block.timestamp);
    }

    /// @notice Claim accumulated rewards without unstaking
    function claimRewards() public {
        updatePool();

        StakerInfo storage staker = stakers[msg.sender];
        require(staker.stakedAmount > 0, "Staking: no stake found");

        uint256 pending = (staker.stakedAmount * pool.accRewardPerShare / PRECISION) - staker.rewardDebt;
        require(pending > 0, "Staking: no rewards to claim");

        staker.rewardDebt = staker.stakedAmount * pool.accRewardPerShare / PRECISION;

        _safeRewardTransfer(msg.sender, pending);
    }

    /// @notice Internal function to safely transfer rewards
    /// @param to Recipient address
    /// @param amount Amount to transfer
    function _safeRewardTransfer(address to, uint256 amount) internal {
        uint256 rewardBal = rewardTokenBalance;
        uint256 transferAmount = amount > rewardBal ? rewardBal : amount;

        if (transferAmount > 0) {
            rewardTokenBalance -= transferAmount;
            totalRewardsDistributed += transferAmount;
            stakers[to].totalClaimed += transferAmount;
            emit RewardClaimed(to, transferAmount, block.timestamp);
        }
    }

    /// @notice Deposit reward tokens (owner only)
    /// @param amount Amount of reward tokens to deposit
    function depositRewards(uint256 amount) public onlyOwner {
        require(amount > 0, "Staking: amount must be positive");
        rewardTokenBalance += amount;
        emit RewardsDeposited(amount);
    }

    /// @notice Emergency withdraw without caring about rewards
    function emergencyWithdraw() public {
        StakerInfo storage staker = stakers[msg.sender];
        uint256 amount = staker.stakedAmount;
        require(amount > 0, "Staking: no stake found");

        staker.stakedAmount = 0;
        staker.rewardDebt = 0;
        pool.totalStaked -= amount;

        emit EmergencyWithdraw(msg.sender, amount);
    }

    /// @notice Update reward rate (owner only)
    /// @param newRewardPerSecond New reward rate
    function setRewardRate(uint256 newRewardPerSecond) public onlyOwner {
        updatePool();
        uint256 oldRate = pool.rewardPerSecond;
        pool.rewardPerSecond = newRewardPerSecond;
        emit RewardRateUpdated(oldRate, newRewardPerSecond);
    }

    /// @notice Update minimum stake amount (owner only)
    /// @param newMinStake New minimum stake amount
    function setMinStakeAmount(uint256 newMinStake) public onlyOwner {
        pool.minStakeAmount = newMinStake;
    }

    /// @notice Update lock period (owner only)
    /// @param newLockPeriod New lock period in seconds
    function setLockPeriod(uint256 newLockPeriod) public onlyOwner {
        pool.lockPeriod = newLockPeriod;
    }

    /// @notice Pause/unpause pool (owner only)
    /// @param active New active state
    function setPoolActive(bool active) public onlyOwner {
        pool.active = active;
    }

    /// @notice Get staker information
    /// @param user The staker address
    /// @return stakedAmount Amount staked
    /// @return pending Pending rewards
    /// @return lastStakeTime Time of last stake
    /// @return totalClaimed Total rewards claimed
    function getStakerInfo(address user) public view returns (
        uint256 stakedAmount,
        uint256 pending,
        uint256 lastStakeTime,
        uint256 totalClaimed
    ) {
        StakerInfo storage staker = stakers[user];
        return (
            staker.stakedAmount,
            pendingRewards(user),
            staker.lastStakeTime,
            staker.totalClaimed
        );
    }

    /// @notice Check if user can unstake
    /// @param user The user address
    /// @return True if lock period has passed
    function canUnstake(address user) public view returns (bool) {
        StakerInfo storage staker = stakers[user];
        if (staker.stakedAmount == 0) return false;
        return block.timestamp >= staker.lastStakeTime + pool.lockPeriod;
    }

    /// @notice Get time until unlock
    /// @param user The user address
    /// @return Seconds until unlock, 0 if unlocked
    function timeUntilUnlock(address user) public view returns (uint256) {
        StakerInfo storage staker = stakers[user];
        if (staker.stakedAmount == 0) return 0;
        uint256 unlockTime = staker.lastStakeTime + pool.lockPeriod;
        if (block.timestamp >= unlockTime) return 0;
        return unlockTime - block.timestamp;
    }

    /// @notice Get pool statistics
    /// @return totalStaked Total amount staked
    /// @return rewardPerSecond Current reward rate
    /// @return totalDistributed Total rewards distributed
    /// @return rewardBalance Available reward balance
    function getPoolStats() public view returns (
        uint256 totalStaked,
        uint256 rewardPerSecond,
        uint256 totalDistributed,
        uint256 rewardBalance
    ) {
        return (pool.totalStaked, pool.rewardPerSecond, totalRewardsDistributed, rewardTokenBalance);
    }

    /// @notice Get total number of stakers
    /// @return Number of unique stakers
    function getStakerCount() public view returns (uint256) {
        return stakerList.length;
    }

    /// @notice Calculate APY based on current parameters
    /// @return Annual percentage yield (scaled by 100 for precision)
    function calculateAPY() public view returns (uint256) {
        if (pool.totalStaked == 0) return 0;
        uint256 yearlyRewards = pool.rewardPerSecond * 365 * 24 * 60 * 60;
        return (yearlyRewards * 10000) / pool.totalStaked; // Returns APY in basis points
    }
}
