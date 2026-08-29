// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title OracleShowcase
 * @notice Demonstrates the Neo N3 Oracle request/callback pattern
 *         using NativeCalls for off-chain data retrieval.
 */

import "../../devpack/contracts/NativeCalls.sol";
import "../../devpack/contracts/Syscalls.sol";

contract OracleShowcase {
    event OracleRequested(string url, string filter);
    event OracleResponseReceived(string url, bytes result, uint8 code);

    address public owner;
    mapping(string => bytes) public cachedResponses;

    constructor() {
        owner = msg.sender;
    }

    /// @notice Request price data from an oracle endpoint
    function requestPrice(string memory url, string memory jsonFilter) public {
        require(msg.sender == owner, "only owner");

        uint256 gasForResponse = 100_000_000; // 1 GAS
        NativeCalls.requestOracleData(
            url,
            jsonFilter,
            "onOracleResponse",
            "",
            gasForResponse
        );

        emit OracleRequested(url, jsonFilter);
    }

    /// @notice Callback invoked by the Oracle native contract
    function onOracleResponse(
        string memory url,
        bytes memory userData,
        uint8 code,
        bytes memory result
    ) public {
        // Only the Oracle contract may call this
        require(
            msg.sender == NativeCalls.ORACLE_CONTRACT,
            "unauthorized caller"
        );

        cachedResponses[url] = result;
        emit OracleResponseReceived(url, result, code);
    }

    function getCachedResponse(string memory url)
        public
        view
        returns (bytes memory)
    {
        return cachedResponses[url];
    }
}
