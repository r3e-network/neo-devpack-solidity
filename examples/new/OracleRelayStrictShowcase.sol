// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title OracleRelayStrictShowcase
 * @notice Strict-manifest-safe oracle request/response relay for Neo N3.
 */
contract OracleRelayStrictShowcase {
    struct OracleResult {
        string url;
        uint8 code;
        bytes result;
        uint256 respondedAt;
        address requester;
    }

    uint256 public nextRequestId = 1;

    mapping(uint256 => address) public requestRequester;
    mapping(uint256 => bool) private _requestExists;
    mapping(uint256 => OracleResult) private _results;

    event OracleRequestSent(uint256 indexed requestId, address indexed requester, string url, string filter);
    event OracleResultStored(uint256 indexed requestId, address indexed requester, uint8 code, bytes result);

    error UnsupportedOracleCaller(address caller);
    error UnknownRequest(uint256 requestId);

    function request(
        string calldata url,
        string calldata filter,
        uint256 gasForResponse
    ) external returns (uint256 requestId) {
        requestId = nextRequestId;
        nextRequestId = requestId + 1;

        requestRequester[requestId] = msg.sender;
        _requestExists[requestId] = true;

        bytes memory userData = abi.encode(requestId, msg.sender);
        NativeCalls.requestOracleData(url, filter, "onOracleResponse", userData, gasForResponse);

        emit OracleRequestSent(requestId, msg.sender, url, filter);
    }

    function onOracleResponse(
        string calldata url,
        bytes calldata userData,
        uint8 code,
        bytes calldata result
    ) external {
        if (msg.sender != NativeCalls.ORACLE_CONTRACT) {
            revert UnsupportedOracleCaller(msg.sender);
        }

        (uint256 requestId, address requester) = abi.decode(userData, (uint256, address));
        if (!_requestExists[requestId]) {
            revert UnknownRequest(requestId);
        }

        _results[requestId] = OracleResult({
            url: url,
            code: code,
            result: result,
            respondedAt: block.timestamp,
            requester: requester
        });

        emit OracleResultStored(requestId, requester, code, result);
    }

    function getResult(uint256 requestId) external view returns (OracleResult memory) {
        if (!_requestExists[requestId]) revert UnknownRequest(requestId);
        OracleResult memory item = _results[requestId];
        return item;
    }
}
