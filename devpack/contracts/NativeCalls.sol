// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title Neo N3 Native Contract Calls
 * @dev Direct integration with Neo N3 native contracts
 * @author Jimmy <jimmy@r3e.network>
 * 
 * This library provides direct access to Neo N3 native contracts:
 * - NEO: Native NEO token and governance
 * - GAS: Native GAS token
 * - ContractManagement: Contract deployment and management
 * - Policy: Network policy management
 * - Oracle: Oracle services
 * - RoleManagement: Role and permission management
 * - Notary: Notary deposit services
 * - Treasury: Treasury funds management
 * - Ledger: Blockchain data access
 *
 * CryptoLib and StdLib methods are exposed through `Syscalls.sol` directly
 * (e.g. `Syscalls.sha256()`, `Syscalls.serialize()`, `Syscalls.base64Encode()`).
 * Their contract hash constants are defined here for reference.
 *
 * DOMAIN-SPECIFIC LIBRARIES: For better code organization, this library has been
 * split into focused domain modules under `native/`:
 *
 *   native/NativeNEO.sol           — NEO token & governance
 *   native/NativeGAS.sol           — GAS token
 *   native/NativeContractMgmt.sol  — Contract deployment & management
 *   native/NativePolicy.sol        — Network policy parameters
 *   native/NativeOracle.sol        — Oracle services
 *   native/NativeRole.sol          — Role management
 *   native/NativeLedger.sol        — Blockchain data access
 *   native/NativeNotary.sol        — Notary deposit services
 *   native/NativeTreasury.sol      — Treasury funds management
 *   native/NativeTypes.sol         — Data structure definitions
 *
 * Import domain libraries directly for finer-grained dependencies:
 *   import "./native/NativeNEO.sol";
 */

import "./Syscalls.sol";
import "./NativeContracts.sol";

