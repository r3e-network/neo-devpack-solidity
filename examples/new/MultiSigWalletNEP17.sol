// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title MultiSigWalletNEP17
 * @notice Neo N3 multisig-style wallet for native GAS/NEO (NEP-17).
 *
 * Neo N3 does not support EVM-style `{value: ...}` calls. To move value you must
 * explicitly transfer NEP-17 tokens and handle deposits via `onNEP17Payment`.
 */
contract MultiSigWalletNEP17 {
    // Native contract hashes exposed by the devpack (lowered as compiler intrinsics).
    address private constant NEO = NativeCalls.NEO_CONTRACT;
    address private constant GAS = NativeCalls.GAS_CONTRACT;

    struct TransferTx {
        address token;
        address to;
        uint256 amount;
        bytes data;
        bool executed;
        uint256 confirmations;
    }

    address[] public owners;
    mapping(address => bool) public isOwner;
    uint256 public required;

    uint256 public transactionCount;
    mapping(uint256 => TransferTx) public transactions;
    mapping(uint256 => mapping(address => bool)) public confirmed;

    event Deposit(address token, address from, uint256 amount, bytes data);
    event Submission(uint256 indexed txId, address indexed token, address indexed to, uint256 amount);
    event Confirmation(address indexed owner, uint256 indexed txId);
    event Execution(uint256 indexed txId);

    modifier onlyOwner() {
        require(isOwner[msg.sender], "not owner");
        _;
    }

    modifier txExists(uint256 txId) {
        require(txId < transactionCount, "tx does not exist");
        _;
    }

    modifier notExecuted(uint256 txId) {
        require(!transactions[txId].executed, "tx already executed");
        _;
    }

    modifier notConfirmed(uint256 txId, address owner) {
        require(!confirmed[txId][owner], "tx already confirmed");
        _;
    }

    constructor(address[] memory _owners, uint256 _required) {
        require(_owners.length > 0, "owners required");
        require(_required > 0 && _required <= _owners.length, "invalid required");

        required = _required;

        for (uint256 i = 0; i < _owners.length; i++) {
            address owner = _owners[i];
            require(owner != address(0), "invalid owner");
            require(!isOwner[owner], "duplicate owner");
            isOwner[owner] = true;
            owners.push(owner);
        }
    }

    function onNEP17Payment(address from, uint256 amount, bytes memory data) external {
        address token = Syscalls.getCallingScriptHash();
        require(token == GAS || token == NEO, "unsupported token");
        emit Deposit(token, from, amount, data);
    }

    function submitTransfer(
        address token,
        address to,
        uint256 amount,
        bytes memory data
    ) external onlyOwner returns (uint256 txId) {
        require(token == GAS || token == NEO, "unsupported token");
        require(to != address(0), "invalid recipient");

        txId = transactionCount;
        transactionCount = txId + 1;

        TransferTx storage txn = transactions[txId];
        txn.token = token;
        txn.to = to;
        txn.amount = amount;
        txn.data = data;
        txn.executed = false;
        txn.confirmations = 0;

        emit Submission(txId, token, to, amount);

        confirm(txId);
    }

    function confirm(uint256 txId)
        public
        onlyOwner
        txExists(txId)
        notExecuted(txId)
        notConfirmed(txId, msg.sender)
    {
        confirmed[txId][msg.sender] = true;
        transactions[txId].confirmations += 1;
        emit Confirmation(msg.sender, txId);

        if (transactions[txId].confirmations >= required) {
            execute(txId);
        }
    }

    function execute(uint256 txId) public onlyOwner txExists(txId) notExecuted(txId) {
        TransferTx storage txn = transactions[txId];
        require(txn.confirmations >= required, "insufficient confirmations");

        txn.executed = true;
        bool success = _transfer(txn.token, txn.to, txn.amount, txn.data);
        require(success, "transfer failed");

        emit Execution(txId);
    }

    function _transfer(
        address token,
        address to,
        uint256 amount,
        bytes memory data
    ) internal returns (bool) {
        if (token == GAS) {
            return NativeCalls.gasTransfer(address(this), to, amount, data);
        }
        if (token == NEO) {
            return NativeCalls.neoTransfer(address(this), to, amount, data);
        }
        return false;
    }
}

