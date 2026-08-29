// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title SolidityFeaturesComplete
 * @notice Comprehensive test contract exercising ALL major Solidity features
 *         for Neo N3 compilation verification.
 *
 * Covers: types, operators, control flow, functions, modifiers, events,
 * errors, inheritance, interfaces, libraries, structs, enums, mappings,
 * arrays, strings, type casting, unchecked blocks, try/catch, and more.
 */

// ========== Interfaces ==========
interface ICounter {
    function count() external view returns (uint256);
    function increment() external;
}

// ========== Libraries ==========
library MathLib {
    function add(uint256 a, uint256 b) internal pure returns (uint256) {
        return a + b;
    }

    function clamp(uint256 value, uint256 min, uint256 max) internal pure returns (uint256) {
        if (value < min) return min;
        if (value > max) return max;
        return value;
    }
}

library ArrayUtils {
    function sum(uint256[] memory arr) internal pure returns (uint256 total) {
        for (uint256 i = 0; i < arr.length; i++) {
            total += arr[i];
        }
    }

    function contains(address[] memory arr, address target) internal pure returns (bool) {
        for (uint256 i = 0; i < arr.length; i++) {
            if (arr[i] == target) return true;
        }
        return false;
    }
}

// ========== Abstract Base ==========
abstract contract Ownable {
    address public owner;

    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);

    error OwnableUnauthorized(address caller);

    modifier onlyOwner() {
        if (msg.sender != owner) revert OwnableUnauthorized(msg.sender);
        _;
    }

    constructor() {
        owner = msg.sender;
        emit OwnershipTransferred(address(0), msg.sender);
    }

    function transferOwnership(address newOwner) public virtual onlyOwner {
        require(newOwner != address(0), "Ownable: zero address");
        emit OwnershipTransferred(owner, newOwner);
        owner = newOwner;
    }
}

abstract contract Pausable is Ownable {
    bool public paused;

    event Paused(address account);
    event Unpaused(address account);

    error EnforcedPause();
    error ExpectedPause();

    modifier whenNotPaused() {
        if (paused) revert EnforcedPause();
        _;
    }

    modifier whenPaused() {
        if (!paused) revert ExpectedPause();
        _;
    }

    function pause() public onlyOwner whenNotPaused {
        paused = true;
        emit Paused(msg.sender);
    }

    function unpause() public onlyOwner whenPaused {
        paused = false;
        emit Unpaused(msg.sender);
    }
}

// ========== Enums ==========
enum Status { Pending, Active, Completed, Cancelled }
enum Priority { Low, Medium, High, Critical }

// ========== Structs ==========
struct Task {
    uint256 id;
    string title;
    Status status;
    Priority priority;
    address assignee;
    uint256 createdAt;
    uint256 completedAt;
}

struct TaskStats {
    uint256 total;
    uint256 pending;
    uint256 active;
    uint256 completed;
    uint256 cancelled;
}

