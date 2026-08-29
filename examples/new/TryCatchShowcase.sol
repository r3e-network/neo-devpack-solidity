// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title TryCatchShowcase
 * @notice Demonstrates try/catch with error data, panic codes,
 *         and raw bytes catch.
 *
 * Neo N3 does not support `new Contract()`. This showcase uses
 * `this.method()` external self-calls to trigger try/catch.
 */
contract TryCatchShowcase {
    event CallSucceeded(uint256 result);
    event CaughtError(string reason);
    event CaughtPanic(uint256 code);
    event CaughtBytes(bytes data);

    // --- External helpers (called via `this.`) ---

    function externalSucceed(uint256 x) external pure returns (uint256) {
        return x * 2;
    }

    function externalSucceedMultiple(uint256 x) external pure returns (uint256, bool) {
        return (x * 2, true);
    }

    function externalRevert(string calldata reason) external pure {
        revert(reason);
    }

    function externalPanic() external pure returns (uint256) {
        uint256 x = 1;
        uint256 y = 0;
        return x / y; // division by zero
    }

    // --- Try/catch demonstrations ---

    function tryCatchSuccess(uint256 val) public {
        try this.externalSucceed(val) returns (uint256 result) {
            emit CallSucceeded(result);
        } catch Error(string memory reason) {
            emit CaughtError(reason);
        }
    }

    function tryCatchMultiple(uint256 val) public {
        try this.externalSucceedMultiple(val) returns (uint256 result, bool successFlag) {
            if (successFlag) {
                emit CallSucceeded(result);
            }
        } catch Error(string memory reason) {
            emit CaughtError(reason);
        }
    }

    function tryCatchRevert() public {
        try this.externalRevert("bad input") {
            // unreachable
        } catch Error(string memory reason) {
            emit CaughtError(reason);
        } catch (bytes memory raw) {
            emit CaughtBytes(raw);
        }
    }

    function tryCatchPanic() public {
        try this.externalPanic() {
            // unreachable
        } catch Panic(uint256 code) {
            emit CaughtPanic(code);
        } catch (bytes memory raw) {
            emit CaughtBytes(raw);
        }
    }
}
