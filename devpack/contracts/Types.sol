// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title Neo N3 Shared Types
 * @dev Common type definitions shared across Neo DevPack contracts
 * @author Jimmy <jimmy@r3e.network>
 *
 * This file provides the canonical `Any` type used by NEP standards.
 * Contracts should import this file rather than redefining `type Any is bytes`
 * in each standard to avoid "type already defined" compilation conflicts
 * when inheriting multiple standards (e.g. NEP-17 + NEP-11).
 */

/// @dev Neo N3 `Any` type — represents any stack item type in NeoVM.
/// NEP-17 and NEP-11 type their `transfer`/`onNEP17Payment`/`onNEP11Payment`
/// `data` parameter as `Any`; this alias makes the compiler emit manifest
/// type `Any` (spec-conformant) while behaving as `bytes` in Solidity.
type Any is bytes;
