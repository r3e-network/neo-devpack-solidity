// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title MultiSigWallet
 * @dev Multi-signature wallet implementation for Neo N3.
 *
 * Neo N3 does not support EVM-style `{value: ...}` calls. This example models "value" as
 * native GAS (NEP-17) and executes transactions as `GAS.transfer(address(this), to, amount, data)`.
 *
 * Features: multi-sig approvals, owner management, daily limits, emergency functions.
 */
contract MultiSigWallet {
    // Constants
    uint256 public constant MAX_OWNER_COUNT = 50;
    uint256 public constant MAX_REQUIRED = 50;
    
    // Events
    event Confirmation(address indexed sender, uint256 indexed transactionId);
    event Revocation(address indexed sender, uint256 indexed transactionId);
    event Submission(uint256 indexed transactionId);
    event Execution(uint256 indexed transactionId);
    event ExecutionFailure(uint256 indexed transactionId);
    event Deposit(address indexed sender, uint256 value);
    event OwnerAddition(address indexed owner);
    event OwnerRemoval(address indexed owner);
    event RequirementChange(uint256 required);
    event DailyLimitChange(uint256 dailyLimit);
    event EmergencyStop();
    event EmergencyResume();
    
    // Storage
    mapping(uint256 => Transaction) public transactions;
    mapping(uint256 => mapping(address => bool)) public confirmations;
    mapping(address => bool) public isOwner;
    address[] public owners;
    uint256 public required;
    uint256 public transactionCount;
    
    // Daily limit functionality
    mapping(address => DailyLimit) public dailyLimits;
    mapping(address => mapping(uint256 => uint256)) public dailySpent;
    uint256 public defaultDailyLimit;
    
    // Emergency functionality
    bool public emergencyStopped;
    address public emergencyAdmin;
    uint256 public emergencyTimeout;
    uint256 public emergencyStartTime;
    
    struct Transaction {
        address destination;
        uint256 value;
        bytes data;
        bool executed;
        uint256 timestamp;
        string description;
    }
    
    struct DailyLimit {
        uint256 limit;
        bool isSet;
    }
    
    // Custom errors
    error OnlyWallet();
    error OwnerDoesNotExist();
    error OwnerExists();
    error TransactionDoesNotExist();
    error TransactionAlreadyExecuted();
    error TransactionAlreadyConfirmed();
    error TransactionNotConfirmed();
    error NotConfirmed();
    error InvalidOwnerCount();
    error InvalidRequirement();
    error ZeroAddress();
    error DailyLimitExceeded();
    error EmergencyActive();
    error NotEmergencyAdmin();
    error EmergencyTimeoutNotReached();
    error ExecutionFailed();
    error InsufficientBalance();
    error InvalidValue();
    
    // Modifiers
    modifier onlyWallet() {
        if (msg.sender != address(this)) revert OnlyWallet();
        _;
    }
    
    modifier ownerDoesNotExist(address owner) {
        if (isOwner[owner]) revert OwnerExists();
        _;
    }
    
    modifier ownerExists(address owner) {
        if (!isOwner[owner]) revert OwnerDoesNotExist();
        _;
    }
    
    modifier transactionExists(uint256 transactionId) {
        if (transactions[transactionId].destination == address(0)) revert TransactionDoesNotExist();
        _;
    }
    
    modifier confirmed(uint256 transactionId, address owner) {
        if (!confirmations[transactionId][owner]) revert TransactionNotConfirmed();
        _;
    }
    
    modifier notConfirmed(uint256 transactionId, address owner) {
        if (confirmations[transactionId][owner]) revert TransactionAlreadyConfirmed();
        _;
    }
    
    modifier notExecuted(uint256 transactionId) {
        if (transactions[transactionId].executed) revert TransactionAlreadyExecuted();
        _;
    }
    
    modifier notNull(address _address) {
        if (_address == address(0)) revert ZeroAddress();
        _;
    }
    
    modifier validRequirement(uint256 ownerCount, uint256 _required) {
        if (ownerCount > MAX_OWNER_COUNT || _required > ownerCount || _required == 0 || ownerCount == 0) {
            revert InvalidRequirement();
        }
        _;
    }
    
    modifier notInEmergency() {
        if (emergencyStopped) revert EmergencyActive();
        _;
    }
    
    modifier onlyEmergencyAdmin() {
        if (msg.sender != emergencyAdmin) revert NotEmergencyAdmin();
        _;
    }
    
    /**
     * @dev Contract constructor sets initial owners, required confirmations, and daily limit
     * @param _owners List of initial owners
     * @param _required Number of required confirmations
     * @param _dailyLimit Default daily spending limit
     * @param _emergencyAdmin Address that can trigger emergency stop
     * @param _emergencyTimeout Time after which emergency can be resolved
     */
    constructor(
        address[] memory _owners,
        uint256 _required,
        uint256 _dailyLimit,
        address _emergencyAdmin,
        uint256 _emergencyTimeout
    )
        validRequirement(_owners.length, _required)
    {
        require(_emergencyAdmin != address(0), "Invalid emergency admin");
        require(_emergencyTimeout > 0, "Invalid emergency timeout");
        
        for (uint256 i = 0; i < _owners.length; i++) {
            require(_owners[i] != address(0), "Invalid owner");
            require(!isOwner[_owners[i]], "Duplicate owner");
            
            isOwner[_owners[i]] = true;
        }
        
        owners = _owners;
        required = _required;
        defaultDailyLimit = _dailyLimit;
        emergencyAdmin = _emergencyAdmin;
        emergencyTimeout = _emergencyTimeout;
        emergencyStopped = false;
    }
    
    /**
     * @dev Neo N3 NEP-17 token deposit hook.
     *
     * When a user transfers GAS to this contract via `GAS.transfer(from, this, amount, data)`,
     * the GAS native contract invokes this callback. We accept GAS deposits and emit a
     * Deposit event using the callback parameters directly (not msg.value, which is
     * meaningless outside onNEP17Payment on Neo N3).
     */
    function onNEP17Payment(address from, uint256 amount, bytes memory /*data*/) external {
        address token = Syscalls.getCallingScriptHash();
        require(token == NativeCalls.GAS_CONTRACT, "MultiSigWallet: only GAS accepted");

        if (amount > 0) {
            emit Deposit(from, amount);
        }
    }
    
    /**
     * @dev Allows to add a new owner. Transaction has to be sent by wallet
     * @param owner Address of new owner
     */
    function addOwner(address owner)
        public
        onlyWallet
        ownerDoesNotExist(owner)
        notNull(owner)
        validRequirement(owners.length + 1, required)
    {
        isOwner[owner] = true;
        owners.push(owner);
        emit OwnerAddition(owner);
    }
    
    /**
     * @dev Allows to remove an owner. Transaction has to be sent by wallet
     * @param owner Address of owner to remove
     */
    function removeOwner(address owner)
        public
        onlyWallet
        ownerExists(owner)
    {
        isOwner[owner] = false;
        
        for (uint256 i = 0; i < owners.length - 1; i++) {
            if (owners[i] == owner) {
                owners[i] = owners[owners.length - 1];
                break;
            }
        }
        owners.pop();
        
        if (required > owners.length) {
            changeRequirement(owners.length);
        }
        
        emit OwnerRemoval(owner);
    }
    
    /**
     * @dev Allows to replace an owner with a new owner. Transaction has to be sent by wallet
     * @param owner Address of owner to be replaced
     * @param newOwner Address of new owner
     */
    function replaceOwner(address owner, address newOwner)
        public
        onlyWallet
        ownerExists(owner)
        ownerDoesNotExist(newOwner)
        notNull(newOwner)
    {
        for (uint256 i = 0; i < owners.length; i++) {
            if (owners[i] == owner) {
                owners[i] = newOwner;
                break;
            }
        }
        
        isOwner[owner] = false;
        isOwner[newOwner] = true;
        
        emit OwnerRemoval(owner);
        emit OwnerAddition(newOwner);
    }
    
    /**
     * @dev Allows to change the number of required confirmations. Transaction has to be sent by wallet
     * @param _required Number of required confirmations
     */
    function changeRequirement(uint256 _required)
        public
        onlyWallet
        validRequirement(owners.length, _required)
    {
        required = _required;
        emit RequirementChange(_required);
    }
    
    /**
     * @dev Allows an owner to submit and confirm a transaction
     * @param destination Transaction target address
     * @param value Transaction ether value
     * @param data Transaction data payload
     * @param description Human readable description of the transaction
     * @return transactionId Returns transaction ID
     */
    function submitTransaction(
        address destination,
        uint256 value,
        bytes memory data,
        string memory description
    )
        public
        ownerExists(msg.sender)
        notInEmergency
        returns (uint256 transactionId)
    {
        transactionId = addTransaction(destination, value, data, description);
        confirmTransaction(transactionId);
    }
    
    /**
     * @dev Allows an owner to confirm a transaction
     * @param transactionId Transaction ID
     */
    function confirmTransaction(uint256 transactionId)
        public
        ownerExists(msg.sender)
        transactionExists(transactionId)
        notConfirmed(transactionId, msg.sender)
        notInEmergency
    {
        confirmations[transactionId][msg.sender] = true;
        emit Confirmation(msg.sender, transactionId);
        
        executeTransaction(transactionId);
    }
    
    /**
     * @dev Allows an owner to revoke a confirmation for a transaction
     * @param transactionId Transaction ID
     */
    function revokeConfirmation(uint256 transactionId)
        public
        ownerExists(msg.sender)
        confirmed(transactionId, msg.sender)
        notExecuted(transactionId)
    {
        confirmations[transactionId][msg.sender] = false;
        emit Revocation(msg.sender, transactionId);
    }
    
    /**
     * @dev Allows anyone to execute a confirmed transaction
     * @param transactionId Transaction ID
     */
    function executeTransaction(uint256 transactionId)
        public
        ownerExists(msg.sender)
        confirmed(transactionId, msg.sender)
        notExecuted(transactionId)
        notInEmergency
    {
        if (isConfirmed(transactionId)) {
            Transaction storage txn = transactions[transactionId];
            
            // Check daily limit
            if (txn.value > 0) {
                checkDailyLimit(txn.destination, txn.value);
                updateDailySpent(txn.destination, txn.value);
            }
            
            // Check balance
            if (txn.value > address(this).balance) {
                revert InsufficientBalance();
            }
            
            txn.executed = true;
            
            bool success = NativeCalls.gasTransfer(address(this), txn.destination, txn.value, txn.data);
            if (success) {
                emit Execution(transactionId);
            } else {
                emit ExecutionFailure(transactionId);
                txn.executed = false;
            }
        }
    }
    
    /**
     * @dev Returns the confirmation status of a transaction
     * @param transactionId Transaction ID
     * @return Confirmation status
     */
    function isConfirmed(uint256 transactionId)
        public
        view
        returns (bool)
    {
        uint256 count = 0;
        for (uint256 i = 0; i < owners.length; i++) {
            if (confirmations[transactionId][owners[i]]) {
                count += 1;
            }
            if (count == required) {
                return true;
            }
        }
        return false;
    }
    
    /**
     * @dev Adds a new transaction to the transaction mapping, if transaction does not exist yet
     * @param destination Transaction target address
     * @param value Transaction ether value
     * @param data Transaction data payload
     * @param description Human readable description
     * @return transactionId Returns transaction ID
     */
    function addTransaction(
        address destination,
        uint256 value,
        bytes memory data,
        string memory description
    )
        internal
        notNull(destination)
        returns (uint256 transactionId)
    {
        transactionId = transactionCount;
        transactions[transactionId] = Transaction({
            destination: destination,
            value: value,
            data: data,
            executed: false,
            timestamp: block.timestamp,
            description: description
        });
        transactionCount += 1;
        emit Submission(transactionId);
    }
    
    /**
     * @dev Returns number of confirmations of a transaction
     * @param transactionId Transaction ID
     * @return count Number of confirmations
     */
    function getConfirmationCount(uint256 transactionId)
        public
        view
        returns (uint256 count)
    {
        for (uint256 i = 0; i < owners.length; i++) {
            if (confirmations[transactionId][owners[i]]) {
                count += 1;
            }
        }
    }
    
    /**
     * @dev Returns total number of transactions after filters are applied
     * @param pending Include pending transactions
     * @param executed Include executed transactions
     * @return count Total number of transactions after filters are applied
     */
    function getTransactionCount(bool pending, bool executed)
        public
        view
        returns (uint256 count)
    {
        for (uint256 i = 0; i < transactionCount; i++) {
            if ((pending && !transactions[i].executed) || (executed && transactions[i].executed)) {
                count += 1;
            }
        }
    }
    
    /**
     * @dev Returns list of owners
     * @return List of owner addresses
     */
    function getOwners()
        public
        view
        returns (address[] memory)
    {
        return owners;
    }
    
    /**
     * @dev Returns array with owner addresses, which confirmed transaction
     * @param transactionId Transaction ID
     * @return _confirmations Returns array of owner addresses
     */
    function getConfirmations(uint256 transactionId)
        public
        view
        returns (address[] memory _confirmations)
    {
        address[] memory confirmationsTemp = new address[](owners.length);
        uint256 count = 0;
        for (uint256 i = 0; i < owners.length; i++) {
            if (confirmations[transactionId][owners[i]]) {
                confirmationsTemp[count] = owners[i];
                count += 1;
            }
        }
        _confirmations = new address[](count);
        for (uint256 i = 0; i < count; i++) {
            _confirmations[i] = confirmationsTemp[i];
        }
    }
    
    /**
     * @dev Returns list of transaction IDs in defined range
     * @param from Index start position of transaction array
     * @param to Index end position of transaction array
     * @param pending Include pending transactions
     * @param executed Include executed transactions
     * @return _transactionIds Returns array of transaction IDs
     */
    function getTransactionIds(
        uint256 from,
        uint256 to,
        bool pending,
        bool executed
    )
        public
        view
        returns (uint256[] memory _transactionIds)
    {
        uint256[] memory transactionIdsTemp = new uint256[](transactionCount);
        uint256 count = 0;
        for (uint256 i = 0; i < transactionCount; i++) {
            if ((pending && !transactions[i].executed) || (executed && transactions[i].executed)) {
                transactionIdsTemp[count] = i;
                count += 1;
            }
        }
        _transactionIds = new uint256[](to - from);
        for (uint256 i = from; i < to; i++) {
            _transactionIds[i - from] = transactionIdsTemp[i];
        }
    }
    
    // Daily limit functionality
    
    /**
     * @dev Set daily limit for a specific destination
     * @param destination Address to set limit for
     * @param limit Daily limit amount
     */
    function setDailyLimit(address destination, uint256 limit)
        public
        onlyWallet
    {
        dailyLimits[destination] = DailyLimit(limit, true);
        emit DailyLimitChange(limit);
    }
    
    /**
     * @dev Set default daily limit
     * @param limit Daily limit amount
     */
    function setDefaultDailyLimit(uint256 limit)
        public
        onlyWallet
    {
        defaultDailyLimit = limit;
        emit DailyLimitChange(limit);
    }
    
    /**
     * @dev Check if transaction is within daily limit
     * @param destination Transaction destination
     * @param value Transaction value
     */
    function checkDailyLimit(address destination, uint256 value)
        internal
        view
    {
        uint256 limit = dailyLimits[destination].isSet 
            ? dailyLimits[destination].limit 
            : defaultDailyLimit;
            
        if (limit == 0) return; // No limit
        
        uint256 today = block.timestamp / 1 days;
        if (dailySpent[destination][today] + value > limit) {
            revert DailyLimitExceeded();
        }
    }
    
    /**
     * @dev Update daily spent amount
     * @param destination Transaction destination
     * @param value Transaction value
     */
    function updateDailySpent(address destination, uint256 value)
        internal
    {
        uint256 today = block.timestamp / 1 days;
        dailySpent[destination][today] += value;
    }
    
    /**
     * @dev Get remaining daily limit for destination
     * @param destination Address to check
     * @return remaining Remaining daily limit
     */
    function getRemainingDailyLimit(address destination)
        public
        view
        returns (uint256 remaining)
    {
        uint256 limit = dailyLimits[destination].isSet 
            ? dailyLimits[destination].limit 
            : defaultDailyLimit;
            
        if (limit == 0) return type(uint256).max; // No limit
        
        uint256 today = block.timestamp / 1 days;
        uint256 spent = dailySpent[destination][today];
        
        return spent >= limit ? 0 : limit - spent;
    }
    
    // Emergency functionality
    
    /**
     * @dev Trigger emergency stop (only emergency admin)
     */
    function emergencyStop()
        public
        onlyEmergencyAdmin
    {
        emergencyStopped = true;
        emergencyStartTime = block.timestamp;
        emit EmergencyStop();
    }
    
    /**
     * @dev Resume operations after emergency timeout
     */
    function emergencyResume()
        public
    {
        require(emergencyStopped, "No emergency active");
        require(
            block.timestamp >= emergencyStartTime + emergencyTimeout,
            "Emergency timeout not reached"
        );
        
        emergencyStopped = false;
        emergencyStartTime = 0;
        emit EmergencyResume();
    }
    
    /**
     * @dev Emergency transaction execution (only when stopped and by admin)
     * @param destination Transaction target address
     * @param value Transaction ether value
     * @param data Transaction data payload
     */
    function emergencyExecute(
        address destination,
        uint256 value,
        bytes memory data
    )
        public
        onlyEmergencyAdmin
    {
        require(emergencyStopped, "Emergency not active");
        
        if (value > address(this).balance) {
            revert InsufficientBalance();
        }
        
        bool success = NativeCalls.gasTransfer(address(this), destination, value, data);
        if (!success) {
            revert ExecutionFailed();
        }
    }
    
    /**
     * @dev Change emergency admin (only wallet can call)
     * @param newAdmin New emergency admin address
     */
    function changeEmergencyAdmin(address newAdmin)
        public
        onlyWallet
        notNull(newAdmin)
    {
        emergencyAdmin = newAdmin;
    }
    
    /**
     * @dev Change emergency timeout (only wallet can call)
     * @param newTimeout New emergency timeout
     */
    function changeEmergencyTimeout(uint256 newTimeout)
        public
        onlyWallet
    {
        require(newTimeout > 0, "Invalid timeout");
        emergencyTimeout = newTimeout;
    }
    
    // Batch operations
    
    /**
     * @dev Submit multiple transactions at once
     * @param destinations Array of transaction target addresses
     * @param values Array of transaction values
     * @param dataArray Array of transaction data payloads
     * @param descriptions Array of descriptions
     * @return transactionIds Array of created transaction IDs
     */
    function submitMultipleTransactions(
        address[] memory destinations,
        uint256[] memory values,
        bytes[] memory dataArray,
        string[] memory descriptions
    )
        public
        ownerExists(msg.sender)
        notInEmergency
        returns (uint256[] memory transactionIds)
    {
        require(destinations.length == values.length, "Array length mismatch");
        require(destinations.length == dataArray.length, "Array length mismatch");
        require(destinations.length == descriptions.length, "Array length mismatch");
        require(destinations.length > 0, "Empty arrays");
        require(destinations.length <= 10, "Too many transactions");
        
        transactionIds = new uint256[](destinations.length);
        
        for (uint256 i = 0; i < destinations.length; i++) {
            transactionIds[i] = addTransaction(
                destinations[i],
                values[i],
                dataArray[i],
                descriptions[i]
            );
            confirmTransaction(transactionIds[i]);
        }
    }
    
    /**
     * @dev Confirm multiple transactions at once
     * @param transactionIds Array of transaction IDs to confirm
     */
    function confirmMultipleTransactions(uint256[] memory transactionIds)
        public
        ownerExists(msg.sender)
        notInEmergency
    {
        require(transactionIds.length > 0, "Empty array");
        require(transactionIds.length <= 20, "Too many transactions");
        
        for (uint256 i = 0; i < transactionIds.length; i++) {
            if (!confirmations[transactionIds[i]][msg.sender] &&
                transactions[transactionIds[i]].destination != address(0) &&
                !transactions[transactionIds[i]].executed) {
                confirmTransaction(transactionIds[i]);
            }
        }
    }
    
    // View functions for web3 integration
    
    /**
     * @dev Get transaction details
     * @param transactionId Transaction ID
     * @return destination Transaction target
     * @return value Transaction value
     * @return data Transaction data
     * @return executed Whether transaction is executed
     * @return timestamp Transaction creation time
     * @return description Transaction description
     */
    function getTransactionDetails(uint256 transactionId)
        public
        view
        returns (
            address destination,
            uint256 value,
            bytes memory data,
            bool executed,
            uint256 timestamp,
            string memory description
        )
    {
        Transaction memory txn = transactions[transactionId];
        return (
            txn.destination,
            txn.value,
            txn.data,
            txn.executed,
            txn.timestamp,
            txn.description
        );
    }
    
    /**
     * @dev Get wallet info
     * @return ownerCount Number of owners
     * @return requiredConfirmations Required confirmations
     * @return transactionCount Total transactions
     * @return balance Wallet balance
     * @return emergencyStatus Emergency stop status
     */
    function getWalletInfo()
        public
        view
        returns (
            uint256 ownerCount,
            uint256 requiredConfirmations,
            uint256 transactionCount_,
            uint256 balance,
            bool emergencyStatus
        )
    {
        return (
            owners.length,
            required,
            transactionCount,
            address(this).balance,
            emergencyStopped
        );
    }
}
