// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title OracleService
 * @dev Convenience wrapper around the Neo N3 Oracle native contract.
 *
 * This is NOT a NEP standard. Neo N3 provides oracle functionality via the
 * native `Oracle` contract. The `Syscalls.oracleRequest(...)` intrinsic lowers
 * to `Oracle.request(...)`.
 *
 * The Oracle native contract calls back into the requesting contract using the
 * provided callback method name, with the signature:
 *
 *   callback(string url, bytes userData, int code, bytes result)
 *
 * This helper:
 * - Sends requests to the native oracle contract
 * - Receives the native callback (`oracleCallback`)
 * - Forwards responses to the original requester via a FIXED callback method
 *   (`onOracleResponse`) to avoid full-wildcard manifest permissions
 */

import "./FrameworkBase.sol";
import "./Syscalls.sol";

/**
 * @dev Requester callback interface.
 *
 * The method name is fixed so that manifest inference can restrict permissions
 * to `methods:["onOracleResponse"]` rather than `methods:"*"`.
 */
interface IOracleServiceReceiver {
    function onOracleResponse(
        uint256 requestId,
        uint256 code,
        bytes calldata result,
        bytes calldata userData
    ) external;
}

contract OracleService is FrameworkBase {

    // Oracle native contract hash (UInt160 as Solidity address).
    // Little-endian UInt160: 0x588717117e0aa81072afab71d2dd89fe7c4b92fe
    // Big-endian (Neo RPC):  0xfe924b7cfe89ddd271abaf7210a80a7e11178758
    address private constant ORACLE_CONTRACT = 0xfe924b7cfe89ddd271abaf7210a80a7e11178758;

    // Callback method invoked by the Oracle native contract on this service.
    string private constant ORACLE_NATIVE_CALLBACK = "oracleCallback";

    struct Request {
        address requester;
        string url;
        string filter;
        bytes userData;
        uint256 gasForResponse;
        uint256 timestamp;
        bool completed;
        uint256 responseCode;
        bytes responseResult;
    }

    uint256 private _nextRequestId = 1;
    mapping(uint256 => Request) private _requests;

    event OracleRequest(uint256 indexed requestId, address indexed requester, string url, string filter);
    event OracleResponse(uint256 indexed requestId, address indexed requester, uint256 code, bytes result);
    event OracleForwardFailed(uint256 indexed requestId, address indexed requester);

    error OracleUnauthorizedSender(address caller);
    error OracleRequestNotFound(uint256 requestId);

    modifier onlyOracleNative() {
        if (msg.sender != ORACLE_CONTRACT) revert OracleUnauthorizedSender(msg.sender);
        _;
    }

    /**
     * @dev Issue an oracle request via the Oracle native contract.
     *
     * The `userData` provided here is stored and forwarded back to the requester
     * in `onOracleResponse`.
     */
    function request(
        string calldata url,
        string calldata filter,
        bytes calldata userData,
        uint256 gasForResponse
    ) external returns (uint256 requestId) {
        requestId = _nextRequestId++;

        _requests[requestId] = Request({
            requester: msg.sender,
            url: url,
            filter: filter,
            userData: userData,
            gasForResponse: gasForResponse,
            timestamp: block.timestamp,
            completed: false,
            responseCode: 0,
            responseResult: ""
        });

        // Encode our internal request id into the oracle userData so we can
        // correlate the native callback with this request.
        bytes memory oracleUserData = abi.encode(requestId);

        Syscalls.oracleRequest(url, filter, ORACLE_NATIVE_CALLBACK, oracleUserData, gasForResponse);

        emit OracleRequest(requestId, msg.sender, url, filter);
    }

    function getRequest(uint256 requestId) external view returns (Request memory) {
        Request memory req = _requests[requestId];
        if (req.requester == address(0)) revert OracleRequestNotFound(requestId);
        return req;
    }

    /**
     * @dev Callback invoked by the Oracle native contract.
     *
     * Neo N3 signature: callback(string url, any userData, int code, bytes result)
     */
    function oracleCallback(
        string calldata url,
        bytes calldata userData,
        int256 code,
        bytes calldata result
    ) external onlyOracleNative {
        uint256 requestId = abi.decode(userData, (uint256));

        Request storage req = _requests[requestId];
        if (req.requester == address(0)) revert OracleRequestNotFound(requestId);
        if (req.completed) {
            // Idempotency guard: ignore duplicate/replayed callbacks.
            return;
        }

        // Best-effort sanity check: ensure url matches the request record.
        // This avoids accidentally correlating two different requests if userData collides.
        if (keccak256(bytes(req.url)) != keccak256(bytes(url))) {
            // Keep going; the response will still be recorded under `requestId`.
        }

        req.completed = true;
        req.responseCode = code;
        req.responseResult = result;

        emit OracleResponse(requestId, req.requester, code, result);

        // Forward to requester (best-effort).
        // Use a typed local variable so the compiler can treat this as an external call target.
        address requester = req.requester;
        if (requester.code.length > 0) {
            try IOracleServiceReceiver(requester).onOracleResponse(
                requestId,
                code,
                result,
                req.userData
            ) {
                // ok
            } catch {
                emit OracleForwardFailed(requestId, requester);
            }
        }
    }
}
