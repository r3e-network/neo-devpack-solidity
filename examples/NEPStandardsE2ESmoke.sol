// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "../devpack/standards/NEP17.sol";
import "../devpack/standards/NEP11.sol";

/// Minimal no-argument NEP smoke wrappers. CompleteNEP17Token and
/// CompleteNEP11NFT remain compile/manifest fixtures; these wrappers are the
/// deployable Neo-Express E2E contracts so the smoke is not blocked by the
/// Complete examples' large generated scripts.
contract NEP17E2ESmoke is NEP17 {
    constructor() NEP17("Smoke NEP17", "S17", 8, 1_000_000, 1_000_000_000) {}
}

contract NEP11E2ESmoke is NEP11 {
    constructor() NEP11("Smoke NEP11", "S11", 0, "https://example.invalid/", 100, false) {}
}
