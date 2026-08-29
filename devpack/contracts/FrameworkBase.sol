// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title Neo N3 Framework Base
 * @dev Base framework providing Neo N3 blockchain integration for Solidity contracts.
 *
 * This base intentionally avoids exposing fully-dynamic contract call helpers
 * (dynamic target + dynamic method name), because those force full wildcard
 * permissions in the Neo N3 manifest (`{"contract":"*","methods":"*"}`).
 *
 * If you need a public dynamic call surface, use `Framework.sol` instead.
 */

import "./Syscalls.sol";
import "./NativeCalls.sol";
import "../libraries/Neo.sol";
import "../libraries/Storage.sol";
import "../libraries/Runtime.sol";

contract FrameworkBase {
    using Neo for *;
    using Storage for *;
    using Runtime for *;

    // Framework state
    address private _owner;
    bool private _initialized;
    uint256 private _version;

    // Neo N3 integration
    bytes4 private constant NEO_MAGIC = 0x3346454E; // "NEF3"
    uint256 private constant MIN_GAS_LIMIT = 20000000; // 0.2 GAS

    // Events
    event FrameworkInitialized(address indexed owner, uint256 version);
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
    event ContractUpgraded(uint256 indexed oldVersion, uint256 indexed newVersion);
    event EmergencyStop(address indexed caller, string reason);

    // Modifiers
    modifier onlyOwner() {
        require(msg.sender == _owner, "Framework: caller is not the owner");
        _;
    }

    modifier whenInitialized() {
        require(_initialized, "Framework: not initialized");
        _;
    }

    modifier withWitness() {
        require(Runtime.checkWitness(msg.sender), "Framework: invalid witness");
        _;
    }

    modifier withGasLimit(uint256 minGas) {
        require(Runtime.gasLeft() >= minGas, "Framework: insufficient gas");
        _;
    }

    /**
     * @dev Initialize the framework
     */
    constructor() {
        _owner = msg.sender;
        _version = 1;
        _initialized = true;

        emit FrameworkInitialized(_owner, _version);
    }

    /**
     * @dev Get framework version
     */
    function version() public view returns (uint256) {
        return _version;
    }

    /**
     * @dev Get contract owner
     */
    function owner() public view returns (address) {
        return _owner;
    }

    /**
     * @dev Check if framework is initialized
     */
    function initialized() public view returns (bool) {
        return _initialized;
    }

    /**
     * @dev Transfer ownership of the contract
     */
    function transferOwnership(address newOwner) public onlyOwner withWitness {
        require(newOwner != address(0), "Framework: new owner is the zero address");
        require(newOwner != _owner, "Framework: new owner is the same as current owner");

        emit OwnershipTransferred(_owner, newOwner);
        _owner = newOwner;
    }

    /**
     * @dev Renounce ownership of the contract
     */
    function renounceOwnership() public onlyOwner withWitness {
        emit OwnershipTransferred(_owner, address(0));
        _owner = address(0);
    }

    /**
     * @dev Upgrade contract version
     */
    function upgradeContract(bytes calldata nef, bytes calldata manifest, uint256 newVersion)
        public
        onlyOwner
        withWitness
        withGasLimit(MIN_GAS_LIMIT)
    {
        require(newVersion > _version, "Framework: version must be higher");
        require(nef.length > 0, "Framework: NEF cannot be empty");
        require(manifest.length > 0, "Framework: manifest cannot be empty");

        uint256 oldVersion = _version;
        _version = newVersion;

        // Call ContractManagement.update
        NativeCalls.updateContract(nef, manifest);

        emit ContractUpgraded(oldVersion, newVersion);
    }

    /**
     * @dev Emergency stop function
     */
    function emergencyStop(string calldata reason) public onlyOwner withWitness {
        // Emit emergency event
        emit EmergencyStop(msg.sender, reason);

        // Optional: Call self-destruct via ContractManagement
        // NativeCalls.destroyContract();
    }

    /**
     * @dev Get current block information.
     * @notice `hash` and `merkleRoot` return zero because Neo N3 does not
     *         expose per-block hash or merkle root via lightweight syscalls.
     *         Use `Syscalls.contractCall(LEDGER_CONTRACT, "getBlock", ...)`
     *         for full block data when needed.
     */
    function getCurrentBlock()
        public
        view
        returns (uint256 index, bytes32 hash, uint256 timestamp, bytes32 merkleRoot)
    {
        index = Syscalls.getCurrentIndex();
        hash = bytes32(0);           // Not available via syscall; see @notice
        timestamp = Syscalls.getTime();
        merkleRoot = bytes32(0);     // Not available via syscall; see @notice
    }

    /**
     * @dev Get transaction information
     */
    function getTransaction(bytes32 txHash)
        public
        view
        returns (bytes32 hash, uint256 nonce, address sender, uint256 gasLimit, uint256 gasPrice)
    {
        // Neo N3 exposes full transaction information via the Ledger native contract,
        // but its shape does not map 1:1 to EVM transaction fields.
        //
        // Provide a minimal, deterministic subset that is safe for diagnostics-style use.
        Syscalls.Transaction memory tx = Syscalls.getTransaction(txHash);
        hash = tx.hash;
        nonce = tx.nonce;
        sender = tx.sender;
        gasLimit = tx.systemFee;
        gasPrice = Neo.getGasPrice();
    }

    /**
     * @dev Get contract balance (GAS)
     */
    function getBalance() public view returns (uint256) {
        return Neo.getGasBalance(address(this));
    }

    /**
     * @dev Get NEO balance
     */
    function getNeoBalance(address account) public view returns (uint256) {
        return Neo.getNeoBalance(account);
    }

    /**
     * @dev Transfer GAS to another address
     */
    function transferGas(address to, uint256 amount) public onlyOwner withWitness {
        require(to != address(0), "Framework: cannot transfer to zero address");
        require(amount > 0, "Framework: amount must be greater than zero");

        bool success = Neo.transferGas(address(this), to, amount);
        require(success, "Framework: GAS transfer failed");
    }

    /**
     * @dev Advanced storage operations
     */
    function setStorageValue(bytes calldata key, bytes calldata value) public onlyOwner {
        Storage.put(key, value);
    }

    function getStorageValue(bytes calldata key) public view returns (bytes memory) {
        return Storage.get(key);
    }

    function deleteStorageValue(bytes calldata key) public onlyOwner {
        Storage.remove(key);
    }

    /**
     * @dev Iterate through storage
     */
    function findStorageValues(bytes calldata prefix) public view returns (bytes[] memory values) {
        Storage.Iterator memory iterator = Storage.find(prefix);
        bytes[] memory temp = new bytes[](100); // Max 100 results
        uint256 count = 0;

        while (iterator.next() && count < 100) {
            temp[count] = iterator.value();
            count++;
        }

        values = new bytes[](count);
        for (uint256 i = 0; i < count; i++) {
            values[i] = temp[i];
        }
    }

    /**
     * @dev Deploy new contract
     */
    function deployContract(bytes calldata nef, bytes calldata manifest)
        public
        onlyOwner
        withWitness
        withGasLimit(MIN_GAS_LIMIT)
        returns (address)
    {
        return NativeCalls.deployContract(nef, manifest);
    }

    /**
     * @dev Get contract information via ContractManagement.getContract
     */
    function getContractInfo(address contractHash)
        public
        view
        returns (string memory name, bytes memory script, bytes memory manifest)
    {
        NativeCalls.ContractState memory state = NativeCalls.getContract(contractHash);
        name = "Contract"; // Neo N3 ContractState does not expose a name field
        script = state.nef;
        manifest = state.manifest;
    }

    /**
     * @dev Verify signature
     */
    function verifySignature(bytes32 hash, bytes calldata publicKey, bytes calldata signature)
        public
        pure
        returns (bool)
    {
        return Neo.verifySignature(hash, publicKey, signature);
    }

    /**
     * @dev Get random number (using Neo's randomness)
     */
    function getRandom() public view returns (uint256) {
        return Neo.getRandom();
    }

    /**
     * @dev Check if account is committee member
     */
    function isCommittee(address account) public view returns (bool) {
        return NativeCalls.isCommittee(account);
    }

    /**
     * @dev Get current committee members
     */
    function getCommittee() public view returns (bytes[] memory) {
        return Neo.getCommittee();
    }

    /**
     * @dev Batch operations for gas efficiency
     */
    function batchStorageOperations(bytes[] calldata keys, bytes[] calldata values, bool[] calldata isDelete)
        public
        onlyOwner
    {
        require(keys.length == values.length, "Framework: array length mismatch");
        require(keys.length == isDelete.length, "Framework: array length mismatch");
        require(keys.length > 0, "Framework: empty arrays");
        require(keys.length <= 100, "Framework: too many operations");

        for (uint256 i = 0; i < keys.length; i++) {
            if (isDelete[i]) {
                Storage.remove(keys[i]);
            } else {
                Storage.put(keys[i], values[i]);
            }
        }
    }

    /**
     * @dev Get gas consumption for operation
     */
    function estimateGas(bytes calldata operation) public view returns (uint256) {
        // Base gas estimation based on operation size.
        uint256 baseGas = 1000000; // 0.01 GAS
        uint256 dataGas = operation.length * 1000; // 0.00001 GAS per byte
        return baseGas + dataGas;
    }

    /**
     * @dev Get current gas price
     */
    function getGasPrice() public view returns (uint256) {
        return Neo.getGasPrice();
    }

    /**
     * @dev Get storage price per byte
     */
    function getStoragePrice() public view returns (uint256) {
        return Neo.getStoragePrice();
    }

    /**
     * @dev Framework diagnostics
     */
    function getDiagnostics()
        public
        view
        returns (
            uint256 currentBlock,
            uint256 gasBalance,
            uint256 neoBalance,
            bool isCommitteeMember
        )
    {
        currentBlock = block.number;
        gasBalance = Neo.getGasBalance(address(this));
        neoBalance = Neo.getNeoBalance(address(this));
        isCommitteeMember = isCommittee(address(this));
    }

    /**
     * @dev Framework metadata
     */
    function getFrameworkInfo()
        public
        pure
        returns (string memory name, string memory version, string memory author, string memory repository)
    {
        return (
            "Neo N3 Solidity Framework",
            "1.0.0",
            "Jimmy <jimmy@r3e.network>",
            "https://github.com/r3e-network/neo-devpack-solidity"
        );
    }
}
