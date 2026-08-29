// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title TypedTx — EIP-2718 typed-transaction envelope demo, compiled to Neo N3.
/// @notice EIP-2718 introduced a tx-type byte (0x01 / 0x02 / 0x03 / 0x04) so EVM
/// can introduce new transaction shapes without breaking RLP. Neo never had this
/// problem: there's exactly one transaction type, defined by Neo.Network.P2P.Payloads.Transaction.
/// This contract exposes the constant tx-version that Neo uses (0).
contract TypedTx {
    string public buildTag = "typed-tx-v1";

    /// Neo only has one transaction format. This is its version field.
    function neoTxVersion() public pure returns (uint8) { return 0; }

    /// Reference values for the EVM tx-type bytes that this constant replaces.
    function evmLegacyType()       public pure returns (uint8) { return 0x00; }
    function evmAccessListType()   public pure returns (uint8) { return 0x01; }
    function evmDynamicFeeType()   public pure returns (uint8) { return 0x02; }
    function evmBlobType()         public pure returns (uint8) { return 0x03; }
    function evmSetCodeType()      public pure returns (uint8) { return 0x04; }
}
