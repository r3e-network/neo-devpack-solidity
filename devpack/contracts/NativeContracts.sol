// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title Neo N3 Native Contract Addresses
 * @dev Canonical script hash constants for Neo N3 native contracts and core libraries.
 *
 * These addresses are deterministic and identical on all Neo N3 networks
 * (MainNet/TestNet/private networks).
 *
 * NOTE: These are written in Neo RPC-style big-endian hex (same values you
 * see via RPC/explorer). The compiler lowers `address` literals to NeoVM
 * `Hash160` stack representation as needed.
 */
library NativeContracts {
    address constant NEO_CONTRACT = 0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5;
    address constant GAS_CONTRACT = 0xd2a4cff31913016155e38e474a2c06d08be276cf;
    address constant CONTRACT_MANAGEMENT = 0xfffdc93764dbaddd97c48f252a53ea4643faa3fd;
    address constant POLICY_CONTRACT = 0xcc5e4edd9f5f8dba8bb65734541df7a1c081c67b;
    address constant ORACLE_CONTRACT = 0xfe924b7cfe89ddd271abaf7210a80a7e11178758;
    address constant ROLE_MANAGEMENT = 0x49cf4e5378ffcd4dec034fd98a174c5491e395e2;
    address constant NOTARY_CONTRACT = 0xc1e14f19c3e60d0b9244d06dd7ba9b113135ec3b;
    address constant TREASURY_CONTRACT = 0x156326f25b1b5d839a4d326aeaa75383c9563ac1;
    address constant LEDGER_CONTRACT = 0xda65b600f7124ce6c79950c1772a36403104f2be;
    address constant CRYPTO_LIB = 0x726cb6e0cd8628a1350a611384688911ab75f51b;
    address constant STD_LIB = 0xacce6fd80d44e1796aa0c2c625e9e4e0ce39efc0;
}
