// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title Native Calls Data Structures
 */

library NativeTypes {
    struct NeoCandidate {
        bytes publicKey;
        uint256 votes;
    }
    /// Mirror of NeoToken NeoAccountState:
    /// [balance, balanceHeight, voteTo (ECPoint), lastGasPerVote]
    struct AccountState {
        uint256 balance;
        uint256 balanceHeight;
        bytes voteTo;
        uint256 lastGasPerVote;
    }
    /// NeoVM ContractState — matches the on-chain `ToStackItem` order:
    /// [id, updateCounter, hash, nef, manifest] (see ContractState.cs).
    /// `hash` is Hash160 (address), `id` is a signed integer, and
    /// `updateCounter` is an unsigned integer. Field ORDER matters: the
    /// struct is decoded positionally from the native call's returned array.
    struct ContractState {
        int256 id;
        uint256 updateCounter;
        address hash;
        bytes nef;
        bytes manifest;
    }
    // NOTE: The former `NetworkConfig` struct here declared a field set that
    // no native method produces and was never referenced; the authoritative
    // `NetworkConfig` lives in `NativeCalls.sol` (matching the Rust lowering
    // in `member_nativecalls/network_config.rs`).
}
