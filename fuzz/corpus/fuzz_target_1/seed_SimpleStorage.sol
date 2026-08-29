// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/// @title SimpleStorage - Basic storage operations for Neo N3
/// @author Neo DevPack for Solidity Team
/// @notice A simple contract demonstrating storage read/write operations
/// @dev Uses Neo N3 storage primitives under the hood
contract SimpleStorage {
    // State variables
    uint256 private storedNumber;
    string private storedString;
    address private owner;
    bool private locked;

    // Mapping for key-value storage
    mapping(bytes32 => uint256) private numberStore;
    mapping(bytes32 => string) private stringStore;
    mapping(address => uint256) private balances;

    // Events
    event NumberStored(uint256 indexed oldValue, uint256 indexed newValue, address indexed setter);
    event StringStored(string value, address indexed setter);
    event KeyValueStored(bytes32 indexed key, uint256 value);
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);

    /// @notice Contract constructor
    /// @dev Sets the deployer as the owner
    constructor() {
        owner = msg.sender;
        storedNumber = 0;
        storedString = "";
        locked = false;
    }

    /// @notice Modifier to restrict access to owner only
    modifier onlyOwner() {
        require(msg.sender == owner, "SimpleStorage: caller is not the owner");
        _;
    }

    /// @notice Modifier to prevent reentrancy
    modifier nonReentrant() {
        require(!locked, "SimpleStorage: reentrant call");
        locked = true;
        _;
        locked = false;
    }

    /// @notice Store a number
    /// @param num The number to store
    function setNumber(uint256 num) public {
        uint256 oldValue = storedNumber;
        storedNumber = num;
        emit NumberStored(oldValue, num, msg.sender);
    }

    /// @notice Retrieve the stored number
    /// @return The stored number
    function getNumber() public view returns (uint256) {
        return storedNumber;
    }

    /// @notice Store a string
    /// @param str The string to store
    function setString(string memory str) public {
        storedString = str;
        emit StringStored(str, msg.sender);
    }

    /// @notice Retrieve the stored string
    /// @return The stored string
    function getString() public view returns (string memory) {
        return storedString;
    }

    /// @notice Store a number with a key
    /// @param key The key to store under
    /// @param value The value to store
    function setKeyValue(bytes32 key, uint256 value) public {
        numberStore[key] = value;
        emit KeyValueStored(key, value);
    }

    /// @notice Retrieve a number by key
    /// @param key The key to look up
    /// @return The stored value
    function getKeyValue(bytes32 key) public view returns (uint256) {
        return numberStore[key];
    }

    /// @notice Store a string with a key
    /// @param key The key to store under
    /// @param value The string value to store
    function setKeyString(bytes32 key, string memory value) public {
        stringStore[key] = value;
    }

    /// @notice Retrieve a string by key
    /// @param key The key to look up
    /// @return The stored string
    function getKeyString(bytes32 key) public view returns (string memory) {
        return stringStore[key];
    }

    /// @notice Increment the stored number
    /// @return The new value after increment
    function increment() public returns (uint256) {
        storedNumber += 1;
        return storedNumber;
    }

    /// @notice Decrement the stored number
    /// @return The new value after decrement
    function decrement() public returns (uint256) {
        require(storedNumber > 0, "SimpleStorage: cannot decrement below zero");
        storedNumber -= 1;
        return storedNumber;
    }

    /// @notice Add to the stored number
    /// @param amount The amount to add
    /// @return The new value
    function add(uint256 amount) public returns (uint256) {
        storedNumber += amount;
        return storedNumber;
    }

    /// @notice Get the contract owner
    /// @return The owner address
    function getOwner() public view returns (address) {
        return owner;
    }

    /// @notice Transfer ownership to a new address
    /// @param newOwner The new owner address
    function transferOwnership(address newOwner) public onlyOwner {
        require(newOwner != address(0), "SimpleStorage: new owner is zero address");
        address oldOwner = owner;
        owner = newOwner;
        emit OwnershipTransferred(oldOwner, newOwner);
    }

    /// @notice Reset all stored values (owner only)
    function reset() public onlyOwner nonReentrant {
        storedNumber = 0;
        storedString = "";
    }

    /// @notice Check if a key exists in number store
    /// @param key The key to check
    /// @return True if key has a non-zero value
    function hasKey(bytes32 key) public view returns (bool) {
        return numberStore[key] != 0;
    }

    /// @notice Get balance of an address
    /// @param account The address to check
    /// @return The balance
    function balanceOf(address account) public view returns (uint256) {
        return balances[account];
    }

    /// @notice Set balance for an address (owner only)
    /// @param account The address
    /// @param amount The balance to set
    function setBalance(address account, uint256 amount) public onlyOwner {
        balances[account] = amount;
    }
}
