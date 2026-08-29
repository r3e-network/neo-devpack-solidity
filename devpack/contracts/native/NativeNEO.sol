// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./NativeTypes.sol";
import "../NativeContracts.sol";
import "../Syscalls.sol";

/**
 * @title Native NEO — Neo N3 NEO native contract operations
 */

library NativeNEO {
    address constant NEO_CONTRACT = NativeContracts.NEO_CONTRACT;
    // ========== NEO Token Native Contract ==========
    
    /**
     * @dev Get NEO total supply
     */
    function neoTotalSupply() internal view returns (uint256) {
        bytes memory result = Syscalls.contractCall(NativeContracts.NEO_CONTRACT, "totalSupply", "");
        return abi.decode(result, (uint256));
    }
    
    /**
     * @dev Get NEO balance of account
     */
    function neoBalanceOf(address account) internal view returns (uint256) {
        bytes memory params = abi.encode(account);
        bytes memory result = Syscalls.contractCall(NativeContracts.NEO_CONTRACT, "balanceOf", params);
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
        bytes memory result = Syscalls.contractCall(NativeContracts.NEO_CONTRACT, "transfer", params);
        return abi.decode(result, (bool));
    }
    
    /**
     * @dev Get NEO decimals
     */
    function neoDecimals() internal view returns (uint8) {
        bytes memory result = Syscalls.contractCall(NativeContracts.NEO_CONTRACT, "decimals", "");
        return abi.decode(result, (uint8));
    }
    
    /**
     * @dev Get NEO symbol
     */
    function neoSymbol() internal view returns (string memory) {
        bytes memory result = Syscalls.contractCall(NativeContracts.NEO_CONTRACT, "symbol", "");
        return abi.decode(result, (string));
    }
    
    /**
     * @dev Vote for validator
     */
    function vote(address account, bytes memory publicKey) internal returns (bool) {
        bytes memory params = abi.encode(account, publicKey);
        bytes memory result = Syscalls.contractCall(NativeContracts.NEO_CONTRACT, "vote", params);
        return abi.decode(result, (bool));
    }
    
    /**
     * @dev Get candidates
     *
     * Neo N3: `NeoToken.getCandidates()` returns `(ECPoint publicKey, BigInteger votes)[]`
     * for the first 256 registered (non-blocked) candidates.
     */
    function getCandidates() internal view returns (NeoCandidate[] memory) {
        bytes memory result = Syscalls.contractCall(NativeContracts.NEO_CONTRACT, "getCandidates", "");
        return abi.decode(result, (NeoCandidate[]));
    }

    /**
     * @dev Get all registered candidates (iterator)
     */
    function getAllCandidates() internal view returns (Syscalls.Iterator memory) {
        bytes memory result = Syscalls.contractCall(NativeContracts.NEO_CONTRACT, "getAllCandidates", "");
        return abi.decode(result, (Syscalls.Iterator));
    }

    /**
     * @dev Get candidate vote count (returns -1 if not found)
     *
     * Neo N3 `NeoToken.getCandidateVote` returns a signed BigInteger: -1 when
     * the candidate is not registered. Decoding as `uint256` turned -1 into
     * `2^256-1`, making non-candidates look like they hold a maximum vote
     * weight. Mirrors `NativeCalls.getCandidateVote` (int256).
     */
    function getCandidateVote(bytes memory publicKey) internal view returns (int256) {
        bytes memory params = abi.encode(publicKey);
        bytes memory result = Syscalls.contractCall(NativeContracts.NEO_CONTRACT, "getCandidateVote", params);
        return abi.decode(result, (int256));
    }
    
    /**
     * @dev Register as candidate
     */
    function registerCandidate(bytes memory publicKey) internal returns (bool) {
        bytes memory params = abi.encode(publicKey);
        bytes memory result = Syscalls.contractCall(NativeContracts.NEO_CONTRACT, "registerCandidate", params);
        return abi.decode(result, (bool));
    }
    
    /**
     * @dev Unregister candidate
     */
    function unregisterCandidate(bytes memory publicKey) internal returns (bool) {
        bytes memory params = abi.encode(publicKey);
        bytes memory result = Syscalls.contractCall(NativeContracts.NEO_CONTRACT, "unregisterCandidate", params);
        return abi.decode(result, (bool));
    }
    
    /**
     * @dev Get GAS per block
     */
    function getGasPerBlock() internal view returns (uint256) {
        bytes memory result = Syscalls.contractCall(NativeContracts.NEO_CONTRACT, "getGasPerBlock", "");
        return abi.decode(result, (uint256));
    }

    /**
     * @dev Get register price for candidates
     */
    function getRegisterPrice() internal view returns (uint256) {
        bytes memory result = Syscalls.contractCall(NativeContracts.NEO_CONTRACT, "getRegisterPrice", "");
        return abi.decode(result, (uint256));
    }

    /**
     * @dev Set register price for candidates (committee only)
     */
    function setRegisterPrice(uint256 value) internal {
        bytes memory params = abi.encode(value);
        Syscalls.contractCall(NativeContracts.NEO_CONTRACT, "setRegisterPrice", params);
    }
    
    /**
     * @dev Set GAS per block (committee only)
     */
    function setGasPerBlock(uint256 gasPerBlock) internal {
        bytes memory params = abi.encode(gasPerBlock);
        Syscalls.contractCall(NativeContracts.NEO_CONTRACT, "setGasPerBlock", params);
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
        bytes memory result = Syscalls.contractCall(NativeContracts.NEO_CONTRACT, "getAccountState", params);
        return abi.decode(result, (AccountState));
    }

    /**
     * @dev Get unclaimed GAS for account at a specific block height
     */
    function unclaimedGas(address account, uint256 end) internal view returns (uint256) {
        bytes memory params = abi.encode(account, end);
        bytes memory result = Syscalls.contractCall(NativeContracts.NEO_CONTRACT, "unclaimedGas", params);
        return abi.decode(result, (uint256));
    }
    
}