// ========== Main Contract ==========
contract SolidityFeaturesComplete is Pausable, ICounter {
    using MathLib for uint256;
    using ArrayUtils for uint256[];
    using ArrayUtils for address[];

    // ===== State Variables =====
    uint256 private _count;
    uint256 public immutable deployBlock;
    string public constant VERSION = "1.0.0";
    uint256 public constant MAX_TASKS = 1000;

    // Complex types
    mapping(uint256 => Task) public tasks;
    mapping(address => uint256[]) private userTasks;
    mapping(address => mapping(address => bool)) public approvals;
    uint256 public nextTaskId;

    // Arrays
    address[] public members;
    uint256[] private scores;

    // Nested mappings
    mapping(bytes32 => mapping(address => uint256)) public roleWeights;

    // ===== Events =====
    event TaskCreated(uint256 indexed id, address indexed assignee, string title);
    event TaskUpdated(uint256 indexed id, Status oldStatus, Status newStatus);
    event MemberAdded(address indexed member);
    event ScoreRecorded(address indexed member, uint256 score);

    // ===== Custom Errors =====
    error TaskNotFound(uint256 id);
    error InvalidStatus(Status current, Status requested);
    error MemberAlreadyExists(address member);
    error MaxTasksReached();

    // ===== Constructor =====
    constructor() Pausable() {
        deployBlock = block.number;
        nextTaskId = 1;
    }

    // ===== ICounter Implementation =====
    function count() external view override returns (uint256) {
        return _count;
    }

    function increment() external override whenNotPaused {
        _count++;
    }

    // ===== Task Management =====
    function createTask(
        string calldata title,
        Priority priority,
        address assignee
    ) external whenNotPaused returns (uint256 taskId) {
        if (nextTaskId > MAX_TASKS) revert MaxTasksReached();

        taskId = nextTaskId++;
        tasks[taskId] = Task({
            id: taskId,
            title: title,
            status: Status.Pending,
            priority: priority,
            assignee: assignee,
            createdAt: block.timestamp,
            completedAt: 0
        });
        userTasks[assignee].push(taskId);

        emit TaskCreated(taskId, assignee, title);
    }

    function updateTaskStatus(uint256 taskId, Status newStatus) external whenNotPaused {
        Task storage task = tasks[taskId];
        if (task.id == 0) revert TaskNotFound(taskId);

        Status oldStatus = task.status;

        // Status transition validation
        if (oldStatus == Status.Cancelled) revert InvalidStatus(oldStatus, newStatus);
        if (oldStatus == Status.Completed && newStatus != Status.Cancelled) {
            revert InvalidStatus(oldStatus, newStatus);
        }

        task.status = newStatus;
        if (newStatus == Status.Completed) {
            task.completedAt = block.timestamp;
        }

        emit TaskUpdated(taskId, oldStatus, newStatus);
    }

    // ===== Member Management =====
    function addMember(address member) external onlyOwner {
        if (members.contains(member)) revert MemberAlreadyExists(member);
        members.push(member);
        emit MemberAdded(member);
    }

    function getMemberCount() external view returns (uint256) {
        return members.length;
    }

    // ===== Arithmetic & Type Features =====
    function testArithmetic(uint256 a, uint256 b) external pure returns (
        uint256 sum,
        uint256 diff,
        uint256 product,
        uint256 quotient,
        uint256 remainder
    ) {
        sum = a + b;
        diff = a > b ? a - b : b - a;
        product = a * b;
        quotient = b > 0 ? a / b : 0;
        remainder = b > 0 ? a % b : 0;
    }

    function testBitwise(uint256 a, uint256 b) external pure returns (
        uint256 andResult,
        uint256 orResult,
        uint256 xorResult,
        uint256 notResult,
        uint256 shiftLeft,
        uint256 shiftRight
    ) {
        andResult = a & b;
        orResult = a | b;
        xorResult = a ^ b;
        notResult = ~a;
        shiftLeft = a << 2;
        shiftRight = a >> 2;
    }

    function testComparison(uint256 a, uint256 b) external pure returns (
        bool eq, bool neq, bool lt, bool gt, bool lte, bool gte
    ) {
        eq = a == b;
        neq = a != b;
        lt = a < b;
        gt = a > b;
        lte = a <= b;
        gte = a >= b;
    }

    function testTypeCasting() external pure returns (
        uint8 small,
        uint256 big,
        int256 signed,
        bool flag,
        bytes32 hash
    ) {
        big = 256;
        small = uint8(big); // Truncation: 256 → 0
        signed = int256(big);
        flag = big > 0;
        hash = bytes32(big);
    }

    // ===== Unchecked Arithmetic =====
    function testUnchecked(uint256 a) external pure returns (uint256) {
        unchecked {
            return a + type(uint256).max; // Wraps around
        }
    }

    // ===== String & Bytes =====
    function testStringOps(string calldata a, string calldata b) external pure returns (
        bytes32 hashA,
        bytes32 hashB,
        uint256 lenA,
        bytes memory concat
    ) {
        hashA = keccak256(bytes(a));
        hashB = keccak256(bytes(b));
        lenA = bytes(a).length;
        concat = abi.encodePacked(a, b);
    }

    // ===== ABI Encoding =====
    function testAbiEncode(address addr, uint256 val) external pure returns (
        bytes memory encoded,
        bytes memory packed,
        bytes4 selector
    ) {
        encoded = abi.encode(addr, val);
        packed = abi.encodePacked(addr, val);
        selector = bytes4(keccak256("transfer(address,uint256)"));
    }

    // ===== Control Flow =====
    function testControlFlow(uint256 n) external pure returns (uint256 result) {
        // For loop
        for (uint256 i = 0; i < n && i < 100; i++) {
            result += i;
        }

        // While loop with break/continue
        uint256 j = 0;
        while (j < n) {
            j++;
            if (j == 50) break;
            if (j % 2 == 0) continue;
            result += 1;
        }

        // Ternary
        result = result > 1000 ? 1000 : result;
    }

    // ===== Using Library =====
    function testLibrary(uint256 a, uint256 b) external pure returns (uint256) {
        return a.add(b);
    }

    function testClamp(uint256 val) external pure returns (uint256) {
        return val.clamp(10, 100);
    }

    // ===== Mapping & Storage Patterns =====
    function setApproval(address operator, bool approved) external {
        approvals[msg.sender][operator] = approved;
    }

    function isApproved(address owner, address operator) external view returns (bool) {
        return approvals[owner][operator];
    }

    function setRoleWeight(bytes32 role, address member, uint256 weight) external onlyOwner {
        roleWeights[role][member] = weight;
    }

    // ===== Task Statistics (struct return) =====
    function getTaskStats() external view returns (TaskStats memory stats) {
        stats.total = nextTaskId - 1;
        for (uint256 i = 1; i < nextTaskId; i++) {
            Status s = tasks[i].status;
            if (s == Status.Pending) stats.pending++;
            else if (s == Status.Active) stats.active++;
            else if (s == Status.Completed) stats.completed++;
            else if (s == Status.Cancelled) stats.cancelled++;
        }
    }

    // ===== Array Operations =====
    function recordScore(uint256 score) external {
        scores.push(score);
        emit ScoreRecorded(msg.sender, score);
    }

    function getTotalScore() external view returns (uint256) {
        return scores.sum();
    }

    // ===== Multi-return =====
    function getContractInfo() external view returns (
        address contractOwner,
        bool isPaused,
        uint256 taskCount,
        uint256 memberCount,
        string memory version,
        uint256 deployed
    ) {
        return (owner, paused, nextTaskId - 1, members.length, VERSION, deployBlock);
    }
}
