// SPDX-License-Identifier: MIT
// A minimal TimelockController, modeled on the OpenZeppelin governance timelock.
// Reference: https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v4.9.3/contracts/governance/TimelockController.sol
// Self-contained, dependency-free condensation: proposer/executor roles, a minimum
// delay, and the schedule -> (wait) -> execute lifecycle keyed by operation id.
pragma solidity ^0.8.0;

contract TimelockController {
    uint256 internal constant _DONE_TIMESTAMP = uint256(1);

    // operation id => timestamp at which it becomes ready (0 = unset, 1 = done)
    mapping(bytes32 => uint256) private _timestamps;
    uint256 private _minDelay;

    address public admin;
    mapping(address => bool) public isProposer;
    mapping(address => bool) public isExecutor;

    event CallScheduled(bytes32 indexed id, address target, uint256 value, bytes data, uint256 delay);
    event CallExecuted(bytes32 indexed id, address target, uint256 value, bytes data);
    event Cancelled(bytes32 indexed id);
    event MinDelayChange(uint256 oldDuration, uint256 newDuration);

    constructor(uint256 minDelay, address[] memory proposers, address[] memory executors) {
        admin = msg.sender;
        _minDelay = minDelay;
        emit MinDelayChange(0, minDelay);

        for (uint256 i = 0; i < proposers.length; i++) {
            isProposer[proposers[i]] = true;
        }
        for (uint256 i = 0; i < executors.length; i++) {
            isExecutor[executors[i]] = true;
        }
    }

    modifier onlyProposer() {
        require(isProposer[msg.sender], "Timelock: not proposer");
        _;
    }

    modifier onlyExecutor() {
        require(isExecutor[msg.sender], "Timelock: not executor");
        _;
    }

    /// @dev Only the timelock itself may reconfigure delay/roles (via a scheduled op).
    modifier onlySelf() {
        require(msg.sender == address(this), "Timelock: caller is not timelock");
        _;
    }

    function getMinDelay() external view returns (uint256) {
        return _minDelay;
    }

    function getTimestamp(bytes32 id) public view returns (uint256) {
        return _timestamps[id];
    }

    function isOperation(bytes32 id) public view returns (bool) {
        return getTimestamp(id) > 0;
    }

    function isOperationPending(bytes32 id) public view returns (bool) {
        return getTimestamp(id) > _DONE_TIMESTAMP;
    }

    function isOperationReady(bytes32 id) public view returns (bool) {
        uint256 ts = getTimestamp(id);
        return ts > _DONE_TIMESTAMP && ts <= block.timestamp;
    }

    function isOperationDone(bytes32 id) public view returns (bool) {
        return getTimestamp(id) == _DONE_TIMESTAMP;
    }

    function hashOperation(
        address target,
        uint256 value,
        bytes calldata data,
        bytes32 salt
    ) public pure returns (bytes32) {
        return keccak256(abi.encode(target, value, data, salt));
    }

    function schedule(
        address target,
        uint256 value,
        bytes calldata data,
        bytes32 salt,
        uint256 delay
    ) external onlyProposer {
        bytes32 id = hashOperation(target, value, data, salt);
        require(_timestamps[id] == 0, "Timelock: operation already scheduled");
        require(delay >= _minDelay, "Timelock: insufficient delay");
        _timestamps[id] = block.timestamp + delay;
        emit CallScheduled(id, target, value, data, delay);
    }

    function cancel(bytes32 id) external onlyProposer {
        require(isOperationPending(id), "Timelock: operation cannot be cancelled");
        _timestamps[id] = 0;
        emit Cancelled(id);
    }

    function execute(
        address target,
        uint256 value,
        bytes calldata data,
        bytes32 salt
    ) external onlyExecutor {
        bytes32 id = hashOperation(target, value, data, salt);
        require(isOperationReady(id), "Timelock: operation is not ready");
        _timestamps[id] = _DONE_TIMESTAMP;
        emit CallExecuted(id, target, value, data);
    }

    function updateDelay(uint256 newDelay) external onlySelf {
        emit MinDelayChange(_minDelay, newDelay);
        _minDelay = newDelay;
    }
}
