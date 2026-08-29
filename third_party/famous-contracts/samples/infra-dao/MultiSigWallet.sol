// SPDX-License-Identifier: LGPL-3.0-only
// A minimal Gnosis-Safe-style M-of-N multisig wallet.
// Inspired by the Gnosis Safe threshold model (owners + threshold + per-tx
// confirmation), condensed into a single self-contained, dependency-free file.
// Transaction proposals are confirmed by owners and executed once the threshold
// number of confirmations is reached.
pragma solidity ^0.8.0;

contract MultiSigWallet {
    event OwnerAdded(address indexed owner);
    event SubmitTransaction(address indexed owner, uint256 indexed txId, address indexed to, uint256 value);
    event ConfirmTransaction(address indexed owner, uint256 indexed txId);
    event RevokeConfirmation(address indexed owner, uint256 indexed txId);
    event ExecuteTransaction(address indexed owner, uint256 indexed txId);

    struct Transaction {
        address to;
        uint256 value;
        bytes data;
        bool executed;
        uint256 numConfirmations;
    }

    address[] public owners;
    mapping(address => bool) public isOwner;
    uint256 public threshold;

    Transaction[] public transactions;
    // txId => owner => confirmed
    mapping(uint256 => mapping(address => bool)) public isConfirmed;

    modifier onlyOwner() {
        require(isOwner[msg.sender], "not owner");
        _;
    }

    modifier txExists(uint256 txId) {
        require(txId < transactions.length, "tx does not exist");
        _;
    }

    modifier notExecuted(uint256 txId) {
        require(!transactions[txId].executed, "tx already executed");
        _;
    }

    modifier notConfirmed(uint256 txId) {
        require(!isConfirmed[txId][msg.sender], "tx already confirmed");
        _;
    }

    constructor(address[] memory _owners, uint256 _threshold) {
        require(_owners.length > 0, "owners required");
        require(_threshold > 0 && _threshold <= _owners.length, "invalid threshold");

        for (uint256 i = 0; i < _owners.length; i++) {
            address owner = _owners[i];
            require(owner != address(0), "invalid owner");
            require(!isOwner[owner], "owner not unique");

            isOwner[owner] = true;
            owners.push(owner);
            emit OwnerAdded(owner);
        }

        threshold = _threshold;
    }

    function submitTransaction(
        address to,
        uint256 value,
        bytes memory data
    ) external onlyOwner returns (uint256 txId) {
        txId = transactions.length;
        transactions.push(
            Transaction({to: to, value: value, data: data, executed: false, numConfirmations: 0})
        );
        emit SubmitTransaction(msg.sender, txId, to, value);
    }

    function confirmTransaction(uint256 txId)
        external
        onlyOwner
        txExists(txId)
        notExecuted(txId)
        notConfirmed(txId)
    {
        Transaction storage transaction = transactions[txId];
        transaction.numConfirmations += 1;
        isConfirmed[txId][msg.sender] = true;
        emit ConfirmTransaction(msg.sender, txId);
    }

    function revokeConfirmation(uint256 txId)
        external
        onlyOwner
        txExists(txId)
        notExecuted(txId)
    {
        Transaction storage transaction = transactions[txId];
        require(isConfirmed[txId][msg.sender], "tx not confirmed");
        transaction.numConfirmations -= 1;
        isConfirmed[txId][msg.sender] = false;
        emit RevokeConfirmation(msg.sender, txId);
    }

    function executeTransaction(uint256 txId)
        external
        onlyOwner
        txExists(txId)
        notExecuted(txId)
    {
        Transaction storage transaction = transactions[txId];
        require(transaction.numConfirmations >= threshold, "not enough confirmations");
        transaction.executed = true;
        emit ExecuteTransaction(msg.sender, txId);
    }

    function getOwners() external view returns (address[] memory) {
        return owners;
    }

    function getTransactionCount() external view returns (uint256) {
        return transactions.length;
    }
}
