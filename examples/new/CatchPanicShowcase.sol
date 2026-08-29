// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title CatchPanicShowcase
 * @notice Demonstrates catch Panic(uint256), catch Error(string),
 *         and bare catch(bytes) on Neo N3.
 *
 * NeoVM has a single exception type — it cannot distinguish between
 * Error, Panic, and raw bytes at the VM level.  The compiler selects
 * the first parameterised catch clause and binds the exception value.
 */

interface IExternal {
    function riskyCall() external returns (uint256);
}

contract CatchPanicShowcase {
    event Success(uint256 result);
    event CaughtError(string reason);
    event CaughtPanic(uint256 code);
    event CaughtRaw(bytes data);

    /// Try an external call with Error + bare-bytes catch clauses.
    function tryCatchBasic(address target) public returns (string memory) {
        try IExternal(target).riskyCall() returns (uint256 result) {
            emit Success(result);
            return "success";
        } catch Error(string memory reason) {
            emit CaughtError(reason);
            return reason;
        } catch (bytes memory) {
            emit CaughtRaw("");
            return "unknown error";
        }
    }

    /// Try an external call with only a Panic catch clause.
    function tryCatchPanicOnly(address target) public returns (uint256) {
        try IExternal(target).riskyCall() returns (uint256 result) {
            return result;
        } catch Panic(uint256 code) {
            emit CaughtPanic(code);
            return code;
        }
    }

    /// Single bare catch — simplest form.
    function tryCatchBare(address target) public returns (string memory) {
        try IExternal(target).riskyCall() {
            return "ok";
        } catch {
            return "failed";
        }
    }
}