library NativeCalls {
    using Syscalls for *;

    // Native contract addresses — imported from NativeContracts.sol (single source of truth).
    // NOTE: These aliases exist for backward compatibility. New code should use
    // `NativeContracts.<NAME>` directly.
    address constant NEO_CONTRACT = NativeContracts.NEO_CONTRACT;
    address constant GAS_CONTRACT = NativeContracts.GAS_CONTRACT;
    address constant CONTRACT_MANAGEMENT = NativeContracts.CONTRACT_MANAGEMENT;
    address constant POLICY_CONTRACT = NativeContracts.POLICY_CONTRACT;
    address constant ORACLE_CONTRACT = NativeContracts.ORACLE_CONTRACT;
    address constant ROLE_MANAGEMENT = NativeContracts.ROLE_MANAGEMENT;
    address constant NOTARY_CONTRACT = NativeContracts.NOTARY_CONTRACT;
    address constant TREASURY_CONTRACT = NativeContracts.TREASURY_CONTRACT;
    address constant LEDGER_CONTRACT = NativeContracts.LEDGER_CONTRACT;
    address constant CRYPTO_LIB = NativeContracts.CRYPTO_LIB;
    address constant STD_LIB = NativeContracts.STD_LIB;
    
    // ========== NEO Token Native Contract ==========
    
    /**
     * @dev Get NEO total supply
     */
    function neoTotalSupply() internal view returns (uint256) {
        bytes memory result = Syscalls.contractCall(NEO_CONTRACT, "totalSupply", "");
        return abi.decode(result, (uint256));
    }
    
    /**
     * @dev Get NEO balance of account
     */
    function neoBalanceOf(address account) internal view returns (uint256) {
        bytes memory params = abi.encode(account);
        bytes memory result = Syscalls.contractCall(NEO_CONTRACT, "balanceOf", params);
        return abi.decode(result, (uint256));
    }
    
    /**
     * @dev Transfer NEO tokens
     */
    function neoTransfer(address from, address to, uint256 amount, bytes memory data) 
        internal 
        returns (bool) 
    {
        bytes memory params = abi.encode(from, to, amount, data);
        bytes memory result = Syscalls.contractCall(NEO_CONTRACT, "transfer", params);
        return abi.decode(result, (bool));
    }
    
    /**
     * @dev Get NEO decimals
     */
    function neoDecimals() internal view returns (uint8) {
        bytes memory result = Syscalls.contractCall(NEO_CONTRACT, "decimals", "");
        return abi.decode(result, (uint8));
    }
    
    /**
     * @dev Get NEO symbol
     */
    function neoSymbol() internal view returns (string memory) {
        bytes memory result = Syscalls.contractCall(NEO_CONTRACT, "symbol", "");
        return abi.decode(result, (string));
    }
    
    /**
     * @dev Vote for validator
     */
    function vote(address account, bytes memory publicKey) internal returns (bool) {
        bytes memory params = abi.encode(account, publicKey);
        bytes memory result = Syscalls.contractCall(NEO_CONTRACT, "vote", params);
        return abi.decode(result, (bool));
    }
    
    /**
     * @dev Get candidates
     *
     * Neo N3: `NeoToken.getCandidates()` returns `(ECPoint publicKey, BigInteger votes)[]`
     * for the first 256 registered (non-blocked) candidates.
     */
    function getCandidates() internal view returns (NeoCandidate[] memory) {
        bytes memory result = Syscalls.contractCall(NEO_CONTRACT, "getCandidates", "");
        return abi.decode(result, (NeoCandidate[]));
    }

    /**
     * @dev Get all registered candidates (iterator)
     */
    function getAllCandidates() internal view returns (Syscalls.Iterator memory) {
        bytes memory result = Syscalls.contractCall(NEO_CONTRACT, "getAllCandidates", "");
        return abi.decode(result, (Syscalls.Iterator));
    }

    /**
     * @dev Get candidate vote count (returns -1 if not found)
     */
    function getCandidateVote(bytes memory publicKey) internal view returns (int256) {
        bytes memory params = abi.encode(publicKey);
        bytes memory result = Syscalls.contractCall(NEO_CONTRACT, "getCandidateVote", params);
        return abi.decode(result, (int256));
    }
    
    /**
     * @dev Register as candidate
     */
    function registerCandidate(bytes memory publicKey) internal returns (bool) {
        bytes memory params = abi.encode(publicKey);
        bytes memory result = Syscalls.contractCall(NEO_CONTRACT, "registerCandidate", params);
        return abi.decode(result, (bool));
    }
    
    /**
     * @dev Unregister candidate
     */
    function unregisterCandidate(bytes memory publicKey) internal returns (bool) {
        bytes memory params = abi.encode(publicKey);
        bytes memory result = Syscalls.contractCall(NEO_CONTRACT, "unregisterCandidate", params);
        return abi.decode(result, (bool));
    }
    
    /**
     * @dev Get GAS per block
     */
    function getGasPerBlock() internal view returns (uint256) {
        bytes memory result = Syscalls.contractCall(NEO_CONTRACT, "getGasPerBlock", "");
        return abi.decode(result, (uint256));
    }

    /**
     * @dev Get register price for candidates
     */
    function getRegisterPrice() internal view returns (uint256) {
        bytes memory result = Syscalls.contractCall(NEO_CONTRACT, "getRegisterPrice", "");
        return abi.decode(result, (uint256));
    }

    /**
     * @dev Set register price for candidates (committee only)
     */
    function setRegisterPrice(uint256 value) internal {
        bytes memory params = abi.encode(value);
        Syscalls.contractCall(NEO_CONTRACT, "setRegisterPrice", params);
    }
    
    /**
     * @dev Set GAS per block (committee only)
     */
    function setGasPerBlock(uint256 gasPerBlock) internal {
        bytes memory params = abi.encode(gasPerBlock);
        Syscalls.contractCall(NEO_CONTRACT, "setGasPerBlock", params);
    }
    
    /**
     * @dev Get account state
     *
     * Neo N3: `NeoToken.getAccountState(UInt160)` returns `NeoAccountState?` (nullable).
     * The compiler lowers this helper to return a default (zeroed) struct when the
     * native contract returns `null`, matching Solidity's "missing mapping key"
     * semantics.
     */
    function getAccountState(address account) internal view returns (AccountState memory) {
        bytes memory params = abi.encode(account);
        bytes memory result = Syscalls.contractCall(NEO_CONTRACT, "getAccountState", params);
        return abi.decode(result, (AccountState));
    }

    /**
     * @dev Get unclaimed GAS for account at a specific block height
     */
    function unclaimedGas(address account, uint256 end) internal view returns (uint256) {
        bytes memory params = abi.encode(account, end);
        bytes memory result = Syscalls.contractCall(NEO_CONTRACT, "unclaimedGas", params);
        return abi.decode(result, (uint256));
    }
    
    // ========== GAS Token Native Contract ==========
    
    /**
     * @dev Get GAS total supply
     */
    function gasTotalSupply() internal view returns (uint256) {
        bytes memory result = Syscalls.contractCall(GAS_CONTRACT, "totalSupply", "");
        return abi.decode(result, (uint256));
    }
    
    /**
     * @dev Get GAS balance of account
     */
    function gasBalanceOf(address account) internal view returns (uint256) {
        bytes memory params = abi.encode(account);
        bytes memory result = Syscalls.contractCall(GAS_CONTRACT, "balanceOf", params);
        return abi.decode(result, (uint256));
    }
    
    /**
     * @dev Transfer GAS tokens
     */
    function gasTransfer(address from, address to, uint256 amount, bytes memory data) 
        internal 
        returns (bool) 
    {
        bytes memory params = abi.encode(from, to, amount, data);
        bytes memory result = Syscalls.contractCall(GAS_CONTRACT, "transfer", params);
        return abi.decode(result, (bool));
    }
    
    /**
     * @dev Get GAS decimals
     */
    function gasDecimals() internal view returns (uint8) {
        bytes memory result = Syscalls.contractCall(GAS_CONTRACT, "decimals", "");
        return abi.decode(result, (uint8));
    }
    
    /**
     * @dev Get GAS symbol
     */
    function gasSymbol() internal view returns (string memory) {
        bytes memory result = Syscalls.contractCall(GAS_CONTRACT, "symbol", "");
        return abi.decode(result, (string));
    }
    
    // ========== Contract Management Native Contract ==========
    
    /**
     * @dev Deploy new contract
     */
    function deployContract(bytes memory nef, bytes memory manifest) internal returns (address) {
        bytes memory params = abi.encode(nef, manifest);
        bytes memory result = Syscalls.contractCall(CONTRACT_MANAGEMENT, "deploy", params);
        return abi.decode(result, (address));
    }

    /**
     * @dev Deploy new contract and pass deployment data to `_deploy(data, false)`
     *
     * The `data` parameter is forwarded to the deployed contract's `_deploy(data, update)` entrypoint
     * (with `update == false`). For neo-devpack-solidity-compiled contracts with parameterised constructors,
     * this is typically a JSON-encoded array (e.g. `[7]`) or StdLib.serialize(...) bytes.
     */
    function deployContract(bytes memory nef, bytes memory manifest, bytes memory data)
        internal
        returns (address)
    {
        bytes memory params = abi.encode(nef, manifest, data);
        bytes memory result = Syscalls.contractCall(CONTRACT_MANAGEMENT, "deploy", params);
        return abi.decode(result, (address));
    }
    
    /**
     * @dev Update contract
     */
    function updateContract(bytes memory nef, bytes memory manifest) internal {
        bytes memory params = abi.encode(nef, manifest);
        Syscalls.contractCall(CONTRACT_MANAGEMENT, "update", params);
    }

    /**
     * @dev Update contract and pass update data to `_deploy(data, true)`
     *
     * The `data` parameter is forwarded to the updated contract's `_deploy(data, update)` entrypoint
     * (with `update == true`). This can be used for migration flows when a contract implements custom
     * `_deploy` logic.
     */
    function updateContract(bytes memory nef, bytes memory manifest, bytes memory data) internal {
        bytes memory params = abi.encode(nef, manifest, data);
        Syscalls.contractCall(CONTRACT_MANAGEMENT, "update", params);
    }
    
    /**
     * @dev Destroy contract
     */
    function destroyContract() internal {
        Syscalls.contractCall(CONTRACT_MANAGEMENT, "destroy", "");
    }
    
    /**
     * @dev Get contract by hash
     */
    function getContract(address hash) internal view returns (ContractState memory) {
        bytes memory params = abi.encode(hash);
        bytes memory result = Syscalls.contractCall(CONTRACT_MANAGEMENT, "getContract", params);
        return abi.decode(result, (ContractState));
    }

    /**
     * @dev Get contract by id
     */
    function getContractById(int256 id) internal view returns (ContractState memory) {
        bytes memory params = abi.encode(id);
        bytes memory result = Syscalls.contractCall(CONTRACT_MANAGEMENT, "getContractById", params);
        return abi.decode(result, (ContractState));
    }
    
    /**
     * @dev List all deployed contract hashes (iterator)
     *
     * Neo N3's ContractManagement exposes this method as `getContractHashes`
     * (there is no `listContracts` method — calling that name faults on-chain
     * with "Method not found"). Returns an iterator of 32-byte (hash, block)
     * key/value entries; consume it via:
     *   while (it.next()) { bytes memory hash = it.value(); ... }
     */
    function getContractHashes() internal view returns (Syscalls.Iterator memory) {
        bytes memory params;
        bytes memory result = Syscalls.contractCall(CONTRACT_MANAGEMENT, "getContractHashes", params);
        return abi.decode(result, (Syscalls.Iterator));
    }

    /**
     * @dev Deprecated alias for `getContractHashes` (kept for source
     * compatibility). The on-chain method is `getContractHashes`.
     */
    function listContracts() internal view returns (Syscalls.Iterator memory) {
        return getContractHashes();
    }

    /**
     * @dev Check if contract has method
     */
    function hasMethod(address hash, string memory method, uint32 paramCount)
        internal
        view
        returns (bool)
    {
        bytes memory params = abi.encode(hash, method, paramCount);
        bytes memory result = Syscalls.contractCall(CONTRACT_MANAGEMENT, "hasMethod", params);
        return abi.decode(result, (bool));
    }

    /**
     * @dev Check if contract exists
     */
    function isContract(address hash) internal view returns (bool) {
        bytes memory params = abi.encode(hash);
        bytes memory result = Syscalls.contractCall(CONTRACT_MANAGEMENT, "isContract", params);
        return abi.decode(result, (bool));
    }
    
    /**
     * @dev Get minimum deployment fee
     */
    function getMinimumDeploymentFee() internal view returns (uint256) {
        bytes memory result = Syscalls.contractCall(CONTRACT_MANAGEMENT, "getMinimumDeploymentFee", "");
        return abi.decode(result, (uint256));
    }
    
    /**
     * @dev Set minimum deployment fee
     */
    function setMinimumDeploymentFee(uint256 value) internal {
        bytes memory params = abi.encode(value);
        Syscalls.contractCall(CONTRACT_MANAGEMENT, "setMinimumDeploymentFee", params);
    }
    
    // ========== Policy Native Contract ==========
    
    /**
     * @dev Get fee per byte
     */
    function getFeePerByte() internal view returns (uint256) {
        bytes memory result = Syscalls.contractCall(POLICY_CONTRACT, "getFeePerByte", "");
        return abi.decode(result, (uint256));
    }
    
    /**
     * @dev Set fee per byte
     */
    function setFeePerByte(uint256 value) internal {
        bytes memory params = abi.encode(value);
        Syscalls.contractCall(POLICY_CONTRACT, "setFeePerByte", params);
    }
    
    /**
     * @dev Get execution fee factor
     */
    function getExecFeeFactor() internal view returns (uint32) {
        bytes memory result = Syscalls.contractCall(POLICY_CONTRACT, "getExecFeeFactor", "");
        return abi.decode(result, (uint32));
    }

    /**
     * @dev Get execution fee factor in picoGAS
     */
    function getExecPicoFeeFactor() internal view returns (uint256) {
        bytes memory result = Syscalls.contractCall(POLICY_CONTRACT, "getExecPicoFeeFactor", "");
        return abi.decode(result, (uint256));
    }
    
    /**
     * @dev Set execution fee factor
     *
     * C# signature: `SetExecFeeFactor(ulong value)` — uint64, not uint32.
     */
    function setExecFeeFactor(uint64 value) internal {
        bytes memory params = abi.encode(value);
        Syscalls.contractCall(POLICY_CONTRACT, "setExecFeeFactor", params);
    }
    
    /**
     * @dev Get storage price
     */
    function getStoragePrice() internal view returns (uint256) {
        bytes memory result = Syscalls.contractCall(POLICY_CONTRACT, "getStoragePrice", "");
        return abi.decode(result, (uint256));
    }

    /**
     * @dev Get milliseconds per block
     */
    function getMillisecondsPerBlock() internal view returns (uint256) {
        bytes memory result = Syscalls.contractCall(POLICY_CONTRACT, "getMillisecondsPerBlock", "");
        return abi.decode(result, (uint256));
    }

    /**
     * @dev Set milliseconds per block
     */
    function setMillisecondsPerBlock(uint256 value) internal {
        bytes memory params = abi.encode(value);
        Syscalls.contractCall(POLICY_CONTRACT, "setMillisecondsPerBlock", params);
    }

    /**
     * @dev Get max valid-until-block increment
     */
    function getMaxValidUntilBlockIncrement() internal view returns (uint256) {
        bytes memory result = Syscalls.contractCall(POLICY_CONTRACT, "getMaxValidUntilBlockIncrement", "");
        return abi.decode(result, (uint256));
    }

    /**
     * @dev Set max valid-until-block increment
     */
    function setMaxValidUntilBlockIncrement(uint256 value) internal {
        bytes memory params = abi.encode(value);
        Syscalls.contractCall(POLICY_CONTRACT, "setMaxValidUntilBlockIncrement", params);
    }

    /**
     * @dev Get max traceable blocks
     */
    function getMaxTraceableBlocks() internal view returns (uint256) {
        bytes memory result = Syscalls.contractCall(POLICY_CONTRACT, "getMaxTraceableBlocks", "");
        return abi.decode(result, (uint256));
    }

    /**
     * @dev Set max traceable blocks
     */
    function setMaxTraceableBlocks(uint256 value) internal {
        bytes memory params = abi.encode(value);
        Syscalls.contractCall(POLICY_CONTRACT, "setMaxTraceableBlocks", params);
    }

    /**
     * @dev Get attribute fee
     */
    function getAttributeFee(uint8 attributeType) internal view returns (uint256) {
        bytes memory params = abi.encode(attributeType);
        bytes memory result = Syscalls.contractCall(POLICY_CONTRACT, "getAttributeFee", params);
        return abi.decode(result, (uint256));
    }

    /**
     * @dev Set attribute fee
     */
    function setAttributeFee(uint8 attributeType, uint256 value) internal {
        bytes memory params = abi.encode(attributeType, value);
        Syscalls.contractCall(POLICY_CONTRACT, "setAttributeFee", params);
    }
    
    /**
     * @dev Set storage price
     */
    function setStoragePrice(uint256 value) internal {
        bytes memory params = abi.encode(value);
        Syscalls.contractCall(POLICY_CONTRACT, "setStoragePrice", params);
    }
    
    /**
     * @dev Block account
     *
     * C# `BlockAccount` returns bool (false when the account was already
     * blocked or the committee check failed).
     */
    function blockAccount(address account) internal returns (bool) {
        bytes memory params = abi.encode(account);
        bytes memory result = Syscalls.contractCall(POLICY_CONTRACT, "blockAccount", params);
        return abi.decode(result, (bool));
    }

    /**
     * @dev Unblock account
     *
     * C# `UnblockAccount` returns bool (false when the account was not
     * blocked).
     */
    function unblockAccount(address account) internal returns (bool) {
        bytes memory params = abi.encode(account);
        bytes memory result = Syscalls.contractCall(POLICY_CONTRACT, "unblockAccount", params);
        return abi.decode(result, (bool));
    }
    
    /**
     * @dev Check if account is blocked
     */
    function isBlocked(address account) internal view returns (bool) {
        bytes memory params = abi.encode(account);
        bytes memory result = Syscalls.contractCall(POLICY_CONTRACT, "isBlocked", params);
        return abi.decode(result, (bool));
    }

    /**
     * @dev Get blocked accounts iterator
     */
    function getBlockedAccounts() internal view returns (Syscalls.Iterator memory) {
        bytes memory result = Syscalls.contractCall(POLICY_CONTRACT, "getBlockedAccounts", "");
        return abi.decode(result, (Syscalls.Iterator));
    }

    /**
     * @dev Recover blocked funds (committee only)
     */
    function recoverFund(address account, address token) internal returns (bool) {
        bytes memory params = abi.encode(account, token);
        bytes memory result = Syscalls.contractCall(POLICY_CONTRACT, "recoverFund", params);
        return abi.decode(result, (bool));
    }

    /**
     * @dev Set whitelist fee contract (committee only)
     */
    function setWhitelistFeeContract(
        address contractHash,
        string memory method,
        uint8 argCount,
        uint256 fixedFee
    ) internal {
        bytes memory params = abi.encode(contractHash, method, argCount, fixedFee);
        Syscalls.contractCall(POLICY_CONTRACT, "setWhitelistFeeContract", params);
    }

    /**
     * @dev Remove whitelist fee contract (committee only)
     */
    function removeWhitelistFeeContract(
        address contractHash,
        string memory method,
        uint8 argCount
    ) internal {
        bytes memory params = abi.encode(contractHash, method, argCount);
        Syscalls.contractCall(POLICY_CONTRACT, "removeWhitelistFeeContract", params);
    }

    /**
     * @dev Get whitelisted fee contracts iterator
     */
    function getWhitelistFeeContracts() internal view returns (Syscalls.Iterator memory) {
        bytes memory result = Syscalls.contractCall(POLICY_CONTRACT, "getWhitelistFeeContracts", "");
        return abi.decode(result, (Syscalls.Iterator));
    }
    
    // ========== Oracle Native Contract ==========
    
    /**
     * @dev Request oracle data
     */
    function requestOracleData(
        string memory url,
        string memory filter,
        string memory callback,
        bytes memory userData,
        uint256 gasForResponse
    ) internal {
        bytes memory params = abi.encode(url, filter, callback, userData, gasForResponse);
        Syscalls.contractCall(ORACLE_CONTRACT, "request", params);
    }
    
    /**
     * @dev Get oracle price
     */
    function getOraclePrice() internal view returns (uint256) {
        bytes memory result = Syscalls.contractCall(ORACLE_CONTRACT, "getPrice", "");
        return abi.decode(result, (uint256));
    }
    
    /**
     * @dev Set oracle price
     */
    function setOraclePrice(uint256 price) internal {
        bytes memory params = abi.encode(price);
        Syscalls.contractCall(ORACLE_CONTRACT, "setPrice", params);
    }

    /**
     * @dev Finish oracle response (oracle nodes)
     */
    function oracleFinish() internal {
        Syscalls.contractCall(ORACLE_CONTRACT, "finish", "");
    }

    /**
     * @dev Verify oracle response transaction
     */
    function oracleVerify() internal view returns (bool) {
        bytes memory result = Syscalls.contractCall(ORACLE_CONTRACT, "verify", "");
        return abi.decode(result, (bool));
    }
    
    // ========== Role Management Native Contract ==========
    
    /**
     * @dev Designate as role
     */
    function designateAsRole(bytes1 role, bytes[] memory publicKeys) internal {
        bytes memory params = abi.encode(role, publicKeys);
        Syscalls.contractCall(ROLE_MANAGEMENT, "designateAsRole", params);
    }
    
    /**
     * @dev Get designated by role
     */
    function getDesignatedByRole(bytes1 role, uint256 index) internal view returns (bytes[] memory) {
        bytes memory params = abi.encode(role, index);
        bytes memory result = Syscalls.contractCall(ROLE_MANAGEMENT, "getDesignatedByRole", params);
        return abi.decode(result, (bytes[]));
    }

    // ========== Ledger Native Contract ==========

    /**
     * @dev Get current block index
     */
    function currentIndex() internal view returns (uint256) {
        bytes memory result = Syscalls.contractCall(LEDGER_CONTRACT, "currentIndex", "");
        return abi.decode(result, (uint256));
    }

    /**
     * @dev Get current block hash
     */
    function currentHash() internal view returns (bytes32) {
        bytes memory result = Syscalls.contractCall(LEDGER_CONTRACT, "currentHash", "");
        return abi.decode(result, (bytes32));
    }

    /**
     * @dev Get block by index
     */
    function getBlock(uint256 index) internal view returns (Syscalls.Block memory) {
        bytes memory params = abi.encode(index);
        bytes memory result = Syscalls.contractCall(LEDGER_CONTRACT, "getBlock", params);
        return abi.decode(result, (Syscalls.Block));
    }

    /**
     * @dev Get block by hash
     */
    function getBlock(bytes32 hash) internal view returns (Syscalls.Block memory) {
        bytes memory params = abi.encode(hash);
        bytes memory result = Syscalls.contractCall(LEDGER_CONTRACT, "getBlock", params);
        return abi.decode(result, (Syscalls.Block));
    }

    /**
     * @dev Get transaction by hash
     */
    function getTransaction(bytes32 hash) internal view returns (Syscalls.Transaction memory) {
        bytes memory params = abi.encode(hash);
        bytes memory result = Syscalls.contractCall(LEDGER_CONTRACT, "getTransaction", params);
        return abi.decode(result, (Syscalls.Transaction));
    }

    /**
     * @dev Get transaction height
     */
    function getTransactionHeight(bytes32 hash) internal view returns (int256) {
        bytes memory params = abi.encode(hash);
        bytes memory result = Syscalls.contractCall(LEDGER_CONTRACT, "getTransactionHeight", params);
        return abi.decode(result, (int256));
    }

    /**
     * @dev Get transaction from block
     */
    function getTransactionFromBlock(uint256 blockIndex, uint256 txIndex)
        internal
        view
        returns (Syscalls.Transaction memory)
    {
        bytes memory params = abi.encode(blockIndex, txIndex);
        bytes memory result = Syscalls.contractCall(LEDGER_CONTRACT, "getTransactionFromBlock", params);
        return abi.decode(result, (Syscalls.Transaction));
    }

    /**
     * @dev Get transaction from block by hash
     */
    function getTransactionFromBlock(bytes32 blockHash, uint256 txIndex)
        internal
        view
        returns (Syscalls.Transaction memory)
    {
        bytes memory params = abi.encode(blockHash, txIndex);
        bytes memory result = Syscalls.contractCall(LEDGER_CONTRACT, "getTransactionFromBlock", params);
        return abi.decode(result, (Syscalls.Transaction));
    }

    /**
     * @dev Get transaction signers
     */
    function getTransactionSigners(bytes32 hash) internal view returns (Syscalls.Signer[] memory) {
        bytes memory params = abi.encode(hash);
        bytes memory result = Syscalls.contractCall(LEDGER_CONTRACT, "getTransactionSigners", params);
        return abi.decode(result, (Syscalls.Signer[]));
    }

    /**
     * @dev Get transaction VM state
     */
    function getTransactionVMState(bytes32 hash) internal view returns (uint8) {
        bytes memory params = abi.encode(hash);
        bytes memory result = Syscalls.contractCall(LEDGER_CONTRACT, "getTransactionVMState", params);
        return abi.decode(result, (uint8));
    }

    // ========== Notary Native Contract ==========

    /**
     * @dev Verify notary-assisted transaction
     */
    function notaryVerify(bytes memory signature) internal view returns (bool) {
        bytes memory params = abi.encode(signature);
        bytes memory result = Syscalls.contractCall(NOTARY_CONTRACT, "verify", params);
        return abi.decode(result, (bool));
    }

    /**
     * @dev Get notary deposit balance
     */
    function notaryBalanceOf(address account) internal view returns (uint256) {
        bytes memory params = abi.encode(account);
        bytes memory result = Syscalls.contractCall(NOTARY_CONTRACT, "balanceOf", params);
        return abi.decode(result, (uint256));
    }

    /**
     * @dev Get notary deposit expiration height
     */
    function notaryExpirationOf(address account) internal view returns (uint256) {
        bytes memory params = abi.encode(account);
        bytes memory result = Syscalls.contractCall(NOTARY_CONTRACT, "expirationOf", params);
        return abi.decode(result, (uint256));
    }

    /**
     * @dev Lock notary deposit until a specific block height
     */
    function notaryLockDepositUntil(address account, uint256 till) internal returns (bool) {
        bytes memory params = abi.encode(account, till);
        bytes memory result = Syscalls.contractCall(NOTARY_CONTRACT, "lockDepositUntil", params);
        return abi.decode(result, (bool));
    }

    /**
     * @dev Withdraw notary deposit
     */
    function notaryWithdraw(address from, address to) internal returns (bool) {
        bytes memory params = abi.encode(from, to);
        bytes memory result = Syscalls.contractCall(NOTARY_CONTRACT, "withdraw", params);
        return abi.decode(result, (bool));
    }

    /**
     * @dev Get maximum NotValidBefore delta
     */
    function notaryGetMaxNotValidBeforeDelta() internal view returns (uint256) {
        bytes memory result = Syscalls.contractCall(NOTARY_CONTRACT, "getMaxNotValidBeforeDelta", "");
        return abi.decode(result, (uint256));
    }

    /**
     * @dev Set maximum NotValidBefore delta
     */
    function notarySetMaxNotValidBeforeDelta(uint256 value) internal {
        bytes memory params = abi.encode(value);
        Syscalls.contractCall(NOTARY_CONTRACT, "setMaxNotValidBeforeDelta", params);
    }

    /**
     * @dev Handle NEP-17 payment (notary deposit)
     */
    function notaryOnNEP17Payment(address from, uint256 amount, bytes memory data) internal {
        bytes memory params = abi.encode(from, amount, data);
        Syscalls.contractCall(NOTARY_CONTRACT, "onNEP17Payment", params);
    }

    // ========== Treasury Native Contract ==========

    /**
     * @dev Verify treasury transaction
     */
    function treasuryVerify() internal view returns (bool) {
        bytes memory result = Syscalls.contractCall(TREASURY_CONTRACT, "verify", "");
        return abi.decode(result, (bool));
    }

    /**
     * @dev Handle NEP-17 payment
     */
    function treasuryOnNEP17Payment(address from, uint256 amount, bytes memory data) internal {
        bytes memory params = abi.encode(from, amount, data);
        Syscalls.contractCall(TREASURY_CONTRACT, "onNEP17Payment", params);
    }

    /**
     * @dev Handle NEP-11 payment
     */
    function treasuryOnNEP11Payment(
        address from,
        uint256 amount,
        bytes memory tokenId,
        bytes memory data
    ) internal {
        bytes memory params = abi.encode(from, amount, tokenId, data);
        Syscalls.contractCall(TREASURY_CONTRACT, "onNEP11Payment", params);
    }

    // ========== Data Structures ==========
    
    struct NeoCandidate {
        bytes publicKey;
        uint256 votes;
    }
    
    struct AccountState {
        // NeoToken NeoAccountState:
        // [balance, balanceHeight, voteTo (ECPoint bytes or null), lastGasPerVote]
        uint256 balance;
        uint256 balanceHeight;
        bytes voteTo;
        uint256 lastGasPerVote;
    }
    
    struct ContractState {
        // Matches Syscalls.sol's ContractStateNative field order:
        // ContractManagement.getContract returns [id, updateCounter, hash, nef, manifest]
        int256 id;
        uint256 updateCounter;
        address hash;
        bytes nef;
        bytes manifest;
    }

    struct WhitelistedContract {
        address contractHash;
        string method;
        uint256 argCount;
        int256 fixedFee;
    }
    
    // ========== Helper Functions ==========
    
    /**
     * @dev Check if native contract exists
     */
    function isNativeContract(address contractHash) internal pure returns (bool) {
        return contractHash == NEO_CONTRACT ||
               contractHash == GAS_CONTRACT ||
               contractHash == CONTRACT_MANAGEMENT ||
               contractHash == POLICY_CONTRACT ||
               contractHash == ORACLE_CONTRACT ||
               contractHash == ROLE_MANAGEMENT ||
               contractHash == NOTARY_CONTRACT ||
               contractHash == TREASURY_CONTRACT ||
               contractHash == LEDGER_CONTRACT ||
               contractHash == CRYPTO_LIB ||
               contractHash == STD_LIB;
    }
    
    /**
     * @dev Get native contract name
     */
    function getNativeContractName(address contractHash) internal pure returns (string memory) {
        if (contractHash == NEO_CONTRACT) return "NeoToken";
        if (contractHash == GAS_CONTRACT) return "GasToken";
        if (contractHash == CONTRACT_MANAGEMENT) return "ContractManagement";
        if (contractHash == POLICY_CONTRACT) return "PolicyContract";
        if (contractHash == ORACLE_CONTRACT) return "OracleContract";
        if (contractHash == ROLE_MANAGEMENT) return "RoleManagement";
        if (contractHash == NOTARY_CONTRACT) return "Notary";
        if (contractHash == TREASURY_CONTRACT) return "Treasury";
        if (contractHash == LEDGER_CONTRACT) return "LedgerContract";
        if (contractHash == CRYPTO_LIB) return "CryptoLib";
        if (contractHash == STD_LIB) return "StdLib";
        return "Unknown";
    }
    
    /**
     * @dev Get all native contract addresses
     */
    function getAllNativeContracts() internal pure returns (address[] memory) {
        address[] memory contracts = new address[](11);
        contracts[0] = NEO_CONTRACT;
        contracts[1] = GAS_CONTRACT;
        contracts[2] = CONTRACT_MANAGEMENT;
        contracts[3] = POLICY_CONTRACT;
        contracts[4] = ORACLE_CONTRACT;
        contracts[5] = ROLE_MANAGEMENT;
        contracts[6] = NOTARY_CONTRACT;
        contracts[7] = TREASURY_CONTRACT;
        contracts[8] = LEDGER_CONTRACT;
        contracts[9] = CRYPTO_LIB;
        contracts[10] = STD_LIB;
        return contracts;
    }
    
    /**
     * @dev Estimate gas for native contract call
     */
    function estimateNativeCallGas(
        address contractHash,
        string memory method,
        bytes memory params
    ) internal view returns (uint256) {
        // Base gas cost for native contract calls
        uint256 baseGas = 1000000; // 0.01 GAS
        
        // Additional gas based on method complexity
        if (contractHash == NEO_CONTRACT) {
            if (keccak256(bytes(method)) == keccak256("vote")) return baseGas * 100;
            if (keccak256(bytes(method)) == keccak256("registerCandidate")) return baseGas * 1000;
        }
        
        if (contractHash == CONTRACT_MANAGEMENT) {
            if (keccak256(bytes(method)) == keccak256("deploy")) return baseGas * 500;
            if (keccak256(bytes(method)) == keccak256("update")) return baseGas * 300;
        }
        
        if (contractHash == ORACLE_CONTRACT) {
            if (keccak256(bytes(method)) == keccak256("request")) return baseGas * 50;
        }
        
        return baseGas;
    }
    
    /**
     * @dev Batch native contract calls
     */
    function batchNativeCalls(
        address[] memory /*contracts*/,
        string[] memory /*methods*/,
        bytes[] memory /*params*/
    ) internal pure returns (bytes[] memory /*results*/) {
        revert(
            "NativeCalls: batchNativeCalls is handled by the compiler as a builtin lowering "
            "(with 10-call cap and safety checks). This Solidity wrapper is intentionally disabled "
            "to avoid wildcard manifest permissions. Use individual native wrapper functions or "
            "let the compiler handle batchNativeCalls() calls directly."
        );
    }
    
    /**
     * @dev Get network configuration
     */
    function getNetworkConfiguration() internal view returns (NetworkConfig memory) {
        return NetworkConfig({
            feePerByte: getFeePerByte(),
            execFeeFactor: getExecFeeFactor(),
            storagePrice: getStoragePrice(),
            gasPerBlock: getGasPerBlock(),
            oraclePrice: getOraclePrice(),
            minimumDeploymentFee: getMinimumDeploymentFee()
        });
    }
    
    struct NetworkConfig {
        uint256 feePerByte;
        uint32 execFeeFactor;
        uint256 storagePrice;
        uint256 gasPerBlock;
        uint256 oraclePrice;
        uint256 minimumDeploymentFee;
    }
    
    // ========== Governance Functions ==========
    
    /**
     * @dev Get committee members
     */
    function getCommittee() internal view returns (bytes[] memory) {
        bytes memory result = Syscalls.contractCall(NEO_CONTRACT, "getCommittee", "");
        return abi.decode(result, (bytes[]));
    }

    /**
     * @dev Get committee multi-sig address
     */
    function getCommitteeAddress() internal view returns (address) {
        bytes memory result = Syscalls.contractCall(NEO_CONTRACT, "getCommitteeAddress", "");
        return abi.decode(result, (address));
    }
    
    /**
     * @dev Get next block validators
     *
     * Neo N3: `NeoToken.getNextBlockValidators()` returns `ECPoint[]` (33-byte
     * public keys), NOT `address[]` (20-byte script hashes). Callers that need
     * addresses must hash the public keys to obtain script hashes.
     */
    function getNextBlockValidators() internal view returns (bytes[] memory) {
        bytes memory result = Syscalls.contractCall(NEO_CONTRACT, "getNextBlockValidators", "");
        return abi.decode(result, (bytes[]));
    }
    
    /**
     * @dev Check if address matches the committee multi-sig address
     *
     * NOTE: This only checks if `account` equals the committee multi-sig
     * address returned by NeoToken.getCommitteeAddress(). It does NOT check
     * individual committee membership. For individual membership verification
     * at invocation time, use `Runtime.checkWitness(account)`.
     *
     * Previously named `isCommittee`, which was misleading because it
     * suggested individual membership checking.
     */
    function isCommitteeMultisigAddress(address account) internal view returns (bool) {
        address committeeAddr = getCommitteeAddress();
        return account == committeeAddr;
    }

    /**
     * @dev Deprecated: use isCommitteeMultisigAddress() instead.
     *
     * WARNING: This function checks whether `account` equals the committee
     * multi-signature address (the aggregate address derived from the committee
     * members' public keys). It does NOT check whether `account` is an
     * individual committee member.
     *
     * For individual committee membership check, use Neo.isCommittee(account)
     * which maps ECPoint[] committee keys to their UInt160 addresses and
     * performs a proper membership test.
     */
    function isCommittee(address account) internal view returns (bool) {
        return isCommitteeMultisigAddress(account);
    }
    
    /**
     * @dev Check if address is validator
     *
     * `getNextBlockValidators()` returns raw ECPoint public keys (`bytes[]`),
     * so each key is converted to its UInt160 script hash via
     * `System.Contract.CreateStandardAccount` before the comparison.
     */
    function isValidator(address account) internal view returns (bool) {
        bytes[] memory validators = getNextBlockValidators();
        for (uint256 i = 0; i < validators.length; i++) {
            if (Syscalls.createStandardAccount(validators[i]) == account) {
                return true;
            }
        }
        return false;
    }
    
    // ========== Native Contract Utilities ==========
    
    /**
     * @dev Get native contract manifest
     */
    function getNativeContractManifest(address contractHash) internal view returns (bytes memory) {
        ContractState memory state = getContract(contractHash);
        return state.manifest;
    }
    
    /**
     * @dev Safe native contract call with error handling
     */
    function safeNativeCall(
        address /*contractHash*/,
        string memory /*method*/,
        bytes memory /*params*/
    ) internal pure returns (bool success, bytes memory result) {
        return (false, "");
    }
    
    /**
     * @dev External wrapper for try/catch
     */
    function externalNativeCall(
        address /*contractHash*/,
        string calldata /*method*/,
        bytes calldata /*params*/
    ) external pure returns (bytes memory) {
        revert("NativeCalls: externalNativeCall disabled in strict-manifest mode");
    }
}
