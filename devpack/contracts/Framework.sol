// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title Neo N3 Framework
 * @dev Extended framework for Neo N3 Solidity contracts.
 *
 * `FrameworkBase.sol` is designed to keep manifests minimally-permissioned by
 * default. This `Framework.sol` contract previously exposed a fully dynamic
 * `callContract` surface, but that forced wildcard permissions in the manifest
 * (`{"contract":"*","methods":"*"}`), which is a security anti-pattern.
 *
 * STRICT-MANIFEST MODE (current default):
 *   `callContract` is intentionally disabled (reverts at runtime) so that the
 *   compiler can infer precise contract-hash permissions from explicit
 *   `NativeCalls.*` or `Syscalls.contractCall(KNOWN_HASH, ...)` calls.
 *
 * If you need dynamic calls, use one of these alternatives:
 *   1. `Syscalls.contractCall(knownHash, method, params)` with a constant hash
 *      — produces exact manifest permissions.
 *   2. `NativeCalls.*` wrappers for native Neo contracts (GAS, NEO, etc.)
 *      — produces exact native contract permissions.
 *   3. Compile with `--manifest-permissions` to supply explicit overrides
 *      when truly dynamic dispatch is unavoidable.
 */

import "./FrameworkBase.sol";

contract Framework is FrameworkBase {
    /**
     * @dev Call another contract (fully dynamic).
     *
     * Because both the target and method can be user-controlled, Neo N3
     * permission inference cannot restrict this safely without wildcards.
     */
    function callContract(address contractHash, string calldata method, bytes calldata params)
        public
        withWitness
        returns (bytes memory)
    {
        contractHash;
        method;
        params;
        revert(
            "Framework: dynamic call surface disabled in strict-manifest mode; use explicit NativeCalls/Syscalls wrappers"
        );
    }
}

